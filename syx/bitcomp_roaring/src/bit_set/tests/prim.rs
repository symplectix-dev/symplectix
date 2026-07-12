use crate::bit_set::ops::*;

#[test]
fn access() {
    let slice = [1u64, 0b10101100001, 0b0000100000];
    assert!(slice.access(0));
    assert!(slice.access(70));
}

#[test]
fn count() {
    let slice = [0u64, 0b10101100000, 0b0000100000];
    assert_eq!(slice.count1(), 5);
}

#[test]
fn rank() {
    let slice = [0u8, 0b0110_0000, 0b0001_0000];
    assert_eq!(slice.rank1(10), 0);
    assert_eq!(slice.rank1(14), 1);
    assert_eq!(slice.rank1(15), 2);
    assert_eq!(slice.rank1(16), 2);
    assert_eq!(slice.rank1(10), 10 - slice.rank0(10)); // rank1(i) + rank0(i) == i
    assert_eq!(slice.rank1(slice.size()), slice.count1());
}

#[test]
fn select() {
    let w: u64 = 0b_0000_0100_1001_0000;
    assert_eq!(w.select1(0), 4);
    assert_eq!(w.select1(1), 7);
    assert_eq!(w.select1(2), 10);
    assert_eq!(w.rank1(w.select1(2)), 2);

    let w: u128 = (0b_00001100_u128 << 64) | 0b_00000100_u128;
    assert_eq!(w.select1(0), 2);
    assert_eq!(w.select1(1), 66);
    assert_eq!(w.select1(2), 67);

    let slice = [0b_0000_u64, 0b_0100, 0b_1001];
    assert_eq!(slice.select1(0), 66);
    assert_eq!(slice.select1(1), 128);
    assert_eq!(slice.select1(2), 131);

    let slice = [0b_11110111_u8, 0b_11111110, 0b_10010011];
    assert_eq!(slice.select0(0), 3);
    assert_eq!(slice.select0(1), 8);
    assert_eq!(slice.select0(2), 18);
}

#[test]
fn insert() {
    let mut slice = [0u64, 0b10101100000, 0b0000100000];
    assert!(!slice.insert(0));
    assert!(slice.insert(0));
    assert!(slice.access(0));
}

#[test]
fn remove() {
    let mut slice = [0u64, 0b10101100001, 0b0000100000];
    assert!(slice.remove(64));
    assert!(!slice.remove(64));
    assert!(!slice.access(64));
}
