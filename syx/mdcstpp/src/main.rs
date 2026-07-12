#![allow(missing_docs)]
use std::path::PathBuf;
use std::{
    env,
    fs,
    io,
};

use clap::Parser;

#[derive(Debug, Clone, clap::Parser)]
struct Flags {
    /// Path to a markdown file.
    #[arg()]
    path: PathBuf,
}

fn main() -> io::Result<()> {
    if let Ok(bwd) = env::var("BUILD_WORKING_DIRECTORY") {
        env::set_current_dir(bwd)?;
    }

    let flags = Flags::parse();
    let md = fs::read_to_string(&flags.path)?;

    let cst = parse_md(&md).ok_or(io::Error::other("failed to parse"))?;
    let mut cur = cst.walk();
    visit(&mut cur, 0, &md);
    Ok(())
}

fn parse_md<T: AsRef<[u8]>>(src: T) -> Option<tree_sitter::Tree> {
    let mut parser = tree_sitter::Parser::new();
    let lang = &tree_sitter_md::LANGUAGE.into();
    parser.set_language(lang).expect("error loading a grammar");
    parser.parse(&src, None)
}

fn visit<T: AsRef<[u8]>>(cursor: &mut tree_sitter::TreeCursor, depth: usize, src: T) {
    let src = src.as_ref();
    let indent = 2 * depth;
    let node = cursor.node();
    let text = node.utf8_text(src).expect("invalid utf-8 text");

    println!(
        "{:>indent$} {kind} [{s}-{e}] \"{view}\"",
        indent,
        kind = node.kind(),
        s = node.start_position(),
        e = node.end_position(),
        view = if let Some(i) = text.find('\n') { &text[..i] } else { &text },
    );

    if cursor.goto_first_child() {
        visit(cursor, depth + 1, src);
        while cursor.goto_next_sibling() {
            visit(cursor, depth + 1, src);
        }
        cursor.goto_parent();
    }
}
