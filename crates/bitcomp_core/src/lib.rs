//! Provides basic bit operations and utilities.

use std::cmp::Ordering;
use std::iter::successors;
use std::ops::{
    Bound,
    RangeBounds,
};

mod bits;
mod bits_mut;
mod block;
mod mask;
mod page;
mod word;

pub use bits::Bits;
pub use bits_mut::BitsMut;
pub use block::{
    Block,
    FromBlocks,
    IntoBlocks,
};
pub use mask::Mask;
pub use page::Page;
pub use word::Word;

mod and;
mod not;
mod or;
mod xor;

pub use and::And;
pub use not::Not;
pub use or::Or;
pub use xor::Xor;

/// Calculates the minimum number of blocks to store `n` bits.
#[inline]
pub const fn blocks(n: u64, b: u64) -> usize {
    let len = (n / b) as usize;
    len + (!n.is_multiple_of(b) as usize)
}

/// Returns a pair of numbers.
#[inline]
pub const fn index(i: u64, b: u64) -> (usize, u64) {
    let j = (i / b) as usize;
    let k = i % b;
    (j, k)
}

/// A utility to clamp the given range, which is possibly unbounded,
/// into a bounded `[i, j)`. Panics when debug is enabled and if
/// `!(min <= i && i <= j && j <= max)`.
pub fn range<R>(r: &R, min: u64, max: u64) -> (u64, u64)
where
    R: RangeBounds<u64>,
{
    let i = min_index_inclusive(r.start_bound(), min);
    let j = max_index_exclusive(r.end_bound(), max);
    debug_assert!(min <= i && i <= j && j <= max);
    (i, j)
}

#[inline]
const fn min_index_inclusive(bound: Bound<&u64>, min: u64) -> u64 {
    match bound {
        Bound::Included(&s) => s,
        Bound::Excluded(&s) => s + 1,
        Bound::Unbounded => min,
    }
}

#[inline]
const fn max_index_exclusive(bound: Bound<&u64>, max: u64) -> u64 {
    match bound {
        Bound::Included(&e) => e + 1,
        Bound::Excluded(&e) => e,
        Bound::Unbounded => max,
    }
}

/// Splits a given range [start, end) into chunks.
/// Each chunk is represented as a (index, len) tuple, and its rhs, index+len, is aligned to a
/// multiple of n.
///
/// # Examples
///
/// ```
/// let mut it = bitcomp_core::chunks(10, 0, 3);
/// assert_eq!(it.next(), None);
///
/// let mut it = bitcomp_core::chunks(10, 10, 3);
/// assert_eq!(it.next(), None);
///
/// let mut it = bitcomp_core::chunks(10, 12, 3);
/// assert_eq!(it.next(), Some((10, 2)));
/// assert_eq!(it.next(), None);
///
/// let mut it = bitcomp_core::chunks(10, 20, 3);
/// assert_eq!(it.next(), Some((10, 2)));
/// assert_eq!(it.next(), Some((12, 3)));
/// assert_eq!(it.next(), Some((15, 3)));
/// assert_eq!(it.next(), Some((18, 2)));
/// assert_eq!(it.next(), None);
///
/// let mut it = bitcomp_core::chunks(10, 21, 3);
/// assert_eq!(it.next(), Some((10, 2)));
/// assert_eq!(it.next(), Some((12, 3)));
/// assert_eq!(it.next(), Some((15, 3)));
/// assert_eq!(it.next(), Some((18, 3)));
/// assert_eq!(it.next(), None);
/// ```
pub fn chunks(start: u64, end: u64, n: u64) -> impl Iterator<Item = (u64, u64)> {
    let step = move |i| (i < end).then(|| (i, next_multiple_of(i, n).min(end) - i));
    successors(step(start), move |&(index, len)| step(index + len))
}

#[inline]
const fn next_multiple_of(x: u64, n: u64) -> u64 {
    // TODO: Use usize::checked_next_multiple_of
    // https://doc.rust-lang.org/std/primitive.usize.html#method.checked_next_multiple_of
    // https://github.com/rust-lang/rust/issues/88581
    x + (n - x % n)
}

pub(crate) fn compare<X, Y>(
    x: Option<&(usize, X)>,
    y: Option<&(usize, Y)>,
    when_x_is_none: Ordering,
    when_y_is_none: Ordering,
) -> Ordering {
    match (x, y) {
        (None, _) => when_x_is_none,
        (_, None) => when_y_is_none,
        (Some((i, _x)), Some((j, _y))) => i.cmp(j),
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn next_multiple_of() {
        use super::next_multiple_of;
        assert_eq!(next_multiple_of(0, 8), 8);
        assert_eq!(next_multiple_of(12, 3), 15);
        assert_eq!(next_multiple_of(16, 8), 24);
        assert_eq!(next_multiple_of(23, 8), 24);
        assert_eq!(next_multiple_of(9, 3), 12);
    }
}
