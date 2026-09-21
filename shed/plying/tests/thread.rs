#![allow(missing_docs)]

use plying::{
    Fold,
    StepFn,
};

mod helper;
use helper::*;

#[test]
fn check_clone() {
    fn check<T: Clone>(_: T) {}
    check(conj::<i32>());
    check(conj::<i32>().map(pow2));
    check(conj::<i32>().map(pow2).filter(even::<i32>));
    check(conj::<i32>().map(pow2).filter(even::<i32>).take(10));
}

#[test]
fn check_send() {
    fn check<T: Send>(_: T) {}
    check(conj::<i32>());
    check(conj::<i32>().map(pow2));
    check(conj::<i32>().map(pow2).filter(even::<i32>));
    check(conj::<i32>().map(pow2).filter(even::<i32>).take(10));
}

#[test]
fn thread_scope_fold() {
    let data = [1, 2, 3, 4, 5, 6];
    let r = sum::<_, i32>().par(sum::<_, i32>().map(mul3)).fold(data.chunks(3));
    assert_eq!(r.unwrap(), 63);
    let r = sum::<_, i32>().par(sum::<_, i32>().map(mul3)).fold(data.chunks(7));
    assert_eq!(r.unwrap(), 63);
    let r = sum::<_, usize>().par(count().map(mul3)).fold(data.chunks(4));
    assert_eq!(r.unwrap(), 6);
}
