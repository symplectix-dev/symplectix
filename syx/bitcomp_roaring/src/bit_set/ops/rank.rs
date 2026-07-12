use crate::bit_set;
use crate::bit_set::ops::{
    Access,
    Capacity,
    Count,
};
use crate::bit_set::{
    Word,
    word,
};

/// `Rank` is a generization of `Count`.
///
/// `rank1` and `rank0` have default implementation, but these are cycled.
/// So either `rank1` or `rank0` need to be redefined.
pub trait Rank: Count {
    /// Returns the number of non-zero bit in `[0, i)`, for `i <= size`.
    ///
    /// - `i == size`: rank1 is equal to count1.
    /// - `i >  size`: rank1 should panic to satisfy the following rule, `rank1(i) + rank0(i) == i`.
    fn rank1(&self, i: u64) -> u64 {
        assert!(i <= self.size(), "index out of bounds");
        i - self.rank0(i)
    }

    /// Returns the number of zero bit in `[0, i)`, for `i <= size`.
    ///
    /// - `i == size`: rank0 is equal to count0.
    /// - `i >  size`: rank0 should panic to satisfy the following rule, `rank1(i) + rank0(i) == i`.
    fn rank0(&self, i: u64) -> u64 {
        assert!(i <= self.size(), "index out of bounds");
        i - self.rank1(i)
    }

    /// Returns `|rank1(i) - rank0(i)|`.
    #[doc(hidden)]
    fn excess(&self, i: u64) -> u64 {
        let rank1 = self.rank1(i);
        let rank0 = self.rank0(i);
        rank1.abs_diff(rank0)
        // if rank1 >= rank0 { rank1 - rank0 } else { rank0 - rank1 }
    }

    /// Returns `rank1(i) - rank0(i)`.
    ///
    /// # Panics
    /// Panics if `rank1(i) < rank0(i)`.
    #[doc(hidden)]
    fn excess1(&self, i: u64) -> u64 {
        let rank1 = self.rank1(i);
        let rank0 = self.rank0(i);
        assert!(rank1 >= rank0);
        rank1 - rank0
    }

    /// Returns `rank0(i) - rank1(i)`.
    ///
    /// # Panics
    /// Panics if `rank0(i) < rank1(i)`.
    #[doc(hidden)]
    fn excess0(&self, i: u64) -> u64 {
        let rank1 = self.rank1(i);
        let rank0 = self.rank0(i);
        assert!(rank0 >= rank1);
        rank0 - rank1
    }

    /// Select1 by binary search.
    ///
    /// # Panics
    /// Panics if `n >= self.count1()`.
    #[doc(hidden)]
    fn search1(&self, n: u64) -> u64 {
        assert!(n < self.count1());
        Word::search(self.size(), |k| self.rank1(k) > n) - 1
    }

    /// Select0 by binary search.
    ///
    /// # Panics
    ///
    /// Panic if `n >= self.count0()`.
    #[doc(hidden)]
    fn search0(&self, n: u64) -> u64 {
        assert!(n < self.count0());
        Word::search(self.size(), |k| self.rank0(k) > n) - 1
    }
}

macro_rules! impl_Rank_for_words {
    ($($ty:ty),*) => ($(
        impl Rank for $ty {
            fn rank1(&self, i: u64) -> u64 {
                assert!(i <= Self::CAPACITY, "index out of bounds");
                if i == Self::CAPACITY {
                    self.count1()
                } else {
                    let mask = *self & Self::mask(word::cast(i));
                    mask.count1()
                }
            }

            fn rank0(&self, i: u64) -> u64 {
                assert!(i <= Self::CAPACITY, "index out of bounds");
                (!self).rank1(i)
            }
        }
    )*)
}
impl_Rank_for_words!(u8, u16, u32, u64, u128, usize);

impl<T: Capacity + Rank> Rank for [T] {
    fn rank1(&self, i: u64) -> u64 {
        assert!(i <= self.size(), "index out of bounds");
        let (index, offset) = bit_set::address(i, T::CAPACITY);
        let c = self.iter().take(index).map(|b| b.count1()).sum::<u64>();
        let r = self.get(index).map_or(0, |b| b.rank1(offset));
        c + r
    }
}
