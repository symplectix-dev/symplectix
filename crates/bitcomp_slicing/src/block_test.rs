use bitcomp_core::BitsMut;

use crate::block::Blocks;
use crate::consts::DENSE_BLOCK_BYTES;

fn set_header(header: &mut [u8], k: usize, index: u8, bits: u16) {
    header[k * 2] = index;
    header[k * 2 + 1] = (bits - 1) as u8;
}

fn set_sparse(data: &mut [u8]) {
    data[0] = 200;
    data[1] = 255;
}

fn set_dense(data: &mut [u8]) {
    for b in 0..80 {
        data.set1(b);
    }
}

#[test]
fn iterate_blocks() {
    const SPARSE_BLOCKS: usize = 1;
    const DENSE_BLOCKS: usize = 1;
    const NBLOCKS: usize = SPARSE_BLOCKS + DENSE_BLOCKS;

    let mut bytes = vec![0u8; NBLOCKS * 2 + SPARSE_BLOCKS * 2 + DENSE_BLOCKS * 32];
    let mut blocks = {
        let (header, data) = bytes.split_at_mut(NBLOCKS * 2);
        set_header(header, 0, 1, 2);
        set_header(header, 1, 255, 80);
        set_sparse(&mut data[..2]);
        set_dense(&mut data[2..]);
        Blocks::new(NBLOCKS * 2, bytes.as_slice())
    };

    assert_eq!(blocks.header.len(), NBLOCKS);

    let b = blocks.next().unwrap();
    assert!(b.is_sparse());
    assert_eq!(b.index, 1);
    assert_eq!(b.bits, 2);
    assert_eq!(b.data(), &[200, 255]);

    let b = blocks.next().unwrap();
    assert!(b.is_dense());
    assert_eq!(b.index, 255);
    assert_eq!(b.bits, 80);
    assert_eq!(b.data().len(), DENSE_BLOCK_BYTES);
    assert_eq!(&b.data()[..10], vec![0xff; 10]);
    assert_eq!(&b.data()[10..], vec![0; 22]);

    assert_eq!(blocks.header.remainder().len(), 0);
}
