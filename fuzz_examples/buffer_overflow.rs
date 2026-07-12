#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| do_fuzz(data));

fn do_fuzz(data: &[u8]) {
    println!("data = {data:?}");
    let x = unsafe { *data.as_ptr().add(data.len()) };
    println!("x = {x:?}");
}
