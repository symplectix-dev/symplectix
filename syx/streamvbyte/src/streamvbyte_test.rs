use quickcheck_macros::quickcheck;

use super::{
    decode_scalar,
    encode_scalar,
};

#[quickcheck]
fn scalar_encode_decode(v: u32) {
    let mut buf = vec![0u8; 10];
    let n = encode_scalar(&mut buf, v);
    let m = decode_scalar(&buf[..n]);
    assert_eq!(v, m);
}

fn test_debug(v: u32) {
    let mut buf = [0; 4];
    let n = encode_scalar(&mut buf[..], v);
    print!("[");
    for byte in &buf[..n] {
        print!(" {:08b}", byte);
    }
    println!(" ]");
}

#[test]
fn not_implemented_yet() {
    test_debug(0);
    test_debug((1 << 16) - 1);
    test_debug((1 << 16) + 3);
    test_debug(u32::MAX);
    test_debug(1209387123);
}
