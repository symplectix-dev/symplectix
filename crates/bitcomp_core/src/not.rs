use std::cmp::Ordering::*;
use std::iter::{
    Fuse,
    Peekable,
};

use crate::{
    IntoBlocks,
    Mask,
    compare,
};

/// The difference of two sets A and B.
pub struct Not<A, B> {
    pub(crate) a: A,
    pub(crate) b: B,
}

pub struct Difference<A: Iterator, B: Iterator> {
    a: Peekable<Fuse<A>>,
    b: Peekable<Fuse<B>>,
}

impl<A, B> IntoIterator for Not<A, B>
where
    Self: IntoBlocks,
{
    type Item = (usize, <Self as IntoBlocks>::Block);
    type IntoIter = <Self as IntoBlocks>::Blocks;
    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.into_blocks()
    }
}

impl<A: IntoBlocks, B: IntoBlocks> IntoBlocks for Not<A, B>
where
    A::Block: Mask<B::Block>,
{
    type Block = A::Block;
    type Blocks = Difference<A::Blocks, B::Blocks>;
    #[inline]
    fn into_blocks(self) -> Self::Blocks {
        Difference {
            a: self.a.into_blocks().fuse().peekable(),
            b: self.b.into_blocks().fuse().peekable(),
        }
    }
}

impl<A, B, S1, S2> Iterator for Difference<A, B>
where
    A: Iterator<Item = (usize, S1)>,
    B: Iterator<Item = (usize, S2)>,
    S1: Mask<S2>,
{
    type Item = (usize, S1);
    fn next(&mut self) -> Option<Self::Item> {
        let a = &mut self.a;
        let b = &mut self.b;
        loop {
            match compare(a.peek(), b.peek(), Less, Less) {
                Less => return a.next(),
                Equal => {
                    let (i, mut s1) = a.next().expect("unreachable");
                    let (j, s2) = b.next().expect("unreachable");
                    debug_assert_eq!(i, j);
                    Mask::not(&mut s1, &s2);
                    return Some((i, s1));
                }
                Greater => {
                    b.next();
                }
            };
        }
    }
}
