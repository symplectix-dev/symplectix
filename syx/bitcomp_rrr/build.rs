//! Build script for RRR.

use std::io::Write;
use std::path::Path;
use std::{
    env,
    fs,
    io,
};

fn main() -> io::Result<()> {
    // Choose `b + 1` as a power of two, so that the bits for `class` can be fully used
    // (bitpacking).
    //
    // 7:  0b_00000111
    // 15: 0b_00001111
    // 31: 0b_00011111
    // 63: 0b_00111111
    write_mod("u8", 4)?;
    write_mod("u16", 15)?;
    write_mod("u32", 31)?;
    write_mod("u64", 63)?;
    Ok(())
}

fn comb_table(size: usize) -> Vec<Vec<u64>> {
    let mut table = vec![vec![0; size]; size];

    #[allow(clippy::needless_range_loop)]
    for i in 0..size {
        table[i][i] = 1; // initialize diagonal
        table[i][0] = 1; // initialize first col
    }

    for n in 1..size {
        for k in 1..size {
            table[n][k] = table[n - 1][k - 1] + table[n - 1][k];
        }
    }
    table
}

/// Build time utilities for rrr, which writes static values
/// and functions to OUT_DIR.
fn write_mod(item_type: &str, b: usize) -> io::Result<()> {
    env::var_os("OUT_DIR").ok_or(io::Error::other("OUT_DIR not found")).and_then(|out_dir| {
        let table = comb_table(b + 1);
        let table_len = table.len();
        let class_size = u8::BITS - (b as u8).leading_zeros();
        let mut file = fs::File::create(Path::new(&out_dir).join(format!("rrr{b}.rs")))?;
        writeln!(
            file,
            r#"
// Size of the rrr block in bits.
const SIZE: usize = {b};

/// Minimum bits size to represents class value.
pub const CLASS_SIZE: u8 = {class_size};

/// Encodes data into a pair of `class` and `offset`.
#[inline]
pub fn encode(data: {item_type}) -> (u8, {item_type}) {{
    rrr_encode!(data)
}}

/// Decodes data from a pair of `class` and `offset`.
#[inline]
pub fn decode(class: u8, offset: {item_type}) -> {item_type} {{
    rrr_decode!(class, offset)
}}

#[allow(clippy::unreadable_literal)]
pub(crate) static COMB: [[{item_type}; {table_len}]; {table_len}] = {table:?};
"#,
        )
    })
}
