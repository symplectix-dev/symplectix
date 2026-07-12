use crate::bit_set;
use crate::bit_set::{
    Word,
    word,
};

/// A constant value of [Access::size](trait.Access.html#tymethod.size).
pub trait Capacity {
    const CAPACITY: u64;
}

pub trait Access {
    /// The potential bit size of the container.
    ///
    /// But the container is not guaranteed to be able to reach that size:
    /// It can fail to allocate at any point before that size is reached.
    fn size(&self) -> u64;

    /// Check whether bit at `i` is enabled.
    fn access(&self, i: u64) -> bool;
}

macro_rules! impl_Access_for_words {
    ($($ty:ty),*) => ($(
        impl Capacity for $ty {
            const CAPACITY: u64 = std::mem::size_of::<$ty>() as u64 * 8;
        }

        impl Access for $ty {
            #[inline]
            fn size(&self) -> u64 {
                Self::CAPACITY
            }
            #[inline]
            fn access(&self, i: u64) -> bool {
                (*self & Self::bit(word::cast(i))) != Self::ZERO
            }
        }
    )*)
}
impl_Access_for_words!(u8, u16, u32, u64, u128, usize);

impl<T: Capacity + Access> Access for [T] {
    fn size(&self) -> u64 {
        T::CAPACITY * word::cast::<usize, u64>(self.len())
    }
    fn access(&self, i: u64) -> bool {
        let (index, offset) = bit_set::address(i, T::CAPACITY);
        self.get(index).is_some_and(|w| w.access(offset))
    }
}
