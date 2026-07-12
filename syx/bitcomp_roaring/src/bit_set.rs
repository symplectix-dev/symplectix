#![allow(dead_code, missing_docs)]

// References
// - Compact Data Structures: A Practical Approach
// - Fast, Small, Simple Rank/Select on Bitmaps
// - Space-Efficient, High-Performance Rank & Select Structures on Uncompressed Bit Sequences

#[cfg(test)]
mod tests;

mod mask;
mod ops;
mod raw_vec;
mod word;

mod private {
    use std::ops;
    pub trait Sealed {}

    macro_rules! impl_Sealed {
        ( $( $Type:ty ),* ) => {
            $( impl Sealed for $Type {} )*
        }
    }
    impl_Sealed!(
        u8,
        u16,
        u32,
        u64,
        u128,
        usize,
        ops::Range<u64>,
        ops::RangeTo<u64>,
        ops::RangeFrom<u64>,
        ops::RangeFull
    );
}

use std::borrow::Cow;

pub use self::mask::{
    And,
    Fold,
    // Not,
    Or,
    Xor,
    and,
    // not,
    or,
    xor,
};
use self::ops::*;
use self::word::Word;

const LARGE_BIT_MAX: u64 = 1 << 63;
const SHORT_BIT_MAX: u64 = 1 << 16;

// Panic message.
static OUT_OF_BOUNDS: &str = "index out of bounds";

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BitSet<T>(RawVec<Entry<Box<[T]>>>);

// Core component of BitSet.
#[derive(Clone, Debug, PartialEq, Eq)]
struct RawVec<B> {
    ones: u64,
    bits: Vec<B>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Words<B>(Option<B>);

type BoxWords<T> = Words<Box<[T]>>;
type CowWords<'a, T> = Words<Cow<'a, [T]>>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry<B> {
    // index  = offset / block::capacity
    // offset = index  * block::capacity
    index: usize,
    block: Words<B>,
}

pub trait Index<T>: private::Sealed {
    type Output;
    fn get(&self, target: T) -> Self::Output;
}

/// All types that implements `Access` can be indexed by `u64`.
impl<'a, T: ops::Access> Index<&'a T> for u64 {
    type Output = bool;
    #[inline]
    fn get(&self, bv: &'a T) -> bool {
        bv.access(*self)
    }
}

// Bits is a range of bits in `T`.
// #[derive(Clone, Debug, PartialEq, Eq)]
// pub struct Bits<'a, T: 'a>(mask::And<&'a T, Run>);

// #[derive(Copy, Clone, Debug, PartialEq, Eq)]
// struct Run {
//     index: u64,
//     len:   u64,
// }

/// Return an iterator that will iterate over all non-zero bit's index.
///
/// # Examples
///
/// ```rust
/// use bitcomp_roaring::{
///     BitSet,
///     bit_set,
/// };
/// let bits = vec![0, 1 << 30, 1 << 40, 1 << 50, 1 << 51];
/// let bv = BitSet::<u64>::from(&bits[..]);
/// let all = bit_set::iterate(&bv).collect::<Vec<u64>>();
/// assert_eq!(all, bits);
/// ```
pub fn iterate<'a, T: Word>(
    it: impl IntoIterator<Item = Entry<Cow<'a, [T]>>> + 'a,
) -> impl Iterator<Item = u64> + 'a {
    let it = it.into_iter();
    it.flat_map(|entry| {
        let offset = entry.offset();
        let mut bits = Vec::with_capacity(1 << 12);
        for (i, word) in entry.block.iter().enumerate() {
            let offset = i as u64 * T::CAPACITY + offset;
            for j in 0..word.size() {
                if word.access(j) {
                    bits.push(offset + j)
                }
            }
        }
        bits
    })
}

impl<T> Entry<T> {
    fn new<U: Into<Words<T>>>(index: usize, data: U) -> Self {
        let block = data.into();
        Self { index, block }
    }

    // bit offset of this entry.
    fn offset(&self) -> u64 {
        self.index as u64 * Words::<T>::CAPACITY
    }
}

/// Convert index into a bit block index and offset.
#[inline]
fn address(i: u64, block_size: u64) -> (usize, u64) {
    let index = word::cast(i / block_size);
    let offset = i % block_size;
    (index, offset)
}

macro_rules! impl_mask {
    ( $([ $($constraints:tt)*] for $Type:ty ;)+ ) => {
        $(
            impl<$($constraints)*> $Type {
                pub fn and<Rhs>(self, rhs: Rhs) -> And<Self, Rhs> {
                    and(self, rhs)
                }
                pub fn or<Rhs>(self, rhs: Rhs) -> Or<Self, Rhs> {
                    or(self, rhs)
                }
                pub fn xor<Rhs>(self, rhs: Rhs) -> Xor<Self, Rhs> {
                    xor(self, rhs)
                }
                // pub fn not(self) -> Not<Self> {
                //     not(self)
                // }
            }
        )+
    }
}
impl_mask!(
    // ['a, T] for Bits<'a, T>;
    [L, R]  for And<L, R>;
    [L, R]  for Or<L, R>;
    [L, R]  for Xor<L, R>;
    // [T]     for Not<T>;
);

impl<T: Word> BitSet<T> {
    pub fn new() -> Self {
        Default::default()
    }

    #[inline]
    pub fn shrink_to_fit(&mut self) {
        self.0.shrink_to_fit()
    }

    /// # Examples
    ///
    /// ```
    /// use bitcomp_roaring::BitSet;
    /// let mut bv = BitSet::<u64>::from(vec![1, 1000, 1000000]);
    ///
    /// assert!(bv.get(1) && bv.get(1000) && bv.get(1000000));
    /// ```
    #[inline]
    pub fn get<'a, Ix>(&'a self, index: Ix) -> Ix::Output
    where
        Ix: Index<&'a Self>,
    {
        index.get(self)
    }

    /// # Examples
    ///
    /// ```
    /// use bitcomp_roaring::BitSet;
    /// let mut bv = BitSet::<u64>::new();
    /// assert!(!bv.insert(1));
    /// assert!(bv.remove(1));
    /// ```
    #[inline]
    pub fn insert(&mut self, i: u64) -> bool {
        self.0.insert(i)
    }

    /// # Examples
    ///
    /// ```
    /// use bitcomp_roaring::BitSet;
    /// let mut bv = BitSet::<u64>::new();
    /// assert!(!bv.insert(1));
    /// assert!(bv.remove(1));
    /// ```
    #[inline]
    pub fn remove(&mut self, i: u64) -> bool {
        self.0.remove(i)
    }

    #[inline]
    pub fn access(&self, i: u64) -> bool {
        self.0.access(i)
    }

    #[inline]
    pub fn size(&self) -> u64 {
        self.0.size()
    }

    #[inline]
    pub fn count1(&self) -> u64 {
        self.0.count1()
    }

    #[inline]
    pub fn count0(&self) -> u64 {
        self.0.count0()
    }

    #[inline]
    pub fn rank1(&self, i: u64) -> u64 {
        self.0.rank1(i)
    }

    #[inline]
    pub fn rank0(&self, i: u64) -> u64 {
        self.0.rank1(i)
    }

    #[inline]
    pub fn excess1(&self, i: u64) -> u64 {
        self.0.excess1(i)
    }

    #[inline]
    pub fn excess0(&self, i: u64) -> u64 {
        self.0.excess0(i)
    }

    #[inline]
    pub fn select1(&self, i: u64) -> u64 {
        self.0.select1(i)
    }

    #[inline]
    pub fn select0(&self, i: u64) -> u64 {
        self.0.select0(i)
    }

    pub fn and<Rhs>(&self, rhs: Rhs) -> And<&Self, Rhs> {
        and(self, rhs)
    }
    pub fn or<Rhs>(&self, rhs: Rhs) -> Or<&Self, Rhs> {
        or(self, rhs)
    }
    pub fn xor<Rhs>(&self, rhs: Rhs) -> Xor<&Self, Rhs> {
        xor(self, rhs)
    }
    // pub fn not(&self) -> Not<&Self> {
    //     not(self)
    // }
}

impl<T: Word> Capacity for BitSet<T> {
    const CAPACITY: u64 = LARGE_BIT_MAX;
}

impl<T: Word> Access for BitSet<T> {
    #[inline]
    fn size(&self) -> u64 {
        Self::CAPACITY
    }
    #[inline]
    fn access(&self, i: u64) -> bool {
        assert!(i < Self::CAPACITY);
        self.0.access(i)
    }
}

impl<T: Word> Count for BitSet<T> {
    #[inline]
    fn count1(&self) -> u64 {
        self.0.count1()
    }
}

impl<T: Word> Rank for BitSet<T> {
    #[inline]
    fn rank1(&self, i: u64) -> u64 {
        self.0.rank1(i)
    }
}

impl<T: Word> Select1 for BitSet<T> {
    #[inline]
    fn select1(&self, c: u64) -> u64 {
        self.0.select1(c)
    }
}
impl<T: Word> Select0 for BitSet<T> {
    fn select0(&self, c: u64) -> u64 {
        self.0.select0(c)
    }
}

impl<T: Word> Insert for BitSet<T> {
    #[inline]
    fn insert(&mut self, i: u64) -> bool {
        self.0.insert(i)
    }
}
impl<T: Word> Remove for BitSet<T> {
    #[inline]
    fn remove(&mut self, i: u64) -> bool {
        self.0.remove(i)
    }
}

impl<T: Word> From<Vec<u64>> for BitSet<T> {
    fn from(vec: Vec<u64>) -> Self {
        let mut bits = <BitSet<T> as Default>::default();
        for bit in vec.into_iter() {
            bits.insert(bit);
        }
        bits
    }
}
impl<T: Word> From<&[u64]> for BitSet<T> {
    fn from(vec: &[u64]) -> Self {
        let mut bits = <BitSet<T> as Default>::default();
        for &bit in vec.iter() {
            bits.insert(bit);
        }
        bits
    }
}

impl<'a, T: Word> IntoIterator for &'a BitSet<T> {
    type Item = Entry<Cow<'a, [T]>>;
    type IntoIter = raw_vec::Entries<'a, Box<[T]>>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a, T: Word> std::iter::FromIterator<Entry<Cow<'a, [T]>>> for BitSet<T> {
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = Entry<Cow<'a, [T]>>>,
    {
        let inner = iter.into_iter().collect::<RawVec<Entry<Box<[T]>>>>();
        BitSet(inner)
    }
}
