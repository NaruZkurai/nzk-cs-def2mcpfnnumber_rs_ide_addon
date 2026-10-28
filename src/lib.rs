//! `def2mcpfnnumber` — stable token numbers for every definition in the
//! csgraph database, for the Rust-only cs viewer v3 IDE, Scratch blocks and
//! MCP links.
//!
//! The model is intentionally small:
//!
//! * [`defaults`] owns the **static** `g` tokens `1..=199` — C# words first,
//!   then system types (`100..=149`, defined statically and before any project
//!   type), then default methods (`150..=199`).
//! * [`dict::Dictionary`] allocates project `g` tokens from `1000` upwards and
//!   persists them, so a rebuild never renumbers an existing definition.
//! * [`locals::Locals`] is a mutable per-function `lt` table.
//! * [`comments::Comments`] is a static `c` table; identical comments share a
//!   token.
//! * [`condensed`] is the wire spelling (`g(1001) = [g(1), g(3), lt(1), c(7)]`).
//!
//! Nothing here compiles C#. The addon only assigns and resolves tokens; the
//! generated/condensed text still goes through the existing compile pipeline.

#![forbid(unsafe_code)]

pub mod comments;
pub mod condensed;
pub mod defaults;
pub mod dict;
pub mod locals;
pub mod shorthand;
pub mod syntax;
pub mod token;
pub mod util;

pub use comments::Comments;
pub use condensed::{def_line, local_decl, parse_def_line, parse_local_decl};
pub use defaults::{DefaultKind, DefaultToken};
pub use dict::{Access, Def, DefKind, Describe, Dictionary, Entry, FIRST_PROJECT_TOKEN, LoadError};
pub use locals::Locals;
pub use shorthand::{Feature, LineFeatures, detect_line, detect_lines};
pub use syntax::{Span, TokenKind};
pub use token::{Db, Token, TokenRef};

/// Crate version (kept in one place: `Cargo.toml`).
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// The three tables together — the smallest useful unit for an IDE session.
///
/// A document window owns one of these; the global dictionary is the shared,
/// persisted one, while [`Locals`] are created per function body.
#[derive(Clone, Debug, Default)]
pub struct TokenIndex {
    pub globals: Dictionary,
    pub comments: Comments,
}

impl TokenIndex {
    pub fn new() -> Self {
        TokenIndex {
            globals: Dictionary::new(),
            comments: Comments::new(),
        }
    }

    /// Number every definition, in the order supplied, and return the tokens in
    /// that same order. Re-indexing the same definitions is idempotent.
    pub fn index_defs(&mut self, defs: &[Def]) -> Vec<Token> {
        defs.iter()
            .cloned()
            .map(|d| self.globals.assign(d))
            .collect()
    }

    /// Intern a comment and return its `c(n)` reference.
    pub fn comment_ref(&mut self, text: &str) -> TokenRef {
        TokenRef::comment(self.comments.intern(text))
    }

    /// A fresh local table for one function body.
    pub fn function_locals(&self) -> Locals {
        Locals::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_session_numbers_defs_comments_and_locals_together() {
        let mut index = TokenIndex::new();
        let defs = [
            Def::new("NamespaceRwby", DefKind::Namespace),
            Def::new("NamespaceRwby.Hero", DefKind::Method)
                .access(Access::Public)
                .modifiers(["public", "static"]),
        ];
        assert_eq!(
            index.index_defs(&defs),
            vec![Token(FIRST_PROJECT_TOKEN), Token(FIRST_PROJECT_TOKEN + 1)]
        );

        let c = index.comment_ref("/* hero */");
        assert_eq!(c, TokenRef::comment(1));

        let mut locals = index.function_locals();
        let value = locals.intern("value");
        assert_eq!(local_decl(value, "value"), "lt(1,\"value\")");

        let hero = TokenRef::global(index.globals.token_of("NamespaceRwby.Hero").unwrap().0);
        assert_eq!(
            def_line(
                Token(FIRST_PROJECT_TOKEN + 1),
                &[
                    TokenRef::global(defaults::PUBLIC),
                    TokenRef::global(defaults::STATIC),
                    TokenRef::global(101),
                    TokenRef::local(value),
                    c,
                ]
            ),
            format!(
                "g({}) = [g(1), g(3), g(101), lt(1), c(1)]",
                FIRST_PROJECT_TOKEN + 1
            )
        );
        assert_eq!(hero, TokenRef::global(FIRST_PROJECT_TOKEN + 1));
    }
}
