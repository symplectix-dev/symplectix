use std::ops::RangeBounds;

use crate::{
    Bits,
    BitsMut,
    Block,
    Mask,
    Word,
};

mod private {
    use crate::Word;

    pub trait Array: Sized {}

    impl<B: Word, const N: usize> Array for [B; N] {}
}

/// Page is a fixed size bit array.
#[derive(Debug, Clone, Default)]
pub struct Page<A: private::Array>(
    // Note that `None` does not imply empty bits. `None` is
    // a bit sequence all set to 0.
    //
    // Null Pointer Optimization:
    // https://doc.rust-lang.org/std/option/#representation
    Option<Box<A>>,
);

impl<B: Word, const N: usize> Page<[B; N]> {
    /// Constructs a new page.
    pub fn new() -> Self {
        Page(None)
    }

    // TODO: const fn when as_deref and as_deref_mut is const fn.
    // https://github.com/rust-lang/rust/issues/14387

    /// Returns a slice of buffer, None if empty.
    #[inline]
    pub fn as_ref(&self) -> Option<&[B]> {
        self.0.as_deref().map(|a| a.as_slice())
    }

    /// Returns a mutable slice of buffer, None if empty.
    #[inline]
    pub fn as_mut(&mut self) -> Option<&mut [B]> {
        self.0.as_deref_mut().map(|a| a.as_mut_slice())
    }

    /// Returns a mutable slice of buffer, allocate if empty.
    #[inline]
    pub fn or_empty(&mut self) -> &mut [B; N] {
        self.0.get_or_insert_with(|| Box::new([B::empty(); N]))
    }
}

impl<B: Word, const N: usize> From<[B; N]> for Page<[B; N]> {
    fn from(array: [B; N]) -> Self {
        Page(Some(Box::new(array)))
    }
}

impl<B: Word, const N: usize> Bits for Page<[B; N]> {
    #[inline]
    fn bits(&self) -> u64 {
        <[B; N]>::BITS
    }

    #[inline]
    fn count1(&self) -> u64 {
        self.0.as_ref().map_or(0, Bits::count1)
    }
    #[inline]
    fn count0(&self) -> u64 {
        self.0.as_ref().map_or(<[B; N]>::BITS, Bits::count0)
    }

    #[inline]
    fn all(&self) -> bool {
        self.0.as_ref().is_some_and(Bits::all)
    }

    #[inline]
    fn any(&self) -> bool {
        self.0.as_ref().is_some_and(Bits::any)
    }

    #[inline]
    fn bit(&self, i: u64) -> bool {
        self.0.as_ref().is_some_and(|b| b.bit(i))
    }

    #[inline]
    fn word<T: Word>(&self, i: u64, len: u64) -> T {
        self.0.as_ref().map_or(T::empty(), |b| b.word(i, len))
    }

    #[inline]
    fn rank1<R: RangeBounds<u64>>(&self, r: R) -> u64 {
        self.0.as_ref().map_or(0, |b| b.rank1(r))
    }
    #[inline]
    fn rank0<R: RangeBounds<u64>>(&self, r: R) -> u64 {
        let (i, j) = crate::range(&r, 0, self.bits());
        self.0.as_ref().map_or(j - i, |b| b.rank0(r))
    }

    #[inline]
    fn select1(&self, n: u64) -> Option<u64> {
        self.0.as_ref().and_then(|b| b.select1(n))
    }
    #[inline]
    fn select0(&self, n: u64) -> Option<u64> {
        match self.as_ref() {
            Some(b) => b.select0(n),
            None => (n < Self::BITS).then_some(n),
        }
    }
}

impl<B: Word, const N: usize> BitsMut for Page<[B; N]> {
    #[inline]
    fn set1(&mut self, i: u64) {
        self.or_empty().set1(i)
    }

    #[inline]
    fn set0(&mut self, i: u64) {
        self.or_empty().set0(i)
    }
}

impl<B: Word, const N: usize> Block for Page<[B; N]> {
    const BITS: u64 = <[B; N]>::BITS;

    #[inline]
    fn empty() -> Self {
        Page::new()
    }
}

macro_rules! impls_for_page {
    ($( $Ty:ty ),*) => ($(
        impl<const N: usize> Mask<Self> for Page<[$Ty; N]> {
            fn and(data: &mut Self, that: &Self) {
                match (data.as_mut(), that.as_ref()) {
                    (Some(this), Some(that)) => {
                        for (a, b) in this.iter_mut().zip(that) {
                            *a &= *b;
                        }
                    }
                    (Some(_), None) => {
                        *data = Page::new();
                    }
                    _ => {}
                }
            }

            fn or(data: &mut Self, that: &Self) {
                match (data.as_mut(), that.as_ref()) {
                    (Some(this), Some(that)) => {
                        for (a, b) in this.iter_mut().zip(that) {
                            *a |= *b;
                        }
                    }
                    (None, Some(that)) => {
                        data.or_empty().copy_from_slice(that);
                    }
                    _ => {}
                }
            }

            fn not(data: &mut Self, that: &Self) {
                if let (Some(this), Some(that)) = (data.as_mut(), that.as_ref()) {
                    for (a, b) in this.iter_mut().zip(that) {
                        *a &= !*b;
                    }
                }
            }

            fn xor(data: &mut Self, that: &Self) {
                match (data.as_mut(), that.as_ref()) {
                    (Some(this), Some(that)) => {
                        for (a, b) in this.iter_mut().zip(that) {
                            *a ^= *b;
                        }
                    }
                    (None, Some(that)) => {
                        data.or_empty().copy_from_slice(that);
                    }
                    _ => {}
                }
            }
        }
    )*)
}
impls_for_page!(u8, u16, u32, u64, u128, usize);
