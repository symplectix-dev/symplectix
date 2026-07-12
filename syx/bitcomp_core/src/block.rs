use std::borrow::Cow;

use crate::{
    Bits,
    BitsMut,
    Word,
};

/// Fixed sized bits.
// TODO: remove BitsMut from constraints.
pub trait Block: Clone + Bits + BitsMut {
    /// The number of bits, which must always be equal to `Bits::bits`.
    const BITS: u64;

    /// Constructs an empty bits block.
    fn empty() -> Self;
}

/// Helper trait for blockwise iteration.
pub trait IntoBlocks: Sized {
    /// Type of a bit container.
    type Block;

    /// An iterator which yields `Block`s with its index.
    type Blocks: Iterator<Item = (usize, Self::Block)>;

    /// Returns an iterator.
    fn into_blocks(self) -> Self::Blocks;
}

/// Helper trait for blockwise iteration.
pub trait FromBlocks<B>: Sized {
    /// Constructs a value from blocks.
    fn from_blocks<T: IntoBlocks<Block = B>>(iter: T) -> Self;
}

impl<'a, B: Block> IntoBlocks for &'a [B] {
    type Block = Cow<'a, B>;
    type Blocks = slice::Blocks<'a, B>;
    fn into_blocks(self) -> Self::Blocks {
        slice::Blocks { blocks: self.iter().enumerate() }
    }
}

mod slice {
    use std::borrow::Cow;
    use std::iter::Enumerate;

    use crate::Block;

    pub struct Blocks<'a, B> {
        pub(crate) blocks: Enumerate<std::slice::Iter<'a, B>>,
    }
    impl<'a, B: Block> Iterator for Blocks<'a, B> {
        type Item = (usize, Cow<'a, B>);
        #[inline]
        fn next(&mut self) -> Option<Self::Item> {
            self.blocks.find_map(|(i, b)| b.any().then(|| (i, Cow::Borrowed(b))))
        }
    }
}

impl<B: Word, const N: usize> Block for [B; N] {
    const BITS: u64 = B::BITS * N as u64;
    #[inline]
    fn empty() -> Self {
        [B::empty(); N]
    }
}
impl<'a, B, const N: usize> IntoBlocks for &'a [B; N]
where
    &'a [B]: IntoBlocks,
{
    type Block = <&'a [B] as IntoBlocks>::Block;
    type Blocks = <&'a [B] as IntoBlocks>::Blocks;
    #[inline]
    fn into_blocks(self) -> Self::Blocks {
        self.as_ref().into_blocks()
    }
}

impl<'a, B: Block> IntoBlocks for &'a Vec<B> {
    type Block = <&'a [B] as IntoBlocks>::Block;
    type Blocks = <&'a [B] as IntoBlocks>::Blocks;
    fn into_blocks(self) -> Self::Blocks {
        self.as_slice().into_blocks()
    }
}

impl<B: Block> Block for Box<B> {
    const BITS: u64 = B::BITS;
    #[inline]
    fn empty() -> Self {
        Box::new(B::empty())
    }
}

impl<T, B> Block for Cow<'_, T>
where
    T: ?Sized + ToOwned<Owned = B> + Bits,
    B: Block,
{
    const BITS: u64 = B::BITS;
    #[inline]
    fn empty() -> Self {
        Cow::Owned(B::empty())
    }
}

impl<'inner, T: ?Sized> IntoBlocks for &&'inner T
where
    &'inner T: IntoBlocks,
{
    type Block = <&'inner T as IntoBlocks>::Block;
    type Blocks = <&'inner T as IntoBlocks>::Blocks;
    #[inline]
    fn into_blocks(self) -> Self::Blocks {
        IntoBlocks::into_blocks(*self)
    }
}
