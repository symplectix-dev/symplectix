#![allow(missing_docs)]
use compacts::{
    BitArray,
    WaveletMatrix,
};
use quickcheck::quickcheck;

quickcheck! {
    fn index_all(vec: Vec<u64>) -> bool {
        let mut xs = vec.clone();
        let wm = WaveletMatrix::<u64, BitArray<u64>>::from(&mut xs[..]);
        vec.iter().enumerate().all(|(i, v)| wm.get(i).unwrap() == *v)
    }
}
