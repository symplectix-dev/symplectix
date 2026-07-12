include!(concat!(env!("OUT_DIR"), "/rrr4.rs"));

#[cfg(test)]
mod rrr04_test {
    use quickcheck_macros::quickcheck;

    use super::*;

    #[test]
    fn class_values() {
        assert_eq!(CLASS_SIZE, 3);
    }

    fn check(mut bits: u8, class: u8, offset: u8) {
        bits &= (1 << 4) - 1;
        let (c, o) = encode(bits);
        assert_eq!(c, class, "class does not match");
        assert_eq!(o, offset, "offset does not match");
        let decoded = decode(c, o);
        assert_eq!(bits, decoded, "decoded does not match");
    }

    #[test]
    fn class_0() {
        check(0b_0000, 0, 0);
    }

    #[test]
    fn class_1() {
        check(0b_0001, 1, 0);
        check(0b_0010, 1, 1);
        check(0b_0100, 1, 2);
        check(0b_1000, 1, 3);
    }

    #[test]
    fn class_2() {
        check(0b_0011, 2, 0);
        check(0b_0101, 2, 1);
        check(0b_0110, 2, 2);
        check(0b_1001, 2, 3);
        check(0b_1010, 2, 4);
        check(0b_1100, 2, 5);
    }

    #[test]
    fn class_3() {
        check(0b_0111, 3, 0);
        check(0b_1011, 3, 1);
        check(0b_1101, 3, 2);
        check(0b_1110, 3, 3);
    }

    #[test]
    fn class_4() {
        check(0b_1111, 4, 0);
    }

    #[quickcheck]
    fn encode_decode(b: u8) -> bool {
        let b = b & ((1 << 4) - 1);
        let (c, o) = encode(b);
        let decoded = decode(c, o);
        b == decoded
    }
}
