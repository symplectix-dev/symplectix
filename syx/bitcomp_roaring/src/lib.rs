#![allow(missing_docs)]

pub mod bit_set;
pub use crate::bit_set::BitSet;

// pub trait Count<T> {
//     type Symbol;
//     /// Returns occurences of symbol `c` in self.
//     fn count(&self, c: &Self::Symbol) -> T;
// }

// /// Generalization of `count`.
// pub trait Rank<T> {
//     type Symbol;
//     /// Returns occurences of symbol `c` in `0..i`.
//     fn rank(&self, c: &Self::Symbol, i: T) -> T;
// }

// /// `Select` is a right inverse of `Rank`.
// pub trait Select<T> {
//     type Symbol;
//     /// Returns the position of 'i+1'th appearance of symbol `c`.
//     fn select(&self, c: &Self::Symbol, i: T) -> Option<T>;
// }
