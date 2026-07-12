use crate::bit_set;
use crate::bit_set::Word;
use crate::bit_set::ops::{
    Access,
    Capacity,
};
use crate::bit_set::word::cast;

/// Trait to manipulate bits.
pub trait Remove: Access {
    /// Remove bit, and return a **previous** value.
    fn remove(&mut self, i: u64) -> bool;
}

macro_rules! impl_Remove_for_words {
    ($($ty:ty),*) => ($(
        impl Remove for $ty {
            #[inline]
            fn remove(&mut self, i: u64) -> bool {
                assert!(i < Self::CAPACITY);
                if self.access(i) {
                    *self &= !Self::bit(cast(i));
                    true
                } else {
                    false
                }
            }
        }
    )*)
}
impl_Remove_for_words!(u8, u16, u32, u64, u128, usize);

impl<T: Capacity + Remove> Remove for [T] {
    fn remove(&mut self, i: u64) -> bool {
        let (index, offset) = bit_set::address(i, T::CAPACITY);
        if self[index].access(offset) {
            self[index].remove(offset);
            true
        } else {
            false
        }
    }
}
