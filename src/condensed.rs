//! The condensed spelling used by the store and by Scratch/IDE links.
//!
//! ```text
//! g(201) = [g(1), g(3), g(100), lt(1), c(7)]
//! lt(1,"value")
//! c(7)
//! ```
//!
//! `g(n)` is a global definition, `lt(n)` a local and `c(n)` a comment. The
//! definition line's left side is the token being defined; the bracketed list
//! is the token stream of its body/signature, in source order.

use crate::token::{Token, TokenRef};

/// Render one condensed definition line.
///
/// An empty reference list still renders `[]`, so "definition with an empty
/// body" is distinct from "no definition line at all".
pub fn def_line(token: Token, refs: &[TokenRef]) -> String {
    let body = refs
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(", ");
    format!("g({}) = [{}]", token.0, body)
}

/// Parse one condensed definition line.
pub fn parse_def_line(line: &str) -> Option<(Token, Vec<TokenRef>)> {
    let (lhs, rhs) = line.split_once('=')?;
    let lhs = lhs.trim();
    let token = Token(
        lhs.strip_prefix("g(")?
            .strip_suffix(')')?
            .trim()
            .parse()
            .ok()?,
    );
    let rhs = rhs.trim();
    let body = rhs.strip_prefix('[')?.strip_suffix(']')?;
    let refs = if body.trim().is_empty() {
        Vec::new()
    } else {
        body.split(',')
            .map(TokenRef::parse)
            .collect::<Option<Vec<_>>>()?
    };
    Some((token, refs))
}

/// Render a local declaration: `lt(1,"value")`.
///
/// The name is inline (and escaped) because the common case reads better with
/// it; the authoritative table is still [`crate::locals::Locals`].
pub fn local_decl(token: u32, name: &str) -> String {
    let mut escaped = String::with_capacity(name.len());
    for c in name.chars() {
        match c {
            '\\' => escaped.push_str("\\\\"),
            '"' => escaped.push_str("\\\""),
            _ => escaped.push(c),
        }
    }
    format!("lt({token},\"{escaped}\")")
}

/// Parse `lt(n,"name")`.
pub fn parse_local_decl(s: &str) -> Option<(u32, String)> {
    let s = s.trim();
    let rest = s.strip_prefix("lt(")?.strip_suffix(')')?;
    let (n, quoted) = rest.split_once(',')?;
    let token: u32 = n.trim().parse().ok()?;
    let inner = quoted.trim().strip_prefix('"')?.strip_suffix('"')?;

    let mut name = String::with_capacity(inner.len());
    let mut chars = inner.chars();
    while let Some(c) = chars.next() {
        if c != '\\' {
            name.push(c);
            continue;
        }
        match chars.next() {
            Some('\\') => name.push('\\'),
            Some('"') => name.push('"'),
            Some(other) => {
                name.push('\\');
                name.push(other);
            }
            None => name.push('\\'),
        }
    }
    Some((token, name))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::token::Db;

    #[test]
    fn definition_lines_round_trip() {
        let refs = [
            TokenRef::global(1),
            TokenRef::global(3),
            TokenRef::global(101),
            TokenRef::local(1),
            TokenRef::comment(7),
        ];
        let line = def_line(Token(201), &refs);
        assert_eq!(line, "g(201) = [g(1), g(3), g(101), lt(1), c(7)]");
        let (token, back) = parse_def_line(&line).expect("parse");
        assert_eq!(token, Token(201));
        assert_eq!(back, refs);
    }

    #[test]
    fn empty_body_is_preserved() {
        let line = def_line(Token(200), &[]);
        assert_eq!(line, "g(200) = []");
        assert_eq!(parse_def_line(&line), Some((Token(200), vec![])));
    }

    #[test]
    fn local_decl_escapes_names() {
        for name in ["value", "with space", "a\"b", "a\\b"] {
            let text = local_decl(1, name);
            let (n, back) = parse_local_decl(&text).expect(&text);
            assert_eq!(n, 1);
            assert_eq!(back, name);
        }
    }

    #[test]
    fn parse_rejects_non_condensed_text() {
        for bad in [
            "",
            "g(1)",
            "g(1) =",
            "g(1) = [g(2)",
            "g(1) = [x(2)]",
            "lt()",
            "lt(1,value)",
        ] {
            assert!(parse_def_line(bad).is_none(), "{bad:?}");
        }
        for bad in ["lt(1,value)", "lt(,)", "g(1)", "lt(1,\"unterminated)"] {
            assert!(parse_local_decl(bad).is_none(), "{bad:?}");
        }
    }

    #[test]
    fn prefixes_map_to_the_right_space() {
        assert_eq!(TokenRef::parse("g(1)").unwrap().db, Db::Global);
        assert_eq!(TokenRef::parse("lt(1)").unwrap().db, Db::Local);
        assert_eq!(TokenRef::parse("c(1)").unwrap().db, Db::Comment);
    }
}
