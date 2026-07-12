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

/// The union of two sets A and B.
pub struct Or<A, B> {
    pub(crate) a: A,
    pub(crate) b: B,
}

pub struct Union<A: Iterator, B: Iterator> {
    a: Peekable<Fuse<A>>,
    b: Peekable<Fuse<B>>,
}

impl<A, B> IntoIterator for Or<A, B>
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

impl<A: IntoBlocks, B: IntoBlocks<Block = A::Block>> IntoBlocks for Or<A, B>
where
    A::Block: Mask<B::Block>,
{
    type Block = A::Block;
    type Blocks = Union<A::Blocks, B::Blocks>;
    #[inline]
    fn into_blocks(self) -> Self::Blocks {
        Union {
            a: self.a.into_blocks().fuse().peekable(),
            b: self.b.into_blocks().fuse().peekable(),
        }
    }
}

impl<A, B, S> Iterator for Union<A, B>
where
    A: Iterator<Item = (usize, S)>,
    B: Iterator<Item = (usize, S)>,
    S: Mask<S>,
{
    type Item = (usize, S);
    fn next(&mut self) -> Option<Self::Item> {
        let x = &mut self.a;
        let y = &mut self.b;
        match compare(x.peek(), y.peek(), Greater, Less) {
            Less => x.next(),
            Equal => {
                let (i, mut l) = x.next().expect("unreachable");
                let (j, r) = y.next().expect("unreachable");
                debug_assert_eq!(i, j);
                Mask::or(&mut l, &r);
                Some((i, l))
            }
            Greater => y.next(),
        }
    }
}
