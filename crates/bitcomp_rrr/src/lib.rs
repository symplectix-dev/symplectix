#![doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/README.md"))]

// Encode using a static table.
macro_rules! rrr_encode {
    ($data:expr) => {{
        let data = $data & ((1 << SIZE) - 1);

        let class = data.count_ones();
        let offset = {
            let mut b = SIZE;
            let mut c = class as usize;
            let mut o = 0;

            while b > 0 && c > 0 {
                b -= 1;
                if data & (1 << b) != 0 {
                    o += COMB[b][c];
                    c -= 1;
                }
            }
            o
        };
        (class as u8, offset)
    }};
}

// Decode using a static table.
macro_rules! rrr_decode {
    ($class:expr, $offset:expr) => {{
        let mut data = 0;
        let mut b = SIZE;
        let mut c = $class as usize;
        let mut o = $offset;

        while b > 0 && c > 0 {
            b -= 1;
            if o >= COMB[b][c] {
                data |= 1 << b;
                o -= COMB[b][c];
                c -= 1;
            }
        }
        data
    }};
}

#[cfg(test)]
mod test_helper {
    use std::fs;

    pub(crate) fn read_json(n: usize) -> fs::File {
        let path =
            testing::rlocation(format!("_main/crates/bitcomp_rrr/testdata/comb_table_{n}.json"));
        fs::OpenOptions::new().read(true).open(path).expect("failed to open a json file")
    }
}

#[cfg(test)]
mod rrr04;
#[cfg(test)]
mod rrr31;
#[cfg(test)]
mod rrr63;

mod rrr15;
pub use rrr15::*;
