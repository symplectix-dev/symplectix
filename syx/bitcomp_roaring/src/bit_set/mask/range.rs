// use std::borrow::Cow;
// use std::marker::PhantomData;
// use std::{
//     iter,
//     ops,
// };

// use super::{
//     And,
//     AndEntries,
// };
// use crate::bit_set;
// use crate::bit_set::ops::*;
// use crate::bit_set::{
//     Index,
//     Word,
// };

// impl<'r, 'a, T> Index<&'r Bits<'a, T>> for ops::RangeFull {
//     type Output = Bits<'a, T>;
//     fn get(&self, range: &'r Bits<'a, T>) -> Self::Output {
//         let lhs = range.0.lhs;
//         let rhs = range.0.rhs;
//         Bits(And { lhs, rhs })
//     }
// }

// impl<'r, 'a, T: bit_set::ops::Capacity> Index<&'r Bits<'a, T>> for ops::Range<u64> {
//     type Output = Bits<'a, T>;
//     fn get(&self, range: &'r Bits<'a, T>) -> Self::Output {
//         let start = self.start;
//         let end = self.end;
//         assert!(start <= end && start < T::CAPACITY && end <= T::CAPACITY);
//         let this = range.0.rhs;
//         let that = Run::new(start, end - start);
//         Bits::with_run(range.0.lhs, this.intersect(&that))
//     }
// }

// impl<'r, 'a, T: bit_set::ops::Capacity> Index<&'r Bits<'a, T>> for ops::RangeFrom<u64> {
//     type Output = Bits<'a, T>;
//     fn get(&self, range: &'r Bits<'a, T>) -> Self::Output {
//         let start = self.start;
//         let end = T::CAPACITY;
//         assert!(start <= end && start < T::CAPACITY && end <= T::CAPACITY);
//         let this = range.0.rhs;
//         let that = Run::new(start, end - start);
//         Bits::with_run(range.0.lhs, this.intersect(&that))
//     }
// }

// impl<'r, 'a, T: bit_set::ops::Capacity> Index<&'r Bits<'a, T>> for ops::RangeTo<u64> {
//     type Output = Bits<'a, T>;
//     fn get(&self, range: &'r Bits<'a, T>) -> Self::Output {
//         let start = 0;
//         let end = self.end;
//         assert!(start <= end && start < T::CAPACITY && end <= T::CAPACITY);
//         let this = range.0.rhs;
//         let that = Run::new(start, end - start);
//         Bits::with_run(range.0.lhs, this.intersect(&that))
//     }
// }

// macro_rules! impl_Index {
//     ($( [ $($tts:tt)* ] for $Type:ty ; )+) => {
//         $(
//             impl<'a, $($tts)*> Index<&'a $Type> for ops::RangeFull {
//                 type Output = Bits<'a, $Type>;
//                 fn get(&self, lhs: &'a $Type) -> Self::Output {
//                     let cap = bit_set::LARGE_BIT_MAX;
//                     let rhs = Run::new(0, cap);
//                     Bits(And { lhs, rhs })
//                 }
//             }

//             impl<'a, $($tts)*> Index<&'a $Type> for ops::Range<u64> {
//                 type Output = Bits<'a, $Type>;
//                 fn get(&self, lhs: &'a $Type) -> Self::Output {
//                     let cap = bit_set::LARGE_BIT_MAX;
//                     let start = self.start;
//                     let end = self.end;
//                     assert!(start <= end && start < cap && end <= cap);
//                     let rhs = Run::new(start, end - start);
//                     Bits(And { lhs, rhs })
//                 }
//             }

//             impl<'a, $($tts)*> Index<&'a $Type> for ops::RangeFrom<u64> {
//                 type Output = Bits<'a, $Type>;
//                 fn get(&self, lhs: &'a $Type) -> Self::Output {
//                     let cap = bit_set::LARGE_BIT_MAX;
//                     let start = self.start;
//                     let end = cap;
//                     assert!(start <= end && start < cap && end <= cap);
//                     let rhs = Run::new(start, end - start);
//                     Bits(And { lhs, rhs })
//                 }
//             }

//             impl<'a, $($tts)*> Index<&'a $Type> for ops::RangeTo<u64> {
//                 type Output = Bits<'a, $Type>;
//                 fn get(&self, lhs: &'a $Type) -> Self::Output {
//                     let cap = bit_set::LARGE_BIT_MAX;
//                     let start = 0;
//                     let end = self.end;
//                     assert!(start <= end && start < cap && end <= cap);
//                     let rhs = Run::new(start, end - start);
//                     Bits(And { lhs, rhs })
//                 }
//             }
//         )+
//     }
// }
// impl_Index!(
//     [T] for bit_set::BitSet<T>;
//     [T] for bit_set::RawVec<T>;
// );

// pub(crate) type Entries<'a, T> = Box<dyn Iterator<Item = bit_set::Entry<Cow<'a, [T]>>>>;

// impl<'a, L, T: Word> IntoIterator for Bits<'a, L>
// where
//     &'a L: IntoIterator<Item = bit_set::Entry<Cow<'a, [T]>>>,
// {
//     type Item = <&'a L as IntoIterator>::Item;
//     type IntoIter = AndEntries<<&'a L as IntoIterator>::IntoIter, Entries<'a, T>, Cow<'a, [T]>>;

//     fn into_iter(self) -> Self::IntoIter {
//         AndEntries {
//             lhs: self.0.lhs.into_iter().peekable(),
//             rhs: self.0.rhs.head_to_last::<T>().peekable(),
//             _ty: PhantomData,
//         }
//     }
// }

// impl<'a, T> Bits<'a, T> {
//     // fn new(bits: &'a T, index: u64, len: u64) -> Self {
//     //     let run = Run::new(index, len);
//     //     Self::with_run(bits, run)
//     // }

//     fn with_run(bits: &'a T, run: Run) -> Self {
//         Bits(And { lhs: bits, rhs: run })
//     }

//     pub fn get<'r, Ix: Index<&'r Self>>(&'r self, index: Ix) -> Ix::Output {
//         index.get(self)
//     }
// }

// impl<'a, T: Rank> Bits<'a, T> {
//     // #[inline]
//     // pub fn size(&self) -> u64
//     // where
//     //     T: Access,
//     // {
//     //     <Self as Access>::size(self)
//     // }

//     // #[inline]
//     // pub fn access(&self, i: u64) -> bool
//     // where
//     //     T: Access,
//     // {
//     //     <Self as Access>::access(self, i)
//     // }

//     #[inline]
//     pub fn count1(&self) -> u64 {
//         <Self as Count>::count1(self)
//     }

//     #[inline]
//     pub fn count0(&self) -> u64 {
//         <Self as Count>::count0(self)
//     }

//     #[inline]
//     pub fn rank1(&self, i: u64) -> u64 {
//         <Self as Rank>::rank1(self, i)
//     }

//     #[inline]
//     pub fn rank0(&self, i: u64) -> u64 {
//         <Self as Rank>::rank0(self, i)
//     }

//     #[inline]
//     pub fn excess1(&self, i: u64) -> u64 {
//         <Self as Rank>::excess1(self, i)
//     }

//     #[inline]
//     pub fn excess0(&self, i: u64) -> u64 {
//         <Self as Rank>::excess0(self, i)
//     }
// }

// impl<'a, T: bit_set::ops::Capacity> bit_set::ops::Capacity for Bits<'a, T> {
//     const CAPACITY: u64 = T::CAPACITY;
// }

// impl<'a, T: bit_set::ops::Access> bit_set::ops::Access for Bits<'a, T> {
//     fn size(&self) -> u64 {
//         self.0.lhs.size()
//     }

//     fn access(&self, i: u64) -> bool {
//         self.0.rhs.contains(i) && self.0.lhs.access(i)
//     }
// }

// impl<'a, T: bit_set::ops::Rank> bit_set::ops::Count for Bits<'a, T> {
//     fn count1(&self) -> u64 {
//         let lhs = self.0.lhs;
//         let rhs = self.0.rhs;

//         if rhs.len == 0 {
//             0
//         } else {
//             let i = rhs.index;
//             let j = rhs.index + rhs.len;
//             lhs.rank1(j) - lhs.rank1(i)
//         }
//     }
// }

// impl<'a, T: bit_set::ops::Rank> bit_set::ops::Rank for Bits<'a, T> {
//     fn rank1(&self, i: u64) -> u64 {
//         let lhs = self.0.lhs;
//         let run = self.0.rhs;

//         if run.len == 0 || i <= run.index {
//             0
//         } else if run.index < i && i <= run.index + run.len {
//             lhs.rank1(i) - lhs.rank1(run.index)
//         } else {
//             self.count1() // range.index + range.len < i
//         }
//     }
// }

// impl Run {
//     fn new(index: u64, len: u64) -> Self {
//         assert!(index + len <= bit_set::LARGE_BIT_MAX, "{}", bit_set::OUT_OF_BOUNDS);
//         Run { index, len }
//     }

//     fn contains(&self, i: u64) -> bool {
//         self.index <= i && i < self.index + self.len
//     }

//     fn intersect(&self, that: &Run) -> Run {
//         if !self.overlap(that) {
//             return Run::new(0, 0);
//         }

//         let index = if self.index >= that.index { self.index } else { that.index };

//         let self_len = self.index + self.len;
//         let that_len = that.index + that.len;
//         let len = if self_len >= that_len { that_len - index } else { self_len - index };

//         Run::new(index, len)
//     }

//     fn overlap(&self, that: &Run) -> bool {
//         self.index < that.index + that.len && that.index < self.index + self.len
//     }

//     // pub(crate) fn into_head_to_last<'a, T: Word>(self) -> Entries<'a, T> {
//     //     self.head_to_last()
//     // }

//     fn head_to_last<'a, T: Word>(&self) -> Entries<'a, T> {
//         if self.len == 0 {
//             return Box::new(iter::empty());
//         }

//         let head_index = (self.index / bit_set::SHORT_BIT_MAX) as usize;
//         let last_index = ((self.index + self.len) / bit_set::SHORT_BIT_MAX) as usize;

//         let head_offset = self.index % bit_set::SHORT_BIT_MAX;
//         let last_offset = (self.index + self.len) % bit_set::SHORT_BIT_MAX;

//         assert!(head_index <= last_index);
//         let words = bit_set::CowWords::<'a, T>::LEN;

//         let trim_head = |vec: &mut Vec<T>, offset: u64| {
//             let q = (offset / T::CAPACITY) as usize;
//             let r = offset % T::CAPACITY;
//             for i in 0..q {
//                 vec[i] = T::ZERO;
//             }
//             for i in 0..r {
//                 vec[q].remove(i);
//             }
//         };

//         let trim_last = move |vec: &mut Vec<T>, offset: u64| {
//             let q = (offset / T::CAPACITY) as usize;
//             let r = offset % T::CAPACITY;
//             if q < words {
//                 for i in r..T::CAPACITY {
//                     vec[q].remove(i);
//                 }
//                 for i in (q + 1)..words {
//                     vec[i] = T::ZERO;
//                 }
//             }
//         };

//         let head = if head_index == last_index && head_offset == 0 && last_offset == 0 {
//             bit_set::Entry::new(head_index, bit_set::CowWords::<'_, T>::splat(!T::ZERO))
//         } else {
//             let words = {
//                 let mut vec = vec![!T::ZERO; words];
//                 trim_head(&mut vec, head_offset);
//                 if head_index == last_index {
//                     trim_last(&mut vec, last_offset);
//                 }
//                 bit_set::CowWords::from(vec)
//             };
//             bit_set::Entry::new(head_index, words)
//         };

//         return if head_index == last_index {
//             // head only
//             Box::new(iter::once(head))
//         } else if last_offset == 0 {
//             // omit last; because last is empty
//             Box::new(iter::once(head).chain({
//                 let range = head_index + 1..last_index;
//                 range.map(|i| bit_set::Entry::new(i, bit_set::CowWords::<'_,
// T>::splat(!T::ZERO)))             }))
//         } else {
//             Box::new(iter::once(head).chain({
//                 let range = head_index + 1..=last_index;
//                 range.map(move |i| {
//                     if i == last_index {
//                         let words = {
//                             let mut vec = vec![!T::ZERO; words];
//                             trim_last(&mut vec, last_offset);
//                             bit_set::CowWords::from(vec)
//                         };
//                         bit_set::Entry::new(i, words)
//                     } else {
//                         bit_set::Entry::new(i, bit_set::CowWords::<'_, T>::splat(!T::ZERO))
//                     }
//                 })
//             }))
//         };
//     }
// }

// #[cfg(test)]
// #[test]
// fn head_to_last() {
//     use crate::bit_set::ops::Count;

//     macro_rules! check {
//         ($rangefn:expr) => {
//             for i in 0..5 {
//                 let range = $rangefn(i);
//                 let count = range.len;
//                 let mut accum = 0;
//                 for entry in range.head_to_last::<u64>() {
//                     accum += entry.block.count1();
//                     println!("i={} ({}, {:?})", i, entry.index, entry.block.count1());
//                 }
//                 assert_eq!(count, accum);
//             }
//         };
//     }

//     check!(|i| Run::new(100_000_000_u64 * i, 0));
//     check!(|i| Run::new(100_000_000_u64 * i, 1));

//     check!(|i| Run::new(65536_u64 * i, 0));
//     check!(|i| Run::new(65536_u64 * i, 65536));
//     check!(|i| Run::new(12384_u64 * i, 65536));
//     check!(|i| Run::new(16384_u64 * i, 1_000_000));
//     check!(|i| Run::new(65536_u64 * i, 10000));
//     check!(|i| Run::new(65536_u64 * i, 1_000_000));
//     check!(|i| Run::new(65536_u64 * i, 100_000_000));
// }
