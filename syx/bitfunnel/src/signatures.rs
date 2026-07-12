#![allow(missing_docs)]
use std::borrow::{
    Borrow,
    BorrowMut,
};
use std::collections::hash_map::RandomState;
use std::fmt;
use std::hash::{
    BuildHasher,
    Hash,
    Hasher,
};
use std::marker::PhantomData;

type Repr = bitcomp_roaring::BitSet<u64>;

// n: 256, fp: 0.06
const DEFAULT_SIZEOF_FILTER: usize = 1500;
const DEFAULT_SIZEOF_HASHES: usize = 5;

/// Signs is an index for filtering data probabilistically.
#[derive(Clone)]
pub struct Signs<S = RandomState> {
    signs: Vec<Repr>, // inverted signatures.
    khash: usize,     // number of hashes for each signatures.
    state: S,         // hash builder.
}

/// Sign is a row of Signs, that represents fingerprint of data (a.k.a bloom filter).
pub struct Sign<B: Borrow<Signs<S>>, S = RandomState> {
    index: u64,
    signs: B,
    state: PhantomData<S>,
}

impl Default for Signs {
    fn default() -> Self {
        new(DEFAULT_SIZEOF_FILTER, DEFAULT_SIZEOF_HASHES)
    }
}

/// m: signature's bit size.
/// k: number of hashes.
pub fn new(m: usize, k: usize) -> Signs {
    Signs::with_hasher(m, k, Default::default())
}

/// n:  number of items you expect to add to the signature.
/// fp: false positive rate.
pub fn optimal(n: usize, fp: f64) -> Signs {
    let (m, k) = optimal_params(n, fp);
    new(m, k)
}

fn optimal_params(n: usize, fp: f64) -> (usize, usize) {
    assert!(n > 0);
    assert!(0.0 < fp && fp < 1.0);

    let m = {
        let t = 2.0f64;
        -((n as f64) * fp.ln() / t.ln().powi(2))
    };

    let k = {
        let t = 2.0f64;
        t.ln() * m / (n as f64)
    };

    (m.ceil() as usize, k.ceil() as usize)
}

impl<S> Signs<S>
where
    S: BuildHasher,
{
    pub fn with_hasher(m: usize, khash: usize, state: S) -> Signs<S> {
        let signs = vec![Repr::default(); m];
        Signs { signs, khash, state }
    }
}

impl<S> Signs<S> {
    pub fn kbits(&self) -> usize {
        self.signs.len()
    }
    pub fn khash(&self) -> usize {
        self.khash
    }

    pub fn bits(&self, i: usize) -> &Repr {
        &self.signs[i]
    }

    pub fn bits_mut(&mut self, i: usize) -> &mut Repr {
        &mut self.signs[i]
    }

    pub fn sign(&self, index: u64) -> Sign<&Signs<S>, S> {
        let signs = self;
        let state = PhantomData;
        Sign { index, signs, state }
    }

    pub fn sign_mut(&mut self, index: u64) -> Sign<&mut Signs<S>, S> {
        let signs = self;
        let state = PhantomData;
        Sign { index, signs, state }
    }
}

fn make_hashes<S, H>(state: &S, t: &H) -> [u64; 4]
where
    H: Hash + ?Sized,
    S: BuildHasher,
{
    let hasher = &mut state.build_hasher();
    let mut hashes = [0; 4];
    for h in &mut hashes {
        t.hash(hasher);
        *h = hasher.finish();
    }
    hashes
}

fn hash_at(h: [u64; 4], i: usize) -> usize {
    let p = h[i % 2].wrapping_add((i as u64).wrapping_mul(h[2 + (((i + (i % 2)) % 4) / 2)]));
    p as usize
}

impl<B, S> Sign<B, S>
where
    B: Borrow<Signs<S>>,
    S: BuildHasher,
{
    pub fn test<H>(&self, h: &H) -> bool
    where
        H: Hash + ?Sized,
    {
        let hashes = make_hashes(&self.signs.borrow().state, h);
        self.test_hashes(hashes)
    }

    fn test_hashes(&self, hs: [u64; 4]) -> bool {
        let signs = self.signs.borrow();
        let kbits = signs.kbits();
        let khash = signs.khash();
        let hashi = |i| hash_at(hs, i) % kbits;
        (0..khash).all(|k| signs.bits(hashi(k)).get(self.index))
    }
}

impl<B, S> Sign<B, S>
where
    B: BorrowMut<Signs<S>>,
    S: BuildHasher,
{
    pub fn add<H>(&mut self, h: &H)
    where
        H: Hash + ?Sized,
    {
        let hashes = make_hashes(&self.signs.borrow().state, h);
        self.add_hashes(hashes)
    }

    fn add_hashes(&mut self, hs: [u64; 4]) {
        let signs = self.signs.borrow_mut();
        let kbits = signs.kbits();
        let khash = signs.khash();
        let hashi = |i| hash_at(hs, i) % kbits;
        for k in 0..khash {
            signs.bits_mut(hashi(k)).insert(self.index);
        }
    }
}

mod disp {
    pub trait Format {}
    pub struct Bin;
    pub struct Hex;
    impl Format for Bin {}
    impl Format for Hex {}
}

pub struct Dump<'a, S: 'a, F: disp::Format> {
    index: u64,
    signs: &'a Signs<S>,
    _fmt:  PhantomData<F>,
}

impl<B: Borrow<Signs<S>>, S> Sign<B, S> {
    pub fn bin_dump(&self) -> Dump<'_, S, disp::Bin> {
        let index = self.index;
        let signs = self.signs.borrow();
        let _fmt = PhantomData;
        Dump { index, signs, _fmt }
    }

    pub fn hex_dump(&self) -> Dump<'_, S, disp::Hex> {
        let index = self.index;
        let signs = self.signs.borrow();
        let _fmt = PhantomData;
        Dump { index, signs, _fmt }
    }
}

macro_rules! fmtdebug {
    ($signs:expr, $index:expr, $f:expr, $p:expr, $rev:expr) => {{
        let mut n: u8 = 0;

        write!($f, "index:0x{:08X}", $index)?;
        macro_rules! putn {
            ($p2: expr) => {
                if $rev {
                    write!($f, $p2, reverse_bits(n) as u8)?;
                } else {
                    write!($f, $p2, n)?;
                }
            };
        }

        for (i, sign) in $signs.signs.iter().enumerate() {
            if sign.get($index) {
                n |= 1 << (i % 8);
            }

            if i % 8 == 7 {
                putn!(concat!(" ", $p));
                n = 0;
            }
        }
        putn!(concat!(" ", $p));
        Ok(())
    }};
}

fn reverse_bits<T: Into<u64>>(n: T) -> u64 {
    // https://graphics.stanford.edu/~seander/bithacks.html
    // use `reverse_bits` when the method is stabled.
    let n = n.into();
    ((n * 0x0002_0202_0202_u64) & 0x0108_8442_2010_u64) % 1023
}

impl<'a, S: 'a> fmt::Debug for Dump<'a, S, disp::Bin> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmtdebug!(self.signs, self.index, f, "{:08b}", true)
    }
}

impl<'a, S: 'a> fmt::Debug for Dump<'a, S, disp::Hex> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmtdebug!(self.signs, self.index, f, "{:02x}", false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reverse_bits() {
        assert_eq!(0b00000000, reverse_bits(0b00000000_u8) as u8);
        assert_eq!(0b10000000, reverse_bits(0b00000001_u8) as u8);
        assert_eq!(0b10010100, reverse_bits(0b00101001_u8) as u8);
        assert_eq!(0b10010111, reverse_bits(0b11101001_u8) as u8);
        assert_eq!(0b11111111, reverse_bits(0b11111111_u8) as u8);
    }

    #[test]
    #[ignore]
    fn debug_optimal_params() {
        let docheck = |n: usize, fp: f64| {
            let (kbits, khash) = optimal_params(n, fp);
            let bytes_per_point = (kbits as f64 / n as f64) / 8.0;
            println!(
                "n:{:5} fp: {:.4}, bits:{:5} hash:{} {:.4}",
                n, fp, kbits, khash, bytes_per_point
            );
        };

        let from_fp = |fp: f64| {
            docheck(1 << 5, fp);
            docheck(1 << 6, fp);
            docheck(1 << 7, fp);
            docheck(1 << 8, fp);
            docheck(1 << 9, fp);
            docheck(1 << 10, fp);
            docheck(1 << 11, fp);
            docheck(1 << 12, fp);
            docheck(1 << 13, fp);
        };

        from_fp(0.01);
        from_fp(0.02);
        from_fp(0.03);
        from_fp(0.04);
        from_fp(0.05);
        from_fp(0.055);
        from_fp(0.06);
        from_fp(0.10);
        from_fp(0.15);
    }
}
