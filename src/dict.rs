//! The persistent global dictionary (`g` token space).
//!
//! # Allocation
//!
//! * The static table in [`crate::defaults`] owns `1..=199`. It is compiled in,
//!   never indexed, and system types occupy `100..=149` so they are resolved
//!   before anything a project defines.
//! * Project definitions are allocated from [`FIRST_PROJECT_TOKEN`] (1000)
//!   upwards, in the order the caller indexes them.
//! * Tokens are **sticky**: re-assigning a `full` name returns its previous
//!   token. [`Dictionary::dump`] persists the map, and [`Dictionary::load`]
//!   restores it, so a rebuild that adds a method never renumbers the methods
//!   that were already there. Removing a definition leaves its entry in place
//!   (a tombstone) rather than reusing the number for something else.
//!
//! # Sub-dictionaries
//!
//! Every owner (`namespace`, `type`, or the synthetic module name) gets an
//! ordered member list. That is the "type gets its own sub dictionary" rule:
//! `Rwby.Hero` is addressable as `g(201)` globally *and* as member 2 of the
//! `NamespaceRwby` sub-dictionary. The sub-dictionary stores the same global
//! number, not a second numbering space, so a token is always globally unique.

use std::collections::{BTreeMap, HashMap};

use crate::defaults::{self, DefaultToken};
use crate::token::Token;
use crate::util::{escape, unescape};

/// First token handed to a project definition. Everything below this belongs to
/// the static default table.
pub const FIRST_PROJECT_TOKEN: u32 = 1000;

/// What a project definition is.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum DefKind {
    Namespace,
    Type,
    Delegate,
    EnumMember,
    Constructor,
    Method,
    Property,
    Field,
    Event,
    Local,
}

impl DefKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            DefKind::Namespace => "namespace",
            DefKind::Type => "type",
            DefKind::Delegate => "delegate",
            DefKind::EnumMember => "enum-member",
            DefKind::Constructor => "constructor",
            DefKind::Method => "method",
            DefKind::Property => "property",
            DefKind::Field => "field",
            DefKind::Event => "event",
            DefKind::Local => "local",
        }
    }

    pub fn parse(s: &str) -> Option<DefKind> {
        match s {
            "namespace" => Some(DefKind::Namespace),
            "type" => Some(DefKind::Type),
            "delegate" => Some(DefKind::Delegate),
            "enum-member" => Some(DefKind::EnumMember),
            "constructor" => Some(DefKind::Constructor),
            "method" => Some(DefKind::Method),
            "property" => Some(DefKind::Property),
            "field" => Some(DefKind::Field),
            "event" => Some(DefKind::Event),
            "local" => Some(DefKind::Local),
            _ => None,
        }
    }
}

/// Declared accessibility. Kept as a value rather than only a modifier list so
/// "no declared access" is distinct from "explicitly public".
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Access {
    Public,
    Internal,
    Protected,
    Private,
    ProtectedInternal,
    PrivateProtected,
    None,
}

impl Access {
    pub const fn as_str(self) -> &'static str {
        match self {
            Access::Public => "public",
            Access::Internal => "internal",
            Access::Protected => "protected",
            Access::Private => "private",
            Access::ProtectedInternal => "protected-internal",
            Access::PrivateProtected => "private-protected",
            Access::None => "none",
        }
    }

    pub fn parse(s: &str) -> Option<Access> {
        match s {
            "public" => Some(Access::Public),
            "internal" => Some(Access::Internal),
            "protected" => Some(Access::Protected),
            "private" => Some(Access::Private),
            "protected-internal" => Some(Access::ProtectedInternal),
            "private-protected" => Some(Access::PrivateProtected),
            "none" => Some(Access::None),
            _ => None,
        }
    }
}

/// A definition as the indexer hands it over.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Def {
    /// Fully qualified name, e.g. `NamespaceRwby.Hero`.
    pub full: String,
    /// Last segment, e.g. `Hero`.
    pub name: String,
    pub kind: DefKind,
    /// Fully qualified owner (namespace or enclosing type); empty at the root.
    pub owner: String,
    pub file: String,
    pub line: u32,
    pub end_line: u32,
    pub access: Access,
    /// Modifier names as the parser saw them (`public`, `static`, `const`, …).
    pub modifiers: Vec<String>,
    /// Declared type (`System.String`, `NamespaceRwby.Hero`, …). Empty for a
    /// namespace and for a constructor.
    pub type_name: String,
}

impl Def {
    /// Build a definition from its full name; `name` and `owner` are derived on
    /// the last `.`. Use the builders to fill in the location and flags.
    pub fn new(full: impl Into<String>, kind: DefKind) -> Self {
        let full = full.into();
        let (owner, name) = match full.rsplit_once('.') {
            Some((owner, name)) => (owner.to_string(), name.to_string()),
            None => (String::new(), full.clone()),
        };
        Def {
            full,
            name,
            kind,
            owner,
            file: String::new(),
            line: 0,
            end_line: 0,
            access: Access::None,
            modifiers: Vec::new(),
            type_name: String::new(),
        }
    }

    pub fn at(mut self, file: impl Into<String>, line: u32, end_line: u32) -> Self {
        self.file = file.into();
        self.line = line;
        self.end_line = end_line;
        self
    }

    pub fn access(mut self, access: Access) -> Self {
        self.access = access;
        self
    }

    pub fn modifiers<I, S>(mut self, modifiers: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.modifiers = modifiers.into_iter().map(Into::into).collect();
        self
    }

    /// Declare this definition's type. Everything that has a type records it so
    /// the condensed store never has to re-parse a signature to answer "what
    /// type is this?".
    pub fn type_ref(mut self, ty: impl Into<String>) -> Self {
        self.type_name = ty.into();
        self
    }

    /// Override the derived owner (a nested type's owner may not be the dotted
    /// prefix when the parser normalises names differently).
    pub fn owner(mut self, owner: impl Into<String>) -> Self {
        self.owner = owner.into();
        self
    }
}

/// A project definition after it has a token.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry {
    pub token: Token,
    pub full: String,
    pub name: String,
    pub kind: DefKind,
    pub owner: String,
    pub file: String,
    pub line: u32,
    pub end_line: u32,
    pub access: Access,
    pub modifiers: Vec<String>,
    /// Declared type name as indexed.
    pub type_name: String,
    /// Resolved type token, filled by [`Dictionary::resolve_types`].
    pub type_token: Option<Token>,
}

impl Entry {
    /// Resolve this entry's modifier names to their default tokens. Unknown
    /// names are skipped rather than invented; a caller that needs strictness
    /// can compare lengths.
    pub fn modifier_tokens(&self) -> Vec<Token> {
        self.modifiers
            .iter()
            .filter_map(|m| defaults::token(m))
            .map(Token)
            .collect()
    }

    /// The access keyword's token, when it has one.
    pub fn access_token(&self) -> Option<Token> {
        defaults::token(self.access.as_str()).map(Token)
    }
}

/// A token resolved to either the static table or the project table.
#[derive(Clone, Copy, Debug)]
pub enum Describe<'a> {
    Default(&'static DefaultToken),
    Project(&'a Entry),
}

impl<'a> Describe<'a> {
    pub fn name(&self) -> &'a str {
        match self {
            Describe::Default(e) => e.name,
            Describe::Project(e) => e.full.as_str(),
        }
    }

    pub fn kind(&self) -> &'static str {
        match self {
            Describe::Default(e) => e.kind.as_str(),
            Describe::Project(e) => e.kind.as_str(),
        }
    }

    pub fn project(&self) -> Option<&'a Entry> {
        match self {
            Describe::Default(_) => None,
            Describe::Project(e) => Some(e),
        }
    }
}

/// Error from [`Dictionary::load`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LoadError(pub String);

impl std::fmt::Display for LoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for LoadError {}

/// Header written by [`Dictionary::dump`].
pub const DUMP_HEADER: &str = "def2mcpfnnumber-globals";
/// Format version for the dump body.
pub const DUMP_VERSION: u32 = 1;

/// The global dictionary.
#[derive(Clone, Debug)]
pub struct Dictionary {
    next: u32,
    entries: BTreeMap<u32, Entry>,
    by_full: HashMap<String, u32>,
    members: BTreeMap<String, Vec<u32>>,
}

impl Default for Dictionary {
    fn default() -> Self {
        Self::new()
    }
}

impl Dictionary {
    /// An empty project table; the static defaults are always resolvable.
    pub fn new() -> Self {
        Dictionary {
            next: FIRST_PROJECT_TOKEN,
            entries: BTreeMap::new(),
            by_full: HashMap::new(),
            members: BTreeMap::new(),
        }
    }

    /// The next number that will be handed out (without allocating it).
    pub fn next_token(&self) -> Token {
        Token(self.next)
    }

    /// Number of project (non-default) entries.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Iterate project entries in token order.
    pub fn iter(&self) -> impl Iterator<Item = &Entry> {
        self.entries.values()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Assign a token, or return the existing one when `full` is already known.
    pub fn assign(&mut self, def: Def) -> Token {
        if let Some(&token) = self.by_full.get(&def.full) {
            return Token(token);
        }
        let token = Token(self.next.max(FIRST_PROJECT_TOKEN));
        self.next = token.0 + 1;

        let owner = def.owner.clone();
        let entry = Entry {
            token,
            full: def.full.clone(),
            name: def.name,
            kind: def.kind,
            owner: owner.clone(),
            file: def.file,
            line: def.line,
            end_line: def.end_line,
            access: def.access,
            modifiers: def.modifiers,
            type_name: def.type_name,
            type_token: None,
        };

        if !owner.is_empty() {
            self.members.entry(owner).or_default().push(token.0);
        }
        self.by_full.insert(def.full, token.0);
        self.entries.insert(token.0, entry);
        token
    }

    /// Assign a batch in order, returning the tokens in the same order.
    pub fn assign_all<I: IntoIterator<Item = Def>>(&mut self, defs: I) -> Vec<Token> {
        defs.into_iter().map(|d| self.assign(d)).collect()
    }

    /// Resolve every entry's declared type to a token.
    ///
    /// Run this **after** the whole batch has been assigned: a method's return
    /// type may be declared later in the same index. Returns the number of
    /// entries whose type is still unresolved (a foreign type outside the
    /// corpus, or a typo), without failing the rest.
    pub fn resolve_types(&mut self) -> usize {
        let by_full = &self.by_full;
        let mut unresolved = 0;
        for entry in self.entries.values_mut() {
            if entry.type_name.is_empty() {
                entry.type_token = None;
                continue;
            }
            let base = defaults::type_base(&entry.type_name);
            entry.type_token = by_full
                .get(&entry.type_name)
                .copied()
                .or_else(|| by_full.get(base).copied())
                .map(Token)
                .or_else(|| defaults::token(base).map(Token));
            if entry.type_token.is_none() {
                unresolved += 1;
            }
        }
        unresolved
    }

    pub fn get(&self, token: Token) -> Option<&Entry> {
        self.entries.get(&token.0)
    }

    /// Exact fully qualified project name.
    pub fn token_of(&self, full: &str) -> Option<Token> {
        self.by_full.get(full).copied().map(Token)
    }

    /// Resolve a name in this order: static default, project full name. This is
    /// the order that guarantees `g(1)` stays `public` and a project cannot
    /// shadow a system type by declaring the same name.
    pub fn resolve(&self, name: &str) -> Option<Token> {
        if let Some(n) = defaults::token(name) {
            return Some(Token(n));
        }
        self.token_of(name)
    }

    /// Ordered members of one owner (namespace, type, or module name).
    pub fn members_of(&self, owner: &str) -> &[u32] {
        self.members.get(owner).map(Vec::as_slice).unwrap_or(&[])
    }

    /// Alias for [`Dictionary::members_of`]: the `thismodule` dictionary — the
    /// definitions owned by the module currently being edited.
    pub fn module_tokens(&self, module: &str) -> &[u32] {
        self.members_of(module)
    }

    /// Resolve an unqualified name against `module` first, then globally.
    ///
    /// This is how `thismodule.Hero` shortens to `Hero` inside the module that
    /// owns it, without making the name ambiguous elsewhere.
    pub fn resolve_in_module(&self, module: &str, name: &str) -> Option<Token> {
        let qualified = if module.is_empty() {
            name.to_string()
        } else {
            format!("{module}.{name}")
        };
        if let Some(t) = self.token_of(&qualified) {
            return Some(t);
        }
        if let Some(t) = self.token_of(name) {
            return Some(t);
        }
        if let Some(n) = defaults::token(name) {
            return Some(Token(n));
        }
        for &m in self.members_of(module) {
            if let Some(e) = self.entries.get(&m)
                && e.name == name
            {
                return Some(e.token);
            }
        }
        None
    }

    /// Resolve a token to its static or project meaning.
    pub fn describe(&self, token: Token) -> Option<Describe<'_>> {
        if let Some(e) = self.entries.get(&token.0) {
            return Some(Describe::Project(e));
        }
        defaults::get(token.0).map(Describe::Default)
    }

    /// Human-readable name for any resolvable token.
    pub fn name_of(&self, token: Token) -> Option<&str> {
        if let Some(e) = self.entries.get(&token.0) {
            return Some(e.full.as_str());
        }
        defaults::get(token.0).map(|e| e.name)
    }

    /// Case-insensitive substring search over project `full`/`name`, ordered by
    /// token. This is the "search by token" entry point when the query is a
    /// name; `get` is the direct token lookup.
    pub fn search(&self, query: &str) -> Vec<&Entry> {
        let q = query.to_lowercase();
        let mut hits: Vec<&Entry> = self
            .entries
            .values()
            .filter(|e| e.full.to_lowercase().contains(&q) || e.name.to_lowercase().contains(&q))
            .collect();
        hits.sort_by_key(|e| e.token.0);
        hits
    }

    /// Definitions owned by `owner`, resolved.
    pub fn members(&self, owner: &str) -> Vec<&Entry> {
        self.members_of(owner)
            .iter()
            .filter_map(|n| self.entries.get(n))
            .collect()
    }

    /// Serialize the project table. Static entries are not repeated: they are
    /// code, and [`defaults::TABLE_REV`] is what proves the code is the same.
    pub fn dump(&self) -> String {
        let mut out = String::new();
        out.push_str(&format!("{DUMP_HEADER} {DUMP_VERSION}\n"));
        out.push_str(&format!("defaults {}\n", defaults::TABLE_REV));
        out.push_str(&format!("next {}\n", self.next));
        for e in self.entries.values() {
            let mods = e
                .modifiers
                .iter()
                .map(|m| escape(m))
                .collect::<Vec<_>>()
                .join(",");
            out.push_str(&format!(
                "entry\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
                e.token.0,
                e.kind.as_str(),
                e.access.as_str(),
                e.line,
                e.end_line,
                escape(&e.full),
                escape(&e.name),
                escape(&e.owner),
                escape(&e.file),
                escape(&e.type_name),
                mods
            ));
        }
        out
    }

    /// Restore a dump produced by [`Dictionary::dump`].
    pub fn load(text: &str) -> Result<Dictionary, LoadError> {
        let mut lines = text.lines();
        let header = lines
            .next()
            .ok_or_else(|| LoadError("empty dump".to_string()))?;
        let mut hp = header.split_whitespace();
        if hp.next() != Some(DUMP_HEADER) {
            return Err(LoadError(format!("not a {DUMP_HEADER} dump: {header:?}")));
        }
        let version: u32 = hp
            .next()
            .and_then(|s| s.parse().ok())
            .ok_or_else(|| LoadError("missing dump version".to_string()))?;
        if version != DUMP_VERSION {
            return Err(LoadError(format!(
                "unsupported dump version {version} (expected {DUMP_VERSION})"
            )));
        }

        let defaults_line = lines
            .next()
            .ok_or_else(|| LoadError("missing defaults revision".to_string()))?;
        let defaults_rev: u32 = defaults_line
            .strip_prefix("defaults ")
            .and_then(|s| s.trim().parse().ok())
            .ok_or_else(|| LoadError(format!("bad defaults line {defaults_line:?}")))?;
        if defaults_rev != defaults::TABLE_REV {
            return Err(LoadError(format!(
                "defaults revision {defaults_rev} does not match this build's {}",
                defaults::TABLE_REV
            )));
        }

        let next_line = lines
            .next()
            .ok_or_else(|| LoadError("missing next counter".to_string()))?;
        let next: u32 = next_line
            .strip_prefix("next ")
            .and_then(|s| s.trim().parse().ok())
            .ok_or_else(|| LoadError(format!("bad next line {next_line:?}")))?;

        let mut dict = Dictionary::new();
        dict.next = next.max(FIRST_PROJECT_TOKEN);

        for line in lines {
            if line.trim().is_empty() {
                continue;
            }
            let fields: Vec<&str> = line.split('\t').collect();
            if fields.len() != 12 || fields[0] != "entry" {
                return Err(LoadError(format!("bad entry line: {line:?}")));
            }
            let token = parse_u32(fields[1], "token")?;
            if dict.entries.contains_key(&token) {
                return Err(LoadError(format!("duplicate token {token}")));
            }
            let kind = DefKind::parse(fields[2])
                .ok_or_else(|| LoadError(format!("bad kind {:?}", fields[2])))?;
            let access = Access::parse(fields[3])
                .ok_or_else(|| LoadError(format!("bad access {:?}", fields[3])))?;
            let entry = Entry {
                token: Token(token),
                full: unescape(fields[6]),
                name: unescape(fields[7]),
                kind,
                owner: unescape(fields[8]),
                file: unescape(fields[9]),
                line: parse_u32(fields[4], "line")?,
                end_line: parse_u32(fields[5], "endLine")?,
                access,
                modifiers: if fields[11].is_empty() {
                    Vec::new()
                } else {
                    fields[11].split(',').map(unescape).collect()
                },
                type_name: unescape(fields[10]),
                type_token: None,
            };
            if dict.by_full.contains_key(&entry.full) {
                return Err(LoadError(format!("duplicate full name {:?}", entry.full)));
            }
            if !entry.owner.is_empty() {
                dict.members
                    .entry(entry.owner.clone())
                    .or_default()
                    .push(entry.token.0);
            }
            dict.by_full.insert(entry.full.clone(), entry.token.0);
            dict.entries.insert(entry.token.0, entry);
        }

        for members in dict.members.values_mut() {
            members.sort_unstable();
            members.dedup();
        }
        dict.resolve_types();
        Ok(dict)
    }
}

fn parse_u32(s: &str, what: &str) -> Result<u32, LoadError> {
    s.trim()
        .parse()
        .map_err(|_| LoadError(format!("bad {what}: {s:?}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn def(full: &str, kind: DefKind) -> Def {
        Def::new(full, kind)
    }

    #[test]
    fn defaults_resolve_before_project_and_project_starts_at_first_project() {
        let mut d = Dictionary::new();
        assert_eq!(d.resolve("public"), Some(Token(1)));
        assert_eq!(d.resolve("System.String"), Some(Token(101)));
        assert_eq!(d.resolve("NamespaceRwby"), None);

        let t = d.assign(def("NamespaceRwby", DefKind::Namespace));
        assert_eq!(t, Token(FIRST_PROJECT_TOKEN));
        assert_eq!(d.resolve("NamespaceRwby"), Some(t));
        assert_eq!(d.next_token(), Token(FIRST_PROJECT_TOKEN + 1));
    }

    #[test]
    fn assignment_is_sticky_and_never_reuses_a_number() {
        let mut d = Dictionary::new();
        let a = d.assign(def("A", DefKind::Namespace));
        let b = d.assign(def("A.B", DefKind::Method));
        assert_eq!(d.assign(def("A", DefKind::Namespace)), a);
        assert_eq!(d.assign(def("A.B", DefKind::Method)), b);
        let c = d.assign(def("A.C", DefKind::Method));
        assert_eq!(c, Token(FIRST_PROJECT_TOKEN + 2));
        assert_ne!(b, c);
    }

    #[test]
    fn sub_dictionary_keeps_global_numbers() {
        let mut d = Dictionary::new();
        let ns = d.assign(def("NamespaceRwby", DefKind::Namespace));
        let hero = d.assign(def("NamespaceRwby.Hero", DefKind::Method));
        let villan = d.assign(def("NamespaceRwby.Villan", DefKind::Method));
        assert_eq!(d.members_of("NamespaceRwby"), &[hero.0, villan.0]);
        assert_eq!(d.module_tokens("NamespaceRwby"), &[hero.0, villan.0]);
        assert_eq!(ns, Token(FIRST_PROJECT_TOKEN));
        assert_eq!(d.resolve_in_module("NamespaceRwby", "Hero"), Some(hero));
        assert_eq!(d.resolve_in_module("NamespaceRwby", "Nope"), None);
    }

    #[test]
    fn dump_load_round_trips_and_preserves_next() {
        let mut d = Dictionary::new();
        d.assign(def("A", DefKind::Namespace));
        d.assign(
            def("A.B", DefKind::Method)
                .at("x y.cs", 3, 9)
                .access(Access::Public)
                .modifiers(["public", "static"]),
        );
        let text = d.dump();
        let mut back = Dictionary::load(&text).expect("load");
        assert_eq!(back.dump(), text);
        assert_eq!(back.next_token(), d.next_token());
        let c = back.assign(def("A.C", DefKind::Method));
        assert_eq!(c, Token(FIRST_PROJECT_TOKEN + 2));
    }

    #[test]
    fn load_rejects_a_stale_default_table_or_bad_header() {
        let mut d = Dictionary::new();
        d.assign(def("A", DefKind::Namespace));
        let mut text = d.dump();
        text = text.replace("defaults 2", "defaults 999");
        assert!(Dictionary::load(&text).is_err());

        assert!(Dictionary::load("nope 1").is_err());
    }

    #[test]
    fn search_is_case_insensitive_and_ordered() {
        let mut d = Dictionary::new();
        d.assign(def("NamespaceRwby.Hero", DefKind::Method));
        d.assign(def("NamespaceRwby.Villan", DefKind::Method));
        let hits = d.search("hero");
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].full, "NamespaceRwby.Hero");
        assert_eq!(hits[0].token, Token(FIRST_PROJECT_TOKEN));
    }

    #[test]
    fn modifiers_resolve_to_default_tokens() {
        let mut d = Dictionary::new();
        let token = d.assign(
            def("A.B", DefKind::Method)
                .access(Access::Public)
                .modifiers(["public", "static", "not-a-keyword"]),
        );
        let e = d.get(token).unwrap();
        assert_eq!(e.access_token(), Some(Token(1)));
        assert_eq!(e.modifier_tokens(), vec![Token(1), Token(3)]);
    }
}
