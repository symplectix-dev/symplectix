#![allow(missing_docs)]
use std::collections::HashMap;
use std::io;

use shtok::{
    Tokens,
    parse,
};

#[test]
fn no_entries() {
    let rc = r#""#;
    assert!(parse(io::Cursor::new(rc), None).is_ok());

    let rc = r#"
"#;
    assert!(parse(io::Cursor::new(rc), None).is_ok());

    let rc = r#"

        "#;
    assert!(parse(io::Cursor::new(rc), None).is_ok());
}

#[test]
fn many_entries() {
    #[rustfmt::skip]
        let rc = r#"
test0

test1 gunicorn -b \
:$PORT \
main:app

test2 \
-x 10s \
--env A"B"C=def \
-- foo \ bar

test3 \
"C:\path" D:\path

test4 "10m\n" \
           -- \
foo a=1

test5 a b \ \ c

test6 a  b \c \d \
e

test7 a  b \c \ \
\d e
"#;

    let envs = {
        let mut envs = HashMap::new();
        envs.insert("PORT".to_owned(), "8080".to_owned());
        Some(envs)
    };
    let entries = parse(io::Cursor::new(rc), envs).expect("reading from a cursor never fails");

    assert_eq!(entries[0].flag, ["test0"]);
    assert_eq!(entries[1].flag, ["test1", "gunicorn", "-b", ":8080", "main:app"]);
    assert_eq!(entries[2].flag, ["test2", "-x", "10s", "--env", "ABC=def", "--", "foo", " bar"]);
    assert_eq!(entries[3].flag, ["test3", "C:\\path", "D:path"]);
    assert_eq!(entries[4].flag, ["test4", "10m\\n", "--", "foo", "a=1"]);
    assert_eq!(entries[5].flag, ["test5", "a", "b", "  c"]);
    assert_eq!(entries[6].flag, ["test6", "a", "b", "c", "d", "e"]);
    assert_eq!(entries[7].flag, ["test7", "a", "b", "c", " d", "e"]);
}

fn tokens_opt(bytes: impl IntoIterator<Item = u8>) -> Option<Vec<String>> {
    let envs = {
        let mut envs = HashMap::new();
        envs.insert("PORT".to_owned(), "8080".to_owned());
        envs.insert("TEST".to_owned(), "ch".to_owned());
        Some(envs)
    };

    let mut t = Tokens::new(bytes, envs);
    let tokens = t.next();
    // Check no more tokens.
    assert_eq!(t.next(), None);
    tokens
}

fn tokens(bytes: impl IntoIterator<Item = u8>) -> Vec<String> {
    tokens_opt(bytes).expect("no token")
}

#[test]
fn empty_token() {
    assert_eq!(tokens_opt("".bytes()), None);
    assert_eq!(tokens_opt(" ".bytes()), None);
}

#[test]
fn get_single_token() {
    assert_eq!(tokens("\\ ".bytes()), [" "]);
    assert_eq!(tokens("\\ e".bytes()), [" e"]);
    assert_eq!(tokens("'\ne'".bytes()), ["\ne"]);

    assert_eq!(tokens("A\"PPL\"E".bytes()), ["APPLE"]);
    assert_eq!(tokens("\"APPL\"E".bytes()), ["APPLE"]);
    assert_eq!(tokens("A\"PPLE\"".bytes()), ["APPLE"]);

    assert_eq!(tokens("'A\"PPL\"E'".bytes()), ["A\"PPL\"E"]);
    assert_eq!(tokens("'\"APPL\"E'".bytes()), ["\"APPL\"E"]);
    assert_eq!(tokens("'A\"PPLE\"'".bytes()), ["A\"PPLE\""]);

    assert_eq!(tokens("A\\e".bytes()), ["Ae"]);
    assert_eq!(tokens("\"'A\\e\"".bytes()), ["'A\\e"]);
}

#[test]
fn get_tokens() {
    assert_eq!(tokens("foo bar".bytes()), ["foo", "bar"]);
    assert_eq!(tokens("foo\"ba\"r baz".bytes()), ["foobar", "baz"]);
    assert_eq!(tokens("\"foo\nbar\" baz".bytes()), ["foo\nbar", "baz"]);
    assert_eq!(tokens("foo; bar; baz".bytes()), ["foo;", "bar;", "baz"]);
    assert_eq!(tokens("'foo; bar'; baz".bytes()), ["foo; bar;", "baz"]);

    assert_eq!(tokens("え び".bytes()), ["え", "び"]);
    assert_eq!(tokens("え\\ び".bytes()), ["え び"]);
    assert_eq!(tokens("え \\ び".bytes()), ["え", " び"]);
    assert_eq!(tokens("え \\\nび".bytes()), ["え", "び"]);

    // Shell prints "for" in this case.
    // https://aosabook.org/en/v1/bash.html
    //
    // But procrc does not support defining a new var.
    assert_eq!(
        tokens("for for in for; do for=for; done; echo $for".bytes()),
        ["for", "for", "in", "for;", "do", "for=for;", "done;", "echo", ""],
    );
}

#[test]
fn ignore_whitespaces() {
    assert_eq!(tokens("     A\"PPL\"E".bytes()), ["APPLE"]);
    assert_eq!(tokens("     \"APPL\"E".bytes()), ["APPLE"]);
    assert_eq!(tokens("     A\"PPLE\"".bytes()), ["APPLE"]);
    assert_eq!(tokens("A\"PPL\"E     ".bytes()), ["APPLE"]);
    assert_eq!(tokens("\"APPL\"E     ".bytes()), ["APPLE"]);
    assert_eq!(tokens("A\"PPLE\"     ".bytes()), ["APPLE"]);
}

#[test]
fn delimited_in_quote() {
    assert_eq!(tokens("\"foobar  baz\"".bytes()), ["foobar  baz"]);
}

#[test]
fn quote_in_another_quote() {
    assert_eq!(tokens("foo=\"1'0'1\"".bytes()), ["foo=1'0'1"]);
}

#[test]
fn no_matching_quote() {
    assert_eq!(tokens("foo\"bar".bytes()), ["foobar"]);
}

#[test]
fn expand_envs() {
    assert_eq!(tokens("\\$TEST".bytes()), ["$TEST"]);
    assert_eq!(tokens("$TEST".bytes()), ["ch"]);
    assert_eq!(tokens("$TEST#a".bytes()), ["ch#a"]);
    assert_eq!(tokens("$TEST-a".bytes()), ["ch-a"]);
    assert_eq!(tokens("e\"$TEST\"o".bytes()), ["echo"]);
    assert_eq!(tokens("$TEST\\A".bytes()), ["chA"]);
    assert_eq!(tokens("\"$TEST\\A\"".bytes()), ["ch\\A"]);
    assert_eq!(tokens("$TEST{cc".bytes()), ["ch{cc"]);
    assert_eq!(tokens("$TEST\"cc\"".bytes()), ["chcc"]);
    assert_eq!(tokens("e\"$TEST\"o hello".bytes()), ["echo", "hello"]);
}
