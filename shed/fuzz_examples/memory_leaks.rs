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

    let data = Vec::from(data);
    std::mem::forget(data);

    Corpus::Keep
}
