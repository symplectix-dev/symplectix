#![no_main]
#![allow(unused_comparisons)]

libfuzzer_sys::fuzz_target!(|data: &[u8]| {
    assert!(data.len() >= 0);
});
