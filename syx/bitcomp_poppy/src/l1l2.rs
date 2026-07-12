use std::cmp::Ordering;
use std::fmt::{
    self,
    Debug,
    Formatter,
};
use std::iter::Sum;
use std::ops::{
    Add,
    AddAssign,
    Sub,
    SubAssign,
};

#[derive(Copy, Clone, Default, PartialEq, Eq)]
pub(crate) struct L1L2(u64);

impl Debug for L1L2 {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_tuple("L1L2")
            .field(&self.l1())
            .field(&self.l2::<0>())
            .field(&self.l2::<1>())
            .field(&self.l2::<2>())
            .finish()
    }
}

pub(crate) const LEN: usize = 4;

const L1_MASK: u64 = 0x_FFFF_FFFF; // lowest 32 bits
const L2_MASK: u64 = 0x_03FF; // lowest 10 bits

const L2_0_SHIFT: u64 = 32;
const L2_1_SHIFT: u64 = 42;
const L2_2_SHIFT: u64 = 52;
const L2_SHIFT: [u64; LEN - 1] = [L2_0_SHIFT, L2_1_SHIFT, L2_2_SHIFT];

impl L1L2 {
    #[inline]
    pub(crate) const fn zero() -> Self {
        Self(0)
    }

    #[inline]
    pub(crate) const fn merge(mut arr: [u64; LEN]) -> Self {
        debug_assert!(arr[0] < L1_MASK && arr[1] < 1024 && arr[2] < 1024 && arr[3] < 1024);

        arr[0] |= arr[1] << L2_SHIFT[0];
        arr[0] |= arr[2] << L2_SHIFT[1];
        arr[0] |= arr[3] << L2_SHIFT[2];
        L1L2(arr[0])
    }

    #[inline]
    pub(crate) const fn split(ll: Self) -> [u64; LEN] {
        [ll.l1(), ll.l2::<0>(), ll.l2::<1>(), ll.l2::<2>()]
    }

    #[inline]
    pub(crate) const fn l1(self) -> u64 {
        let L1L2(l1l2) = self;
        l1l2 & L1_MASK
    }

    #[inline]
    pub(crate) const fn l2<const N: usize>(self) -> u64 {
        let L1L2(l1l2) = self;
        (l1l2 >> L2_SHIFT[N]) & L2_MASK
    }

    // Sum of basic blocks. [0,i)
    #[inline]
    pub(crate) fn l2_sum(self, i: usize) -> u64 {
        match i {
            0 => 0,
            1 => self.l2::<0>(),
            2 => self.l2::<0>() + self.l2::<1>(),
            3 => self.l2::<0>() + self.l2::<1>() + self.l2::<2>(),
            _ => unreachable!("basic block: index out of bounds"),
        }
    }
}

// for lower_bound
impl PartialEq<u64> for L1L2 {
    #[inline]
    fn eq(&self, u: &u64) -> bool {
        self.l1().eq(u)
    }
}
impl PartialOrd<u64> for L1L2 {
    #[inline]
    fn partial_cmp(&self, u: &u64) -> Option<Ordering> {
        self.l1().partial_cmp(u)
    }
}

// for init
impl AddAssign<L1L2> for L1L2 {
    #[inline]
    fn add_assign(&mut self, delta: L1L2) {
        self.0 += delta.l1();
    }
}
// not used for now but here to keep symmetric
impl SubAssign<L1L2> for L1L2 {
    #[inline]
    fn sub_assign(&mut self, delta: L1L2) {
        self.0 -= delta.l1();
    }
}

// for add
impl AddAssign<u64> for L1L2 {
    #[inline]
    fn add_assign(&mut self, delta: u64) {
        self.0 += delta;
    }
}
// for sub
impl SubAssign<u64> for L1L2 {
    #[inline]
    fn sub_assign(&mut self, delta: u64) {
        self.0 -= delta;
    }
}

// lower_bound
impl From<L1L2> for u64 {
    #[inline]
    fn from(ll: L1L2) -> u64 {
        ll.l1()
    }
}

// for accum
impl Add<u64> for L1L2 {
    type Output = u64;
    #[inline]
    fn add(mut self, delta: u64) -> Self::Output {
        self += delta;
        self.l1()
    }
}
impl Sub<u64> for L1L2 {
    type Output = u64;
    #[inline]
    fn sub(mut self, delta: u64) -> Self::Output {
        self -= delta;
        self.l1()
    }
}

impl Sum<L1L2> for u64 {
    #[inline]
    fn sum<I: Iterator<Item = L1L2>>(iter: I) -> Self {
        iter.map(|l1l2| l1l2.l1()).sum()
    }
}

// not used for now but here to keep symmetric
impl Add<L1L2> for u64 {
    type Output = u64;
    #[inline]
    fn add(self, delta: L1L2) -> Self::Output {
        self + delta.l1()
    }
}
impl AddAssign<L1L2> for u64 {
    #[inline]
    fn add_assign(&mut self, delta: L1L2) {
        *self += delta.l1();
    }
}

impl Sub<L1L2> for u64 {
    type Output = u64;
    #[inline]
    fn sub(self, delta: L1L2) -> Self::Output {
        self - delta.l1()
    }
}
impl SubAssign<L1L2> for u64 {
    #[inline]
    fn sub_assign(&mut self, delta: L1L2) {
        *self -= delta.l1();
    }
}
