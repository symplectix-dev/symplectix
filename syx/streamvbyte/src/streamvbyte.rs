#![allow(missing_docs)]
//! stream vbyte: https://arxiv.org/pdf/1709.08990.pdf

use byteorder::{
    ByteOrder,
    LittleEndian,
};

#[cfg(test)]
mod streamvbyte_test;

pub fn encode(buf: &mut [u8], ctl: &mut [u8], vs: &[u32]) -> usize {
    // assert_eq!(ctl.len(), (vs.len() + 3) / 4);
    assert_eq!(ctl.len(), vs.len().div_ceil(4));

    let mut encoded = 0;
    // FIXME: use exact_chunks
    for (i, chunk) in vs.chunks(4).enumerate() {
        let num0 = chunk[0];
        let num1 = chunk[1];
        let num2 = chunk[2];
        let num3 = chunk[3];

        let len0 = encode_scalar(&mut buf[encoded..], num0);
        let len1 = encode_scalar(&mut buf[encoded + len0..], num1);
        let len2 = encode_scalar(&mut buf[encoded + len0 + len1..], num2);
        let len3 = encode_scalar(&mut buf[encoded + len0 + len1 + len2..], num3);

        ctl[i] = ((len0 - 1) | (len1 - 1) << 2 | (len2 - 1) << 4 | (len3 - 1) << 6) as u8;
        encoded += len0 + len1 + len2 + len3;
    }
    encoded
}

pub fn encode_scalar(buf: &mut [u8], n: u32) -> usize {
    let len = std::cmp::max(1, 4 - n.leading_zeros() as usize / 8);
    let mut tmp = [0; 4];
    LittleEndian::write_u32(&mut tmp, n);
    buf[..len].copy_from_slice(&tmp[..len]);
    len
}

#[cfg(test)]
fn decode_scalar(buf: &[u8]) -> u32 {
    assert!(buf.len() <= 4);
    let mut tmp = [0u8; 4];
    tmp[..buf.len()].copy_from_slice(buf);
    LittleEndian::read_u32(&tmp[..])
}
