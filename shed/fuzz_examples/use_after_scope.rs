#![no_main]

use libfuzzer_sys::{
    Corpus,
    fuzz_target,
};

fuzz_target!(|data: &[u8]| -> Corpus { do_fuzz(data) });

fn do_fuzz(data: &[u8]) -> Corpus {
    if data.is_empty() {
        return Corpus::Reject;
    }

    static mut P: *mut u8 = std::ptr::null_mut();

    unsafe {
        {
            let mut x = data[0];
            P = &mut x;
        }
        std::ptr::write_volatile(P, 123);
    }

    Corpus::Keep
}
