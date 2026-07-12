mod primitive;

#[test]
fn default_value() {
    use std::borrow::Cow;

    use crate::bit_set;

    let words = bit_set::Words::<Box<[u64]>>::default();
    assert!(words.as_ref().is_none());

    let words = bit_set::Words::<Cow<[u64]>>::default();
    assert!(words.as_ref().is_none());

    let zero = <u64 as bit_set::Word>::ZERO;
    assert_eq!(zero, <u64 as Default>::default());
}

macro_rules! gentest {
    ($Type:tt, $Words:ty) => {
        use crate::bit_set::*;

        type BlockTy = $Words;
        type RawVecType = $Type<BlockTy>;

        // macro_rules! associative {
        //     ($x: expr,$y: expr,$z: expr,$fn: ident) => {{
        //         let r1 = $x.$fn($y).$fn($z).into_iter().collect::<RawVecType>();
        //         let r2 = $x.$fn($y.$fn($z)).into_iter().collect::<RawVecType>();
        //         r1 == r2
        //     }};
        // }

        // macro_rules! commutative {
        //     ($x: expr,$y: expr,$fn: ident) => {{
        //         let r1 = $x.$fn($y).into_iter().collect::<RawVecType>();
        //         let r2 = $y.$fn($x).into_iter().collect::<RawVecType>();
        //         r1 == r2
        //     }};
        // }

        // quickcheck! {
        // fn identity(vec: Vec<u64>) -> bool {
        //     let bv = &RawVecType::from(&vec[..]);
        //     let r1 = bv.get(0..990_000).into_iter().collect::<RawVecType>();
        //     let r2 =
        // bv.get(0..660_000).or(bv.get(660_000..990_000)).into_iter().collect::<RawVecType>();
        //     let r3 = {
        //         let x = bv.get(0..330_000);
        //         let y = bv.get(330_000..660_000);
        //         let z = bv.get(660_000..990_000);
        //         x.or(y).or(z).into_iter().collect::<RawVecType>()
        //     };
        //     r1 == r2 && r2 == r3
        // }

        // fn associative(vec1: Vec<u64>, vec2: Vec<u64>, vec3: Vec<u64>) -> bool {
        //     let b1 = &RawVecType::from(&vec1[..]);
        //     let b2 = &RawVecType::from(&vec2[..]);
        //     let b3 = &RawVecType::from(&vec3[..]);
        //     let r1 = associative!(b1, b2, b3, and);
        //     let r2 = associative!(b1, b2, b3, or);
        //     let r3 = associative!(b1, b2, b3, xor);
        //     r1 && r2 && r3
        // }

        // fn commutative(vec1: Vec<u64>, vec2: Vec<u64>) -> bool {
        //     let b1 = &RawVecType::from(&vec1[..]);
        //     let b2 = &RawVecType::from(&vec2[..]);
        //     let r1 = commutative!(b1, b2, and);
        //     let r2 = commutative!(b1, b2, or);
        //     let r3 = commutative!(b1, b2, xor);
        //     r1 && r2 && r3
        // }
        // }

        #[test]
        fn ops() {
            let mut bits: RawVecType = Default::default();

            assert!(!bits.insert(1000));
            assert!(!bits.insert(2000));
            assert!(!bits.insert(300000));
            assert!(bits.access(1000));
            assert!(bits.access(2000));
            assert!(bits.access(300000));

            assert_eq!(bits.count1(), 3);
            assert_eq!(bits.rank1(2000), 1);
            assert_eq!(bits.rank1(300000), 2);

            assert_eq!(bits.rank1(bits.size()), bits.count1());

            assert_eq!(bits.select1(0), 1000);
            assert_eq!(bits.select1(1), 2000);
            assert_eq!(bits.select1(2), 300000);
            assert_eq!(bits.select0(0), 0);
            assert_eq!(bits.select0(1), 1);
            assert_eq!(bits.select0(2), 2);

            assert!(bits.remove(1000));
            assert!(bits.remove(2000));
            assert!(!bits.access(1000));
            assert!(!bits.access(2000));
            assert!(bits.access(300000));
        }

        // quickcheck! {
        //     fn dict(vec: Vec<u64>) -> bool {
        //         let bits = RawVecType::from(&vec[..]);

        //         for i in 0..bits.count1() {
        //             if bits.rank1(bits.select1(i)) != i {
        //                 return false;
        //             }
        //         }
        //         for i in 0..bits.count1() {
        //             if bits.select1(bits.rank1(i)) < i {
        //                 return false;
        //             }
        //         }
        //         true
        //     }
        // }
    };
}

mod entry {
    mod u8 {
        gentest!(RawVec, Entry<Box<[u8]>>);
    }
    mod u16 {
        gentest!(RawVec, Entry<Box<[u16]>>);
    }
    mod u32 {
        gentest!(RawVec, Entry<Box<[u32]>>);
    }
    mod u64 {
        gentest!(RawVec, Entry<Box<[u64]>>);
    }
    mod u128 {
        gentest!(RawVec, Entry<Box<[u128]>>);
    }
    mod usize {
        gentest!(RawVec, Entry<Box<[usize]>>);
    }
}

mod words {
    use crate::bit_set::ops::*;
    use crate::bit_set::*;

    mod u8 {
        gentest!(RawVec, Words<Box<[u8]>>);
    }
    mod u16 {
        gentest!(RawVec, Words<Box<[u16]>>);
    }
    mod u32 {
        gentest!(RawVec, Words<Box<[u32]>>);
    }
    mod u64 {
        gentest!(RawVec, Words<Box<[u64]>>);
    }
    mod u128 {
        gentest!(RawVec, Words<Box<[u128]>>);
    }
    mod usize {
        gentest!(RawVec, Words<Box<[usize]>>);
    }

    #[test]
    fn access() {
        let bv: Words<Box<[u64]>> = Words::from(vec![1u64, 0b10101100001, 0b0000100000]);
        assert!(bv.access(0));
        assert!(bv.access(64));
        assert!(bv.access(70));
    }

    #[test]
    fn count() {
        let bv: Words<Box<[u64]>> = Words::from(vec![0u64, 0b10101100000, 0b0000100000]);
        assert_eq!(bv.count1(), 5);
    }

    #[test]
    fn rank() {
        let bv: Words<Box<[u8]>> = Words::from(vec![0u8, 0b0110_0000, 0b0001_0000]);
        assert_eq!(bv.rank1(10), 0);
        assert_eq!(bv.rank1(14), 1);
        assert_eq!(bv.rank1(15), 2);
        assert_eq!(bv.rank1(16), 2);
        assert_eq!(bv.rank1(10), 10 - bv.rank0(10));
        assert_eq!(bv.rank1(bv.size()), bv.count1());
    }

    #[test]
    fn select() {
        let bv: Words<Box<[u64]>> = Words::from(vec![0b_0000, 0b_0100, 0b_1001]);
        assert_eq!(bv.select1(0), 66);
        assert_eq!(bv.select1(1), 128);
        assert_eq!(bv.select1(2), 131);

        let bv: Words<Box<[u8]>> = Words::from(vec![0b_11110111, 0b_11111110, 0b_10010011]);
        assert_eq!(bv.select0(0), 3);
        assert_eq!(bv.select0(1), 8);
        assert_eq!(bv.select0(2), 18);
    }

    #[test]
    fn insert() {
        let mut bv: Words<Box<[u64]>> = Words::from(vec![0u64, 0b10101100000, 0b0000100000]);
        assert!(!bv.insert(0));
        assert!(bv.insert(0));
        assert!(bv.access(0));
    }

    #[test]
    fn remove() {
        let mut bv: Words<Box<[u64]>> = Words::from(vec![0u64, 0b10101100001, 0b0000100000]);
        assert!(bv.remove(64));
        assert!(!bv.remove(64));
        assert!(!bv.access(64));
    }

    type Ty<'a> = Words<Cow<'a, [u64]>>;

    #[test]
    fn bitand() {
        let check = |s1, s2, count| {
            let slice: Ty = s1 & s2;
            assert!(slice.len() == 1024 || slice.is_empty());
            let block = BoxWords::from(slice);
            assert_eq!(block.count1(), count);
        };

        check(Ty::empty(), Ty::empty(), 0);
        check(Ty::splat(!0), Ty::splat(!0), 1 << 16);
        check(Ty::empty(), Ty::splat(!0), 0);
        check(Ty::splat(!0), Ty::from(vec![]), 0);
        check(Ty::from(vec![1u64, 1, 1, 1, 1]), Ty::empty(), 0);
        check(Ty::empty(), Ty::from(vec![1u64, 1, 1, 1, 1]), 0);

        check(Ty::from(vec![1u64, 1, 1, 1, 1]), Ty::from(vec![0u64, 0, 0, 0, 0]), 0);
        check(Ty::from(vec![1u64, 1, 1, 1, 1]), Ty::from(vec![0u64, 1, 0, 1]), 2);
        check(Ty::from(vec![1u64, 1, 1]), Ty::from(vec![1u64, 0, 1, 0, 0]), 2);
    }

    #[test]
    fn bitor() {
        let check = |w1, w2, count| {
            let slice: Ty = w1 | w2;
            assert!(slice.len() == 1024 || slice.is_empty());
            let block = BoxWords::from(slice);
            assert_eq!(block.count1(), count);
        };

        check(Ty::empty(), Ty::empty(), 0);
        check(Ty::splat(!0), Ty::splat(!0), 1 << 16);
        check(Ty::empty(), Ty::splat(!0), 1 << 16);
        check(Ty::splat(!0), Ty::from(vec![]), 1 << 16);
        check(Ty::from(vec![1u64, 1, 1, 1, 1]), Ty::from(vec![]), 5);
        check(Ty::from(vec![1u64, 1, 1, 1, 1]), Ty::empty(), 5);
        check(Ty::empty(), Ty::from(vec![1u64, 1, 1, 1, 1]), 5);

        check(Ty::from(vec![1u64, 1, 1, 1, 1]), Ty::from(vec![0u64, 0, 0, 0, 0]), 5);
        check(Ty::from(vec![1u64, 1, 1, 1, 1]), Ty::from(vec![0u64, 1, 0, 1]), 5);
        check(Ty::from(vec![1u64, 1, 1]), Ty::from(vec![1u64, 0, 1, 0, 0]), 3);
    }

    #[test]
    fn bitxor() {
        let check = |w1, w2, count| {
            let slice: Ty = w1 ^ w2;
            assert!(slice.len() == 1024 || slice.is_empty());
            let block = BoxWords::from(slice);
            assert_eq!(block.count1(), count);
        };

        check(Ty::empty(), Ty::empty(), 0);
        check(Ty::splat(!0), Ty::splat(!0), 0);
        check(Ty::empty(), Ty::splat(!0), 1 << 16);
        check(Ty::splat(!0), Ty::from(vec![]), 1 << 16);
        check(Ty::from(vec![1u64, 1, 1, 1, 1]), Ty::from(vec![]), 5);
        check(Ty::from(vec![1u64, 1, 1, 1, 1]), Ty::empty(), 5);
        check(Ty::empty(), Ty::from(vec![1u64, 1, 1, 1, 1]), 5);

        check(Ty::from(vec![1u64, 1, 1, 1, 1]), Ty::from(vec![0u64, 0, 0, 0, 1]), 4);
        check(Ty::from(vec![1u64, 1, 1, 1, 1]), Ty::from(vec![0u64, 1, 0, 1]), 3);
        check(Ty::from(vec![1u64, 1, 1]), Ty::from(vec![1u64, 0, 1, 0, 0]), 1);
    }

    // #[test]
    // fn bitnot() {
    //     let check = |w1: Ty, count| {
    //         let slice: Ty = !w1;
    //         assert!(slice.len() == 1024 || slice.len() == 0);
    //         let block = BoxWords::from(slice);
    //         assert_eq!(block.count1(), count);
    //     };

    //     check(Ty::empty(), 1 << 16);
    //     check(Ty::splat(!0), 0);
    //     check(Ty::from(vec![]), 1 << 16);
    //     check(Ty::from(vec![0, 0, 0, 0, 0]), 1 << 16);
    //     check(Ty::from(vec![1, 1, 1, 1, 1]), (1 << 16) - 5);
    // }
}
