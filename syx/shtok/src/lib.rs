//! A tiny library to tokenize shell lines.

use std::borrow::{
    Borrow,
    Cow,
};
use std::collections::HashMap;
use std::{
    fmt,
    io,
};

use Word::*;
use itertools::Itertools;

/// Procrc entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    /// Indicates the command that you should execute such as:
    /// - `gunicorn -b :$PORT main:app`
    /// - `rake jobs:work`
    pub flag: Vec<String>,
}

/// Parses procrc.
pub fn parse<R>(source: R, vars: Option<HashMap<String, String>>) -> io::Result<Vec<Entry>>
where
    R: io::Read,
{
    let rcfile = io::read_to_string(source)?;
    let tokens = Tokens::new(rcfile.bytes(), vars);
    Ok(tokens.map(|flag| Entry { flag }).collect())
}

/// Transforms an input bytes into a sequence of tokens.
pub struct Tokens<I> {
    lex:  Lexer<I>,
    vars: HashMap<String, String>,
}

impl<I> Tokens<I> {
    /// Creates a new Tokens.
    ///
    /// ```
    /// # use shtok::Tokens;
    /// let mut tokens = Tokens::new("foo\"ba\"r baz".bytes(), None);
    /// assert_eq!(tokens.next().unwrap(), ["foobar", "baz"]);
    /// assert_eq!(tokens.next(), None);
    /// ```
    pub fn new<T>(bytes: T, vars: Option<HashMap<String, String>>) -> Self
    where
        T: IntoIterator<Item = u8, IntoIter = I>,
    {
        Tokens { lex: Lexer::new(bytes), vars: vars.unwrap_or_default() }
    }
}

impl<I> Iterator for Tokens<I>
where
    I: Iterator<Item = u8>,
{
    type Item = Vec<String>;

    fn next(&mut self) -> Option<Self::Item> {
        let token_to_str = |token: Token| {
            let mut out = String::new();
            for word in token.words {
                match word {
                    Lit(v) => {
                        let lit: Cow<str> = String::from_utf8_lossy(&v);
                        out.push_str(lit.borrow());
                    }
                    Var(v) => {
                        let var: Cow<str> = String::from_utf8_lossy(&v);
                        let var: &str = var.borrow();
                        if let Some(var) = self.vars.get(var) {
                            out.push_str(var);
                        }
                    }
                    NewLine(_) => {
                        break;
                    }
                }
            }
            out
        };
        let tokens = self
            .lex
            .by_ref()
            // TODO: Use iter::take_until or something.
            // Rust stdlib may have this someday.
            // https://github.com/rust-lang/rust/issues/62208
            .take_while_inclusive(|t| !matches!(t.words.last(), Some(NewLine(_))))
            .map(token_to_str);

        let tokens = tokens.collect::<Vec<_>>();
        if tokens.is_empty() { None } else { Some(tokens) }
    }
}

struct Lexer<I> {
    bytes: I,
    state: State,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum State {
    // Skipping non ascii whitespaces.
    FindNextNonAsciiWhiteSpace = 0,
    // Not in any of the following states.
    NoQuote = 1,
    // Found "\".
    NoQuoteEscape = 2,
    // Found "$".
    NoQuoteVarStart = 3,
    NoQuoteVar = 4,
    // Found "'", but an another matching quote yet.
    InSingleQuote = 30,
    // Found '"', but an another matching quote yet.
    InDoubleQuote = 40,
    // Found "\" in double quote.
    InDoubleQuoteEscape = 41,
    // Found "$" in double quote.
    InDoubleQuoteVarStart = 42,
    InDoubleQuoteVar = 43,
    // Found "\r".
    CarriageReturn = 100,
}

// Token is a single element that make up a line.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Token {
    words: Vec<Word>,
}

#[derive(Clone, PartialEq, Eq)]
enum Word {
    /// A literal string.
    ///
    /// For examples:
    /// - Lit("$x") is just a string "$x", not a variable.
    /// - Lit("\n") is just a string "\n", not a line break.
    Lit(Vec<u8>),
    /// A variable $LIKE_THIS.
    ///
    /// Not support:
    /// - bracing: ${LIKE_THIS}
    /// - nesting: $$LIKE_THIS
    Var(Vec<u8>),
    /// A word for the *non-escaped* newline delimiter.
    NewLine(u8),
}

impl fmt::Debug for Word {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Lit(v) => f.debug_tuple("Lit").field(&String::from_utf8_lossy(v)).finish(),
            Var(v) => f.debug_tuple("Var").field(&String::from_utf8_lossy(v)).finish(),
            NewLine(b) => f.debug_tuple("NewLine").field(&(*b as char)).finish(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Action {
    // Do not consume an input byte, epsilon transion.
    LeftOver,
    // Consume an input byte, and need the next one.
    NextByte,
    // Token has completed.
    Complete,
}

#[inline]
fn leftover(state: State) -> (State, Action) {
    (state, Action::LeftOver)
}

#[inline]
fn nextbyte(state: State) -> (State, Action) {
    (state, Action::NextByte)
}

#[inline]
fn complete(state: State) -> (State, Action) {
    (state, Action::Complete)
}

impl<I> Iterator for Lexer<I>
where
    I: Iterator<Item = u8>,
{
    type Item = Token;
    fn next(&mut self) -> Option<Token> {
        let mut b = self.bytes.next()?;
        let mut token = Token::new();
        loop {
            match self.next_action(&mut token, b) {
                Action::LeftOver => {
                    // Do not call bytes.next().
                    continue;
                }
                Action::NextByte => {
                    // Keep consuming.
                }
                Action::Complete => {
                    break;
                }
            }
            if let Some(next) = self.bytes.next() {
                b = next;
            } else {
                break;
            }
        }

        if token.words.is_empty() { None } else { Some(token) }
    }
}

impl<I> Lexer<I> {
    fn new<T>(bytes: T) -> Self
    where
        T: IntoIterator<Item = u8, IntoIter = I>,
    {
        Lexer { bytes: bytes.into_iter(), state: State::FindNextNonAsciiWhiteSpace }
    }
}

impl<I> Lexer<I>
where
    I: Iterator<Item = u8>,
{
    #[inline]
    fn next_action(&mut self, token: &mut Token, b: u8) -> Action {
        let (state, action) = self.transition(token, b);
        self.state = state;
        action
    }

    // It may be possible to convert nfa to dfa and create a static transition table. To implement
    // this, I think it seems necessary to move token mutations to somewhere.
    //
    // n.b. If b is treated as a byte, the table size will be too large. By classifying b into a few
    // symbols needed for tokenization and "others", the table size should much smaller.
    fn transition(&mut self, token: &mut Token, b: u8) -> (State, Action) {
        use State::*;
        match self.state {
            FindNextNonAsciiWhiteSpace => match b {
                b if b.is_ascii_whitespace() => nextbyte(FindNextNonAsciiWhiteSpace),
                _ => leftover(NoQuote),
            },
            NoQuote => match b {
                b'\r' => nextbyte(CarriageReturn),
                b'\n' => {
                    token.break_line(b);
                    complete(FindNextNonAsciiWhiteSpace)
                }
                b if b.is_ascii_whitespace() => complete(FindNextNonAsciiWhiteSpace),
                b'\\' => nextbyte(NoQuoteEscape),
                b'\'' => nextbyte(InSingleQuote),
                b'"' => nextbyte(InDoubleQuote),
                b'$' => nextbyte(NoQuoteVarStart),
                _ => {
                    token.lit().push(b);
                    nextbyte(NoQuote)
                }
            },
            NoQuoteEscape => match b {
                b'\r' => nextbyte(FindNextNonAsciiWhiteSpace),
                b'\n' => nextbyte(FindNextNonAsciiWhiteSpace),
                _ => {
                    token.lit().push(b);
                    nextbyte(NoQuote)
                }
            },
            NoQuoteVarStart => match b {
                b if b.is_ascii_alphanumeric() || b == b'_' => leftover(NoQuoteVar),
                _ => unimplemented!("Expected a variable name after $"),
            },
            NoQuoteVar => match b {
                b'\'' => nextbyte(InSingleQuote),
                b'"' => nextbyte(InDoubleQuote),
                b'\\' => nextbyte(NoQuote),
                b if b.is_ascii_whitespace() => complete(FindNextNonAsciiWhiteSpace),
                b if b.is_ascii_alphanumeric() || b == b'_' => {
                    token.var().push(b);
                    nextbyte(NoQuoteVar)
                }
                _ => {
                    token.lit().push(b);
                    nextbyte(NoQuote)
                }
            },
            InSingleQuote => match b {
                b'\'' => nextbyte(NoQuote),
                _ => {
                    token.lit().push(b);
                    nextbyte(InSingleQuote)
                }
            },
            InDoubleQuote => match b {
                b'"' => nextbyte(NoQuote),
                b'\\' => nextbyte(InDoubleQuoteEscape),
                b'$' => nextbyte(InDoubleQuoteVarStart),
                _ => {
                    token.lit().push(b);
                    nextbyte(InDoubleQuote)
                }
            },
            InDoubleQuoteEscape => match b {
                b'$' => {
                    token.lit().push(b);
                    nextbyte(InDoubleQuote)
                }
                _ => {
                    let lit = token.lit();
                    lit.push(b'\\');
                    lit.push(b);
                    nextbyte(InDoubleQuote)
                }
            },
            InDoubleQuoteVarStart => match b {
                b if b.is_ascii_alphanumeric() || b == b'_' => leftover(InDoubleQuoteVar),
                _ => unimplemented!("Expected a variable name after $"),
            },
            InDoubleQuoteVar => match b {
                b'"' => nextbyte(NoQuote),
                b'$' => nextbyte(InDoubleQuoteVarStart),
                b if b.is_ascii_alphanumeric() || b == b'_' => {
                    token.var().push(b);
                    nextbyte(InDoubleQuoteVar)
                }
                _ => {
                    token.lit().push(b);
                    nextbyte(InDoubleQuote)
                }
            },
            CarriageReturn => match b {
                b'\n' => {
                    token.break_line(b);
                    complete(FindNextNonAsciiWhiteSpace)
                }
                _ => {
                    let lit = token.lit();
                    lit.push(b'\r');
                    lit.push(b);
                    nextbyte(NoQuote)
                }
            },
        }
    }
}

impl Token {
    fn new() -> Self {
        Token { words: Vec::new() }
    }

    fn lit(&mut self) -> &mut Vec<u8> {
        self.ensure_last_is_lit();
        let Lit(vec) = self.words.last_mut().unwrap() else {
            unreachable!("ensure_last_is_lit does not work as expected");
        };
        vec
    }

    fn ensure_last_is_lit(&mut self) {
        if let Some(last) = self.words.last_mut() {
            match last {
                Var(_) => {
                    self.words.push(Lit(vec![]));
                }
                Lit(_) => (),
                NewLine(_) => unreachable!("pushing a byte onto NewLine"),
            }
        } else {
            self.words.push(Lit(vec![]));
        }
    }

    fn var(&mut self) -> &mut Vec<u8> {
        self.ensure_last_is_var();
        let Var(vec) = self.words.last_mut().unwrap() else {
            unreachable!("ensure_last_is_var does not work as expected");
        };
        vec
    }

    fn ensure_last_is_var(&mut self) {
        if let Some(last) = self.words.last_mut() {
            match last {
                Lit(_) => {
                    self.words.push(Var(vec![]));
                }
                Var(_) => (),
                NewLine(_) => unreachable!("pushing a byte onto NewLine"),
            }
        } else {
            self.words.push(Var(vec![]));
        }
    }

    fn break_line(&mut self, b: u8) {
        self.words.push(Word::NewLine(b));
    }
}

#[cfg(test)]
mod tests {
    use std::iter;

    use super::Lexer;
    use crate::Word::*;
    use crate::{
        Token,
        Word,
    };

    fn check_empty(source: &str) {
        let mut lex = Lexer::new(source.bytes());
        assert_eq!(lex.next(), None);
    }

    fn tokens(source: &str) -> Vec<Token> {
        let mut lex = Lexer::new(source.bytes());
        iter::from_fn(|| lex.next()).collect()
    }

    #[test]
    fn test_empty() {
        check_empty("");
        check_empty(" ");
        check_empty("  ");
    }

    macro_rules! Token {
        ($( $word:expr ),+ $(,)?) => {
            Token {
                words: vec![
                    $($word),+
                ]
            }
        };
    }

    fn lit(bytes: &[u8]) -> Word {
        Lit(bytes.to_vec())
    }

    fn var(bytes: &[u8]) -> Word {
        Var(bytes.to_vec())
    }

    #[test]
    fn no_tokens() {
        assert_eq!(tokens("\\"), []);
        assert_eq!(tokens("\\\n"), []);
        assert_eq!(tokens("\\\n "), []);
        assert_eq!(tokens("\n \n"), []);
        assert_eq!(tokens("\n\t\n"), []);
        assert_eq!(tokens("\n\n\n"), []);
        assert_eq!(tokens("\n\r\n"), []);
    }

    #[test]
    fn get_tokens() {
        assert_eq!(tokens("foobar"), [Token![lit(b"foobar")]]);
        assert_eq!(tokens("$foo-bar"), [Token![var(b"foo"), lit(b"-bar")]]);
        assert_eq!(tokens("A\\e"), [Token![lit(b"Ae")]]);
        assert_eq!(tokens("\"'A\\e\""), [Token![lit(b"'A\\e")]]);

        assert_eq!(tokens("foo bar"), [Token![lit(b"foo")], Token![lit(b"bar")]]);
        assert_eq!(tokens("$foo bar"), [Token![var(b"foo")], Token![lit(b"bar")]]);
        assert_eq!(
            tokens("\"foo\nbar\r\n\" baz"),
            [Token![lit(b"foo\nbar\r\n")], Token![lit(b"baz")]]
        );
    }

    #[test]
    fn get_tokens_non_ascii() {
        assert_eq!(tokens("えび"), [Token![lit("えび".as_bytes())]]);
        assert_eq!(tokens("え び"), [Token![lit("え".as_bytes())], Token![lit("び".as_bytes())]]);
        assert_eq!(tokens("え\\ び"), [Token![lit("え び".as_bytes())]]);
        assert_eq!(
            tokens("え \\ び"),
            [Token![lit("え".as_bytes())], Token![lit(" び".as_bytes())]]
        );
        assert_eq!(
            tokens("え \\\nび"),
            [Token![lit("え".as_bytes())], Token![lit("び".as_bytes())]]
        );
        assert_eq!(
            tokens("え \\\r\nび"),
            [Token![lit("え".as_bytes())], Token![lit("び".as_bytes())]]
        );
    }

    #[test]
    fn for_for_in_for() {
        // Shell prints "for" in this case.
        // https://aosabook.org/en/v1/bash.html
        assert_eq!(
            tokens("for for in for; do for=for; done; echo $for"),
            [
                Token![lit(b"for")],
                Token![lit(b"for")],
                Token![lit(b"in")],
                Token![lit(b"for;")],
                Token![lit(b"do")],
                Token![lit(b"for=for;")],
                Token![lit(b"done;")],
                Token![lit(b"echo")],
                Token![var(b"for")],
            ]
        );
    }

    #[test]
    fn line_breaks() {
        assert_eq!(
            tokens("$A-foo -x\n$B-bar -y\n$C-baz -z\n"),
            [
                Token![var(b"A"), lit(b"-foo")],
                Token![lit(b"-x"), NewLine(b'\n')],
                Token![var(b"B"), lit(b"-bar")],
                Token![lit(b"-y"), NewLine(b'\n')],
                Token![var(b"C"), lit(b"-baz")],
                Token![lit(b"-z"), NewLine(b'\n')]
            ]
        );
        assert_eq!(
            tokens("$A-foo -x\r\n$B-bar -y\r\n$C-baz -z\r\n"),
            [
                Token![var(b"A"), lit(b"-foo")],
                Token![lit(b"-x"), NewLine(b'\n')],
                Token![var(b"B"), lit(b"-bar")],
                Token![lit(b"-y"), NewLine(b'\n')],
                Token![var(b"C"), lit(b"-baz")],
                Token![lit(b"-z"), NewLine(b'\n')]
            ]
        );
    }

    #[test]
    fn escape_ascii_whitespace() {
        assert_eq!(tokens("\\ "), [Token![lit(b" ")]]);
        assert_eq!(tokens("\\ A"), [Token![lit(b" A")]]);
        assert_eq!(tokens("\\\tA"), [Token![lit(b"\tA")]]);
        assert_eq!(tokens("\\\r\nA"), [Token![lit(b"A")]]);
        assert_eq!(tokens("\\\nA"), [Token![lit(b"A")]]);
    }

    #[test]
    fn escape_in_single_quote() {
        assert_eq!(tokens("'\ne'"), [Token![lit(b"\ne")]]);
        assert_eq!(tokens("'\\\ne'"), [Token![lit(b"\\\ne")]]);
    }

    #[test]
    fn escape_in_double_quote() {
        assert_eq!(tokens("\"\\ VAR\""), [Token![lit(b"\\ VAR")]]);
        assert_eq!(tokens("\"\\$VAR\""), [Token![lit(b"$VAR")]]);
        assert_eq!(tokens("\"\\ $VAR\""), [Token![lit(b"\\ "), var(b"VAR")]]);
    }

    #[test]
    fn double_quote_no_spaces() {
        assert_eq!(tokens("A\"PPL\"E"), [Token![lit(b"APPLE")]]);
        assert_eq!(tokens("\"APPL\"E"), [Token![lit(b"APPLE")]]);
        assert_eq!(tokens("A\"PPLE\""), [Token![lit(b"APPLE")]]);
    }

    #[test]
    fn double_quote_in_single_quote() {
        assert_eq!(tokens("'A\"PPL\"E'"), [Token![lit(b"A\"PPL\"E")]]);
        assert_eq!(tokens("'\"APPL\"E'"), [Token![lit(b"\"APPL\"E")]]);
        assert_eq!(tokens("'A\"PPLE\"'"), [Token![lit(b"A\"PPLE\"")]]);
    }

    #[test]
    fn var_in_many_ways() {
        assert_eq!(tokens("\"$VAR\""), [Token![var(b"VAR")]]);
        assert_eq!(tokens("$VAR"), [Token![var(b"VAR")]]);
        assert_eq!(tokens("$VAR#a"), [Token![var(b"VAR"), lit(b"#a")]]);
        assert_eq!(tokens("$VAR-a"), [Token![var(b"VAR"), lit(b"-a")]]);
        assert_eq!(tokens("e\"$VAR\"o"), [Token![lit(b"e"), var(b"VAR"), lit(b"o")]]);
        assert_eq!(tokens("$VAR\\A"), [Token![var(b"VAR"), lit(b"A")]]);
        assert_eq!(tokens("\"$VAR\\A\""), [Token![var(b"VAR"), lit(b"\\A")]]);
        assert_eq!(tokens("$VAR\"\""), [Token![var(b"VAR")]]);
        assert_eq!(tokens("$VAR\"ABC\""), [Token![var(b"VAR"), lit(b"ABC")]]);
        assert_eq!(
            tokens("e\"$VAR\"o hello"),
            [Token![lit(b"e"), var(b"VAR"), lit(b"o")], Token![lit(b"hello")]]
        );
    }
}
