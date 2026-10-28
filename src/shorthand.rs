//! Detection of C# native shorthand: the ternary operator and the other
//! syntax-level techniques the corpus uses instead of explicit blocks.
//!
//! This is a **line detector**, not a parser. It masks comments, string and
//! char literals first, then looks for operators and keyword shapes on what is
//! left. That is enough to label a source line with the techniques it uses
//! (`g(77)` ternary, `g(78)` `??`, `g(80)` `?.`, …) without claiming to know
//! the expression tree.
//!
//! It deliberately does not try to be Roslyn: a multi-line interpolated string
//! or a multi-line comment is handled only on the line where it starts. The
//! engine already owns the authoritative lexer; this module is the cheap
//! per-line tagger that feeds the condensed store.
//!
//! ## Tokens
//!
//! Every feature maps to a pre-assigned `g` token in the `77..=99` construct
//! range (and, where the construct already has a C# keyword token, to that
//! keyword token instead). See [`Feature::token`].

use crate::token::Token;

/// The construct tokens, matching [`crate::defaults`].
pub const TERNARY: u32 = 77;
pub const NULL_COALESCE: u32 = 78;
pub const NULL_COALESCE_ASSIGN: u32 = 79;
pub const NULL_CONDITIONAL: u32 = 80;
pub const EXPRESSION_BODIED: u32 = 81;
pub const TARGET_TYPED_NEW: u32 = 82;
pub const TUPLE: u32 = 83;
pub const DISCARD: u32 = 84;
pub const RANGE: u32 = 85;
pub const INDEX_FROM_END: u32 = 86;
pub const USING_DECLARATION: u32 = 87;
pub const NAMEOF: u32 = 88;
pub const SWITCH_EXPRESSION: u32 = 89;
pub const WITH_EXPRESSION: u32 = 90;
pub const INIT_ACCESSOR: u32 = 91;
pub const THROW_EXPRESSION: u32 = 92;
pub const LAMBDA: u32 = 93;
pub const INTERPOLATED_STRING: u32 = 94;
pub const VERBATIM_STRING: u32 = 95;
pub const RAW_STRING: u32 = 96;
pub const DECONSTRUCTION: u32 = 97;
pub const CHECKED_EXPRESSION: u32 = 98;

/// One detected C# shorthand technique.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Feature {
    /// `cond ? a : b`
    Ternary,
    /// `a ?? b`
    NullCoalesce,
    /// `a ??= b`
    NullCoalesceAssign,
    /// `a?.b` / `a?[i]`
    NullConditional,
    /// `=>` on a member (`int X => _x;`)
    ExpressionBodied,
    /// `new()` / `new[] { … }` with the type inferred.
    TargetTypedNew,
    /// `(a, b)` value tuple.
    Tuple,
    /// `_` as a discard.
    Discard,
    /// `a..b`
    Range,
    /// `a[^1]`
    IndexFromEnd,
    /// `using var x = …;`
    UsingDeclaration,
    /// `nameof(x)`
    NameOf,
    /// `x switch { … }`
    SwitchExpression,
    /// `x with { … }`
    WithExpression,
    /// `init;` / `init =>`
    InitAccessor,
    /// `cond ? value : throw …`
    ThrowExpression,
    /// `x => …`
    Lambda,
    /// `$"…"`
    InterpolatedString,
    /// `@"…"`
    VerbatimString,
    /// `"""…"""`
    RawString,
    /// `(a, b) = …` / `var (a, b) = …`
    Deconstruction,
    /// `checked(…)` / `unchecked(…)`
    CheckedExpression,
    /// `x is T`
    Pattern,
    /// `var x = …`
    Var,
    /// `default` with no type argument.
    DefaultLiteral,
}

impl Feature {
    /// The pre-assigned token this feature is labelled with.
    pub const fn token(self) -> Token {
        let n = match self {
            Feature::Ternary => TERNARY,
            Feature::NullCoalesce => NULL_COALESCE,
            Feature::NullCoalesceAssign => NULL_COALESCE_ASSIGN,
            Feature::NullConditional => NULL_CONDITIONAL,
            Feature::ExpressionBodied => EXPRESSION_BODIED,
            Feature::TargetTypedNew => TARGET_TYPED_NEW,
            Feature::Tuple => TUPLE,
            Feature::Discard => DISCARD,
            Feature::Range => RANGE,
            Feature::IndexFromEnd => INDEX_FROM_END,
            Feature::UsingDeclaration => USING_DECLARATION,
            Feature::NameOf => NAMEOF,
            Feature::SwitchExpression => SWITCH_EXPRESSION,
            Feature::WithExpression => WITH_EXPRESSION,
            Feature::InitAccessor => INIT_ACCESSOR,
            Feature::ThrowExpression => THROW_EXPRESSION,
            Feature::Lambda => LAMBDA,
            Feature::InterpolatedString => INTERPOLATED_STRING,
            Feature::VerbatimString => VERBATIM_STRING,
            Feature::RawString => RAW_STRING,
            Feature::Deconstruction => DECONSTRUCTION,
            Feature::CheckedExpression => CHECKED_EXPRESSION,
            Feature::Pattern => crate::defaults::IS,
            Feature::Var => crate::defaults::VAR,
            Feature::DefaultLiteral => crate::defaults::DEFAULT,
        };
        Token(n)
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Feature::Ternary => "ternary",
            Feature::NullCoalesce => "null-coalesce",
            Feature::NullCoalesceAssign => "null-coalesce-assign",
            Feature::NullConditional => "null-conditional",
            Feature::ExpressionBodied => "expression-bodied",
            Feature::TargetTypedNew => "target-typed-new",
            Feature::Tuple => "tuple",
            Feature::Discard => "discard",
            Feature::Range => "range",
            Feature::IndexFromEnd => "index-from-end",
            Feature::UsingDeclaration => "using-declaration",
            Feature::NameOf => "nameof",
            Feature::SwitchExpression => "switch-expression",
            Feature::WithExpression => "with-expression",
            Feature::InitAccessor => "init-accessor",
            Feature::ThrowExpression => "throw-expression",
            Feature::Lambda => "lambda",
            Feature::InterpolatedString => "interpolated-string",
            Feature::VerbatimString => "verbatim-string",
            Feature::RawString => "raw-string",
            Feature::Deconstruction => "deconstruction",
            Feature::CheckedExpression => "checked-expression",
            Feature::Pattern => "pattern",
            Feature::Var => "var",
            Feature::DefaultLiteral => "default-literal",
        }
    }
}

/// One source line's techniques, 1-based.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LineFeatures {
    pub line: usize,
    pub features: Vec<Feature>,
}

impl LineFeatures {
    pub fn tokens(&self) -> Vec<Token> {
        self.features.iter().map(|f| f.token()).collect()
    }
}

/// Detect every shorthand technique on one line.
///
/// Comments and string/char literals are masked first; their contents cannot
/// produce a false `?:` or `??`.
pub fn detect_line(line: &str) -> Vec<Feature> {
    let (masked, strings) = mask_and_classify(line);
    let mut out: Vec<Feature> = Vec::new();
    let mut push = |f: Feature| {
        if !out.contains(&f) {
            out.push(f);
        }
    };

    if strings.interpolated {
        push(Feature::InterpolatedString);
    }
    if strings.verbatim {
        push(Feature::VerbatimString);
    }
    if strings.raw {
        push(Feature::RawString);
    }

    if has_ternary(&masked) {
        push(Feature::Ternary);
    }
    if masked.contains("??=") {
        push(Feature::NullCoalesceAssign);
    } else if masked.contains("??") {
        push(Feature::NullCoalesce);
    }
    if masked.contains("?.") || masked.contains("?[") {
        push(Feature::NullConditional);
    }
    if masked.contains("=>") {
        if is_expression_bodied(&masked) {
            push(Feature::ExpressionBodied);
        } else {
            push(Feature::Lambda);
        }
    }
    if has_target_typed_new(&masked) {
        push(Feature::TargetTypedNew);
    }
    if has_tuple(&masked) {
        push(Feature::Tuple);
    }
    if has_discard(&masked) {
        push(Feature::Discard);
    }
    if has_range(&masked) {
        push(Feature::Range);
    }
    if has_index_from_end(&masked) {
        push(Feature::IndexFromEnd);
    }
    if has_using_declaration(&masked) {
        push(Feature::UsingDeclaration);
    }
    if word_followed_by(&masked, "nameof", '(') {
        push(Feature::NameOf);
    }
    if word_followed_by(&masked, "switch", '{') {
        push(Feature::SwitchExpression);
    }
    if word_followed_by(&masked, "with", '{') {
        push(Feature::WithExpression);
    }
    if has_init_accessor(&masked) {
        push(Feature::InitAccessor);
    }
    if has_throw_expression(&masked) {
        push(Feature::ThrowExpression);
    }
    if word_followed_by(&masked, "checked", '(') || word_followed_by(&masked, "unchecked", '(') {
        push(Feature::CheckedExpression);
    }
    if word_followed_by(&masked, "is", ' ') || masked.contains(" is ") {
        push(Feature::Pattern);
    }
    if has_var(&masked) {
        push(Feature::Var);
    }
    if has_default_literal(&masked) {
        push(Feature::DefaultLiteral);
    }
    if has_deconstruction(&masked) {
        push(Feature::Deconstruction);
    }

    out
}

/// Detect techniques for every line of `src`.
pub fn detect_lines(src: &str) -> Vec<LineFeatures> {
    src.lines()
        .enumerate()
        .map(|(i, line)| LineFeatures {
            line: i + 1,
            features: detect_line(line),
        })
        .filter(|l| !l.features.is_empty())
        .collect()
}

/// Count each technique across `src`, in the enum's stable order.
pub fn counts(src: &str) -> Vec<(Feature, usize)> {
    let mut all: Vec<Feature> = Vec::new();
    for line in src.lines() {
        for f in detect_line(line) {
            if !all.contains(&f) {
                all.push(f);
            }
        }
    }
    let mut out: Vec<(Feature, usize)> = all
        .into_iter()
        .map(|f| {
            let n = src.lines().filter(|l| detect_line(l).contains(&f)).count();
            (f, n)
        })
        .collect();
    out.sort_by_key(|(f, _)| *f);
    out
}

// ---------------------------------------------------------------------------
// masking
// ---------------------------------------------------------------------------

#[derive(Default)]
struct StringFlags {
    interpolated: bool,
    verbatim: bool,
    raw: bool,
}

/// Replace comments, strings and char literals with spaces while preserving
/// column positions; report which string forms appeared.
fn mask_and_classify(line: &str) -> (String, StringFlags) {
    let chars: Vec<char> = line.chars().collect();
    let mut out = String::with_capacity(line.len());
    let mut flags = StringFlags::default();
    let mut i = 0usize;
    while i < chars.len() {
        let c = chars[i];
        if c == '/' && chars.get(i + 1) == Some(&'/') {
            while i < chars.len() {
                out.push(' ');
                i += 1;
            }
            break;
        }
        if c == '/' && chars.get(i + 1) == Some(&'*') {
            let mut closed = false;
            while i < chars.len() {
                if chars[i] == '*' && chars.get(i + 1) == Some(&'/') {
                    out.push_str("  ");
                    i += 2;
                    closed = true;
                    break;
                }
                out.push(' ');
                i += 1;
            }
            let _ = closed;
            continue;
        }
        if c == '\'' {
            // char literal: skip to the closing quote, honouring \'
            out.push(' ');
            i += 1;
            while i < chars.len() {
                if chars[i] == '\\' {
                    out.push(' ');
                    i += 1;
                    if i < chars.len() {
                        out.push(' ');
                        i += 1;
                    }
                    continue;
                }
                let done = chars[i] == '\'';
                out.push(' ');
                i += 1;
                if done {
                    break;
                }
            }
            continue;
        }
        if c == '$' || c == '@' {
            // `$"…"`, `@"…"`, `$@"…"`, `@$"…"`.
            let mut j = i;
            let mut interp = false;
            let mut verb = false;
            while j < chars.len() && (chars[j] == '$' || chars[j] == '@') {
                if chars[j] == '$' {
                    interp = true;
                } else {
                    verb = true;
                }
                j += 1;
            }
            if chars.get(j) == Some(&'"') {
                flags.interpolated |= interp;
                flags.verbatim |= verb;
                while i <= j {
                    out.push(' ');
                    i += 1;
                }
                while i < chars.len() {
                    if verb && chars[i] == '"' && chars.get(i + 1) == Some(&'"') {
                        out.push_str("  ");
                        i += 2;
                        continue;
                    }
                    let done = chars[i] == '"';
                    out.push(' ');
                    i += 1;
                    if done {
                        break;
                    }
                }
                continue;
            }
        }
        if c == '"' {
            // Raw string `"""`.
            if chars.get(i + 1) == Some(&'"') && chars.get(i + 2) == Some(&'"') {
                flags.raw = true;
                out.push_str("   ");
                i += 3;
                while i < chars.len() {
                    if chars[i] == '"'
                        && chars.get(i + 1) == Some(&'"')
                        && chars.get(i + 2) == Some(&'"')
                    {
                        out.push_str("   ");
                        i += 3;
                        break;
                    }
                    out.push(' ');
                    i += 1;
                }
                continue;
            }
            // Verbatim `@"` / interpolated `$"` / `$@"` / `@$"`.
            let mut verbatim = false;
            let mut interpolated = false;
            let mut j = i;
            while j < chars.len() && (chars[j] == '$' || chars[j] == '@') {
                if chars[j] == '$' {
                    interpolated = true;
                } else {
                    verbatim = true;
                }
                j += 1;
            }
            if j > i && chars.get(j) == Some(&'"') {
                flags.verbatim |= verbatim;
                flags.interpolated |= interpolated;
                while i <= j {
                    out.push(' ');
                    i += 1;
                }
                while i < chars.len() {
                    if verbatim && chars[i] == '"' && chars.get(i + 1) == Some(&'"') {
                        out.push_str("  ");
                        i += 2;
                        continue;
                    }
                    let done = chars[i] == '"';
                    out.push(' ');
                    i += 1;
                    if done && !verbatim {
                        break;
                    }
                    if done && verbatim {
                        break;
                    }
                }
                continue;
            }
            // Ordinary string.
            out.push(' ');
            i += 1;
            while i < chars.len() {
                if chars[i] == '\\' {
                    out.push(' ');
                    i += 1;
                    if i < chars.len() {
                        out.push(' ');
                        i += 1;
                    }
                    continue;
                }
                let done = chars[i] == '"';
                out.push(' ');
                i += 1;
                if done {
                    break;
                }
            }
            continue;
        }
        out.push(c);
        i += 1;
    }
    (out, flags)
}

// ---------------------------------------------------------------------------
// detectors
// ---------------------------------------------------------------------------

fn next_nonspace(bytes: &[u8], mut i: usize) -> Option<usize> {
    while i < bytes.len() {
        if !bytes[i].is_ascii_whitespace() {
            return Some(i);
        }
        i += 1;
    }
    None
}

fn prev_nonspace(bytes: &[u8], i: usize) -> Option<usize> {
    let mut j = i;
    while j > 0 {
        j -= 1;
        if !bytes[j].is_ascii_whitespace() {
            return Some(j);
        }
    }
    None
}

fn is_word_byte(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}

fn word_at(bytes: &[u8], i: usize, word: &str) -> bool {
    let w = word.as_bytes();
    if i + w.len() > bytes.len() || &bytes[i..i + w.len()] != w {
        return false;
    }
    let before_ok = i == 0 || !is_word_byte(bytes[i - 1]);
    let after = i + w.len();
    let after_ok = after >= bytes.len() || !is_word_byte(bytes[after]);
    before_ok && after_ok
}

fn word_followed_by(masked: &str, word: &str, next: char) -> bool {
    let b = masked.as_bytes();
    let mut i = 0;
    while let Some(pos) = masked[i..].find(word) {
        let at = i + pos;
        if word_at(b, at, word)
            && let Some(n) = next_nonspace(b, at + word.len())
            && b[n] == next as u8
        {
            return true;
        }
        i = at + word.len();
    }
    false
}

fn has_ternary(masked: &str) -> bool {
    let b = masked.as_bytes();
    for i in 0..b.len() {
        if b[i] != b'?' {
            continue;
        }
        let Some(n) = next_nonspace(b, i + 1) else {
            continue;
        };
        if matches!(b[n], b'.' | b'?' | b'[' | b'=') {
            continue;
        }
        // A `:` before the statement ends, skipping `::`.
        let mut j = n;
        while j < b.len() {
            match b[j] {
                b';' | b'{' | b'}' => break,
                b':' => {
                    if j + 1 < b.len() && b[j + 1] == b':' {
                        j += 2;
                        continue;
                    }
                    return true;
                }
                _ => j += 1,
            }
        }
    }
    false
}

fn is_expression_bodied(masked: &str) -> bool {
    let trimmed = masked.trim_start();
    const MODS: [&str; 8] = [
        "public",
        "private",
        "protected",
        "internal",
        "static",
        "override",
        "virtual",
        "abstract",
    ];
    MODS.iter().any(|m| trimmed.starts_with(m))
}

fn has_target_typed_new(masked: &str) -> bool {
    let b = masked.as_bytes();
    let mut i = 0;
    while let Some(pos) = masked[i..].find("new") {
        let at = i + pos;
        if word_at(b, at, "new")
            && let Some(n) = next_nonspace(b, at + 3)
            && matches!(b[n], b'(' | b'[' | b'{')
        {
            return true;
        }
        i = at + 3;
    }
    false
}

fn has_tuple(masked: &str) -> bool {
    let b = masked.as_bytes();
    for i in 0..b.len() {
        if b[i] != b'(' {
            continue;
        }
        if let Some(p) = prev_nonspace(b, i)
            && (is_word_byte(b[p]) || matches!(b[p], b']' | b')'))
        {
            continue;
        }
        let mut depth = 0i32;
        let mut comma = false;
        let mut j = i;
        while j < b.len() {
            match b[j] {
                b'(' => depth += 1,
                b')' => {
                    depth -= 1;
                    if depth == 0 {
                        if comma {
                            return true;
                        }
                        break;
                    }
                }
                b',' if depth == 1 => comma = true,
                b';' => break,
                _ => {}
            }
            j += 1;
        }
    }
    false
}

fn has_deconstruction(masked: &str) -> bool {
    let b = masked.as_bytes();
    // `var (a, b) =` or `(a, b) =`
    if word_followed_by(masked, "var", '(') {
        return true;
    }
    for i in 0..b.len() {
        if b[i] != b')' {
            continue;
        }
        let Some(eq) = next_nonspace(b, i + 1) else {
            continue;
        };
        if b[eq] != b'=' {
            continue;
        }
        // Walk back to the matching '(' and require a top-level comma.
        let mut depth = 0i32;
        let mut j = i;
        while j > 0 {
            j -= 1;
            match b[j] {
                b')' => depth += 1,
                b'(' => {
                    if depth == 0 {
                        let inside = &masked[j + 1..i];
                        if inside.contains(',') && !inside.contains(';') {
                            return true;
                        }
                        break;
                    }
                    depth -= 1;
                }
                _ => {}
            }
        }
    }
    false
}

fn has_discard(masked: &str) -> bool {
    let b = masked.as_bytes();
    for i in 0..b.len() {
        if b[i] != b'_' {
            continue;
        }
        if (i > 0 && is_word_byte(b[i - 1])) || (i + 1 < b.len() && is_word_byte(b[i + 1])) {
            continue;
        }
        if let Some(p) = prev_nonspace(b, i)
            && matches!(b[p], b'=' | b',' | b'(')
        {
            return true;
        }
        if let Some(n) = next_nonspace(b, i + 1)
            && matches!(b[n], b'=' | b',' | b')')
        {
            return true;
        }
    }
    false
}

fn has_range(masked: &str) -> bool {
    let b = masked.as_bytes();
    b.windows(2)
        .enumerate()
        .any(|(i, w)| w == b".." && b.get(i + 2) != Some(&b'.'))
}

fn has_index_from_end(masked: &str) -> bool {
    let b = masked.as_bytes();
    for i in 0..b.len() {
        if b[i] != b'^' {
            continue;
        }
        if i > 0 && is_word_byte(b[i - 1]) {
            continue;
        }
        if let Some(n) = next_nonspace(b, i + 1)
            && (is_word_byte(b[n]) || b[n] == b'(')
        {
            return true;
        }
    }
    false
}

fn has_using_declaration(masked: &str) -> bool {
    if word_followed_by(masked, "using", '(') {
        return false;
    }
    if word_followed_by(masked, "using", 'v') && masked.contains("using var") {
        return true;
    }
    // `using Type name = ...;` — a `using` that is not a directive and has an
    // assignment before the semicolon.
    if let Some(pos) = masked.find("using ") {
        let rest = &masked[pos + 6..];
        if !rest.trim_start().starts_with('(')
            && rest.contains('=')
            && rest
                .find(';')
                .is_some_and(|semi| rest[..semi].contains('='))
        {
            return true;
        }
    }
    false
}

fn has_init_accessor(masked: &str) -> bool {
    let b = masked.as_bytes();
    let mut i = 0;
    while let Some(pos) = masked[i..].find("init") {
        let at = i + pos;
        if word_at(b, at, "init")
            && let Some(n) = next_nonspace(b, at + 4)
            && matches!(b[n], b';' | b'{' | b'=' | b',')
        {
            return true;
        }
        i = at + 4;
    }
    false
}

fn has_throw_expression(masked: &str) -> bool {
    let b = masked.as_bytes();
    let mut i = 0;
    while let Some(pos) = masked[i..].find("throw") {
        let at = i + pos;
        if word_at(b, at, "throw")
            && let Some(p) = prev_nonspace(b, at)
            && matches!(b[p], b'?' | b':' | b',' | b'(' | b'=')
        {
            return true;
        }
        i = at + 5;
    }
    false
}

fn has_var(masked: &str) -> bool {
    let b = masked.as_bytes();
    let mut i = 0;
    while let Some(pos) = masked[i..].find("var") {
        let at = i + pos;
        if word_at(b, at, "var")
            && let Some(n) = next_nonspace(b, at + 3)
            && (is_word_byte(b[n]) || b[n] == b'(')
        {
            return true;
        }
        i = at + 3;
    }
    false
}

fn has_default_literal(masked: &str) -> bool {
    let b = masked.as_bytes();
    let mut i = 0;
    while let Some(pos) = masked[i..].find("default") {
        let at = i + pos;
        if word_at(b, at, "default")
            && let Some(n) = next_nonspace(b, at + 7)
            && !matches!(b[n], b'(' | b':')
        {
            return true;
        }
        i = at + 7;
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::defaults;

    #[test]
    fn construct_tokens_match_the_static_table() {
        for (feature, name) in [
            (Feature::Ternary, "?:"),
            (Feature::NullCoalesce, "??"),
            (Feature::NullCoalesceAssign, "??="),
            (Feature::NullConditional, "?."),
            (Feature::ExpressionBodied, "=>"),
            (Feature::TargetTypedNew, "new()"),
            (Feature::Tuple, "tuple"),
            (Feature::Discard, "discard"),
            (Feature::Range, "range"),
            (Feature::IndexFromEnd, "index-from-end"),
            (Feature::UsingDeclaration, "using-declaration"),
            (Feature::NameOf, "nameof"),
            (Feature::SwitchExpression, "switch-expression"),
            (Feature::WithExpression, "with-expression"),
            (Feature::InitAccessor, "init-accessor"),
            (Feature::ThrowExpression, "throw-expression"),
            (Feature::Lambda, "lambda"),
            (Feature::InterpolatedString, "interpolated-string"),
            (Feature::VerbatimString, "verbatim-string"),
            (Feature::RawString, "raw-string"),
            (Feature::Deconstruction, "deconstruction"),
            (Feature::CheckedExpression, "checked-expression"),
        ] {
            assert_eq!(
                defaults::token(name),
                Some(feature.token().0),
                "{name} token mismatch"
            );
        }
    }

    fn has(line: &str, feature: Feature) -> bool {
        detect_line(line).contains(&feature)
    }

    #[test]
    fn detects_ternary_but_not_nullable_or_strings() {
        assert!(has("var x = a ? b : c;", Feature::Ternary));
        assert!(has("return ok ? 1 : 0;", Feature::Ternary));
        assert!(!has("int? x = null;", Feature::Ternary));
        assert!(!has("var s = \"a ? b : c\";", Feature::Ternary));
        assert!(!has("// a ? b : c", Feature::Ternary));
        assert!(!has("/* a ? b : c */", Feature::Ternary));
    }

    #[test]
    fn detects_null_shorthands() {
        assert!(has("var x = a ?? b;", Feature::NullCoalesce));
        assert!(has("a ??= b;", Feature::NullCoalesceAssign));
        assert!(!has("a ??= b;", Feature::NullCoalesce));
        assert!(has("var x = a?.b;", Feature::NullConditional));
        assert!(has("var x = a?[0];", Feature::NullConditional));
        assert!(!has("var s = \"a ?? b\";", Feature::NullCoalesce));
    }

    #[test]
    fn detects_expression_bodied_and_lambda() {
        assert!(has("public int X => _x;", Feature::ExpressionBodied));
        assert!(!has("public int X => _x;", Feature::Lambda));
        assert!(has("items.Select(x => x + 1);", Feature::Lambda));
        assert!(!has("items.Select(x => x + 1);", Feature::ExpressionBodied));
    }

    #[test]
    fn detects_new_tuple_discard_range_index() {
        assert!(has("List<int> a = new();", Feature::TargetTypedNew));
        assert!(!has("var a = new List<int>();", Feature::TargetTypedNew));
        assert!(has("var t = (1, 2);", Feature::Tuple));
        assert!(!has("Foo(a, b);", Feature::Tuple));
        assert!(has("_ = Foo();", Feature::Discard));
        assert!(has("var s = a[1..];", Feature::Range));
        assert!(has("var last = a[^1];", Feature::IndexFromEnd));
        assert!(has("var (a, b) = Pair();", Feature::Deconstruction));
        assert!(has("(a, b) = (b, a);", Feature::Deconstruction));
    }

    #[test]
    fn detects_using_nameof_switch_with_init_throw_checked() {
        assert!(has(
            "using var f = File.OpenRead(p);",
            Feature::UsingDeclaration
        ));
        assert!(has("var n = nameof(Foo);", Feature::NameOf));
        assert!(has(
            "var y = x switch { 1 => \"a\", _ => \"b\" };",
            Feature::SwitchExpression
        ));
        assert!(has("var y = x with { A = 1 };", Feature::WithExpression));
        assert!(has("public int X { get; init; }", Feature::InitAccessor));
        assert!(has(
            "var x = ok ? 1 : throw new E();",
            Feature::ThrowExpression
        ));
        assert!(has("var x = checked(a + b);", Feature::CheckedExpression));
        assert!(has("if (x is string s) { }", Feature::Pattern));
        assert!(has("var x = Foo();", Feature::Var));
        assert!(has("var x = default;", Feature::DefaultLiteral));
        assert!(!has(
            "switch (x) { case 1: break; }",
            Feature::SwitchExpression
        ));
    }

    #[test]
    fn detects_string_forms_and_does_not_scan_inside_them() {
        assert!(has("var s = $\"a {b}\";", Feature::InterpolatedString));
        assert!(has("var s = @\"a\\b\";", Feature::VerbatimString));
        assert!(has("var s = \"\"\"raw ?: \"\"\";", Feature::RawString));
        assert!(!has("var s = $\"a ?? b\";", Feature::NullCoalesce));
        assert!(!has("var s = \"\"\"a ?? b\"\"\";", Feature::NullCoalesce));
    }

    #[test]
    fn keyword_backed_features_use_the_keyword_token() {
        assert_eq!(Feature::Pattern.token(), Token(62));
        assert_eq!(Feature::Var.token(), Token(64));
        assert_eq!(Feature::DefaultLiteral.token(), Token(33));
    }

    #[test]
    fn line_and_source_helpers_are_stable() {
        let src = "var x = a ? b : c;\nint y = 0;\n";
        let lines = detect_lines(src);
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].line, 1);
        assert_eq!(lines[0].features, vec![Feature::Ternary, Feature::Var]);
        assert_eq!(lines[0].tokens(), vec![Token(77), Token(64)]);

        let c = counts(src);
        assert!(c.contains(&(Feature::Ternary, 1)));
    }
}
