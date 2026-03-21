use std::borrow::Cow;

use crate::{
    Bits,
    Block,
};

/// A mutatable bit sequence.
pub trait BitsMut: Bits {
    /// Set a bit at `i`.
    fn set1(&mut self, i: u64);

    /// Unset a bit at `i`.
    fn set0(&mut self, i: u64);
}

impl<B: Block> BitsMut for [B] {
    #[inline]
    fn set1(&mut self, i: u64) {
        let (i, o) = crate::index(i, B::BITS);
        self[i].set1(o)
    }
    #[inline]
    fn set0(&mut self, i: u64) {
        let (i, o) = crate::index(i, B::BITS);
        self[i].set0(o)
    }
}

impl<B: Block, const N: usize> BitsMut for [B; N] {
    #[inline]
    fn set1(&mut self, i: u64) {
        self.as_mut_slice().set1(i)
    }
    #[inline]
    fn set0(&mut self, i: u64) {
        self.as_mut_slice().set0(i)
    }
}

impl<B: Block> BitsMut for Vec<B> {
    #[inline]
    fn set1(&mut self, i: u64) {
        self.as_mut_slice().set1(i)
    }
    #[inline]
    fn set0(&mut self, i: u64) {
        self.as_mut_slice().set0(i)
    }
}

impl<B: BitsMut> BitsMut for Box<B> {
    #[inline]
    fn set1(&mut self, i: u64) {
        self.as_mut().set1(i)
    }
    #[inline]
    fn set0(&mut self, i: u64) {
        self.as_mut().set0(i)
    }
}

impl<T, B> BitsMut for Cow<'_, T>
where
    T: ?Sized + ToOwned<Owned = B> + Bits,
    B: BitsMut,
{
    #[inline]
    fn set1(&mut self, i: u64) {
        self.to_mut().set1(i)
    }
    #[inline]
    fn set0(&mut self, i: u64) {
        self.to_mut().set0(i)
    }
}
