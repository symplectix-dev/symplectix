#![allow(missing_docs)]
//! 1-indexed FenwickTree (BinaryIndexedTree).

use std::iter::Sum;
use std::ops::{
    AddAssign,
    Sub,
    SubAssign,
};

use bitcomp_core::Word;
pub use index::{
    children,
    prefix,
    search,
    update,
};

pub trait Node: Sized + Copy {}

impl<T> Node for T where T: Sized + Copy {}

pub trait Nodes {
    type Node: Node;

    /// The size of fenwick tree.
    fn nodes(&self) -> usize;
}

impl<T: Node> Nodes for [T] {
    type Node = T;
    #[inline]
    fn nodes(&self) -> usize {
        self.len() - 1 // self[0] is a dummy node
    }
}

impl<T: ?Sized + Nodes> Nodes for &T {
    type Node = <T as Nodes>::Node;
    fn nodes(&self) -> usize {
        <T as Nodes>::nodes(self)
    }
}

/// Builds a fenwick tree.
pub fn build<T: Node + AddAssign>(tr: &mut [T]) {
    assert!(!tr.is_empty());

    for i in 1..tr.len() {
        let j = index::next_for_update(i);
        if j < tr.len() {
            tr[j] += tr[i];
        }
    }
}

/// Resets a fenwick tree.
pub fn reset<T: Node + SubAssign>(tr: &mut [T]) {
    assert!(!tr.is_empty());

    for i in (1..tr.len()).rev() {
        let j = index::next_for_update(i);
        if j < tr.len() {
            tr[j] -= tr[i];
        }
    }
}

pub fn push<T: Node + AddAssign>(bit: &mut Vec<T>, mut x: T) {
    assert!(!bit.is_empty());

    // `bit.nodes()+1` points to the index to which `x` belongs when pushed
    for i in children(bit.nodes() + 1) {
        x += bit[i];
    }
    bit.push(x);
}

pub fn pop<T: Node + SubAssign>(bit: &mut Vec<T>) -> Option<T> {
    // tree[0] is dummy value, popping it doesn't make sense.
    (bit.len() > 1).then(|| {
        let mut x = bit.pop().expect("len > 1");
        for i in children(bit.nodes() + 1) {
            x -= bit[i];
        }
        x
    })
}

mod index {
    use core::iter::{
        Successors,
        successors,
    };
    use core::ops::{
        Add,
        Sub,
    };

    use bitcomp_core::Word;

    // The next node to be updated can be found by adding the node size `n.lsb()`.
    #[inline]
    pub(crate) fn next_for_update<T: Word + Add>(i: T) -> <T as Add>::Output {
        i + i.lsb()
    }

    // The next node to be queried can be found by subtracting the node size `n.lsb()`.
    #[inline]
    pub(crate) fn next_for_prefix<T: Word + Sub>(i: T) -> <T as Sub>::Output {
        i - i.lsb()
    }

    /// # Examples
    ///
    /// ```
    /// for i in fenwicktree::prefix(7) {}
    /// ```
    pub fn prefix(i: usize) -> Successors<usize, fn(&usize) -> Option<usize>> {
        fn next_index(&i: &usize) -> Option<usize> {
            let x = next_for_prefix(i);
            (x > 0).then_some(x)
        }

        // for x := i; x > 0; x -= lsb(x)
        successors((i > 0).then_some(i), next_index)
    }

    pub fn children(i: usize) -> impl Iterator<Item = usize> {
        // The number of children that belongs to the node at `i`, including `i`.
        let nb = i.lsb();

        // Yields `nb`s of each children of `i`. Not an index itself.
        let next_nb = move |&x: &usize| {
            let x = x << 1;
            (x < nb).then_some(x)
        };

        // Maps children's `nb`s to node indices
        let to_index = move |d: usize| i - d;

        successors((nb > 1).then_some(1), next_nb).map(to_index)
    }

    pub fn update(i: usize, nodes: usize) -> impl Iterator<Item = usize> {
        let next_index = move |&i: &usize| -> Option<usize> {
            let x = next_for_update(i);
            (x <= nodes).then_some(x)
        };

        // for x := i; x <= nodes; x += lsb(x)
        successors((i > 0).then_some(i), next_index)
    }

    pub fn search(nodes: usize) -> impl Iterator<Item = usize> {
        // for x := m.msb(); x > 0; x >>= 1
        successors((nodes > 0).then(|| nodes.msb()), |&i| {
            let x = i >> 1;
            (x > 0).then_some(x)
        })
    }
}

pub trait Prefix<S>: Nodes {
    fn sum(&self, index: usize) -> S;
}

pub trait Incr<N>: Nodes {
    /// Corresponds to `T[i] += delta` in `[T]`.
    fn incr(&mut self, i: usize, delta: N);
}

pub trait Decr<N>: Nodes {
    /// Corresponds to `T[i] -= delta` in `[T]`.
    fn decr(&mut self, i: usize, delta: N);
}

pub trait LowerBound<S>: Nodes {
    /// Finds the lowest idnex `i` that satisfies `sum(i) >= threshold`.
    fn lower_bound(&self, threshold: S) -> usize;
}

impl<T: Node, S: Sum<Self::Node>> Prefix<S> for [T] {
    #[inline]
    fn sum(&self, index: usize) -> S {
        prefix(index).map(|i| self[i]).sum()
    }
}

impl<T, U> LowerBound<U> for [T]
where
    T: Node + PartialOrd<U>,
    U: Word + SubAssign<T>,
{
    fn lower_bound(&self, mut w: U) -> usize {
        assert!(!self.is_empty());

        if !w.any() {
            return 0;
        }

        let mut i = 0;
        search(self.nodes()).for_each(|d| {
            if let Some(v) = self.get(i + d).copied()
                && v < w
            {
                w -= v;
                i += d;
            }
        });

        i + 1
    }
}

impl<T, U> Incr<U> for [T]
where
    T: Node + AddAssign<U>,
    U: Copy,
{
    #[inline]
    fn incr(&mut self, i: usize, delta: U) {
        update(i, self.nodes()).for_each(|p| self[p] += delta)
    }
}

impl<T, U> Decr<U> for [T]
where
    T: Node + SubAssign<U>,
    U: Copy,
{
    #[inline]
    fn decr(&mut self, i: usize, delta: U) {
        update(i, self.nodes()).for_each(|p| self[p] -= delta)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Complement<'a, T: ?Sized, U = u64> {
    inner:     &'a T,
    max_bound: U,
}

#[inline]
pub fn complement<T: ?Sized, U>(inner: &T, max_bound: U) -> Complement<'_, T, U> {
    Complement { inner, max_bound }
}

impl<'a, T: Node, U> Nodes for Complement<'a, [T], U> {
    type Node = T;
    #[inline]
    fn nodes(&self) -> usize {
        self.inner.nodes()
    }
}

impl<'a, T: Node> Prefix<u64> for Complement<'a, [T], u64>
where
    [T]: Prefix<u64>,
{
    #[inline]
    fn sum(&self, i: usize) -> u64 {
        (self.max_bound * i as u64) - self.inner.sum(i)
    }
}

// impl<'a, T> Iterator for iter::Prefix<Complement<'a, [T]>>
// where
//     T: Copy,
// {
//     type Item = T;
//     #[inline]
//     fn next(&mut self) -> Option<Self::Item> {
//         self.index.next().map(|i| self.data.inner[i])
//     }
// }

impl<'a, T> LowerBound<u64> for Complement<'a, [T], u64>
where
    T: Node,
    u64: Sub<T, Output = u64>,
{
    fn lower_bound(&self, mut w: u64) -> usize {
        let bit = self.inner;
        let max = self.max_bound;
        assert!(!bit.is_empty());
        if w == 0 {
            return 0;
        }

        let mut i = 0;
        // The size of the segment is halved for each step.
        for d in search(bit.nodes()) {
            if let Some(&v) = bit.get(i + d) {
                let v: u64 = max * (d as u64) - v;
                if v < w {
                    w -= v;
                    i += d; // move to right
                }
            }
        }
        i + 1
    }
}
