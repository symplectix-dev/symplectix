use crate::bit_set;
use crate::bit_set::ops::{
    Capacity,
    Count,
};
use crate::bit_set::word;

pub trait Select1: Count {
    /// Returns the position of 'n+1'th occurences of non-zero bit.
    ///
    /// `select1` is a right inverse of `rank1`, under the condition `i < count1`.
    /// `select1` should panic when it's impossible to satisfy the following rule.
    ///
    /// # Rule
    ///
    /// ```text
    /// select1(y) = x, then rank1(x) = y
    /// ```
    fn select1(&self, n: u64) -> u64;
}

pub trait Select0: Count {
    /// Returns the position of 'n+1'th occurences of zero bit.
    ///
    /// `select0` is a right inverse of `rank0`, under the condition `i < count0`.
    /// `select0` should panic when it's impossible to satisfy the following rule.
    ///
    /// # Rule
    ///
    /// ```text
    /// select0(y) = x, then rank0(x) = y
    /// ```
    fn select0(&self, n: u64) -> u64;
}

const X01: u64 = 0x0101_0101_0101_0101;
const X02: u64 = 0x2020_2020_2020_2020;
const X33: u64 = 0x3333_3333_3333_3333;
const X22: u64 = 0x2222_2222_2222_2222;
const X80: u64 = 0x2010_0804_0201_0080;
const X81: u64 = 0x2010_0804_0201_0081;
const X0F: u64 = 0x0f0f_0f0f_0f0f_0f0f;
const X55: u64 = X22 + X33 + X22 + X33;
const X8X: u64 = X81 + X80 + X80 + X80;

fn le8(x: u64, y: u64) -> u64 {
    let x8 = X02 + X02 + X02 + X02;
    let xs = (y | x8) - (x & !x8);
    (xs ^ x ^ y) & x8
}

fn lt8(x: u64, y: u64) -> u64 {
    let x8 = X02 + X02 + X02 + X02;
    let xs = (x | x8) - (y & !x8);
    (xs ^ x ^ !y) & x8
}

impl Select1 for u64 {
    fn select1(&self, c: u64) -> u64 {
        assert!(c < self.count1());

        let x = self;
        let s0 = x - ((x & X55) >> 1);
        let s1 = (s0 & X33) + ((s0 >> 2) & X33);
        let s2 = ((s1 + (s1 >> 4)) & X0F).wrapping_mul(X01);
        let p0 = (le8(s2, c * X01) >> 7).wrapping_mul(X01);
        let p1 = (p0 >> 53) & !0x7;
        let p2 = p1 as u32;
        let p3 = (s2 << 8).wrapping_shr(p2);
        let p4 = c - (p3 & 0xFF);
        let p5 = lt8(0x0, ((x.wrapping_shr(p2) & 0xFF) * X01) & X8X);
        let s3 = (p5 >> 0x7).wrapping_mul(X01);
        let p6 = (le8(s3, p4 * X01) >> 7).wrapping_mul(X01) >> 56;
        let i = p1 + p6;
        assert!(i < 64);
        i
    }
}

impl Select1 for u128 {
    fn select1(&self, mut c: u64) -> u64 {
        assert!(c < self.count1());

        let lo = *self as u64;
        let hi = (*self >> 64) as u64;
        if c < lo.count1() {
            return lo.select1(c);
        }
        c -= lo.count1();
        hi.select1(c) + 64
    }
}

macro_rules! impl_Select1 {
    ( $( $ty:ty ),* ) => ($(
        impl Select1 for $ty {
            #[inline]
            fn select1(&self, c: u64) -> u64 {
                assert!(c < self.count1());
                (*self as u64).select1(c)
            }
        }
    )*)
}
impl_Select1!(u8, u16, u32, usize);

macro_rules! impl_Select0 {
    ($($ty:ty),*) => ($(
        impl Select0 for $ty {
            #[inline]
            fn select0(&self, c: u64) -> u64 {
                assert!(c < self.count0());
                (!self).select1(c)
            }
        }
    )*)
}
impl_Select0!(u8, u16, u32, u64, u128, usize);

impl<T: Capacity + Select1> Select1 for [T] {
    fn select1(&self, mut c: u64) -> u64 {
        let iter = self.iter().enumerate().filter(|&(_, v)| v.count1() != 0);
        for (i, w) in iter {
            let ones = w.count1();
            if c < ones {
                let index: u64 = word::cast(i);
                return index * T::CAPACITY + w.select1(c);
            }
            c -= ones;
        }

        panic!("{}", bit_set::OUT_OF_BOUNDS);
    }
}

impl<T: Capacity + Select0> Select0 for [T] {
    fn select0(&self, mut c: u64) -> u64 {
        let iter = self.iter().enumerate().filter(|&(_, v)| v.count0() != 0);
        for (i, w) in iter {
            let zeros = w.count0();
            if c < zeros {
                let index: u64 = word::cast(i);
                return index * T::CAPACITY + w.select0(c);
            }
            c -= zeros;
        }

        panic!("{}", bit_set::OUT_OF_BOUNDS);
    }
}
