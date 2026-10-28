//! The static comment dictionary (`c` token space).
//!
//! Comments are interned once and referenced as `c(n)`. The text lives here,
//! not in the condensed definition, so repeated comments cost one number. The
//! table is static: a comment token is never freed or reused, even if the code
//! that referenced it is deleted.

use std::collections::HashMap;

use crate::util::{escape, unescape};

pub const DUMP_HEADER: &str = "def2mcpfnnumber-comments";
pub const DUMP_VERSION: u32 = 1;

/// Interned comment bodies. Numbers start at 1 so `0` stays "no comment".
#[derive(Clone, Debug)]
pub struct Comments {
    next: u32,
    texts: Vec<String>,
    by_text: HashMap<String, u32>,
}

impl Default for Comments {
    fn default() -> Self {
        Self::new()
    }
}

impl Comments {
    pub fn new() -> Self {
        Comments {
            next: 1,
            texts: vec![String::new()],
            by_text: HashMap::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.texts.len() - 1
    }

    pub fn is_empty(&self) -> bool {
        self.texts.len() == 1
    }

    /// Intern a comment body, returning its static number.
    pub fn intern(&mut self, text: &str) -> u32 {
        if let Some(&n) = self.by_text.get(text) {
            return n;
        }
        let n = self.next;
        self.next += 1;
        self.texts.push(text.to_string());
        self.by_text.insert(text.to_string(), n);
        n
    }

    /// Resolve a comment token.
    pub fn text_of(&self, n: u32) -> Option<&str> {
        if n == 0 {
            return None;
        }
        self.texts.get(n as usize).map(String::as_str)
    }

    /// Resolve an already-interned comment body.
    pub fn token_of(&self, text: &str) -> Option<u32> {
        self.by_text.get(text).copied()
    }

    pub fn dump(&self) -> String {
        let mut out = String::new();
        out.push_str(&format!("{DUMP_HEADER} {DUMP_VERSION}\n"));
        out.push_str(&format!("next {}\n", self.next));
        for (n, text) in self.texts.iter().enumerate().skip(1) {
            out.push_str(&format!("{}\t{}\n", n, escape(text)));
        }
        out
    }

    pub fn load(text: &str) -> Result<Comments, String> {
        let mut lines = text.lines();
        let header = lines.next().ok_or("empty comments dump")?;
        let mut hp = header.split_whitespace();
        if hp.next() != Some(DUMP_HEADER) {
            return Err(format!("not a {DUMP_HEADER} dump: {header:?}"));
        }
        let version: u32 = hp
            .next()
            .and_then(|s| s.parse().ok())
            .ok_or("missing comments version")?;
        if version != DUMP_VERSION {
            return Err(format!("unsupported comments version {version}"));
        }
        let next_line = lines.next().ok_or("missing comments next counter")?;
        let next: u32 = next_line
            .strip_prefix("next ")
            .and_then(|s| s.trim().parse().ok())
            .ok_or_else(|| format!("bad comments next line {next_line:?}"))?;

        let mut comments = Comments::new();
        comments.next = next.max(1);
        for line in lines {
            if line.trim().is_empty() {
                continue;
            }
            let (n, text) = line
                .split_once('\t')
                .ok_or_else(|| format!("bad comment line {line:?}"))?;
            let n: u32 = n.parse().map_err(|_| format!("bad comment number {n:?}"))?;
            let text = unescape(text);
            if n as usize != comments.texts.len() {
                return Err(format!("comment {n} out of order"));
            }
            if comments.by_text.insert(text.clone(), n).is_some() {
                return Err("duplicate comment text".to_string());
            }
            comments.texts.push(text);
        }
        if comments.next <= comments.texts.len() as u32 {
            comments.next = comments.texts.len() as u32;
        }
        Ok(comments)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identical_comments_share_one_token() {
        let mut c = Comments::new();
        assert_eq!(c.intern("hello"), 1);
        assert_eq!(c.intern("hello"), 1);
        assert_eq!(c.intern("world"), 2);
        assert_eq!(c.text_of(1), Some("hello"));
        assert_eq!(c.token_of("world"), Some(2));
        assert_eq!(c.len(), 2);
    }

    #[test]
    fn dump_load_round_trips_multiline_comment_text() {
        let mut c = Comments::new();
        c.intern("line one\nline two");
        let text = c.dump();
        assert!(!text.contains("line one\nline two"));
        let back = Comments::load(&text).expect("load");
        assert_eq!(back.dump(), text);
        assert_eq!(back.text_of(1), Some("line one\nline two"));
    }
}
