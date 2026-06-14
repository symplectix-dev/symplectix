#![allow(missing_docs)]
//! Composable left folds.
//!
//! To create a fold, the most simplest way is from a closure.
//!
//! ```
//! use std::ops::ControlFlow::Continue;
//!
//! use plying::Fold;
//!
//! let sum = |acc, item| Continue(acc + item);
//! assert_eq!(sum.fold_with(0, 1..5), 10);
//! ```
//!
//! You can chain to construct a fold pipeline like an iterator
//! by the `StepFn` trait, but they are evaluated in reverse order.
//!
//! ```
//! use std::ops::ControlFlow::Continue;
//!
//! use plying::{
//!     Fold,
//!     StepFn,
//! };
//!
//! let data = vec![1, 2, 3, 4, 5];
//! let sum = |acc, item| Continue(acc + item);
//! let f = sum.map(|x: &i32| x - 1).take(3);
//! assert_eq!(3, f.fold_with(0, &data[..]));
//! ```
//!
//! So in this case, `take` is applied to data first, then `map`, and finally `sum`.

use std::ops::ControlFlow::*;

mod beginning;
mod completing;
mod filter;
mod fuse;
mod map;
mod par;
mod seq;
mod take;
mod zip;

use beginning::Beginning;
use completing::Completing;
use filter::Filter;
use fuse::Fuse;
use map::Map;
use par::{
    FoldInScope,
    Par,
};
use seq::Seq;
use take::Take;
use zip::Zip;

mod internal {
    pub(crate) use std::ops::ControlFlow::*;

    pub(crate) use crate::{
        ControlFlow,
        Fold,
        Fuse,
        InitialState,
        StepFn,
    };
}

/// A composable left fold.
pub trait Fold<A, B> {
    /// The accumulator, used to store the intermediate result while folding.
    type State;

    fn fold_with<T>(self, init: Self::State, iterable: T) -> B
    where
        Self: Sized,
        T: IntoIterator<Item = A>;

    #[inline]
    fn fold<T>(self, iterable: T) -> B
    where
        Self: Sized + InitialState<Self::State>,
        T: IntoIterator<Item = A>,
    {
        let iter = iterable.into_iter();
        let init = self.initial_state(iter.size_hint());
        self.fold_with(init, iter)
    }
}

pub trait InitialState<St> {
    fn initial_state(&self, size_hint: (usize, Option<usize>)) -> St;
}

/// The result of [Fold.step].
pub type ControlFlow<T> = std::ops::ControlFlow<T, T>;

pub trait StepFn<A, B> {
    /// The accumulator, used to store the intermediate result while folding.
    type State;

    /// Runs just a one step of folding.
    // TODO: consider to use Try instead of ControlFlow.
    // https://doc.rust-lang.org/std/ops/trait.Try.html
    // https://github.com/rust-lang/rust/issues/84277
    fn step(&mut self, acc: Self::State, item: A) -> ControlFlow<Self::State>;

    /// Invoked when folding is complete.
    fn complete(self, acc: Self::State) -> B;

    fn beginning<F>(self, f: F) -> Beginning<Self, F>
    where
        Self: Sized,
        F: Fn((usize, Option<usize>)) -> Self::State,
    {
        Beginning::new(self, f)
    }

    fn completing<C, F>(self, f: F) -> Completing<Self, B, F>
    where
        Self: Sized,
        F: FnMut(B) -> C,
    {
        Completing::new(self, f)
    }

    fn map<F>(self, f: F) -> Map<Self, F>
    where
        Self: Sized,
    {
        Map::new(self, f)
    }

    fn filter<P>(self, f: P) -> Filter<Self, P>
    where
        Self: Sized,
    {
        Filter::new(self, f)
    }

    fn take(self, n: usize) -> Take<Self>
    where
        Self: Sized,
    {
        Take::new(self, n)
    }

    fn seq<That>(self, that: That) -> Seq<Self, That>
    where
        Self: Sized,
    {
        Seq::new(self, that)
    }

    fn zip<'a, That>(self, that: That) -> Zip<'a, Self, That>
    where
        Self: Sized,
    {
        Zip::new(self, that)
    }

    fn par<That>(self, that: That) -> Par<A, Self, That>
    where
        Self: Sized,
    {
        Par::new(self, that)
    }
}

impl<A, B, F> StepFn<A, B> for F
where
    F: FnMut(B, A) -> ControlFlow<B>,
{
    type State = B;

    #[inline]
    fn step(&mut self, acc: Self::State, item: A) -> ControlFlow<Self::State> {
        (self)(acc, item)
    }

    #[inline]
    fn complete(self, acc: Self::State) -> B {
        acc
    }
}

// The bracket impl below cause conflicting implementations error:
// error[E0119]: conflicting implementations of trait `Fold<&[_], _>` for type `Par<_, _>`
// downstream crates may implement trait `StepFn<&[_], _>` for type `Par<_, _>`
//
// impl<A, B, Sf> Fold<A, B> for Sf
// where
//     Sf: StepFn<A, B>,
// {
//     type State = <Sf as StepFn<A, B>>::State;
//     def_fold_with!();
// }
//
// n.b. Par does not implement StepFn intentionaly.
//
// All StepFn should be able to automatically implement Fold,
// but this error prevents it from doing so. This is very annoying.
// The boilerplate below is a workaround to avoid this error.
//
// See also: https://github.com/rust-lang/rust/issues/48869

macro_rules! def_fold_with {
    ($Item:ty) => {
        #[inline]
        fn fold_with<Iter>(mut self, init: Self::State, iterable: Iter) -> B
        where
            Self: Sized,
            Iter: IntoIterator<Item = $Item>,
        {
            match iterable.into_iter().try_fold(init, |acc, v| self.step(acc, v)) {
                Continue(c) => self.complete(c),
                Break(b) => self.complete(b),
            }
        }
    };
}

macro_rules! impl_fold {
    (Fold<$A:tt, $B:tt> for $Ty:ident<$($type_params:tt),+>) => {
        impl<$($type_params,)+ $A, $B> Fold<$A, $B> for $Ty<$($type_params),+>
        where
            Self: StepFn<$A, $B>,
        {
            type State = <Self as StepFn<$A, $B>>::State;
            def_fold_with!($A);
        }
    };
}

impl<A, B, F> Fold<A, B> for F
where
    F: FnMut(B, A) -> ControlFlow<B>,
{
    type State = B;
    def_fold_with!(A);
}

impl_fold!(Fold<A, B> for Beginning<Sf, F>);
impl_fold!(Fold<A, B> for Completing<Sf, R, F>);
impl_fold!(Fold<A, B> for Fuse<Sf>);
impl_fold!(Fold<A, B> for Map<Sf, F>);
impl_fold!(Fold<A, B> for Filter<Sf, P>);
impl_fold!(Fold<A, B> for Take<Sf>);
impl_fold!(Fold<A, B> for Seq<F, G>);
impl_fold!(Fold<A, B> for Zip<'a, F, G>);
impl_fold!(Fold<A, B> for FoldInScope<'s, 'e, F>);
