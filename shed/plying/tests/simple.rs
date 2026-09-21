#![allow(missing_docs)]

use std::iter::{
    empty,
    once,
};
use std::ops::ControlFlow::*;
use std::rc::Rc;

use plying::{
    Fold,
    StepFn,
};

mod helper;
use helper::*;

#[test]
fn test_map() {
    assert_eq!(vec![0; 0], conj().map(pow2).fold(empty::<i32>()));
    assert_eq!(vec![81], conj().map(pow2).fold(once::<i32>(9)));
    assert_eq!(vec![1, 4, 9], conj().map(pow2).fold([1, 2, 3]));
    assert_eq!(vec![3, 6, 9], conj().map(mul3).fold([1, 2, 3]));
    assert_eq!(vec![3, 6, 9], conj().map(mul3).fold(&[1, 2, 3]));
}

#[test]
fn test_filter() {
    assert_eq!(vec![0; 0], conj().filter(even).fold(empty::<i32>()));
    assert_eq!(vec![0; 0], conj().filter(even).fold(once(1)));
    assert_eq!(vec![0; 0], conj().filter(even).fold([1, 3, 5]));
    assert_eq!(vec![2, 4], conj().filter(even).fold(1..6));
    assert_eq!(vec![&0; 0], conj().filter(even).fold(&[1, 3, 5]));
}

#[test]
fn test_take() {
    assert_eq!(vec![0; 0], conj().take(0).fold(empty::<i32>()));
    assert_eq!(vec![0; 0], conj().take(0).fold(1..));
    assert_eq!(vec![0; 0], conj().take(1).fold(empty::<i32>()));
    assert_eq!(vec![1, 2], conj().take(2).fold(1..3));
    assert_eq!(vec![1, 2, 3], conj().take(3).fold(1..));
    assert_eq!(vec![&1, &2, &3], conj().take(5).fold(&[1, 2, 3]));

    let mut f = sum().take(2);
    assert_eq!(f.step(0, 1), Continue(1));
    let mut g = f.clone();
    assert_eq!(f.step(1, 2), Break(3));
    assert_eq!(g.step(1, 2), Break(3));
}

#[test]
fn test_map_filter_take() {
    assert_eq!(vec![6, 12], conj().filter(even).take(5).map(mul3).fold(1..));
    assert_eq!(vec![6, 12, 18, 24, 30], conj().take(5).filter(even).map(mul3).fold(1..));
    assert_eq!(vec![6, 12, 18, 24, 30], conj().take(5).map(mul3).filter(even).fold(1..));
    assert_eq!(vec![6, 12, 18, 24, 30], conj().map(mul3).take(5).filter(even).fold(1..));
    assert_eq!(vec![6, 12], conj().map(mul3).filter(even).take(5).fold(1..));
    assert_eq!(vec![6, 12], conj().filter(even).map(mul3).take(5).fold(1..));
}

#[test]
fn test_all() {
    assert!(all(even).fold(empty::<i32>()));
    assert!(all(even).fold([2, 4]));
    assert!(all(even).fold(&[2, 4]));
    assert!(!all(even).fold([1, 2]));
    assert!(!all(even).fold(&[1, 2]));
}

#[test]
fn test_any() {
    assert!(!any(even).fold(empty::<i32>()));
    assert!(any(even).fold([2, 4]));
    assert!(any(even).fold([1, 2]));
    assert!(any(even).fold(&[2, 4]));
    assert!(any(even).fold(&[1, 2]));
}

#[test]
fn test_count() {
    assert_eq!(0, count().fold(empty::<i32>()));
    assert_eq!(9, count().fold(1..10));
    assert_eq!(3, count().take(3).fold(1..));
}

#[test]
fn test_sum() {
    assert_eq!(0, sum().fold(empty::<i32>()));
    assert_eq!(1, sum().fold(once::<i32>(1)));
    assert_eq!(2, sum().fold(once::<i32>(2)));
    assert_eq!(18, sum().take(3).map(mul3).fold(1..));

    assert_eq!(10, sum::<i32, i32>().completing(|acc| acc + 10).fold(empty::<i32>()));
    assert_eq!(20, sum().completing(|acc: i32| acc + 10).take(4).fold(1..5));
    assert_eq!(20, sum().take(4).completing(|acc: i32| acc + 10).fold(1..5));

    assert_eq!("", sum::<_, String>().fold(empty::<&str>()));
    assert_eq!("1", sum::<_, String>().fold(once::<&str>("1")));
    assert_eq!("123", sum::<_, String>().fold(["foo1".trim_start_matches("foo"), "2", "3"]));
}

#[test]
fn test_nested_using() {
    assert_eq!(200, sum().beginning(|_| 10).beginning(|_| 100).fold(once(100)));
}

#[test]
fn test_seq() {
    assert_eq!((vec![1, 2, 3], vec![4, 5, 6, 7, 8]), conj().take(3).seq(conj().take(5)).fold(1..));
}

#[test]
fn test_zip() {
    let f = count().zip(sum());
    let g = sum().zip(count());
    let (a, b) = f.zip(g).fold(&[1, 2]);
    assert_eq!(a, (2, 3));
    assert_eq!(b, (3, 2));

    let f = count().zip(sum());
    let g = sum().zip(count());
    let (a, b) = f.seq(g).fold(&[1, 2]);
    assert_eq!(a, (2, 3));
    assert_eq!(b, (0, 0));

    let f = conj().take(3).map(mul3);
    let g = sum().take(2).map(mul3);
    let acc = f.zip(g).fold(&[1, 2, 3, 4]);
    assert_eq!(acc, (vec![3, 6, 9], 9));

    let f = conj().take(5);
    let g = conj().take(5);
    let acc = f.zip(g).fold(&[1, 2, 3]);
    assert_eq!(acc, (vec![&1, &2, &3], vec![&1, &2, &3]));
}

#[test]
fn test_zip_completing() {
    let f = sum().zip(count()).completing(|(sum, c): (i32, usize)| sum as f64 / c as f64);
    let avg = f.fold(&[1, 2]);
    assert_eq!(avg, 1.5);
}

#[test]
fn test_zip_rc() {
    fn to_rcs<I>(iterable: I) -> impl Iterator<Item = Rc<I::Item>>
    where
        I: IntoIterator,
    {
        iterable.into_iter().map(Rc::new)
    }

    let f = conj().take(3).map(pow2_rc);
    let g = sum().take(2).map(mul3_rc);
    let acc = f.zip(g).fold(to_rcs(1..10));
    assert_eq!(acc, (vec![1, 4, 9], 9));

    let f = count().zip(sum_rc());
    let g = count().zip(sum_rc());
    let ret = f.seq(g).fold(to_rcs([1, 2]));
    assert_eq!(ret, ((2, 3), (0, 0)));

    let f = count().zip(sum_rc());
    let g = count().zip(sum_rc());
    let (a, b) = f.zip(g).fold(to_rcs([1, 2]));
    assert_eq!(a, (2, 3));
    assert_eq!(b, (2, 3));

    let f = sum::<_, i32>().take(3).map(mul3_rc);
    let g = sum::<_, i32>().take(3).map(pow2_rc);
    let (fsum, gsum) = f.zip(g).fold(to_rcs(1..));
    assert_eq!(fsum, 18);
    assert_eq!(gsum, 14);
}
