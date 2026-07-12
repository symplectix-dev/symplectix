use crate::bit_set;

/// `Count` the number of bits in the container.
/// `Count` is a special case of `Rank`.
///
/// `count1` and `count0` have default implementation, but these are cycled.
/// So either `count1` or `count0` need to be redefined.
pub trait Count: bit_set::ops::Access {
    /// Return the number of non-zero bits.
    fn count1(&self) -> u64 {
        self.size() - self.count0()
    }

    /// Return the number of zero bits.
    fn count0(&self) -> u64 {
        self.size() - self.count1()
    }
}

macro_rules! impl_Count_for_words {
    ($($ty:ty),*) => ($(
        impl Count for $ty {
            #[inline]
            fn count1(&self) -> u64 {
                self.count_ones() as u64
            }
            #[inline]
            fn count0(&self) -> u64 {
                self.count_zeros() as u64
            }
        }
    )*)
}
impl_Count_for_words!(u8, u16, u32, u64, u128, usize);

impl<T: bit_set::ops::Capacity + Count> Count for [T] {
    /// Count the number of non-zero bits.
    fn count1(&self) -> u64 {
        self.iter().map(|w| w.count1()).sum()
    }
}
