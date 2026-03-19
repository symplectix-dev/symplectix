#![allow(missing_docs)]

use std::cmp;
use std::iter::Sum;
use std::ops::RangeBounds;

use bits::*;
use fenwicktree::{
    LowerBound,
    Nodes,
    Prefix,
};

mod l1l2;

use l1l2::L1L2;

/// `Pop<T>` stores auxiliary data to compute `Rank` and `Select` more efficiently.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pop<T> {
    aux:  Aux,
    repr: T,
}

// pub type PopVec<T> = Pop<Vec<T>>;

// Modified a little to build a binary indexed tree, instead of accumulating.
// * [Space-Efficient, High-Performance Rank & Select Structures on Uncompressed Bit Sequences](https://www.cs.cmu.edu/~dga/papers/zhou-sea2013.pdf)
//
// It seems good to try the latter to see if efficiency improves.
// * [Engineering Compact Data Structures for Rank and Select Queries on Bit Vectors](https://arxiv.org/pdf/2206.01149)
#[derive(Debug, Clone, PartialEq, Eq)]
struct Aux {
    ubs: Vec<u64>,
    lbs: Vec<L1L2>,
}

const UPPER_BLOCK: u64 = 1 << 32;
const SUPER_BLOCK: u64 = BASIC_BLOCK * 4;
const BASIC_BLOCK: u64 = 512;

const MAX_SB_LEN: usize = (UPPER_BLOCK / SUPER_BLOCK) as usize;

fn build<'a, T, I>(size: u64, super_blocks: I) -> Aux
where
    T: Word + 'a,
    I: IntoIterator<Item = Option<&'a [T]>>,
{
    let mut aux = Aux::new(size);

    for (i, sb) in super_blocks.into_iter().enumerate() {
        let (bbs, sum) = basic_blocks(sb);

        let (q, r) = (i / MAX_SB_LEN, i % MAX_SB_LEN);

        // +1 to skip dummy index
        aux.ubs[q + 1] += sum;
        aux.lb_mut(q)[r + 1] = L1L2::merge([sum, bbs[0], bbs[1], bbs[2]]);
    }

    aux
}

fn basic_blocks<W: Word>(sb: Option<&[W]>) -> ([u64; l1l2::LEN], u64) {
    let mut bbs = [0; l1l2::LEN];
    let mut sum = 0;
    if let Some(sb) = sb {
        let chunk_size = (BASIC_BLOCK / W::BITS) as usize;
        for (i, bb) in sb.chunks(chunk_size).enumerate() {
            let count1 = bb.count1();
            bbs[i] = count1;
            sum += count1;
        }
    }
    (bbs, sum)
}

fn super_blocks_from_words<T: Word>(slice: &[T]) -> impl Iterator<Item = Option<&[T]>> {
    let chunk_size = (SUPER_BLOCK / T::BITS) as usize;
    slice.chunks(chunk_size).map(Some)
}

fn ubs_len(n: u64) -> usize {
    bits::blocks(n, UPPER_BLOCK) + 1
}

fn lbs_len(n: u64) -> usize {
    if n == 0 {
        1
    } else {
        // A minimum and a *logical* length of a vector to store `L1L2`.
        let supers = bits::blocks(n, SUPER_BLOCK);
        supers + {
            // Remenber that fenwicks for L1 and L2 is logically `Vec<Vec<LL>>` but flattened.
            // Need additional space for each fenwicks because of its 1-based indexing.
            let (q, r) = (supers / MAX_SB_LEN, supers % MAX_SB_LEN);
            q + (r > 0) as usize
        }
    }
}

impl<T: Word> From<Vec<T>> for Pop<Vec<T>> {
    fn from(repr: Vec<T>) -> Self {
        let mut aux = build(repr.bits(), super_blocks_from_words(&repr));

        // TODO: should be in the [`build`] loop.
        {
            // initialize upper_blocks as a binary index tree
            fenwicktree::build(&mut aux.ubs);

            // initialize lower_blocks as a binary index tree
            for q in 0..aux.lb_parts() {
                fenwicktree::build(aux.lb_mut(q));
            }
        }

        Pop { aux, repr }
    }
}

impl<B: bits::Block> Pop<Vec<B>> {
    #[inline]
    pub fn new(n: u64) -> Pop<Vec<B>> {
        let repr = vec![B::empty(); bits::blocks(n, B::BITS)];
        Pop { aux: Aux::new(repr.bits()), repr }
    }
}

impl<T> Pop<T> {
    pub fn inner(&self) -> &T {
        &self.repr
    }
}

impl<B: Bits> Pop<B> {
    #[inline]
    pub fn bits(&self) -> u64 {
        self.repr.bits()
    }

    #[inline]
    pub fn bit(&self, i: u64) -> bool {
        self.repr.bit(i)
    }

    #[inline]
    pub fn count1(&self) -> u64 {
        let ubs = &self.aux.ubs;
        ubs.sum(ubs.nodes())
    }

    #[inline]
    pub fn count0(&self) -> u64 {
        self.bits() - self.count1()
    }

    fn rank1_impl(&self, p0: u64) -> u64 {
        if p0 == 0 {
            0
        } else if p0 == self.bits() {
            self.count1()
        } else {
            let (q0, r0) = bits::index(p0, UPPER_BLOCK);
            let (q1, r1) = bits::index(r0, SUPER_BLOCK);
            let (q2, r2) = bits::index(r1, BASIC_BLOCK);

            let hi = &self.aux.ubs;
            let lo = self.aux.lb(q0);
            let c0: u64 = hi.sum(q0);
            let c1: u64 = lo.sum(q1);
            let c2 = lo[q1 + 1].l2_sum(q2);
            c0 + c1 + c2 + self.repr.rank1(p0 - r2..p0)
        }
    }

    #[inline]
    pub fn rank1<R: RangeBounds<u64>>(&self, r: R) -> u64 {
        let (i, j) = bits::range(&r, 0, self.bits());
        self.rank1_impl(j) - self.rank1_impl(i)
    }

    #[inline]
    pub fn rank0<R: RangeBounds<u64>>(&self, r: R) -> u64 {
        let (i, j) = bits::range(&r, 0, self.bits());
        (j - i) - (self.rank1_impl(j) - self.rank1_impl(i))
    }

    pub fn select1(&self, n: u64) -> Option<u64> {
        let mut r = n;

        let (s, e) = {
            let p0 = find_l0(&self.aux.ubs[..], &mut r)?;
            let lo = self.aux.lb(p0);
            let p1 = find_l1(lo, &mut r);
            let ll = lo[p1 + 1];
            let l2 = [ll.l2::<0>(), ll.l2::<1>(), ll.l2::<2>()];
            let p2 = find_l2(&l2, &mut r);

            let s = p0 as u64 * UPPER_BLOCK + p1 as u64 * SUPER_BLOCK + p2 as u64 * BASIC_BLOCK;
            (s, cmp::min(s + BASIC_BLOCK, self.bits()))
        };

        {
            debug_assert!(n - r == self.rank1(..s));
            debug_assert!(r < self.rank1(s..e));
        }

        // i + imp.bit_vec[x..y].select1(r).unwrap()

        const BITS: u64 = <u128 as Block>::BITS;
        for i in (s..e).step_by(BITS as usize) {
            let b = self.repr.word::<u128>(i, BITS);
            let c = b.count1();
            if r < c {
                // #[cfg(test)]
                // {
                //     dbg!(l0, l1, l2);
                //     dbg!(lo[l1 + 1]);
                //     dbg!(s, e);
                // }
                return Some(i + b.select1(r).unwrap());
            }
            r -= c;
        }
        unreachable!()
    }

    pub fn select0(&self, n: u64) -> Option<u64> {
        let mut r = n;

        let (s, e) = {
            const UB: u64 = UPPER_BLOCK;
            const SB: u64 = SUPER_BLOCK;
            const BB: u64 = BASIC_BLOCK;
            let hi_complemented = fenwicktree::complement(&self.aux.ubs[..], UB);
            let p0 = find_l0(&hi_complemented, &mut r)?;
            let lo = self.aux.lb(p0);
            let lo_complemented = fenwicktree::complement(lo, SB);
            let p1 = find_l1(&lo_complemented, &mut r);
            let ll = lo[p1 + 1];
            let l2 = [BB - ll.l2::<0>(), BB - ll.l2::<1>(), BB - ll.l2::<2>()];
            let p2 = find_l2(&l2, &mut r);

            let s = p0 as u64 * UPPER_BLOCK + p1 as u64 * SUPER_BLOCK + p2 as u64 * BASIC_BLOCK;
            (s, cmp::min(s + BASIC_BLOCK, self.bits()))
        };

        {
            debug_assert!(n - r == self.rank0(..s));
            debug_assert!(r < self.rank0(s..e));
        }

        const BITS: u64 = <u128 as Block>::BITS;
        for i in (s..e).step_by(BITS as usize) {
            let b = self.repr.word::<u128>(i, BITS);
            let c = b.count0();
            if r < c {
                return Some(i + b.select0(r).unwrap());
            }
            r -= c;
        }
        unreachable!()
    }
}

fn find_l0<L0>(l0: &L0, r: &mut u64) -> Option<usize>
where
    L0: ?Sized + Nodes + Prefix<u64> + LowerBound<u64>,
    u64: Sum<L0::Node>,
{
    // r: +1 because `select1(n)` returns the position of the n-th one, indexed starting from zero.
    // i: -1 is safe because lower_bound(x) returns 0 iif x is 0
    let p0 = l0.lower_bound(*r + 1) - 1;
    if p0 >= l0.nodes() {
        None
    } else {
        *r -= l0.sum(p0);
        Some(p0)
    }
}

fn find_l1<L1>(l1: &L1, r: &mut u64) -> usize
where
    L1: ?Sized + Nodes + Prefix<u64> + LowerBound<u64>,
    u64: Sum<L1::Node>,
{
    let p1 = l1.lower_bound(*r + 1) - 1;
    *r -= l1.sum(p1);
    p1
}

fn find_l2<'a, L2>(l2: L2, r: &mut u64) -> usize
where
    L2: IntoIterator<Item = &'a u64> + 'a,
{
    let mut p2 = 0;
    for &c in l2.into_iter() {
        if *r < c {
            break;
        }
        *r -= c;
        p2 += 1;
    }
    p2
}

impl<B: bits::BitsMut> Pop<B> {
    /// Swaps a bit at `i` by `bit` and returns the previous value.
    fn swap(&mut self, i: u64, bit: bool) -> bool {
        let before = self.repr.bit(i);
        if bit {
            self.repr.set1(i);
        } else {
            self.repr.set0(i);
        }
        before
    }

    #[inline]
    pub fn set1(&mut self, index: u64) {
        if !self.swap(index, true) {
            self.aux.incr(index, 1);
        }
    }

    #[inline]
    pub fn set0(&mut self, index: u64) {
        if self.swap(index, false) {
            self.aux.decr(index, 1);
        }
    }
}

impl Aux {
    fn new(n: u64) -> Aux {
        let ubs = vec![0; ubs_len(n)];
        let lbs = vec![L1L2::zero(); lbs_len(n)];
        Aux { ubs, lbs }
    }

    #[inline]
    fn lb(&self, i: usize) -> &[L1L2] {
        let s = (MAX_SB_LEN + 1) * i;
        let e = cmp::min(s + (MAX_SB_LEN + 1), self.lbs.len());
        &self.lbs[s..e]
    }

    #[inline]
    fn lb_mut(&mut self, i: usize) -> &mut [L1L2] {
        let s = (MAX_SB_LEN + 1) * i;
        let e = cmp::min(s + (MAX_SB_LEN + 1), self.lbs.len());
        &mut self.lbs[s..e]
    }

    // The logical number of fenwicks hiding at `lo`.
    // #[cfg(test)]
    #[inline]
    fn lb_parts(&self) -> usize {
        // if self.low.len() == 1 {
        //     0
        // } else {
        //     blocks(self.low.len(), MAXL1 + 1)
        // }
        bits::blocks(self.lbs.len() as u64, MAX_SB_LEN as u64 + 1)
    }

    fn incr(&mut self, p0: u64, delta: u64) {
        use fenwicktree::Incr;

        let (q0, r0) = bits::index(p0, UPPER_BLOCK);
        let (q1, r1) = bits::index(r0, SUPER_BLOCK);

        self.ubs.incr(q0 + 1, delta);

        let lo = self.lb_mut(q0);
        lo.incr(q1 + 1, delta);

        // Update L2 array which is interleaved into L1
        let sb = q1 + 1; // +1 because fenwick doesn't use index 0
        let bb = (r1 / BASIC_BLOCK + 1) as usize; // +1 to skip index 0 which is for L1
        if bb < l1l2::LEN {
            lo[sb] = {
                let mut arr = L1L2::split(lo[sb]);
                arr[bb] += delta;
                L1L2::merge(arr)
            };
        }
    }

    fn decr(&mut self, p0: u64, delta: u64) {
        use fenwicktree::Decr;

        let (q0, r0) = bits::index(p0, UPPER_BLOCK);
        let (q1, r1) = bits::index(r0, SUPER_BLOCK);

        let hi = &mut self.ubs;
        hi.decr(q0 + 1, delta);

        let lo = self.lb_mut(q0);
        lo.decr(q1 + 1, delta);

        let sb = q1 + 1;
        let bb = (r1 / BASIC_BLOCK + 1) as usize;
        if bb < l1l2::LEN {
            lo[sb] = {
                let mut arr = L1L2::split(lo[sb]);
                arr[bb] -= delta;
                L1L2::merge(arr)
            };
        }
    }

    // fn resize(&mut self, cur_len: usize, new_len: usize) {
    //     let (cur_top_len, cur_low_len) = (self.top.len(), self.low.len());
    //     let (new_top_len, new_low_len) = (toplen(new_len), lowlen(new_len));
    //
    //     let top = &mut self.top;
    //     let low = &mut self.low;
    //
    //     match dbg!(cur_len.cmp(&new_len)) {
    //         // cur_len == new_len
    //         EQ =>
    //         {
    //             #[cfg(test)]
    //             (|| {
    //                 assert_eq!(cur_top_len, new_top_len);
    //                 assert_eq!(cur_low_len, new_low_len);
    //             })()
    //         }
    //
    //         // cur_len < new_len
    //         LT => {
    //             unimplemented!()
    //         }
    //
    //         // cur_len > new_len
    //         GT => {
    //             top.truncate(new_top_len);
    //             low.truncate(new_low_len);
    //         }
    //     }
    // }
}

// impl<'a, T: Clone> From<Rho<&'a [T]>> for Rho<Vec<T>> {
//     fn from(Rho(imp): Rho<&'a [T]>) -> Self {
//         Rho(Imp { buckets: imp.buckets, samples: None, bit_vec: imp.bit_vec.to_vec() })
//     }
// }

// impl<'a, T: WordArray> From<&'a [Block<T>]> for Rho<&'a [Block<T>]> {
//     fn from(dat: &'a [Block<T>]) -> Self {
//         let (buckets, _) = build(Seq::len(&dat), sbs_from_heaps(&dat));
//         Rho(Imp { buckets: buckets.into(), samples: None, bit_vec: dat })
//     }
// }

// impl<T: Word, U: From<Vec<T>>> From<Vec<T>> for Rho<U> {
//     fn from(dat: Vec<T>) -> Self {
//         let (buckets, _) = build(Seq::len(&dat), sbs_from_words(&dat));
//         Rho(Imp { buckets: buckets.into(), samples: None, bit_vec: dat.into() })
//     }
// }

// impl<T: WordArray, U: From<Vec<Block<T>>>> From<Vec<Block<T>>> for Rho<U> {
//     fn from(dat: Vec<Block<T>>) -> Self {
//         let (buckets, _) = build(Seq::len(&dat), sbs_from_heaps(&dat));
//         Rho(Imp { buckets: buckets.into(), samples: None, bit_vec: dat.into() })
//     }
// }
