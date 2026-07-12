include!(concat!(env!("OUT_DIR"), "/rrr63.rs"));

mod rrr63_test {
    use quickcheck_macros::quickcheck;

    use super::*;

    #[test]
    fn class_values() {
        assert_eq!(CLASS_SIZE, 6);
    }

    #[quickcheck]
    fn encode_decode(b: u64) -> bool {
        let b = b & ((1 << 63) - 1);
        let (c, o) = encode(b);
        let decoded = decode(c, o);
        b == decoded
    }
}
