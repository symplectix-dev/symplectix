include!(concat!(env!("OUT_DIR"), "/rrr31.rs"));

#[cfg(test)]
mod rrr31_test {
    use quickcheck_macros::quickcheck;
    use serde::{
        Deserialize,
        Serialize,
    };

    use super::*;
    use crate::test_helper;

    #[test]
    fn class_values() {
        assert_eq!(CLASS_SIZE, 5);
    }

    #[quickcheck]
    fn encode_decode(b: u32) -> bool {
        let b = b & ((1 << 31) - 1);
        let (c, o) = encode(b);
        let decoded = decode(c, o);
        b == decoded
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct Comb32 {
        table: Vec<Vec<u32>>,
    }

    #[test]
    fn comb_table_32() {
        let comb: Comb32 = {
            let json = test_helper::read_json(32);
            let r = std::io::BufReader::new(json);
            serde_json::from_reader(r).expect("failed to deserialize a table")
        };
        assert_eq!(comb.table, COMB);
    }
}
