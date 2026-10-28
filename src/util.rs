//! Field escaping for the plain-text dump format.
//!
//! The store is deliberately dependency-free, so it is one entry per line with
//! TAB-separated fields. That means TAB, LF, CR and `\` itself have to be
//! escaped or a file path / comment body could break the framing.

/// Escape a field for one line of the dump.
pub fn escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '\t' => out.push_str("\\t"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            _ => out.push(c),
        }
    }
    out
}

/// Inverse of [`escape`]. Unknown escapes are preserved verbatim so a future
/// field never silently loses bytes.
pub fn unescape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        match chars.next() {
            Some('\\') => out.push('\\'),
            Some('t') => out.push('\t'),
            Some('n') => out.push('\n'),
            Some('r') => out.push('\r'),
            Some(other) => {
                out.push('\\');
                out.push(other);
            }
            None => out.push('\\'),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_control_bytes_and_backslashes() {
        let raw = "a\\b\tc\nd\re";
        let enc = escape(raw);
        assert!(!enc.contains('\t'));
        assert!(!enc.contains('\n'));
        assert_eq!(unescape(&enc), raw);
    }

    #[test]
    fn unknown_escapes_are_preserved() {
        assert_eq!(unescape("a\\qb"), "a\\qb");
        assert_eq!(unescape("trailing\\"), "trailing\\");
    }
}
