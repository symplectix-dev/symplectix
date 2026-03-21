#![allow(missing_docs)]
#[macro_use(quickcheck)]
extern crate quickcheck_macros;

use std::{
    iter,
    ops,
};

use bitcomp_core::Word;
use fenwicktree::{
    Incr,
    LowerBound,
    Nodes,
    Prefix,
};

#[test]
fn children() {
    let indices = fenwicktree::children(1);
    assert_eq!(indices.collect::<Vec<usize>>(), []);

    let indices = fenwicktree::children(2);
    assert_eq!(indices.collect::<Vec<usize>>(), [1]);

    let indices = fenwicktree::children(4);
    assert_eq!(indices.collect::<Vec<usize>>(), [3, 2]);

    let indices = fenwicktree::children(7);
    assert_eq!(indices.collect::<Vec<usize>>(), []);

    let indices = fenwicktree::children(8);
    assert_eq!(indices.collect::<Vec<usize>>(), [7, 6, 4]);
}

#[test]
fn lower_bound() {
    {
        let mut tr: Vec<u32> = vec![0, 1, 0, 3, 5];
        fenwicktree::build(&mut tr);

        assert_eq!(4, tr.nodes());

        assert_eq!(0u32, tr.sum(0));
        assert_eq!(1u32, tr.sum(1));
        assert_eq!(1u32, tr.sum(2));
        assert_eq!(4u32, tr.sum(3));
        assert_eq!(9u32, tr.sum(4));

        assert_eq!(tr.lower_bound(1), 1);
        assert_eq!(tr.lower_bound(3), 3);
        assert_eq!(tr.lower_bound(4), 3);
        assert_eq!(tr.lower_bound(5), 4);
    }

    {
        let mut tr: Vec<u32> = vec![0, 0, 1, 0, 0, 3, 0, 2, 4, 2];
        fenwicktree::build(&mut tr);

        assert_eq!(9, tr.nodes());

        assert_eq!(0u32, tr.sum(0));
        assert_eq!(0u32, tr.sum(1));
        assert_eq!(1u32, tr.sum(2));
        assert_eq!(1u32, tr.sum(3));
        assert_eq!(1u32, tr.sum(4));

        assert_eq!(tr.lower_bound(1), 2);
        assert_eq!(tr.lower_bound(4), 5);
        assert_eq!(tr.lower_bound(5), 7);
        assert_eq!(tr.lower_bound(10), 8);
        assert_eq!(tr.lower_bound(11), 9);
        assert_eq!(tr.lower_bound(12), 9);
    }
}

#[test]
fn next_index_for_prefix() {
    let indices = [
        0b_0110_1110_1010_1101_0000, // 453328
        0b_0110_1110_1010_1100_0000, // 453312
        0b_0110_1110_1010_1000_0000, // 453248
        0b_0110_1110_1010_0000_0000, // 453120
        0b_0110_1110_1000_0000_0000, // 452608
        0b_0110_1110_0000_0000_0000, // 450560
        0b_0110_1100_0000_0000_0000, // 442368
        0b_0110_1000_0000_0000_0000, // 425984
        0b_0110_0000_0000_0000_0000, // 393216
        0b_0100_0000_0000_0000_0000, // 262144
    ];

    assert_eq!(fenwicktree::prefix(indices[0]).collect::<Vec<_>>(), &indices[0..]);
}

#[test]
fn prefix() {
    let mut indices = fenwicktree::prefix(0);
    assert_eq!(indices.next(), None);

    let mut indices = fenwicktree::prefix(3);
    assert_eq!(indices.next(), Some(3));
    assert_eq!(indices.next(), Some(2));
    assert_eq!(indices.next(), None);

    let mut indices = fenwicktree::prefix(7);
    assert_eq!(indices.next(), Some(7));
    assert_eq!(indices.next(), Some(6));
    assert_eq!(indices.next(), Some(4));
    assert_eq!(indices.next(), None);

    let mut indices = fenwicktree::prefix(8);
    assert_eq!(indices.next(), Some(8));
    assert_eq!(indices.next(), None);
}

fn build<T: Word + ops::AddAssign>(mut vec: Vec<T>) -> Vec<T> {
    vec.insert(0, T::empty()); // ensure vec.len() > 0
    fenwicktree::build(&mut vec);
    vec
}

#[quickcheck]
fn build_reset(vec: Vec<u32>) -> bool {
    let mut tr = build(vec.clone());
    fenwicktree::reset(&mut tr);
    tr[1..] == vec
}

#[quickcheck]
fn tree_by_incr(vec: Vec<u64>) -> bool {
    let mut tr = vec![0; vec.len() + 1];
    for (i, &d) in vec.iter().enumerate() {
        tr.incr(i + 1, d);
    }

    tr[0] == 0 && tr == build(vec)
}

#[quickcheck]
fn sum_0_is_always_zero(vec: Vec<u64>) -> bool {
    let tr = build(vec);
    let sum: u64 = tr.sum(0);
    sum == 0
}

#[quickcheck]
fn sum_x_eq_vec_sum(vec: Vec<u64>) -> bool {
    let tr = build(vec.clone());
    (0..=tr.nodes()).all(|i| {
        let sum: u64 = tr.sum(i);
        sum == vec[..i].iter().sum()
    })
}

// It takes too long to complete the test when using `Vec<u64>`.
#[quickcheck]
fn lower_bound_sum(vec: Vec<u16>) -> bool {
    let tr = build(vec.clone());
    (0..=vec.iter().sum::<u16>()).all(|w| {
        let i = tr.lower_bound(w);
        let sum: u64 = fenwicktree::prefix(i).map(|i| Into::<u64>::into(tr[i])).sum();
        sum >= w.into()
    })
}

#[quickcheck]
fn pop_all_then_push_all(vec: Vec<u64>) -> bool {
    let tr = build(vec.clone());

    let mut cloned = tr.clone();
    let mut popped = iter::from_fn(|| fenwicktree::pop(&mut cloned)).collect::<Vec<_>>();

    popped.reverse();
    assert_eq!(vec, popped);

    for x in popped.into_iter() {
        fenwicktree::push(&mut cloned, x);
    }

    tr == cloned
}

#[test]
fn next_index_for_update() {
    let indices = [
        0b_0000_0110_1110_1010_1101_0001, // 453329
        0b_0000_0110_1110_1010_1101_0010, // 453330
        0b_0000_0110_1110_1010_1101_0100, // 453332
        0b_0000_0110_1110_1010_1101_1000, // 453336
        0b_0000_0110_1110_1010_1110_0000, // 453344
        0b_0000_0110_1110_1011_0000_0000, // 453376
        0b_0000_0110_1110_1100_0000_0000, // 453632
        0b_0000_0110_1111_0000_0000_0000, // 454656
        0b_0000_0111_0000_0000_0000_0000, // 458752
        0b_0000_1000_0000_0000_0000_0000, // 524288
        0b_0001_0000_0000_0000_0000_0000, // 1048576
        0b_0010_0000_0000_0000_0000_0000, // 2097152
    ];

    assert_eq!(
        fenwicktree::update(indices[0], indices[indices.len() - 1] + 1).collect::<Vec<_>>(),
        &indices[0..]
    );
}

#[test]
fn update() {
    let mut indices = fenwicktree::update(0, 8);
    assert_eq!(indices.next(), None);

    let mut indices = fenwicktree::update(1, 8);
    assert_eq!(indices.next(), Some(1));
    assert_eq!(indices.next(), Some(2));
    assert_eq!(indices.next(), Some(4));
    assert_eq!(indices.next(), Some(8));
    assert_eq!(indices.next(), None);

    let mut indices = fenwicktree::update(3, 8);
    assert_eq!(indices.next(), Some(3));
    assert_eq!(indices.next(), Some(4));
    assert_eq!(indices.next(), Some(8));
    assert_eq!(indices.next(), None);

    let mut indices = fenwicktree::update(7, 8);
    assert_eq!(indices.next(), Some(7));
    assert_eq!(indices.next(), Some(8));
    assert_eq!(indices.next(), None);
}
