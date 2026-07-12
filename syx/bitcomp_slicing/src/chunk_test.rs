use crate::BitVec;

#[test]
fn chunks_header_has_expected_size() {
    fn check(n: usize) {
        let bv = BitVec::with_chunks(n);
        let mut header = bv.chunks_header();
        assert_eq!(header.len(), n);
        for chunk in header.by_ref() {
            println!("{:?}", chunk);
        }
        // chunks header must not have remainder.
        assert_eq!(header.remainder().len(), 0);
    }

    check(0);
    check(1);
    check((1 << 8) + 1);
    check((1 << 9) + 1);
    check(1 << 10);
    check(1 << 16);
}
