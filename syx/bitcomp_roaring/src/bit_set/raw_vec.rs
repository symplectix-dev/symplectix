#![allow(dead_code)]
use std::borrow::Cow;
use std::iter;

use crate::bit_set::ops::*;
use crate::bit_set::{
    self,
    Entry,
    RawVec,
    Word,
    Words,
};

impl<B> Default for RawVec<B> {
    fn default() -> Self {
        Self::new()
    }
}

impl<B> RawVec<B> {
    pub fn new() -> Self {
        RawVec { ones: 0, bits: Vec::new() }
    }

    fn new_unchecked(ones: u64, bits: Vec<B>) -> Self {
        RawVec { ones, bits }
    }

    /// Shrink an internal vector.
    pub fn shrink_to_fit(&mut self) {
        self.bits.shrink_to_fit()
    }

    pub fn and<Rhs>(&self, rhs: Rhs) -> bit_set::And<&Self, Rhs> {
        bit_set::and(self, rhs)
    }

    pub fn or<Rhs>(&self, rhs: Rhs) -> bit_set::Or<&Self, Rhs> {
        bit_set::or(self, rhs)
    }

    pub fn xor<Rhs>(&self, rhs: Rhs) -> bit_set::Xor<&Self, Rhs> {
        bit_set::xor(self, rhs)
    }

    // pub fn not(&self) -> bit_set::Not<&Self> {
    //     bit_set::not(self)
    // }
}

impl<B: Capacity> RawVec<B> {
    fn address(&self, index: u64) -> (usize, u64) {
        bit_set::address(index, B::CAPACITY)
    }
}

impl<T> RawVec<Entry<T>> {
    fn address(&self, index: u64) -> (usize, u64) {
        bit_set::address(index, Words::<T>::CAPACITY)
    }

    fn search_entry(&self, index: usize) -> Result<usize, usize> {
        self.bits.binary_search_by_key(&index, |page| page.index)
    }

    fn entry(&self, index: usize) -> Option<&Entry<T>> {
        self.search_entry(index).ok().map(|i| &self.bits[i])
    }
}

impl<T> Capacity for RawVec<T> {
    const CAPACITY: u64 = bit_set::LARGE_BIT_MAX;
}

impl<B: Capacity + Access> Access for RawVec<B> {
    fn size(&self) -> u64 {
        Self::CAPACITY
    }
    fn access(&self, i: u64) -> bool {
        assert!(i < Self::CAPACITY);
        self.bits.access(i)
    }
}

impl<B> Count for RawVec<B>
where
    Self: Access,
{
    fn count1(&self) -> u64 {
        debug_assert!(self.ones <= Self::CAPACITY);
        self.ones
    }
}

impl<B: Capacity + Rank> Rank for RawVec<B> {
    fn rank1(&self, i: u64) -> u64 {
        assert!(i <= Self::CAPACITY, "{}", bit_set::OUT_OF_BOUNDS);
        let (q, r) = self.address(i);
        let c = self.bits.iter().take(q).map(|b| b.count1()).sum::<u64>();
        let r = self.bits.get(q).map_or(0, |b| b.rank1(r));
        c + r
    }
}

impl<B: Capacity + Select1> Select1 for RawVec<B> {
    fn select1(&self, c: u64) -> u64 {
        assert!(c < self.count1());
        self.bits.select1(c)
    }
}
impl<B: Capacity + Rank> Select0 for RawVec<B> {
    fn select0(&self, n: u64) -> u64 {
        assert!(n < self.count0());
        self.search0(n)
    }
}

impl<B: Word> Insert for RawVec<B> {
    fn insert(&mut self, i: u64) -> bool {
        assert!(i < Self::CAPACITY, "{}", bit_set::OUT_OF_BOUNDS);
        match self.address(i) {
            (index, offset) if index >= self.bits.len() => {
                self.bits.resize(index + 1, B::ZERO);
                self.bits[index].insert(offset);
                self.ones += 1;
                false
            }
            (index, offset) => {
                if !self.bits[index].insert(offset) {
                    self.ones += 1;
                    false
                } else {
                    true
                }
            }
        }
    }
}

impl<B: Word> Insert for RawVec<Words<Box<[B]>>> {
    fn insert(&mut self, i: u64) -> bool {
        assert!(i < Self::CAPACITY, "{}", bit_set::OUT_OF_BOUNDS);
        match self.address(i) {
            (index, offset) if index >= self.bits.len() => {
                self.bits.resize(index + 1, Words::default());
                self.bits[index].insert(offset);
                self.ones += 1;
                false
            }
            (index, offset) => {
                if !self.bits[index].insert(offset) {
                    self.ones += 1;
                    false
                } else {
                    true
                }
            }
        }
    }
}

impl<'a, B: Word> Insert for RawVec<Words<Cow<'a, [B]>>> {
    fn insert(&mut self, i: u64) -> bool {
        assert!(i < Self::CAPACITY, "{}", bit_set::OUT_OF_BOUNDS);
        match self.address(i) {
            (index, offset) if index >= self.bits.len() => {
                self.bits.resize(index + 1, Words::default());
                self.bits[index].insert(offset);
                self.ones += 1;
                false
            }
            (index, offset) => {
                if !self.bits[index].insert(offset) {
                    self.ones += 1;
                    false
                } else {
                    true
                }
            }
        }
    }
}

impl<B: Capacity + Remove> Remove for RawVec<B> {
    fn remove(&mut self, i: u64) -> bool {
        assert!(i < Self::CAPACITY, "{}", bit_set::OUT_OF_BOUNDS);
        match self.address(i) {
            (index, _) if index >= self.bits.len() => false,
            (index, offset) => {
                if self.bits[index].remove(offset) {
                    self.ones -= 1;
                    true
                } else {
                    false
                }
            }
        }
    }
}

macro_rules! impl_for_entry {
    ( $([ $($constraints:tt)*] for $Type:ty ;)+ ) => {
        $(
            impl<$($constraints)*> Access for $Type {
                fn size(&self) -> u64 {
                    Self::CAPACITY
                }
                fn access(&self, i: u64) -> bool {
                    assert!(i < Self::CAPACITY);
                    let (index, offset) = self.address(i);
                    self.entry(index).map_or(false, |e| e.block.access(offset))
                }
            }

            impl<$($constraints)*> Rank for $Type {
                fn rank1(&self, i: u64) -> u64 {
                    assert!(i <= Self::CAPACITY, "{}", bit_set::OUT_OF_BOUNDS);
                    let (index, offset) = self.address(i);
                    let mut rank = 0;
                    for page in &self.bits {
                        if page.index < index {
                            rank += page.block.count1();
                        } else if page.index == index {
                            rank += page.block.rank1(offset);
                            break;
                        } else if page.index > index {
                            break;
                        }
                    }
                    rank
                }
            }

            impl<$($constraints)*> Select1 for $Type {
                fn select1(&self, mut n: u64) -> u64 {
                    assert!(n < self.count1());
                    for entry in &self.bits {
                        let count = entry.block.count1();
                        if n < count {
                            let index = bit_set::word::cast::<usize, u64>(entry.index);
                            return index * Words::<T>::CAPACITY + entry.block.select1(n);
                        }
                        n -= count;
                    }
                    unreachable!();
                }
            }

            impl<$($constraints)*> Select0 for $Type {
                fn select0(&self, n: u64) -> u64 {
                    assert!(n < self.count0());
                    self.search0(n)
                }
            }

            impl<$($constraints)*> Insert for $Type {
                fn insert(&mut self, i: u64) -> bool {
                    assert!(i < Self::CAPACITY, "{}", bit_set::OUT_OF_BOUNDS);
                    let (index, offset) = self.address(i);
                    match self.search_entry(index) {
                        Ok(p) => {
                            if self.bits[p].block.insert(offset) {
                                true // already enabled
                            } else {
                                self.ones += 1;
                                false
                            }
                        }
                        Err(p) => {
                            let mut entry = Entry::new(index, bit_set::BoxWords::splat(T::ZERO));
                            entry.block.insert(offset);
                            self.bits.insert(p, entry);
                            self.ones += 1;
                            false
                        }
                    }
                }
            }

            impl<$($constraints)*> Remove for $Type {
                fn remove(&mut self, i: u64) -> bool {
                    assert!(i < Self::CAPACITY, "{}", bit_set::OUT_OF_BOUNDS);
                    let (index, offset) = self.address(i);
                    match self.search_entry(index) {
                        Ok(index) => {
                            if self.bits[index].block.remove(offset) {
                                self.ones -= 1;
                                if self.bits[index].block.count1() == 0 {
                                    self.bits.remove(index);
                                }
                                true
                            } else {
                                false
                            }
                        }
                        Err(_) => false,
                    }
                }
            }
        )+
    }
}
impl_for_entry!(
    [    T: Word] for RawVec<Entry<Box<[T]>>>;
    ['c, T: Word] for RawVec<Entry<Cow<'c, [T]>>>;
);

macro_rules! impl_for_all {
    ( $([ $($constraints:tt)*] for $Type:ty ;)+ ) => {
        $(
            impl<$($constraints)*> $Type {
                #[inline]
                pub fn get<'a, Ix>(&'a self, index: Ix) -> Ix::Output
                where
                    Ix: bit_set::Index<&'a Self>,
                {
                    index.get(self)
                }
            }

            impl<$($constraints)*> From<Vec<u64>> for $Type {
                fn from(vec: Vec<u64>) -> Self {
                    let mut bits = <$Type as Default>::default();
                    for bit in vec.into_iter() {
                        bits.insert(bit);
                    }
                    bits
                }
            }
            impl<$($constraints)*> From<&[u64]> for $Type {
                fn from(vec: &[u64]) -> Self {
                    let mut bits = <$Type as Default>::default();
                    for &bit in vec.iter() {
                        bits.insert(bit);
                    }
                    bits
                }
            }
        )+
    }
}
impl_for_all!(
    [    T: Word] for RawVec<T>;
    [    T: Word] for RawVec<Words<Box<[T]>>>;
    ['c, T: Word] for RawVec<Words<Cow<'c, [T]>>>;
    [    T: Word] for RawVec<Entry<Box<[T]>>>;
    ['c, T: Word] for RawVec<Entry<Cow<'c, [T]>>>;
);

pub struct Chunks<'a, T: Word> {
    iter: std::iter::Enumerate<std::slice::Chunks<'a, T>>,
}
impl<'a, T: Word> Iterator for Chunks<'a, T> {
    type Item = Entry<Cow<'a, [T]>>;
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|(i, words)| Entry::new(i, words))
    }
}

pub struct Pager<'a, B> {
    iter: std::iter::Enumerate<std::slice::Iter<'a, Words<B>>>,
}

impl<'a, T: Word> Iterator for Pager<'a, Box<[T]>> {
    type Item = Entry<Cow<'a, [T]>>;
    fn next(&mut self) -> Option<Self::Item> {
        // while let Some((i, words)) = self.iter.next() {
        //     if words.is_empty() {
        //         continue;
        //     }
        //     return Some(Entry::new(i, words.as_cow()));
        // }
        for (i, words) in self.iter.by_ref() {
            if words.is_empty() {
                continue;
            }
            return Some(Entry::new(i, words.as_cow()));
        }
        None
    }
}
impl<'a: 'cow, 'cow, T: Word> Iterator for Pager<'a, Cow<'cow, [T]>> {
    type Item = Entry<Cow<'a, [T]>>;
    fn next(&mut self) -> Option<Self::Item> {
        for (i, words) in self.iter.by_ref() {
            if let Some(cow) = words.as_ref() {
                return Some(Entry::new(i, Words::from(cow.as_ref())));
            } else {
                continue;
            }
        }
        None
        // while let Some((i, words)) = self.iter.next() {
        //     if let Some(cow) = words.as_ref() {
        //         return Some(Entry::new(i, Words::from(cow.as_ref())));
        //     } else {
        //         continue;
        //     }
        // }
    }
}

pub struct Entries<'a, T> {
    // all entries should be sorted by index.
    iter: std::slice::Iter<'a, Entry<T>>,
}

impl<'a, T: Word> Iterator for Entries<'a, Box<[T]>> {
    type Item = Entry<Cow<'a, [T]>>;
    fn next(&mut self) -> Option<Self::Item> {
        for entry in self.iter.by_ref() {
            if entry.block.is_empty() {
                continue;
            }
            return Some(Entry::new(entry.index, &entry.block));
        }
        // while let Some(entry) = self.iter.next() {
        //     if entry.block.is_empty() {
        //         continue;
        //     }
        //     return Some(Entry::new(entry.index, &entry.block));
        // }
        None
    }
}
impl<'a: 'cow, 'cow, T: Word> Iterator for Entries<'a, Cow<'cow, [T]>> {
    type Item = Entry<Cow<'cow, [T]>>;
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(entry) = self.iter.next() {
            let index = entry.index;
            return entry.block.as_ref().map(|cow| Entry::new(index, cow.as_ref()));
        }
        None
    }
}

impl<'a, T: Word> IntoIterator for &'a RawVec<T> {
    type Item = Entry<Cow<'a, [T]>>;
    type IntoIter = Chunks<'a, T>;
    fn into_iter(self) -> Self::IntoIter {
        Chunks { iter: self.bits.chunks(bit_set::SHORT_BIT_MAX as usize).enumerate() }
    }
}

impl<'a, T: Word> IntoIterator for &'a RawVec<Words<Box<[T]>>> {
    type Item = Entry<Cow<'a, [T]>>;
    type IntoIter = Pager<'a, Box<[T]>>;
    fn into_iter(self) -> Self::IntoIter {
        Pager { iter: self.bits.iter().enumerate() }
    }
}
impl<'a: 'cow, 'cow, T: Word> IntoIterator for &'a RawVec<Words<Cow<'cow, [T]>>> {
    type Item = Entry<Cow<'cow, [T]>>;
    type IntoIter = Pager<'a, Cow<'cow, [T]>>;
    fn into_iter(self) -> Self::IntoIter {
        Pager { iter: self.bits.iter().enumerate() }
    }
}

impl<'a, T: Word> IntoIterator for &'a RawVec<Entry<Box<[T]>>> {
    type Item = Entry<Cow<'a, [T]>>;
    type IntoIter = Entries<'a, Box<[T]>>;
    fn into_iter(self) -> Self::IntoIter {
        Entries { iter: self.bits.iter() }
    }
}
impl<'a: 'cow, 'cow, T: Word> IntoIterator for &'a RawVec<Entry<Cow<'cow, [T]>>> {
    type Item = Entry<Cow<'cow, [T]>>;
    type IntoIter = Entries<'a, Cow<'cow, [T]>>;
    fn into_iter(self) -> Self::IntoIter {
        Entries { iter: self.bits.iter() }
    }
}

impl<'a, T: Word> iter::FromIterator<Entry<Cow<'a, [T]>>> for RawVec<Words<Box<[T]>>> {
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = Entry<Cow<'a, [T]>>>,
    {
        let mut ones = 0;
        let mut bits = Vec::with_capacity(1 << 10);

        for p in iter {
            let index = p.index;
            let words = p.block; // Words<Cow<[T]>>

            let count = words.count1();
            if count == 0 {
                continue;
            }
            ones += count;
            bits.resize(index + 1, Words::<Box<[T]>>::empty());
            bits.insert(index, words.into());
        }
        bits.shrink_to_fit();
        RawVec::new_unchecked(ones, bits)
    }
}

impl<'a, T: Word> iter::FromIterator<Entry<Cow<'a, [T]>>> for RawVec<Entry<Box<[T]>>> {
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = Entry<Cow<'a, [T]>>>,
    {
        let mut ones = 0;
        let mut bits = Vec::with_capacity(1 << 10);

        for p in iter {
            let index = p.index;
            let block = p.block;

            let count = block.count1();
            if count == 0 {
                continue;
            }
            ones += count;
            bits.push(Entry::new(index, block));
        }
        bits.shrink_to_fit();
        RawVec::new_unchecked(ones, bits)
    }
}
