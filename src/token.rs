//! Token primitives.
//!
//! There are three token spaces, named after the forms the owner sketched:
//!
//! | space | form | meaning |
//! |---|---|---|
//! | [`Db::Global`] | `g(201)` | a definition reachable from anywhere (namespace, type, method, field, modifier, keyword) |
//! | [`Db::Local`] | `lt(1,"value")` | a local/parameter inside one function body; mutable, reset per function |
//! | [`Db::Comment`] | `c(3)` | a comment body, interned once and referenced statically |
//!
//! A bare [`Token`] is just the number; the [`Db`] says which dictionary it
//! belongs to. Keeping them separate is what stops `g(1)` (public) and `c(1)`
//! (the first comment) from ever being confused for one another.

use std::fmt;

/// A token number inside one of the three token spaces.
///
/// `0` is reserved for "no token" so a missing/serialized field can never be
/// mistaken for a real definition.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Default)]
pub struct Token(pub u32);

impl Token {
    /// The reserved "no token" value.
    pub const NULL: Token = Token(0);

    pub const fn new(n: u32) -> Self {
        Token(n)
    }

    pub const fn get(self) -> u32 {
        self.0
    }

    pub const fn is_null(self) -> bool {
        self.0 == 0
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<u32> for Token {
    fn from(n: u32) -> Self {
        Token(n)
    }
}

impl From<Token> for u32 {
    fn from(t: Token) -> Self {
        t.0
    }
}

/// Which token dictionary a number belongs to.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Db {
    /// `g(n)` — a global definition (the main dictionary).
    Global,
    /// `lt(n)` — a local name inside a function body.
    Local,
    /// `c(n)` — a comment body.
    Comment,
}

impl Db {
    pub const fn prefix(self) -> &'static str {
        match self {
            Db::Global => "g",
            Db::Local => "lt",
            Db::Comment => "c",
        }
    }

    pub fn parse_prefix(s: &str) -> Option<Db> {
        match s {
            "g" => Some(Db::Global),
            "lt" => Some(Db::Local),
            "c" => Some(Db::Comment),
            _ => None,
        }
    }
}

/// A token together with the space it lives in — the thing that renders as
/// `g(201)`, `lt(1)` or `c(3)` in the condensed store.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct TokenRef {
    pub db: Db,
    pub token: Token,
}

impl TokenRef {
    pub const fn new(db: Db, token: Token) -> Self {
        TokenRef { db, token }
    }

    pub const fn global(n: u32) -> Self {
        TokenRef {
            db: Db::Global,
            token: Token(n),
        }
    }

    pub const fn local(n: u32) -> Self {
        TokenRef {
            db: Db::Local,
            token: Token(n),
        }
    }

    pub const fn comment(n: u32) -> Self {
        TokenRef {
            db: Db::Comment,
            token: Token(n),
        }
    }

    /// Parse exactly `g(12)`, `lt(3)` or `c(4)` (surrounding whitespace is
    /// ignored). Anything else is `None` rather than a guess.
    pub fn parse(s: &str) -> Option<TokenRef> {
        let s = s.trim();
        let (prefix, rest) = s.split_once('(')?;
        let inner = rest.strip_suffix(')')?.trim();
        let db = Db::parse_prefix(prefix.trim())?;
        let number: u32 = inner.parse().ok()?;
        Some(TokenRef {
            db,
            token: Token(number),
        })
    }
}

impl fmt::Display for TokenRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}({})", self.db.prefix(), self.token.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_and_parses_every_space() {
        for (text, db, n) in [
            ("g(201)", Db::Global, 201),
            ("lt(1)", Db::Local, 1),
            ("c(3)", Db::Comment, 3),
        ] {
            let r = TokenRef::parse(text).expect(text);
            assert_eq!(r.db, db);
            assert_eq!(r.token, Token(n));
            assert_eq!(r.to_string(), text);
        }
    }

    #[test]
    fn parse_rejects_unknown_prefix_and_garbage() {
        for bad in ["", "x(1)", "g()", "g(1", "g(-1)", "g(1)junk", "12"] {
            assert!(TokenRef::parse(bad).is_none(), "{bad:?} must not parse");
        }
    }

    #[test]
    fn null_token_is_zero_and_never_real() {
        assert!(Token::NULL.is_null());
        assert_eq!(Token::NULL.get(), 0);
        assert!(!Token::new(1).is_null());
    }
}
