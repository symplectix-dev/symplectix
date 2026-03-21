#![allow(missing_docs)]
use bitcomp_core::{
    BitsMut,
    Mask,
    Page,
};

#[test]
fn and() {
    let mut a = Page::<[u64; 4]>::new();
    a.set1(0);
    a.set1(1);
    a.set1(2);
    a.set1(128);
    let mut b = Page::<[u64; 4]>::new();
    b.set1(1);
    b.set1(2);
    b.set1(3);
    b.set1(192);
    Mask::and(&mut a, &b);
    assert_eq!(a.as_ref().unwrap(), &[0b_0110, 0, 0, 0]);
    let mut c = Page::<[u64; 4]>::new();
    Mask::and(&mut b, &c);
    assert_eq!(b.as_ref(), None);
    Mask::and(&mut c, &a);
    assert_eq!(c.as_ref(), None);
}

#[test]
fn or() {
    let mut a = Page::<[u64; 4]>::new();
    a.set1(0);
    a.set1(1);
    a.set1(2);
    a.set1(128);
    let mut b = Page::<[u64; 4]>::new();
    b.set1(1);
    b.set1(2);
    b.set1(3);
    b.set1(192);
    Mask::or(&mut a, &b);
    assert_eq!(a.as_ref().unwrap(), &[0b_1111, 0, 1, 1]);
    let mut c = Page::<[u64; 4]>::new();
    Mask::or(&mut c, &b);
    assert_eq!(c.as_ref().unwrap(), &[0b_1110, 0, 0, 1]);
}

#[test]
fn not() {
    let mut a = Page::<[u64; 4]>::new();
    a.set1(0);
    a.set1(1);
    a.set1(2);
    a.set1(128);

    let mut b = Page::<[u64; 4]>::new();
    b.set1(1);
    b.set1(2);
    b.set1(3);
    b.set1(192);

    Mask::not(&mut a, &b);
    assert_eq!(a.as_ref().unwrap(), &[0b_0001, 0, 1, 0]);

    let mut c = Page::<[u64; 4]>::new();
    Mask::not(&mut c, &a);
    assert_eq!(c.as_ref(), None);
}

#[test]
fn xor() {
    let mut a = Page::<[u64; 4]>::new();
    a.set1(0);
    a.set1(1);
    a.set1(2);
    a.set1(128);
    let mut b = Page::<[u64; 4]>::new();
    b.set1(1);
    b.set1(2);
    b.set1(3);
    b.set1(192);
    Mask::xor(&mut a, &b);
    assert_eq!(a.as_ref().unwrap(), &[0b_1001, 0, 1, 1]);
    let mut c = Page::<[u64; 4]>::new();
    Mask::xor(&mut c, &a);
    assert_eq!(c.as_ref().unwrap(), &[0b_1001, 0, 1, 1]);
}
