#![allow(dead_code)]

use std::borrow::Borrow;
use std::ops::ControlFlow::*;
use std::ops::{
    Add,
    Mul,
    Rem,
};
use std::rc::Rc;

use plying::{
    Fold,
    InitialState,
    StepFn,
};

pub fn pow2(x: i32) -> i32 {
    x.pow(2)
}

pub fn pow2_rc(x: Rc<i32>) -> i32 {
    pow2(*x.borrow())
}

pub fn mul3<T>(x: T) -> T::Output
where
    T: Mul<i32>,
{
    x.mul(3)
}

pub fn mul3_rc(x: Rc<i32>) -> i32 {
    mul3::<i32>(*x.borrow())
}

pub fn even<T>(x: &T) -> bool
where
    T: Copy + Rem<i32, Output = i32>,
{
    x.rem(2) == 0
}

pub trait FoldFn<A, B>: Fold<A, B, State = B> + StepFn<A, B, State = B> + InitialState<B> {}
impl<A, B, T> FoldFn<A, B> for T where
    T: Fold<A, B, State = B> + StepFn<A, B, State = B> + InitialState<B>
{
}

pub fn conj<A>() -> impl FoldFn<A, Vec<A>> + Clone {
    let f = |mut acc: Vec<A>, item| {
        acc.push(item);
        Continue(acc)
    };
    f.beginning(|(lo, _hi)| Vec::with_capacity(lo.saturating_add(1)))
}

pub fn all<A, P>(mut pred: P) -> impl FoldFn<A, bool>
where
    P: FnMut(&A) -> bool,
{
    let f = move |_acc, item| {
        if pred(&item) { Continue(true) } else { Break(false) }
    };
    f.beginning(|_| true)
}

#[derive(Debug, Clone)]
pub struct Any<P> {
    pred: P,
}

impl<A, B, P> Fold<A, B> for Any<P>
where
    Self: StepFn<A, B>,
{
    type State = <Self as StepFn<A, B>>::State;
    fn fold_with<T>(mut self, init: Self::State, iterable: T) -> B
    where
        Self: Sized,
        T: IntoIterator<Item = A>,
    {
        match iterable.into_iter().try_fold(init, |acc, v| self.step(acc, v)) {
            Continue(c) => self.complete(c),
            Break(b) => self.complete(b),
        }
    }
}
impl<A, P> StepFn<A, bool> for Any<P>
where
    P: FnMut(&A) -> bool,
{
    type State = bool;

    #[inline]
    fn step(&mut self, _acc: Self::State, item: A) -> plying::ControlFlow<Self::State> {
        if (self.pred)(&item) { Break(true) } else { Continue(false) }
    }

    #[inline]
    fn complete(self, acc: Self::State) -> bool {
        acc
    }
}

impl<P> InitialState<bool> for Any<P> {
    #[inline]
    fn initial_state(&self, _size_hint: (usize, Option<usize>)) -> bool {
        false
    }
}

pub fn any<A, P>(pred: P) -> Any<P>
where
    P: FnMut(&A) -> bool,
{
    Any { pred }
    // let f = move |_acc: bool, item: A| {
    //     if pred(&item) { Break(true) } else { Continue(false) }
    // };
    // f.using(|_| false)
}

pub fn count<A>() -> impl FoldFn<A, usize> + Clone {
    let f = |acc: usize, _item: A| Continue(acc + 1);
    f.beginning(|_| 0)
}

pub fn sum<A, B>() -> impl FoldFn<A, B> + Clone
where
    B: Default + Add<A, Output = B>,
{
    let f = |acc, item| Continue(acc + item);
    f.beginning(|_| B::default())
}

pub fn sum_rc<A, B>() -> impl FoldFn<Rc<A>, B> + Clone
where
    A: Copy,
    B: Default + Add<A, Output = B>,
{
    let f = |acc, item: Rc<A>| Continue(acc + *item.borrow());
    f.beginning(|_| B::default())
}
