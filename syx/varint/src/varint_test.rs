use quickcheck::quickcheck;

use super::*;

// The maximum length of a varint-encoded N-bit integer.
const MAX_VARINT_LEN16: usize = 3;
const MAX_VARINT_LEN32: usize = 5;
const MAX_VARINT_LEN64: usize = 10;

fn check_varint_16<T: Varint + PartialEq>(v: T) -> bool {
    let mut buf = vec![0u8; MAX_VARINT_LEN16];
    let n = Varint::encode(&v, &mut buf);
    let (d, m) = Varint::decode(&buf[..n]).unwrap();
    v == d && n == m
}

fn check_varint_32<T: Varint + PartialEq>(v: T) -> bool {
    let mut buf = vec![0u8; MAX_VARINT_LEN32];
    let n = Varint::encode(&v, &mut buf);
    let (d, m) = Varint::decode(&buf[..n]).unwrap();
    v == d && n == m
}

fn check_varint_64<T: Varint + PartialEq>(v: T) -> bool {
    let mut buf = vec![0u8; MAX_VARINT_LEN64];
    let n = Varint::encode(&v, &mut buf);
    let (d, m) = Varint::decode(&buf[..n]).unwrap();
    v == d && n == m
}

quickcheck! {
    fn varint_16(i: i16, u: u16) -> bool {
        check_varint_16(i) && check_varint_16(u)
    }
    fn varint_32(i: i32, u: u32) -> bool {
        check_varint_32(i) && check_varint_32(u)
    }
    fn varint_64(i: i64, u: u64) -> bool {
        check_varint_64(i) && check_varint_64(u)
    }
}
