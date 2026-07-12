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

/// The symmetric difference of two sets A and B.
pub struct Xor<A, B> {
    pub(crate) a: A,
    pub(crate) b: B,
}

pub struct SymmetricDifference<A: Iterator, B: Iterator> {
    a: Peekable<Fuse<A>>,
    b: Peekable<Fuse<B>>,
}

impl<A, B> IntoIterator for Xor<A, B>
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

impl<A: IntoBlocks, B: IntoBlocks<Block = A::Block>> IntoBlocks for Xor<A, B>
where
    A::Block: Mask<B::Block>,
{
    type Block = A::Block;
    type Blocks = SymmetricDifference<A::Blocks, B::Blocks>;
    #[inline]
    fn into_blocks(self) -> Self::Blocks {
        SymmetricDifference {
            a: self.a.into_blocks().fuse().peekable(),
            b: self.b.into_blocks().fuse().peekable(),
        }
    }
}

impl<A, B, S> Iterator for SymmetricDifference<A, B>
where
    A: Iterator<Item = (usize, S)>,
    B: Iterator<Item = (usize, S)>,
    S: Mask<S>,
{
    type Item = (usize, S);
    fn next(&mut self) -> Option<Self::Item> {
        let a = &mut self.a;
        let b = &mut self.b;
        match compare(a.peek(), b.peek(), Greater, Less) {
            Less => a.next(),
            Equal => {
                let (i, mut l) = a.next().expect("unreachable");
                let (j, r) = b.next().expect("unreachable");
                debug_assert_eq!(i, j);
                Mask::xor(&mut l, &r);
                Some((i, l))
            }
            Greater => b.next(),
        }
    }
}
