use std::borrow::Cow;
use std::cmp::Ordering::*;
use std::iter::Peekable;
use std::marker::PhantomData;

use super::{
    Xor,
    XorEntries,
};
use crate::bit_set;
use crate::bit_set::Word;

impl<L, R, A> IntoIterator for Xor<L, R>
where
    L: IntoIterator<Item = bit_set::Entry<A>>,
    R: IntoIterator<Item = bit_set::Entry<A>>,
    XorEntries<L::IntoIter, R::IntoIter, A>: Iterator<Item = bit_set::Entry<A>>,
{
    type Item = bit_set::Entry<A>;
    type IntoIter = XorEntries<L::IntoIter, R::IntoIter, A>;

    fn into_iter(self) -> Self::IntoIter {
        XorEntries {
            lhs: self.lhs.into_iter().peekable(),
            rhs: self.rhs.into_iter().peekable(),
            _ty: PhantomData,
        }
    }
}

impl<'a, L, R, T: Word> Iterator for XorEntries<L, R, Cow<'a, [T]>>
where
    L: Iterator<Item = bit_set::Entry<Cow<'a, [T]>>>,
    R: Iterator<Item = bit_set::Entry<Cow<'a, [T]>>>,
{
    type Item = bit_set::Entry<Cow<'a, [T]>>;
    fn next(&mut self) -> Option<Self::Item> {
        let lhs_mut = &mut self.lhs;
        let rhs_mut = &mut self.rhs;
        next(lhs_mut, rhs_mut).and_then(|pair| match pair {
            (Some(mut lhs), Some(rhs)) => {
                assert_eq!(lhs.index, rhs.index);
                lhs.block = lhs.block ^ rhs.block;
                Some(lhs)
            }
            (Some(lhs), None) => Some(lhs),
            (None, Some(rhs)) => Some(rhs),
            _ => None,
        })
    }
}

#[allow(clippy::type_complexity)]
#[inline]
fn next<L, R, T>(
    lhs: &mut Peekable<L>,
    rhs: &mut Peekable<R>,
) -> Option<(Option<bit_set::Entry<T>>, Option<bit_set::Entry<T>>)>
where
    L: Iterator<Item = bit_set::Entry<T>>,
    R: Iterator<Item = bit_set::Entry<T>>,
{
    let compared = {
        let lhs_peek = lhs.peek();
        let rhs_peek = rhs.peek();
        super::compare_entry(lhs_peek, rhs_peek, Greater, Less)
    };
    match compared {
        Less => lhs.next().map(|lhs| (Some(lhs), None)),
        Equal => {
            let lhs = lhs.next();
            let rhs = rhs.next();
            Some((lhs, rhs))
        }
        Greater => rhs.next().map(|rhs| (None, Some(rhs))),
    }
}
