use crate::bit_set;
use crate::bit_set::ops::{
    Access,
    Capacity,
};
use crate::bit_set::{
    Word,
    word,
};

/// Trait to manipulate bits.
pub trait Insert: Access {
    /// Insert bit, and return a **previous** value.
    fn insert(&mut self, i: u64) -> bool;
}

macro_rules! impl_Insert_for_words {
    ($($ty:ty),*) => ($(
        impl Insert for $ty {
            #[inline]
            fn insert(&mut self, i: u64) -> bool {
                assert!(i < Self::CAPACITY);
                if self.access(i) {
                    true
                } else {
                    *self |= Self::bit(word::cast(i));
                    false
                }
            }
        }
    )*)
}
impl_Insert_for_words!(u8, u16, u32, u64, u128, usize);

impl<T: Capacity + Insert> Insert for [T] {
    fn insert(&mut self, i: u64) -> bool {
        let (index, offset) = bit_set::address(i, T::CAPACITY);
        if self[index].access(offset) {
            true
        } else {
            self[index].insert(offset);
            false
        }
    }
}
