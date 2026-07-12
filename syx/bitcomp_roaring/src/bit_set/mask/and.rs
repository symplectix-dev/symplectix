use std::borrow::Cow;
use std::cmp::Ordering;
use std::iter::Peekable;
use std::marker::PhantomData;

use super::{
    And,
    AndEntries,
};
use crate::bit_set;
use crate::bit_set::Word;

impl<L, R, A> IntoIterator for And<L, R>
where
    L: IntoIterator<Item = bit_set::Entry<A>>,
    R: IntoIterator<Item = bit_set::Entry<A>>,
    AndEntries<L::IntoIter, R::IntoIter, A>: Iterator<Item = bit_set::Entry<A>>,
{
    type Item = bit_set::Entry<A>;
    type IntoIter = AndEntries<L::IntoIter, R::IntoIter, A>;

    fn into_iter(self) -> Self::IntoIter {
        AndEntries {
            lhs: self.lhs.into_iter().peekable(),
            rhs: self.rhs.into_iter().peekable(),
            _ty: PhantomData,
        }
    }
}

impl<'a, L, R, T: Word> Iterator for AndEntries<L, R, Cow<'a, [T]>>
where
    L: Iterator<Item = bit_set::Entry<Cow<'a, [T]>>>,
    R: Iterator<Item = bit_set::Entry<Cow<'a, [T]>>>,
{
    type Item = bit_set::Entry<Cow<'a, [T]>>;
    fn next(&mut self) -> Option<Self::Item> {
        let lhs_mut = &mut self.lhs;
        let rhs_mut = &mut self.rhs;
        next(lhs_mut, rhs_mut).map(|(mut lhs, rhs)| {
            assert_eq!(lhs.index, rhs.index);
            lhs.block = lhs.block & rhs.block;
            lhs
        })
    }
}

#[allow(clippy::type_complexity)]
#[inline]
fn next<L, R>(
    lhs: &mut Peekable<impl Iterator<Item = bit_set::Entry<L>>>,
    rhs: &mut Peekable<impl Iterator<Item = bit_set::Entry<R>>>,
) -> Option<(bit_set::Entry<L>, bit_set::Entry<R>)> {
    loop {
        let compared = {
            if let Some(x) = lhs.peek() { rhs.peek().map(|y| x.index.cmp(&y.index)) } else { None }
        };
        match compared {
            Some(Ordering::Equal) => {
                let lhs = lhs.next().unwrap();
                let rhs = rhs.next().unwrap();
                break Some((lhs, rhs));
            }
            Some(Ordering::Less) => {
                lhs.next();
            }
            Some(Ordering::Greater) => {
                rhs.next();
            }
            None => break None,
        }
    }
}
