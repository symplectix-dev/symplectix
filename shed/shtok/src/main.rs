//! Tokenize shell lines.

use std::io;

fn main() -> io::Result<()> {
    if let Ok(entries) = shtok::parse(io::stdin(), None) {
        for e in entries {
            println!("{e:?}");
        }
    }
    Ok(())
}
