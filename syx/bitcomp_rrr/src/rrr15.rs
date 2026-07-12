include!(concat!(env!("OUT_DIR"), "/rrr15.rs"));

#[cfg(test)]
mod rrr15_test {
    use quickcheck_macros::quickcheck;
    use serde::{
        Deserialize,
        Serialize,
    };

    use super::*;
    use crate::test_helper;

    #[test]
    fn class_values() {
        assert_eq!(CLASS_SIZE, 4);
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct Comb16 {
        table: Vec<Vec<u16>>,
    }

    #[test]
    fn comb_table_16() {
        let comb: Comb16 = {
            let json = test_helper::read_json(16);
            let r = std::io::BufReader::new(json);
            serde_json::from_reader(r).expect("failed to deserialize a table")
        };
        assert_eq!(comb.table, COMB);
    }

    fn check(mut bits: u16, class: u8, offset: u16) {
        bits &= (1 << 15) - 1;
        let (c, o) = encode(bits);
        assert_eq!(c, class, "class does not match");
        assert_eq!(o, offset, "offset does not match");
        let decoded = decode(c, o);
        assert_eq!(bits, decoded, "decoded does not match");
    }

    #[test]
    fn class_0() {
        check(0b_0000_0000_0000_0000, 0, 0);
        check(0b_1000_0000_0000_0000, 0, 0);
    }

    #[test]
    fn class_1() {
        check(0b_0000_0000_0000_0001, 1, 0);
        check(0b_0000_0000_0000_0010, 1, 1);
        check(0b_0100_0000_0000_0000, 1, 14);
    }

    #[test]
    fn class_2() {
        check(0b_0000_0000_0000_0011, 2, 0);
        check(0b_0110_0000_0000_0000, 2, 104); // comb(15, 2) is 105
    }

    #[test]
    fn class_15() {
        check(0b_0111_1111_1111_1111, 15, 0);
        check(0b_1111_1111_1111_1111, 15, 0);
    }

    #[quickcheck]
    fn encode_decode(b: u16) -> bool {
        let b = b & ((1 << 15) - 1);
        let (c, o) = encode(b);
        let decoded = decode(c, o);
        b == decoded
    }
}
