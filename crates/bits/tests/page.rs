#![allow(missing_docs)]
use bits::{
    Bits,
    BitsMut,
    Block,
    Page,
};

#[test]
fn count1() {
    let b: Page<[u8; 3]> = Page::new();
    assert_eq!(b.count1(), 0);
    let b: Page<[u8; 3]> = Page::from([0, 1, 0]);
    assert_eq!(b.count1(), 1);
}

#[test]
fn count0() {
    let b: Page<[u8; 3]> = Page::new();
    assert_eq!(b.count0(), 24);
    let b: Page<[u8; 3]> = Page::from([0, 0, 0]);
    assert_eq!(b.count0(), 24);
}

#[test]
fn all() {
    let b: Page<[u8; 3]> = Page::new();
    assert_eq!(b.count0(), 24);
    let b: Page<[u8; 3]> = Page::from([0, 0, 0]);
    assert_eq!(b.count0(), 24);
}

#[test]
fn any() {
    let b: Page<[u8; 3]> = Page::new();
    assert!(!b.any());
    let b: Page<[u8; 3]> = Page::from([0, 0, 0]);
    assert!(!b.any());
    let b: Page<[u8; 3]> = Page::from([0, 1, 0]);
    assert!(b.any());
}

#[test]
fn bit() {
    let b: Page<[u8; 3]> = Page::new();
    assert!(!b.bit(8));
    let b: Page<[u8; 3]> = Page::from([0, 1, 0]);
    assert!(b.bit(8));
}

#[test]
fn word() {
    let b: Page<[u8; 3]> = Page::new();
    assert_eq!(b.word::<u8>(0, 3), 0b_000);
    let b: Page<[u8; 3]> = Page::from([1, 1, 1]);
    assert_eq!(b.word::<u8>(0, 3), 0b_001);
}

#[test]
fn rank1() {
    let b: Page<[u8; 3]> = Page::new();
    assert_eq!(b.rank1(..10), 0);
    let b: Page<[u8; 3]> = Page::from([0, 1, 0]);
    assert_eq!(b.rank1(..10), 1);
}

#[test]
fn rank0() {
    let b: Page<[u8; 3]> = Page::new();
    assert_eq!(b.rank0(..10), 10);
    let b: Page<[u8; 3]> = Page::from([0, 1, 0]);
    assert_eq!(b.rank0(..10), 9);
}

#[test]
fn select1_select0() {
    let b: Page<[u8; 3]> = Page::new();
    assert_eq!(b.select1(0), None);
    let b: Page<[u8; 3]> = Page::from([0, 1, 0]);
    assert_eq!(b.select1(0), Some(8));

    let b: Page<[u8; 3]> = Page::new();
    assert_eq!(b.select0(0), Some(0));
    assert_eq!(b.select0(100), None);
    let b: Page<[u8; 3]> = Page::from([0, 1, 0]);
    assert_eq!(b.select0(10), Some(11));

    let mut b = Page::<[u64; 8]>::empty();
    assert_eq!(b.select1(0), None);
    assert_eq!(b.select0(0), Some(0));
    assert_eq!(b.select0(Page::<[u64; 8]>::BITS - 1), Some(511));
    b.set1(1);
    b.set1(511);
    assert_eq!(b.select1(0), Some(1));
    assert_eq!(b.select1(1), Some(511));
    assert_eq!(b.select0(0), Some(0));
    assert_eq!(b.select0(1), Some(2));
}
