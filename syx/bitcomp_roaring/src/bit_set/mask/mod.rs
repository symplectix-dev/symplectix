mod and;
mod not;
mod or;
mod xor;

mod padding;
mod range;

use std::fmt;
use std::iter::Peekable;
use std::marker::PhantomData;

use crate::bit_set;

/// And
#[derive(Clone, PartialEq, Eq)]
pub struct And<L, R> {
    pub(crate) lhs: L,
    pub(crate) rhs: R,
}

/// Or
#[derive(Clone, PartialEq, Eq)]
pub struct Or<L, R> {
    pub(crate) lhs: L,
    pub(crate) rhs: R,
}

/// Xor
#[derive(Clone, PartialEq, Eq)]
pub struct Xor<L, R> {
    pub(crate) lhs: L,
    pub(crate) rhs: R,
}

/// Not
#[derive(Clone, Default, PartialEq, Eq)]
pub struct Not<I> {
    pub(crate) val: I,
}

pub struct AndEntries<L: Iterator, R: Iterator, A> {
    lhs: Peekable<L>,
    rhs: Peekable<R>,
    _ty: PhantomData<A>,
}

pub struct OrEntries<L: Iterator, R: Iterator, A> {
    lhs: Peekable<L>,
    rhs: Peekable<R>,
    _ty: PhantomData<A>,
}

pub struct XorEntries<L: Iterator, R: Iterator, A> {
    lhs: Peekable<L>,
    rhs: Peekable<R>,
    _ty: PhantomData<A>,
}

pub struct NotEntries<I: Iterator, A> {
    padding: PaddingUnbound<I>,
    _block:  PhantomData<A>,
}

pub struct PaddingUnbound<I: Iterator> {
    dummy: std::ops::RangeFrom<usize>, // dummy iterator
    value: Peekable<I>,
}

pub struct Fold<'a, B>(Option<DynEntries<'a, B>>);

type DynEntries<'a, B> = Box<dyn Iterator<Item = bit_set::Entry<B>> + 'a>;

/// Compare `a` and `b`, but return `x` if a is None and `y` if b is None
#[inline]
fn compare_entry<L, R>(
    a: Option<&bit_set::Entry<L>>,
    b: Option<&bit_set::Entry<R>>,
    x: std::cmp::Ordering,
    y: std::cmp::Ordering,
) -> std::cmp::Ordering {
    match (a, b) {
        (Some(lhs), Some(rhs)) => lhs.index.cmp(&rhs.index),
        (None, _) => x,
        (_, None) => y,
    }
}

impl<L: fmt::Debug, R: fmt::Debug> fmt::Debug for And<L, R> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("And").field(&self.lhs).field(&self.rhs).finish()
    }
}
impl<L: fmt::Debug, R: fmt::Debug> fmt::Debug for Or<L, R> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("Or").field(&self.lhs).field(&self.rhs).finish()
    }
}
impl<L: fmt::Debug, R: fmt::Debug> fmt::Debug for Xor<L, R> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("Xor").field(&self.lhs).field(&self.rhs).finish()
    }
}
// impl<T: fmt::Debug> fmt::Debug for Not<T> {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         f.debug_tuple("Not").field(&self.val).finish()
//     }
// }

impl<'a, B> fmt::Debug for Fold<'a, B> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("Fold").field(&format_args!("_")).finish()
    }
}

pub fn and<L, R>(lhs: L, rhs: R) -> And<L, R> {
    And { lhs, rhs }
}

pub fn or<L, R>(lhs: L, rhs: R) -> Or<L, R> {
    Or { lhs, rhs }
}

pub fn xor<L, R>(lhs: L, rhs: R) -> Xor<L, R> {
    Xor { lhs, rhs }
}

// pub fn not<T>(val: T) -> Not<T> {
//     Not { val }
// }

impl<'a, B: 'a> Iterator for Fold<'a, B> {
    type Item = bit_set::Entry<B>;
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.0.as_mut().and_then(|it| it.next())
    }
}

impl<'a, B: 'a> Fold<'a, B> {
    pub fn and<T>(iters: impl IntoIterator<Item = T>) -> Fold<'a, B>
    where
        T: IntoIterator<Item = bit_set::Entry<B>> + 'a,
        And<DynEntries<'a, B>, T>: IntoIterator<Item = bit_set::Entry<B>>,
    {
        Self::new(iters, and)
    }

    pub fn or<T>(iters: impl IntoIterator<Item = T>) -> Fold<'a, B>
    where
        T: IntoIterator<Item = bit_set::Entry<B>> + 'a,
        Or<DynEntries<'a, B>, T>: IntoIterator<Item = bit_set::Entry<B>>,
    {
        Self::new(iters, or)
    }

    pub fn xor<T>(iters: impl IntoIterator<Item = T>) -> Fold<'a, B>
    where
        T: IntoIterator<Item = bit_set::Entry<B>> + 'a,
        Xor<DynEntries<'a, B>, T>: IntoIterator<Item = bit_set::Entry<B>>,
    {
        Self::new(iters, xor)
    }

    fn new<T, U>(
        iters: impl IntoIterator<Item = T>,
        func: impl Fn(DynEntries<'a, B>, T) -> U,
    ) -> Fold<'a, B>
    where
        T: IntoIterator<Item = bit_set::Entry<B>> + 'a,
        U: IntoIterator<Item = bit_set::Entry<B>> + 'a,
    {
        let mut iters = iters.into_iter();
        Fold(if let Some(head) = iters.next() {
            let head = Box::new(head.into_iter());
            Some(iters.fold(head, |it, x| Box::new(func(it, x).into_iter())))
        } else {
            None
        })
    }
}
