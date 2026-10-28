//! The per-function local dictionary (`lt` token space).
//!
//! Locals are mutable by design: a function body is edited far more often than
//! a declaration is added, and `lt` numbers are scoped to that one body. Two
//! different functions can both use `lt(1,"value")` without collision because
//! a function context owns its own [`Locals`] table.
//!
//! Rendering a local declaration keeps the name inline so a reader of the
//! condensed store does not need a side table for the common case:
//!
//! ```text
//! lt(1,"value")
//! ```
//!
//! Numbers start at 1 so `0` can keep its "no token" meaning in every space.

use std::collections::HashMap;

use crate::util::{escape, unescape};

pub const DUMP_HEADER: &str = "def2mcpfnnumber-locals";
pub const DUMP_VERSION: u32 = 1;

/// Local/parameter names for one function body.
#[derive(Clone, Debug)]
pub struct Locals {
    next: u32,
    /// Index 0 is the sentinel empty name; a real local `n` lives at `names[n]`.
    names: Vec<String>,
    by_name: HashMap<String, u32>,
}

impl Default for Locals {
    fn default() -> Self {
        Self::new()
    }
}

impl Locals {
    /// An empty table; locals allocate from 1.
    pub fn new() -> Self {
        Locals {
            next: 1,
            names: vec![String::new()],
            by_name: HashMap::new(),
        }
    }

    /// Number of interned locals.
    pub fn len(&self) -> usize {
        self.names.len() - 1
    }

    pub fn is_empty(&self) -> bool {
        self.names.len() == 1
    }

    /// Intern a local name, returning its stable number within this body.
    pub fn intern(&mut self, name: &str) -> u32 {
        if let Some(&n) = self.by_name.get(name) {
            return n;
        }
        let n = self.next;
        self.next += 1;
        self.names.push(name.to_string());
        self.by_name.insert(name.to_string(), n);
        n
    }

    /// Resolve an interned number.
    pub fn name_of(&self, n: u32) -> Option<&str> {
        if n == 0 {
            return None;
        }
        self.names.get(n as usize).map(String::as_str)
    }

    /// Resolve an interned name.
    pub fn token_of(&self, name: &str) -> Option<u32> {
        self.by_name.get(name).copied()
    }

    /// Rename an existing local in place. The number does not move; only the
    /// name changes, which is what "mutable" means for this table.
    pub fn rename(&mut self, n: u32, name: &str) -> bool {
        if n == 0 {
            return false;
        }
        let Some(slot) = self.names.get_mut(n as usize) else {
            return false;
        };
        let old = std::mem::replace(slot, name.to_string());
        self.by_name.remove(&old);
        self.by_name.insert(name.to_string(), n);
        true
    }

    /// Drop every local. A new function body starts here.
    pub fn reset(&mut self) {
        self.next = 1;
        self.names.truncate(1);
        self.by_name.clear();
    }

    /// Serialize this body's locals.
    pub fn dump(&self) -> String {
        let mut out = String::new();
        out.push_str(&format!("{DUMP_HEADER} {DUMP_VERSION}\n"));
        out.push_str(&format!("next {}\n", self.next));
        for (n, name) in self.names.iter().enumerate().skip(1) {
            out.push_str(&format!("{}\t{}\n", n, escape(name)));
        }
        out
    }

    /// Restore a dump produced by [`Locals::dump`].
    pub fn load(text: &str) -> Result<Locals, String> {
        let mut lines = text.lines();
        let header = lines.next().ok_or("empty locals dump")?;
        let mut hp = header.split_whitespace();
        if hp.next() != Some(DUMP_HEADER) {
            return Err(format!("not a {DUMP_HEADER} dump: {header:?}"));
        }
        let version: u32 = hp
            .next()
            .and_then(|s| s.parse().ok())
            .ok_or("missing locals version")?;
        if version != DUMP_VERSION {
            return Err(format!("unsupported locals version {version}"));
        }
        let next_line = lines.next().ok_or("missing locals next counter")?;
        let next: u32 = next_line
            .strip_prefix("next ")
            .and_then(|s| s.trim().parse().ok())
            .ok_or_else(|| format!("bad locals next line {next_line:?}"))?;

        let mut locals = Locals::new();
        locals.next = next.max(1);
        for line in lines {
            if line.trim().is_empty() {
                continue;
            }
            let (n, name) = line
                .split_once('\t')
                .ok_or_else(|| format!("bad local line {line:?}"))?;
            let n: u32 = n.parse().map_err(|_| format!("bad local number {n:?}"))?;
            let name = unescape(name);
            if n as usize != locals.names.len() {
                return Err(format!("local {n} out of order"));
            }
            if locals.by_name.insert(name.clone(), n).is_some() {
                return Err(format!("duplicate local name {name:?}"));
            }
            locals.names.push(name);
        }
        if locals.next <= locals.names.len() as u32 {
            locals.next = locals.names.len() as u32;
        }
        Ok(locals)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn locals_are_stable_and_per_body() {
        let mut a = Locals::new();
        let mut b = Locals::new();
        assert_eq!(a.intern("value"), 1);
        assert_eq!(a.intern("value"), 1);
        assert_eq!(a.intern("other"), 2);
        assert_eq!(b.intern("value"), 1, "separate bodies number independently");
        assert_eq!(a.name_of(2), Some("other"));
        assert_eq!(a.token_of("other"), Some(2));
        assert_eq!(a.name_of(0), None);
    }

    #[test]
    fn rename_keeps_the_number_and_reset_clears() {
        let mut l = Locals::new();
        assert_eq!(l.intern("value"), 1);
        assert!(l.rename(1, "renamed"));
        assert_eq!(l.name_of(1), Some("renamed"));
        assert_eq!(l.token_of("value"), None);
        assert_eq!(l.token_of("renamed"), Some(1));
        assert!(!l.rename(0, "nope"));
        l.reset();
        assert!(l.is_empty());
        assert_eq!(l.intern("fresh"), 1);
    }

    #[test]
    fn dump_load_round_trips() {
        let mut l = Locals::new();
        l.intern("value");
        l.intern("tab\there");
        let text = l.dump();
        let back = Locals::load(&text).expect("load");
        assert_eq!(back.dump(), text);
        assert_eq!(back.name_of(2), Some("tab\there"));
    }
}
