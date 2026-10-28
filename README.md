# `def2mcpfnnumber` — definition → global token number

Rust-only addon for **cs viewer v3**, the Scratch panes and the future
`gui.nzkide` / `idlink.nzkide` window: it gives every definition in the indexed
database a **stable global token number**, and stores the database in a
condensed token form that the viewer, Scratch and MCP can search by token.

Nothing here compiles C#. It only assigns/resolves tokens; generated text still
goes through the existing compile pipeline (`core/src/compile_check.rs` / Unity
Roslyn).

> Status: wired into cs viewer v3 behind `--tokens` (the packaged smoke test).
> Persisting the map during `Index::build` is still the next step.

---

## 1. The three token spaces

| Space | Spelling | Lifetime | Owner |
|---|---|---|---|
| Global | `g(1001)` | sticky, persisted | every namespace / type / method / field / modifier / keyword |
| Local | `lt(1,"value")` | one function body, mutable | parameters and locals |
| Comment | `c(7)` | static, interned once | `//` and `/**/` bodies |

`0` is reserved for "no token" in all three spaces.

A token is only ever meaningful **together with its space**: `g(1)` is
`public`, `c(1)` is the first comment, `lt(1)` is the first local of the current
function. `TokenRef` enforces that; a bare `Token` is just the number.

## 2. Number layout (the static table wins)

```
   1 ..  99   C# words + native shorthand constructs (`?:`=77 … `checked`=98)
 100 .. 899   SYSTEM TYPES — pre-assigned, first (335 today)
 900 .. 999   default methods on those types (88 today)
1000 ..       project definitions (allocated in index order)
```

* **System types are static and first.** `System.Object` = 100,
  `System.String` = 101, `UnityEngine.Vector3` = 139, `VRC.SDKBase.VRCPlayerApi`
  = 148, … They are compiled into the addon and the viewer; indexing can never
  invent a different number for them, and a project declaration cannot shadow
  them (`Dictionary::resolve` checks the static table first).
* The owner's explicit numbers are honoured: `public`=1, `private`=2,
  `static`=3, `const`=4, `void`=5, `return`=40.
* Project tokens start at `1000`; the system-type block is deliberately large
  so "pre-assign all system types" does not run out of room after 50 entries.
  The old sketch's `200` was a 50-type placeholder; it is now `1000`.
* Every project entry records `full`, `name`, `kind`, `owner`, `file`, `line`,
  `endLine`, `access`, `modifiers` and its declared **type** (`type_name`).
  `resolve_types()` fills `type_token` after the batch is assigned, so a method
  returning `System.String` carries `g(101)` and one returning `NamespaceRwby.Hero`
  carries `g(1001)`. A foreign/unknown type stays `None` rather than inventing a
  number.
* `resolve_types()` also strips generics/arrays/`?`/`ref` and `global::` before
  lookup (`type_base`), so `List<NZK.B>` resolves to the `List` token rather
  than staying unknown.

The table lives in [`src/defaults.rs`](src/defaults.rs). It is append-only;
changing an existing number requires bumping `defaults::TABLE_REV`, which makes
`Dictionary::load` reject old dumps instead of silently reinterpreting them.

## 3. Sticky allocation and the persisted map

`Dictionary::assign` returns the existing token when the fully qualified name is
already known; otherwise it takes the next number. `dump()` / `load()` persist
the map (one TAB-separated line per entry, no dependencies).

Consequences:

* rebuilding the index with one new method does **not** renumber the others;
* a removed definition leaves a tombstone (its entry stays in the map), so its
  number is never reused for something else;
* a dump written against a different default table is rejected.

## 4. Sub-dictionaries — `thismodule`

Every owner gets an ordered member list of the **same global numbers**:

```text
NamespaceRwby           g(1000)
├─ Hero                 g(1001)  (method)
│  ├─ HeroType0         g(1002)
│  └─ …                 g(1010)
├─ Villan               g(1011)
└─ party                g(1012)
```

* `members_of("NamespaceRwby")` → `[1001, 1011, 1012]`
* `members_of("NamespaceRwby.Hero")` → `[1002 … 1010]`
* `module_tokens(module)` is the `thismodule` view of that list.
* `resolve_in_module("NamespaceRwby", "Hero")` resolves the short name, then
  falls back to the global name and the static table.

The sub-dictionary is for authoring convenience, **not** a second numbering
space: `HeroType0` is `g(1002)` everywhere.

## 5. Condensed spelling

```text
g(1001) = [g(1), g(3), g(101), lt(1), c(7)]
lt(1,"value")
c(7)
```

* left side: the token being defined;
* bracketed list: the token stream of the definition (signature + body), in
  source order;
* `[]` is a real empty body, distinct from a missing definition line.

This is what a Scratch block or `idlink.nzkide` link can store instead of text.
The original comment text still lives in `Comments`, so `c(7)` expands exactly.

## 6. Detecting ternary / C# shorthand

`shorthand.rs` masks comments and string/char literals, then tags each line with
the native techniques it uses. Every feature maps to a pre-assigned token in
`77..=99` (or to the existing keyword token where there is one):

| feature | token | example |
|---|---|---|
| ternary | `g(77)` | `a ? b : c` |
| null-coalesce / assign | `g(78)` / `g(79)` | `a ?? b`, `a ??= b` |
| null-conditional | `g(80)` | `a?.b`, `a?[0]` |
| expression-bodied / lambda | `g(81)` / `g(93)` | `int X => _x;`, `x => x + 1` |
| target-typed `new()` | `g(82)` | `List<int> a = new();` |
| tuple / deconstruction | `g(83)` / `g(97)` | `(1, 2)`, `var (a, b) = …` |
| discard / range / index-from-end | `g(84)` / `g(85)` / `g(86)` | `_ = F();`, `a[1..]`, `a[^1]` |
| using-declaration / nameof | `g(87)` / `g(88)` | `using var f = …;`, `nameof(x)` |
| switch / with / init / throw expression | `g(89)` / `g(90)` / `g(91)` / `g(92)` | `x switch { … }`, … |
| interpolated / verbatim / raw string | `g(94)` / `g(95)` / `g(96)` | `$"…"`, `@"…"`, `"""…"""` |
| checked expression / pattern / var | `g(98)` / `g(62)` / `g(64)` | `checked(a+b)`, `x is T`, `var x` |

`--tokens` in the viewer prints a per-corpus tally (e.g. `ternary=490
null-coalesce=136 …`) on the 60-file `_sealed_src` corpus.

## 6b. The shared syntax tokenizer

`syntax.rs` is the one C# word tokenizer the IDE and Scratch both call. It
returns contiguous byte spans with a `TokenKind`:

```text
Plain  Comment  Str  Keyword  Control  Type  Number
Function  Variable  Op  Preproc
```

* `[Attribute(…)]` brackets are `Op`; the attribute's type/call/string are
  classified normally (this is what stopped Scratch painting a whole attribute
  run one colour).
* `namespace` is `Keyword` and its name is `Type`, never a comment.
* `//`, `/* */`, string/char literals and `#…` lines are single spans.
* An identifier before `(` is `Function`; an upper-initial one is `Type`; the
  rest are `Variable`.

The IDE maps this onto its `SourceTheme`; Scratch maps it onto the same palette.
Adding a colour therefore changes both surfaces at once.

## 7. What is implemented

| Area | API |
|---|---|
| Static table | `defaults::all/get/token`, `DEFAULT_MAX`, `PUBLIC`, `STATIC`, … |
| Global dictionary | `Dictionary::{assign, assign_all, resolve_types, token_of, resolve, get, describe, search, members_of, module_tokens, resolve_in_module}` |
| Persistence | `Dictionary::{dump, load}`; `LoadError` for stale/bad input |
| Locals | `Locals::{intern, name_of, token_of, rename, reset, dump, load}` |
| Comments | `Comments::{intern, text_of, token_of, dump, load}` |
| Condensed wire | `def_line`, `parse_def_line`, `local_decl`, `parse_local_decl`, `TokenRef` |
| Shorthand detect | `shorthand::{detect_line, detect_lines, counts, Feature}` |
| Session | `TokenIndex { globals, comments }`, `index_defs`, `comment_ref`, `function_locals` |

Run the demo and the tests:

```sh
export RUSTUP_HOME=/home/naruzkurai/.rustup CARGO_HOME=/home/naruzkurai/.cargo
export PATH="$CARGO_HOME/bin:$PATH"

cargo run
cargo test
```

## 8. Integration

1. **Index hook.** After `Index::build` has its declaration list (before nodes
   are emitted), load `<root>/.csgraph.v2/tokens.v1` if present, call
   `assign_all`, then `resolve_types()`, write the dump back, and put
   `"token": n` (and `"typeToken": n`) on each node record. This mirrors how
   `cache.rs` already persists parse/graph blobs.
2. **Viewer hook.** `app/src/model.rs` already copies node fields into `Node`;
   add `token`. The source pane and CodeLens can then show `g(1001)` and the
   IDE/Scratch link can resolve a block to a token without re-parsing names.
3. **Scratch / IDE.** `scratch.rs` stays read-only for now; when it writes, it
   emits the condensed form (`g(1001) = [...]`), and the IDE expands tokens back
   to C# through the dictionary and hands the result to the existing compile
   pipeline.

## 9. Deliberate choices / open questions

These points in the sketch were ambiguous; the prototype picks one reading and
keeps it isolated so it is cheap to change:

* **`g(1,3)` vs `g(1), g(3)`.** The sketch writes `G(1,3)` for "public static".
  This prototype stores each token separately (`[g(1), g(3)]`) so a token has
  exactly one meaning and search stays simple. A packed form would need its own
  framing.
* **"type is 300".** The sketch's `300` is now inside the pre-assigned system
  type block; a project type gets `1000+`. If project types should have their
  own reserved range, that is a one-constant change in `defaults.rs` + `dict.rs`.
* **Two token DBs.** Implemented as three spaces (`g`, `lt`, `c`) plus the
  `77..=99` construct range in one
  `TokenIndex`. The global map is the persisted one; locals are per function and
  comments are static. Additional spaces can be added to `token::Db` without
  changing the storage format of the existing ones.
* **File names.** The context mentions `gui.nzkide.cs` and
  `idelink.nzkide.rs`; because the IDE is Rust-only, `gui.nzkide.cs` looks like
  a `.cs`/`.rs` slip. Nothing in this crate depends on those names.
* **Stability on deletion.** Entries are kept as tombstones. If numbers must be
  reclaimed, that is a separate compaction pass with an explicit remap table.
