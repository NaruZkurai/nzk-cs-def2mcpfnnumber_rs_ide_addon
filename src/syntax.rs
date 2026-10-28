//! `syntax` — the one C# word tokenizer shared by the IDE and Scratch.
//!
//! The IDE's source pane and the Scratch puzzle pieces must colour the *same
//! character* the *same* way. Two hand-written colourisers drift: one learns
//! `[Attribute]` handling, the other keeps painting the whole attribute run as
//! one colour; one knows `#if`, the other does not. This module is the single
//! table both call, so a difference in the UI is a difference in the theme,
//! never in what a word *is*.
//!
//! It is display-free: no colours, no `egui`, no host types. Callers map
//! [`TokenKind`] to their own palette (`SourceTheme` in the IDE, the Scratch
//! palette in the puzzle pane).
//!
//! The spans are contiguous and cover every byte, so a painter can append the
//! runs in order and lose no whitespace. Comments, string/char literals and
//! preprocessor lines are single spans; everything else is a word, a number or
//! one run of punctuation.
//!
//! This is a *colouring* tokenizer, not a C# parser. The engine's own lexer
//! (`csgraph_core::lex::mask_noncode`) remains the authority on whether a
//! character is code when the IDE paints a synthesized document; the IDE
//! classifies the code regions the mask leaves. Scratch text is ordinary C#
//! source, so it can use this tokenizer directly.

#![forbid(unsafe_code)]

/// What a run of source characters is, for colouring.
///
/// The variants deliberately mirror the IDE's `Tok` enum and Monaco's Dark+
/// token classes; adding a variant is a UI decision, not a grammar one.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum TokenKind {
    /// Whitespace and anything unclassified.
    Plain,
    /// `// …` or `/* … */`, including `///` doc comments.
    Comment,
    /// A string or character literal (`"…"`, `@"…"`, `$"…"`, `"""…"""`, `'c'`).
    Str,
    /// A non-control C# keyword or a built-in type (`public`, `class`, `int`).
    Keyword,
    /// Control-flow / statement keywords (`if`, `return`, `using`, `yield`).
    Control,
    /// An upper-initial identifier used as a type (`NZK.SPS2`, `Transform`).
    Type,
    /// A numeric literal.
    Number,
    /// An identifier immediately followed by `(` — a method/constructor call.
    Function,
    /// Any other identifier: a local, field, parameter or enum member.
    Variable,
    /// Brackets, operators and punctuation, but **not** `[` attribute runs:
    /// an attribute's brackets are ordinary punctuation and its contents are
    /// classified normally, which is what makes `[Tooltip("x")]` match the IDE.
    Op,
    /// A preprocessor line (`#if`, `#region`, …), starting at `#`.
    Preproc,
}

/// A byte range of source text with one token kind.
///
/// `start`/`end` are byte offsets into the string passed to [`tokens`] and are
/// always on a character boundary.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Span {
    pub start: usize,
    pub end: usize,
    pub kind: TokenKind,
}

impl Span {
    pub fn text<'a>(&self, source: &'a str) -> &'a str {
        &source[self.start..self.end]
    }
}

/// C# keywords that are coloured as keywords (modifiers, declarations,
/// built-in types and literals). Not a grammar.
const KEYWORDS: &[&str] = &[
    "abstract", "as", "base", "bool", "byte", "char", "class", "const", "decimal",
    "delegate", "double", "enum", "event", "explicit", "extern", "false", "fixed",
    "float", "implicit", "int", "interface", "internal", "is", "lock", "long",
    "namespace", "new", "null", "object", "operator", "out", "override", "params",
    "private", "protected", "public", "readonly", "record", "ref", "sbyte",
    "sealed", "short", "sizeof", "stackalloc", "static", "string", "struct",
    "this", "true", "typeof", "uint", "ulong", "unchecked", "unsafe", "ushort",
    "var", "virtual", "void", "volatile", "partial", "where", "get", "set", "add",
    "remove", "init", "when", "nameof", "global", "required", "file", "scoped",
    "dynamic", "async", "await",
];

/// Control-flow / statement keywords, coloured as Monaco `keyword.control`.
const CONTROL: &[&str] = &[
    "if", "else", "for", "foreach", "while", "do", "switch", "case", "default",
    "break", "continue", "return", "goto", "throw", "try", "catch", "finally",
    "using", "checked", "yield", "select", "from", "let", "join", "on", "equals",
    "into", "orderby", "ascending", "descending", "group", "by", "async", "await",
];

pub fn is_keyword(word: &str) -> bool {
    KEYWORDS.contains(&word)
}

pub fn is_control(word: &str) -> bool {
    CONTROL.contains(&word)
}

/// Classify one already-separated identifier.
///
/// `followed_by_paren` is computed by the scanner (it skips whitespace), so a
/// caller that already knows the next token can reuse this rule instead of
/// re-implementing it.
pub fn classify_word(word: &str, followed_by_paren: bool) -> TokenKind {
    if is_control(word) {
        TokenKind::Control
    } else if is_keyword(word) {
        TokenKind::Keyword
    } else if followed_by_paren {
        TokenKind::Function
    } else if word.chars().next().is_some_and(|c| c.is_uppercase()) {
        TokenKind::Type
    } else {
        TokenKind::Variable
    }
}

/// Split `text` into contiguous spans, covering every byte.
///
/// The returned vector is never empty for a non-empty input and always starts
/// at byte 0 / ends at `text.len()`. Adjacent spans may share a kind; callers
/// that want fewer sections can merge them.
pub fn tokens(text: &str) -> Vec<Span> {
    let b = text.as_bytes();
    let n = b.len();
    let mut out: Vec<Span> = Vec::new();
    let mut i = 0usize;
    // True until the first non-whitespace character after a newline: `#` only
    // starts a directive there, exactly like the C# preprocessor.
    let mut line_start = true;
    while i < n {
        let start = i;
        let c = b[i];

        if c.is_ascii_whitespace() {
            while i < n && b[i].is_ascii_whitespace() {
                if b[i] == b'\n' {
                    line_start = true;
                }
                i += 1;
            }
            out.push(Span { start, end: i, kind: TokenKind::Plain });
            continue;
        }

        // A directive owns its whole line. `#if` in a Scratch block is a
        // piece of its own; the IDE paints the same span as preproc.
        if c == b'#' && line_start {
            while i < n && b[i] != b'\n' {
                i += 1;
            }
            out.push(Span { start, end: i, kind: TokenKind::Preproc });
            line_start = false;
            continue;
        }
        line_start = false;

        if c == b'/' && i + 1 < n && b[i + 1] == b'/' {
            i += 2;
            while i < n && b[i] != b'\n' {
                i += 1;
            }
            out.push(Span { start, end: i, kind: TokenKind::Comment });
            continue;
        }
        if c == b'/' && i + 1 < n && b[i + 1] == b'*' {
            i += 2;
            while i + 1 < n && !(b[i] == b'*' && b[i + 1] == b'/') {
                i += 1;
            }
            i = (i + 2).min(n);
            out.push(Span { start, end: i, kind: TokenKind::Comment });
            continue;
        }

        if is_string_start(b, i) {
            i = string_end(b, i);
            out.push(Span { start, end: i, kind: TokenKind::Str });
            continue;
        }

        if c.is_ascii_digit() {
            i += 1;
            while i < n && (b[i].is_ascii_alphanumeric() || b[i] == b'.' || b[i] == b'_') {
                i += 1;
            }
            out.push(Span { start, end: i, kind: TokenKind::Number });
            continue;
        }

        if is_ident_start(b, i) {
            i += 1;
            while i < n && is_ident_continue(b, i) {
                i += word_char_len(b, i, n);
            }
            let word = &text[start..i];
            let mut k = i;
            while k < n && b[k].is_ascii_whitespace() {
                k += 1;
            }
            let call = k < n && b[k] == b'(';
            out.push(Span {
                start,
                end: i,
                kind: classify_word(word, call),
            });
            continue;
        }

        // One run of punctuation/operators, but never swallowing the start of a
        // comment, a string, a directive or a word. `//` inside this run would
        // otherwise become punctuation and lose its comment colour.
        i += 1;
        while i < n {
            let d = b[i];
            if d.is_ascii_whitespace()
                || is_ident_start(b, i)
                || d.is_ascii_digit()
                || is_string_start(b, i)
                || (d == b'/' && i + 1 < n && (b[i + 1] == b'/' || b[i + 1] == b'*'))
            {
                break;
            }
            i += 1;
        }
        out.push(Span { start, end: i, kind: TokenKind::Op });
    }
    out
}

fn is_ident_start(b: &[u8], i: usize) -> bool {
    match b[i] {
        c if c.is_ascii_alphabetic() || c == b'_' => true,
        // `@class` is an escaped identifier. The `@` is part of the word so a
        // verbatim string (`@"…"`) is handled before this branch by
        // `string_end`; an isolated `@` is punctuation.
        b'@' => b.get(i + 1).is_some_and(|c| c.is_ascii_alphabetic() || *c == b'_'),
        c if c >= 0x80 => true,
        _ => false,
    }
}

fn is_ident_continue(b: &[u8], i: usize) -> bool {
    b[i].is_ascii_alphanumeric() || b[i] == b'_' || b[i] >= 0x80
}

/// Number of bytes in the UTF-8 character at `i`, capped at `n`.
fn word_char_len(b: &[u8], i: usize, n: usize) -> usize {
    let c = b[i];
    let len = if c < 0x80 {
        1
    } else if c >> 5 == 0b110 {
        2
    } else if c >> 4 == 0b1110 {
        3
    } else {
        4
    };
    len.min(n - i).max(1)
}

fn is_string_start(b: &[u8], i: usize) -> bool {
    match b[i] {
        b'"' | b'\'' => true,
        b'@' => matches!(b.get(i + 1), Some(b'"')),
        b'$' => matches!(b.get(i + 1), Some(b'"') | Some(b'@') | Some(b'$')),
        _ => false,
    }
}

fn string_end(b: &[u8], start: usize) -> usize {
    let n = b.len();
    let mut i = start;
    while i < n && (b[i] == b'$' || b[i] == b'@') {
        i += 1;
    }
    if i >= n || (b[i] != b'"' && b[i] != b'\'') {
        return i;
    }
    let q = b[i];
    // Count the opening run: 2 = the empty string, 3+ = a raw string.
    let mut run = 0usize;
    while i < n && b[i] == q {
        run += 1;
        i += 1;
    }
    if q == b'"' && run >= 3 {
        let mut r = 0usize;
        while i < n {
            if b[i] == b'"' {
                r += 1;
                if r == run {
                    return i + 1;
                }
            } else {
                r = 0;
            }
            i += 1;
        }
        return n;
    }
    if q == b'"' && run == 2 {
        return i;
    }
    let verbatim = b[start..i].contains(&b'@');
    while i < n {
        if b[i] == b'\\' && !verbatim {
            i += 2;
            continue;
        }
        if b[i] == q {
            // `""` inside a verbatim string is an escaped quote.
            if verbatim && q == b'"' && i + 1 < n && b[i + 1] == b'"' {
                i += 2;
                continue;
            }
            return i + 1;
        }
        i += 1;
    }
    n
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kind_at<'a>(text: &'a str, spans: &[Span], needle: &str) -> TokenKind {
        let i = text.find(needle).unwrap_or_else(|| panic!("{needle} not in {text}"));
        spans
            .iter()
            .find(|s| s.start <= i && i < s.end)
            .unwrap_or_else(|| panic!("no span covers {needle}"))
            .kind
    }

    #[test]
    fn spans_cover_every_byte() {
        let text = "public void A(int n) { /* x */ return; }\n";
        let spans = tokens(text);
        assert_eq!(spans.first().unwrap().start, 0);
        assert_eq!(spans.last().unwrap().end, text.len());
        for pair in spans.windows(2) {
            assert_eq!(pair[0].end, pair[1].start, "no gap between spans");
        }
    }

    #[test]
    fn attributes_are_punctuation_with_normal_contents() {
        // The bug this fixes: Scratch used to paint the whole `[ … ]` run one
        // colour, so the IDE and the blocks disagreed about every attribute.
        let text = "[UnityEngine.Tooltip(\"a;b\")] public string label = \"\";";
        let spans = tokens(text);
        assert_eq!(kind_at(text, &spans, "["), TokenKind::Op);
        assert_eq!(kind_at(text, &spans, "UnityEngine"), TokenKind::Type);
        assert_eq!(kind_at(text, &spans, "Tooltip"), TokenKind::Function);
        assert_eq!(kind_at(text, &spans, "\"a;b\""), TokenKind::Str);
        assert_eq!(kind_at(text, &spans, "]"), TokenKind::Op);
        assert_eq!(kind_at(text, &spans, "public"), TokenKind::Keyword);
        assert_eq!(kind_at(text, &spans, "label"), TokenKind::Variable);
    }

    #[test]
    fn namespace_and_its_name_are_not_comments() {
        let text = "namespace NZK.SPS2 {";
        let spans = tokens(text);
        assert_eq!(kind_at(text, &spans, "namespace"), TokenKind::Keyword);
        assert_eq!(kind_at(text, &spans, "NZK"), TokenKind::Type);
        assert_eq!(kind_at(text, &spans, "SPS2"), TokenKind::Type);
        assert_eq!(kind_at(text, &spans, "{"), TokenKind::Op);
    }

    #[test]
    fn comments_and_strings_win_over_operators() {
        let text = "var s = \"http://x\"; // tail\n/* block */ var t = @\"a\"\"b\";";
        let spans = tokens(text);
        assert_eq!(kind_at(text, &spans, "\"http://x\""), TokenKind::Str);
        assert_eq!(kind_at(text, &spans, "// tail"), TokenKind::Comment);
        assert_eq!(kind_at(text, &spans, "/* block */"), TokenKind::Comment);
        assert_eq!(kind_at(text, &spans, "@\"a\"\"b\""), TokenKind::Str);
        assert_eq!(kind_at(text, &spans, "var"), TokenKind::Keyword);
    }

    #[test]
    fn control_words_are_distinct_from_other_keywords() {
        let text = "if (x) { return; } public static class C { }";
        let spans = tokens(text);
        assert_eq!(kind_at(text, &spans, "if"), TokenKind::Control);
        assert_eq!(kind_at(text, &spans, "return"), TokenKind::Control);
        assert_eq!(kind_at(text, &spans, "public"), TokenKind::Keyword);
        assert_eq!(kind_at(text, &spans, "static"), TokenKind::Keyword);
        assert_eq!(kind_at(text, &spans, "class"), TokenKind::Keyword);
    }

    #[test]
    fn numbers_and_calls_are_classified() {
        let text = "var x = 0.25f; Go(1); Name";
        let spans = tokens(text);
        assert_eq!(kind_at(text, &spans, "0.25f"), TokenKind::Number);
        assert_eq!(kind_at(text, &spans, "Go"), TokenKind::Function);
        assert_eq!(kind_at(text, &spans, "Name"), TokenKind::Type);
    }

    #[test]
    fn a_directive_owns_its_line() {
        let text = "#if UNITY_EDITOR\nvoid A() { }\n#endif";
        let spans = tokens(text);
        assert_eq!(kind_at(text, &spans, "#if UNITY_EDITOR"), TokenKind::Preproc);
        assert_eq!(kind_at(text, &spans, "#endif"), TokenKind::Preproc);
        assert_eq!(kind_at(text, &spans, "void"), TokenKind::Keyword);
    }

    #[test]
    fn empty_input_is_no_spans() {
        assert!(tokens("").is_empty());
    }
}
