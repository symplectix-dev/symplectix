use std::ops::RangeBounds;
use std::{
    fmt,
    ops,
};

use crate::{
    Bits,
    BitsMut,
    Block,
    Mask,
};

/// Unsigned integers as a bits block.
pub trait Word:
    num::PrimInt
    + fmt::Debug
    + fmt::Display
    + fmt::Binary
    + fmt::LowerHex
    + fmt::UpperHex
    + ops::Not<Output = Self>
    + ops::BitAnd<Output = Self>
    + ops::BitOr<Output = Self>
    + ops::BitXor<Output = Self>
    + ops::BitAndAssign
    + ops::BitOrAssign
    + ops::BitXorAssign
    + ops::Shl<usize, Output = Self>
    + ops::Shr<usize, Output = Self>
    + ops::ShlAssign<usize>
    + ops::ShrAssign<usize>
    + Bits
    + BitsMut
    + Block
{
    #[doc(hidden)]
    const BITS_MINUS_1: u32;

    /// Least significant set bit.
    fn lsb(self) -> Self;

    /// Most significant set bit.
    fn msb(self) -> Self;
}

// A helper trait to implement `select1` for `Word`.
trait SelectHelper {
    fn select1(self, n: u64) -> Option<u64>;
}

impl SelectHelper for u64 {
    #[inline]
    fn select1(self, n: u64) -> Option<u64> {
        (n < self.count1()).then(|| {
            #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
            if is_x86_feature_detected!("bmi2") {
                use std::arch::x86_64::{
                    _pdep_u64,
                    _tzcnt_u64,
                };
                return unsafe { _tzcnt_u64(_pdep_u64(1 << n, self)) };
            }
            broadword(self, n)
        })
    }
}

impl SelectHelper for u8 {
    #[inline]
    fn select1(self, c: u64) -> Option<u64> {
        (c < self.count1()).then(|| <u64 as SelectHelper>::select1(self as u64, c).unwrap())
    }
}
impl SelectHelper for u16 {
    #[inline]
    fn select1(self, c: u64) -> Option<u64> {
        (c < self.count1()).then(|| <u64 as SelectHelper>::select1(self as u64, c).unwrap())
    }
}
impl SelectHelper for u32 {
    #[inline]
    fn select1(self, c: u64) -> Option<u64> {
        (c < self.count1()).then(|| <u64 as SelectHelper>::select1(self as u64, c).unwrap())
    }
}

impl SelectHelper for u128 {
    #[inline]
    fn select1(self, c: u64) -> Option<u64> {
        let arr = [self as u64, (self >> 64) as u64];
        arr.select1(c)
    }
}

#[cfg(target_pointer_width = "32")]
impl SelectHelper for usize {
    #[inline]
    fn select1(self, c: u64) -> Option<u64> {
        (c < self.count1()).then(|| <u64 as SelectHelper>::select1(self as u64, c).unwrap())
    }
}

#[cfg(target_pointer_width = "64")]
impl SelectHelper for usize {
    #[inline]
    fn select1(self, c: u64) -> Option<u64> {
        (c < self.count1()).then(|| <u64 as SelectHelper>::select1(self as u64, c).unwrap())
    }
}

// Sebastiano Vigna, “Broadword Implementation of Rank/Select Queries”.
// Returns 72 when not found.
#[allow(clippy::many_single_char_names)]
fn broadword(x: u64, n: u64) -> u64 {
    const L8: u64 = 0x0101_0101_0101_0101; // has the lowest bit of every bytes
    const H8: u64 = 0x8080_8080_8080_8080; // has the highest bit of every bytes

    #[inline]
    const fn le8(x: u64, y: u64) -> u64 {
        (((y | H8) - (x & !H8)) ^ x ^ y) & H8
    }

    #[inline]
    const fn lt8(x: u64, y: u64) -> u64 {
        (((x | H8) - (y & !H8)) ^ x ^ !y) & H8
    }

    #[inline]
    const fn nz8(x: u64) -> u64 {
        lt8(0, x)
    }

    let mut s = x - ((x & 0xAAAA_AAAA_AAAA_AAAA) >> 1);
    s = (s & 0x3333_3333_3333_3333) + ((s >> 2) & 0x3333_3333_3333_3333);
    s = ((s + (s >> 4)) & 0x0F0F_0F0F_0F0F_0F0F).wrapping_mul(L8);

    let b = ((le8(s, n.wrapping_mul(L8)) >> 7).wrapping_mul(L8) >> 53) & !7;
    let l = n - ((s << 8).wrapping_shr(b as u32) & 0xFF);

    s = nz8((x.wrapping_shr(b as u32) & 0xFF).wrapping_mul(L8) & 0x8040_2010_0804_0201);
    s = (s >> 7).wrapping_mul(L8);

    ((le8(s, l * L8) >> 7).wrapping_mul(L8) >> 56) + b
}

mod private {
    use super::Word;

    pub trait Repr: Sized {}

    impl<B: Word, const N: usize> Repr for [B; N] {}
}

/// Buf is a boxed array of `Word` and can be used as a fixed-size bit block.
#[derive(Debug, Clone, Default)]
pub struct Buf<A: private::Repr>(
    // Note that `None` does not imply empty bits. `None` is
    // a bit sequence all set to 0.
    //
    // Null Pointer Optimization:
    // https://doc.rust-lang.org/std/option/#representation
    Option<Box<A>>,
);

impl<B: Word, const N: usize> Buf<[B; N]> {
    /// Constructs a new buffer.
    pub fn new() -> Self {
        Buf(None)
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

impl<B: Word, const N: usize> From<[B; N]> for Buf<[B; N]> {
    fn from(array: [B; N]) -> Self {
        Buf(Some(Box::new(array)))
    }
}

impl<B: Word, const N: usize> Bits for Buf<[B; N]> {
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

impl<B: Word, const N: usize> BitsMut for Buf<[B; N]> {
    #[inline]
    fn set1(&mut self, i: u64) {
        self.or_empty().set1(i)
    }

    #[inline]
    fn set0(&mut self, i: u64) {
        self.or_empty().set0(i)
    }
}

impl<B: Word, const N: usize> Block for Buf<[B; N]> {
    const BITS: u64 = <[B; N]>::BITS;

    #[inline]
    fn empty() -> Self {
        Buf::new()
    }
}

macro_rules! mask {
    ($( $i: expr, $j: expr )*) => ($(
        ((1 << ($j - $i)) - 1) << $i
    )*)
}

macro_rules! impls_for_word {
    ($( $Ty:ty ),*) => ($(
        impl Word for $Ty {
            // const _0: Self = 0;

            // const _1: Self = 1;

            const BITS_MINUS_1: u32 = <$Ty>::BITS - 1;

            #[inline]
            fn lsb(self) -> Self {
                self & self.wrapping_neg()
            }

            #[inline]
            fn msb(self) -> Self {
                if self == 0 {
                    0
                } else {
                    1 << (Self::BITS_MINUS_1 - self.leading_zeros())
                }
            }
        }

        impl Block for $Ty {
            const BITS: u64 = <$Ty>::BITS as u64;

            #[inline]
            fn empty() -> Self {
                0
            }
        }

        impl Bits for $Ty {
            #[inline]
            fn bits(&self) -> u64 {
                <Self as Block>::BITS
            }

            #[inline]
            fn count1(&self) -> u64 {
                self.count_ones() as u64
            }

            #[inline]
            fn count0(&self) -> u64 {
                self.count_zeros() as u64
            }

            #[inline]
            fn all(&self) -> bool {
                *self == !0
            }

            #[inline]
            fn any(&self) -> bool {
                *self != 0
            }

            #[inline]
            fn bit(&self, i: u64) -> bool {
                assert!(i < <$Ty as Block>::BITS);
                (*self & (1 << i)) != 0
            }

            #[inline]
            fn word<T: Word>(&self, i: u64, n: u64) -> T {
                debug_assert!(i < self.bits() && n <= T::BITS);
                // Shift first, then mask — avoids `1 << n` overflow when n == Self::BITS.
                let shifted = *self >> i;
                let masked = if n < <Self as Block>::BITS { shifted & ((1 << n) - 1) } else { shifted };
                num::cast(masked).expect("cast failed")
            }

            // fn write_word<T: Word>(&mut self, i: usize, n: usize, bits: T) {
            //     debug_assert!(i < <$Ty as Block>::BITS && n <= T::BITS);
            //     if n > 0 /*&& bits.any()*/ {
            //         let n = n.clamp(1, T::BITS);
            //         let w = T::BITS - n;
            //         let m1 = num::cast::<T, $Ty>(( bits << w) >> w).expect("bug");
            //         let m0 = num::cast::<T, $Ty>((!bits << w) >> w).expect("bug");
            //         *self |= m1<<i;
            //         *self &= !(m0<<i);
            //     }
            // }

            #[inline]
            fn rank1<R: RangeBounds<u64>>(&self, r: R) -> u64 {
                let (i, j) = crate::range(&r, 0, <$Ty as Block>::BITS);
                (*self & mask!(i, j)).count1()
            }

            #[inline]
            fn rank0<R: RangeBounds<u64>>(&self, r: R) -> u64 {
                (!*self).rank1(r)
            }

            #[inline]
            fn select1(&self, n: u64) -> Option<u64> {
                <Self as SelectHelper>::select1(*self, n)
            }

            #[inline]
            fn select0(&self, n: u64) -> Option<u64> {
                <Self as SelectHelper>::select1(!self, n)
            }
        }

        impl BitsMut for $Ty {
            #[inline]
            fn set1(&mut self, i: u64) {
                *self |= 1 << i;
            }
            #[inline]
            fn set0(&mut self, i: u64) {
                *self &= !(1 << i);
            }
        }

        impl<const N: usize> Mask<Self> for Buf<[$Ty; N]> {
            fn and(data: &mut Self, that: &Self) {
                match (data.as_mut(), that.as_ref()) {
                    (Some(this), Some(that)) => {
                        for (a, b) in this.iter_mut().zip(that) {
                            *a &= *b;
                        }
                    }
                    (Some(_), None) => {
                        *data = Buf::new();
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
impls_for_word!(u8, u16, u32, u64, u128, usize);

#[cfg(test)]
mod tests {
    use crate::{
        Bits,
        BitsMut,
    };

    #[test]
    fn u16_word() {
        let b: u16 = 0b_0011_1000_0011_0010;
        assert_eq!(b.word::<u16>(0, 4), 0b_0010);
        assert_eq!(b.word::<u32>(3, 4), 0b_0110);
        assert_eq!(b.word::<u64>(9, 4), 0b_1100);
    }

    #[test]
    fn u32_word() {
        let b: u32 = 0b_0011_1000_0011_0010_0011_1000_0011_0010;
        assert_eq!(b.word::<u16>(16, 7), 0b011_0010);
        assert_eq!(b.word::<u16>(24, 7), 0b011_1000);
    }

    #[test]
    fn u128_select() {
        let mut b: u128 = 0;
        for i in (0..128).step_by(2) {
            b.set1(i);
        }
        assert_eq!(b.select1(0), Some(0));
        assert_eq!(b.select1(1), Some(2));
        assert_eq!(b.select0(0), Some(1));
        assert_eq!(b.select0(1), Some(3));

        assert_eq!(b.select1(60), Some(120));
        assert_eq!(b.select1(61), Some(122));
    }
}
