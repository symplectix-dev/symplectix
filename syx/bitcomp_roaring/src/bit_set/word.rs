#![allow(dead_code)]
use std::borrow::Cow;
use std::{
    iter,
    ops,
    slice,
};

use crate::bit_set;
use crate::bit_set::ops::*;
use crate::bit_set::{
    BoxWords,
    CowWords,
    Words,
    private,
};

/// Trait for an unsigned int; seen as a bit vector with fixed size
/// or an element of bit vector.
///
/// This trait is public but sealed.
pub trait Word:
    'static
    + Copy
    + Eq
    + Ord
    + Default
    + ops::Add<Output = Self>
    + ops::AddAssign
    + ops::Sub<Output = Self>
    + ops::SubAssign
    + ops::Mul<Output = Self>
    + ops::MulAssign
    + ops::Div<Output = Self>
    + ops::DivAssign
    + ops::Rem<Output = Self>
    + ops::RemAssign
    + ops::Shl<Output = Self>
    + ops::ShlAssign
    + ops::Shr<Output = Self>
    + ops::ShrAssign
    + ops::BitAnd<Output = Self>
    + ops::BitAndAssign
    + ops::BitOr<Output = Self>
    + ops::BitOrAssign
    + ops::BitXor<Output = Self>
    + ops::BitXorAssign
    + ops::Not<Output = Self>
    + iter::Sum
    + CastTo<u8>
    + CastTo<u16>
    + CastTo<u32>
    + CastTo<u64>
    + CastTo<u128>
    + CastTo<usize>
    + Capacity
    + Access
    + Count
    + Rank
    + Select0
    + Select1
    + Insert
    + Remove
    + private::Sealed
{
    const ZERO: Self;

    fn bit(i: Self) -> Self;

    fn mask(i: Self) -> Self;

    /// Search the smallest index in range at which f(i) is true,
    /// assuming that f(i) == true implies f(i+1) == true.
    fn search(end: Self, func: impl Fn(Self) -> bool) -> Self;
}

/// Lossless cast that never fail.
pub trait CastAs<T>: private::Sealed {
    fn cast_as(self) -> T;
}

/// Lossless cast that may fail.
pub trait CastTo<T>: private::Sealed {
    fn cast_to(self) -> Option<T>;
}

/// Short for `cast_to().unwrap()`
pub fn cast<U: CastTo<T>, T>(u: U) -> T {
    u.cast_to().unwrap()
}

// /// Short for `cast_to().unwrap_or_else(|| panic!(msg))`
// pub fn cast_expect<U: CastTo<T>, T>(u: U, msg: &'static str) -> T {
//     u.cast_to().unwrap_or_else(|| panic!(msg))
// }

impl<T: Word> CastAs<T> for T {
    fn cast_as(self) -> T {
        self
    }
}

impl<T: Word, A: CastAs<T>> CastTo<T> for A {
    fn cast_to(self) -> Option<T> {
        Some(self.cast_as())
    }
}

macro_rules! impl_Word {
    ($($ty:ty),*) => ($(
        impl Word for $ty {
            const ZERO: Self = 0;

            fn bit(i: Self) -> Self {
                1 << i
            }

            fn mask(i: Self) -> Self {
                (1 << i) - 1
            }

            fn search(end: $ty, func: impl Fn($ty) -> bool) -> $ty {
                let mut i = Self::ZERO;
                let mut j = end;
                while i < j {
                    let h = i + (j - i) / 2;
                    if func(h) {
                        j = h; // f(j) == true
                    } else {
                        i = h + 1; // f(i-1) == false
                    }
                }
                i // f(i-1) == false && f(i) (= f(j)) == true
            }
        }
    )*)
}
impl_Word!(u8, u16, u32, u64, u128, usize);

macro_rules! impl_CastAs {
    ( $small:ty, $( $large:ty ),* ) => ($(
        impl CastAs<$large> for $small {
            #[inline]
            fn cast_as(self) -> $large {
                self as $large
            }
        }
    )*)
}

impl_CastAs!(u8, u16, u32, u64, u128);
impl_CastAs!(u16, u32, u64, u128);
impl_CastAs!(u32, u64, u128);
impl_CastAs!(u64, u128);

#[cfg(target_pointer_width = "32")]
mod cast_as_for_usize {
    use super::*;
    impl_CastAs!(u8, usize);
    impl_CastAs!(u16, usize);
    impl_CastAs!(u32, usize);
    impl_CastAs!(usize, u32, u64, u128);
}

#[cfg(target_pointer_width = "64")]
mod cast_as_for_usize {
    use super::*;
    impl_CastAs!(u8, usize);
    impl_CastAs!(u16, usize);
    impl_CastAs!(u32, usize);
    impl_CastAs!(u64, usize);
    impl_CastAs!(usize, u64, u128);
}

macro_rules! impl_CastTo {
    ( $large:ty, $( $small:ty ),* ) => ($(
        impl CastTo<$small> for $large {
            #[inline]
            fn cast_to(self) -> Option<$small> {
                const MIN: $small = 0;
                const MAX: $small = !MIN;
                if self <= MAX as $large {
                    Some(self as $small)
                } else {
                    None
                }
            }
        }
    )*)
}

impl_CastTo!(u128, u64, u32, u16, u8);
impl_CastTo!(u64, u32, u16, u8);
impl_CastTo!(u32, u16, u8);
impl_CastTo!(u16, u8);

#[cfg(target_pointer_width = "32")]
mod cast_to_for_usize {
    use super::*;
    impl_CastTo!(u64, usize);
    impl_CastTo!(u128, usize);
    impl_CastTo!(usize, u8, u16);
}

#[cfg(target_pointer_width = "64")]
mod cast_to_for_usize {
    use super::*;
    impl_CastTo!(u128, usize);
    impl_CastTo!(usize, u8, u16, u32);
}

impl<T> Default for Words<T> {
    fn default() -> Self {
        Words(None)
    }
}

impl<T> Words<T> {
    pub fn as_ref(&self) -> Option<&T> {
        self.0.as_ref()
    }

    pub fn as_mut(&mut self) -> Option<&mut T> {
        self.0.as_mut()
    }
}

impl<T> Capacity for Words<T> {
    const CAPACITY: u64 = bit_set::SHORT_BIT_MAX;
}

impl<T: Word> BoxWords<T> {
    pub const LEN: usize = (Self::CAPACITY / T::CAPACITY) as usize;
}

impl<T: Word> CowWords<'_, T> {
    pub const LEN: usize = (Self::CAPACITY / T::CAPACITY) as usize;
}

impl<T: Word> From<CowWords<'_, T>> for BoxWords<T> {
    fn from(Words(block): CowWords<'_, T>) -> Self {
        Words(block.map(|cow| cow.into_owned().into_boxed_slice()))
    }
}
impl<T: Word> From<BoxWords<T>> for CowWords<'_, T> {
    fn from(Words(block): BoxWords<T>) -> Self {
        Words(block.map(|arr| Cow::Owned(arr.into_vec())))
    }
}

impl<'a, T: Word> From<&'a BoxWords<T>> for CowWords<'a, T> {
    fn from(block: &'a BoxWords<T>) -> Self {
        Words(block.as_ref().map(|ws| Cow::Borrowed(&ws[..])))
    }
}
impl<'a, T: Word> From<&'a [T]> for CowWords<'a, T> {
    fn from(slice: &'a [T]) -> Self {
        Words(Some(Cow::Borrowed(&slice[0..Self::LEN])))
    }
}

impl<T: Word> From<Vec<T>> for BoxWords<T> {
    fn from(mut vec: Vec<T>) -> Self {
        vec.resize(Self::LEN, T::ZERO);
        Words(Some(vec.into_boxed_slice()))
    }
}
impl<T: Word> From<Vec<T>> for CowWords<'_, T> {
    fn from(mut vec: Vec<T>) -> Self {
        vec.resize(Self::LEN, T::ZERO);
        Words(Some(Cow::Owned(vec)))
    }
}

impl<T: Word> BoxWords<T> {
    /// Return an empty Words.
    pub fn empty() -> Self {
        Words(None)
    }

    /// Constructs a new instance with each element initialized to value.
    pub fn splat(value: T) -> Self {
        Words(Some(vec![value; Self::LEN].into_boxed_slice()))
    }

    pub fn len(&self) -> usize {
        self.as_cow().len()
    }

    pub fn is_empty(&self) -> bool {
        self.as_cow().is_empty()
    }

    pub fn iter<'r>(&'r self) -> impl Iterator<Item = T> + 'r {
        self.into_iter()
    }
}

impl<T: Word> CowWords<'_, T> {
    /// Return an empty Words.
    pub fn empty() -> Self {
        Words(None)
    }

    /// Constructs a new instance with each element initialized to value.
    pub fn splat(value: T) -> Self {
        Words(Some(Cow::Owned(vec![value; Self::LEN])))
    }

    pub fn len(&self) -> usize {
        match self.as_ref() {
            None => 0,
            Some(vec) => vec.len(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn iter<'r>(&'r self) -> impl Iterator<Item = T> + 'r {
        self.into_iter()
    }
}

impl<T: Word> BoxWords<T> {
    pub(crate) fn as_cow(&self) -> CowWords<'_, T> {
        Words::from(self)
    }

    fn init(&mut self) -> &mut [T] {
        if self.0.is_none() {
            *self = Self::splat(T::ZERO);
        }
        self.0.as_mut().unwrap()
    }
}

impl<'r, T: Word> IntoIterator for &'r BoxWords<T> {
    type Item = T;
    type IntoIter = WordsIter<'r, T>;
    fn into_iter(self) -> Self::IntoIter {
        WordsIter(self.as_ref().map(|b| b.into_iter().cloned()))
    }
}
impl<'r, 'a, T: Word> IntoIterator for &'r CowWords<'a, T> {
    type Item = T;
    type IntoIter = WordsIter<'r, T>;
    fn into_iter(self) -> Self::IntoIter {
        WordsIter(self.as_ref().map(|b| b.iter().cloned()))
    }
}

pub struct WordsIter<'a, T: Word>(Option<iter::Cloned<slice::Iter<'a, T>>>);

impl<'a, T: Word> Iterator for WordsIter<'a, T> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.as_mut().and_then(|i| i.next())
    }
}

impl<T: Word> bit_set::ops::Access for BoxWords<T> {
    fn size(&self) -> u64 {
        Self::CAPACITY
    }
    fn access(&self, i: u64) -> bool {
        assert!(i < Self::CAPACITY, "{}", bit_set::OUT_OF_BOUNDS);
        self.as_cow().access(i)
    }
}
impl<T: Word> bit_set::ops::Access for CowWords<'_, T> {
    fn size(&self) -> u64 {
        Self::CAPACITY
    }
    fn access(&self, i: u64) -> bool {
        assert!(i < Self::CAPACITY, "{}", bit_set::OUT_OF_BOUNDS);
        self.as_ref().is_some_and(|cow| cow.access(i))
    }
}

impl<T: Word> bit_set::ops::Count for BoxWords<T> {
    fn count1(&self) -> u64 {
        self.as_cow().count1()
    }
}
impl<T: Word> bit_set::ops::Count for CowWords<'_, T> {
    fn count1(&self) -> u64 {
        self.as_ref().map_or(0, |cow| cow.count1())
    }
}

impl<T: Word> bit_set::ops::Rank for BoxWords<T> {
    fn rank1(&self, i: u64) -> u64 {
        self.as_cow().rank1(i)
    }
}
impl<T: Word> bit_set::ops::Rank for CowWords<'_, T> {
    fn rank1(&self, i: u64) -> u64 {
        assert!(i <= Self::CAPACITY, "{}", bit_set::OUT_OF_BOUNDS);
        self.as_ref().map_or(0, |cow| cow.rank1(i))
    }
}

impl<T: Word> bit_set::ops::Select1 for BoxWords<T> {
    fn select1(&self, n: u64) -> u64 {
        self.as_cow().select1(n)
    }
}
impl<T: Word> bit_set::ops::Select0 for BoxWords<T> {
    fn select0(&self, n: u64) -> u64 {
        self.as_cow().select0(n)
    }
}
impl<T: Word> bit_set::ops::Select1 for CowWords<'_, T> {
    fn select1(&self, n: u64) -> u64 {
        assert!(n < self.count1());
        self.as_ref().expect("should not happen").select1(n)
    }
}
impl<T: Word> bit_set::ops::Select0 for CowWords<'_, T> {
    fn select0(&self, n: u64) -> u64 {
        assert!(n < self.count0());
        self.as_ref().map_or(n, |bv| bv.select0(n))
    }
}

impl<T: Word> bit_set::ops::Insert for BoxWords<T> {
    fn insert(&mut self, i: u64) -> bool {
        assert!(i < Self::CAPACITY);
        self.init().insert(i)
    }
}
impl<T: Word> bit_set::ops::Insert for CowWords<'_, T> {
    fn insert(&mut self, i: u64) -> bool {
        assert!(i < Self::CAPACITY);
        if self.0.is_none() {
            *self = Self::splat(T::ZERO);
        }
        let bv = self.as_mut().unwrap();
        <[T] as bit_set::ops::Insert>::insert(bv.to_mut(), i)
    }
}

impl<T: Word> bit_set::ops::Remove for BoxWords<T> {
    fn remove(&mut self, i: u64) -> bool {
        assert!(i < Self::CAPACITY);
        if let Some(bv) = self.as_mut() { bv.remove(i) } else { false }
    }
}
impl<T: Word> bit_set::ops::Remove for CowWords<'_, T> {
    fn remove(&mut self, i: u64) -> bool {
        assert!(i < Self::CAPACITY);
        if let Some(bv) = self.as_mut() {
            <[T] as bit_set::ops::Remove>::remove(bv.to_mut(), i)
        } else {
            false
        }
    }
}

impl<'a, T: Word> ops::BitAnd<CowWords<'a, T>> for CowWords<'a, T> {
    type Output = CowWords<'a, T>;
    fn bitand(self, that: CowWords<'a, T>) -> Self::Output {
        Words(match (self.0, that.0) {
            (None, _) | (_, None) => None,
            (Some(ref buf), _) | (_, Some(ref buf)) if buf.is_empty() => None,
            (Some(mut lhs), Some(rhs)) => {
                assert_eq!(lhs.len(), rhs.len());

                let ones = {
                    let zip = lhs.to_mut().iter_mut().zip(rhs.iter());
                    let mut acc = 0;
                    for (x, y) in zip {
                        *x &= *y;
                        acc += x.count1();
                    }
                    acc
                };

                if ones > 0 { Some(lhs) } else { None }
            }
        })
    }
}

impl<'a, T: Word> ops::BitOr<CowWords<'a, T>> for CowWords<'a, T> {
    type Output = CowWords<'a, T>;

    fn bitor(self, that: CowWords<'a, T>) -> Self::Output {
        Words(match (self.0, that.0) {
            (None, None) => None,
            (Some(buf), None) | (None, Some(buf)) => Some(buf),
            (Some(mut lhs), Some(rhs)) => {
                assert_eq!(lhs.len(), rhs.len());
                {
                    let zip = lhs.to_mut().iter_mut().zip(rhs.iter());
                    for (x, y) in zip {
                        *x |= *y;
                    }
                }
                Some(lhs)
            }
        })
    }
}

impl<'a, T: Word> ops::BitXor<CowWords<'a, T>> for CowWords<'a, T> {
    type Output = CowWords<'a, T>;

    fn bitxor(self, that: CowWords<'a, T>) -> Self::Output {
        Words(match (self.0, that.0) {
            (None, None) => None,
            (Some(buf), None) | (None, Some(buf)) => Some(buf),
            (Some(mut lhs), Some(rhs)) => {
                assert_eq!(lhs.len(), rhs.len());
                {
                    let lhs_vec = lhs.to_mut();
                    let zip = lhs_vec.iter_mut().zip(rhs.iter());
                    for (x, y) in zip {
                        *x ^= *y;
                    }
                };
                Some(lhs)
            }
        })
    }
}

impl<'a, T: Word> ops::Not for CowWords<'a, T> {
    type Output = CowWords<'a, T>;
    fn not(self) -> Self::Output {
        Words(match self.0 {
            Some(mut buf) => {
                let ones = {
                    let vec = buf.to_mut();
                    vec.resize(BoxWords::<T>::LEN, T::ZERO);
                    let mut acc = 0;
                    #[allow(clippy::needless_range_loop)]
                    for i in 0..vec.len() {
                        vec[i] = !vec[i];
                        acc += vec[i].count1();
                    }
                    acc
                };
                if ones > 0 { Some(buf) } else { None }
            }
            None => Some(Cow::Owned(vec![!T::ZERO; BoxWords::<T>::LEN])),
        })
    }
}
