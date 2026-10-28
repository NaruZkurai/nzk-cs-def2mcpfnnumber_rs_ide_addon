//! The owner's `NamespaceRwby` sketch, as an executable contract.
//!
//! It fixes the three properties that matter most:
//! 1. system types are static and come before every project token;
//! 2. project tokens start at 1000 and are sticky across a dump/load;
//! 3. each type/namespace owns a sub-dictionary that reuses the global numbers.

use def2mcpfnnumber::{
    Access, Def, DefKind, Dictionary, FIRST_PROJECT_TOKEN, Locals, Token, TokenIndex, TokenRef,
    def_line, local_decl, parse_def_line, parse_local_decl,
};

/// First project token; system types occupy everything below it.
const P: u32 = FIRST_PROJECT_TOKEN;

fn sketch() -> (TokenIndex, Vec<Def>, Vec<Token>) {
    let mut defs = vec![
        Def::new("NamespaceRwby", DefKind::Namespace).at("rwby.cs", 1, 120),
        Def::new("NamespaceRwby.Hero", DefKind::Method)
            .at("rwby.cs", 3, 9)
            .access(Access::Public)
            .modifiers(["public", "static"])
            .type_ref("System.String"),
    ];
    for i in 0..9 {
        defs.push(
            Def::new(format!("NamespaceRwby.Hero.HeroType{i}"), DefKind::Type).at(
                "rwby.cs",
                4 + i,
                4 + i,
            ),
        );
    }
    defs.push(
        Def::new("NamespaceRwby.Villan", DefKind::Method)
            .at("rwby.cs", 14, 22)
            .access(Access::Public)
            .modifiers(["public", "static"])
            .type_ref("NamespaceRwby.Hero"),
    );
    defs.push(
        Def::new("NamespaceRwby.party", DefKind::Type)
            .at("rwby.cs", 30, 30)
            .access(Access::Public)
            .type_ref("NamespaceRwby.Hero"),
    );

    let mut index = TokenIndex::new();
    let tokens = index.index_defs(&defs);
    assert_eq!(
        index.globals.resolve_types(),
        0,
        "every declared type in the sketch resolves"
    );
    (index, defs, tokens)
}

#[test]
fn system_types_are_static_and_precede_every_project_token() {
    // Statically defined, present before any indexing happens.
    assert_eq!(def2mcpfnnumber::defaults::token("System.Object"), Some(100));
    assert_eq!(def2mcpfnnumber::defaults::token("System.String"), Some(101));
    assert_eq!(
        def2mcpfnnumber::defaults::token("UnityEngine.Vector3"),
        Some(139)
    );

    // System types all sit below the first project token.
    for entry in def2mcpfnnumber::defaults::all() {
        if entry.kind == def2mcpfnnumber::DefaultKind::Type {
            assert!(entry.token < def2mcpfnnumber::FIRST_PROJECT_TOKEN);
        }
    }

    let (index, _, tokens) = sketch();
    assert_eq!(tokens[0], Token(P));
    let hero = index.globals.token_of("NamespaceRwby.Hero").unwrap();
    assert!(hero.0 >= P);
}

#[test]
fn project_tokens_are_sequential_and_hierarchical() {
    let (index, defs, tokens) = sketch();
    assert_eq!(tokens[0], Token(P), "namespace first");
    assert_eq!(tokens[1], Token(P + 1), "Hero");

    // Nine nested type definitions, tokens 202..=210, owned by Hero.
    let hero_types: Vec<u32> = (P + 2..=P + 10).collect();
    assert_eq!(
        &tokens[2..11].iter().map(|t| t.0).collect::<Vec<_>>(),
        &hero_types
    );
    assert_eq!(tokens[11], Token(P + 11), "Villan");
    assert_eq!(tokens[12], Token(P + 12), "party");

    // The namespace sub-dictionary holds its direct members, not grandchildren.
    assert_eq!(
        index.globals.members_of("NamespaceRwby"),
        &[P + 1, P + 11, P + 12],
        "Hero, Villan, party"
    );
    // The type sub-dictionary holds its own nested types.
    assert_eq!(
        index.globals.members_of("NamespaceRwby.Hero"),
        &hero_types[..]
    );
    // …and each member is the same global token, not a second numbering space.
    assert_eq!(
        index.globals.get(Token(P + 2)).unwrap().full,
        "NamespaceRwby.Hero.HeroType0"
    );
    assert_eq!(defs[2].full, "NamespaceRwby.Hero.HeroType0");
}

#[test]
fn tokens_are_sticky_across_a_rebuild() {
    let (index, _, _) = sketch();
    let dump = index.globals.dump();
    let mut rebuilt = Dictionary::load(&dump).expect("load");

    // Re-indexing the same definitions must not consume new numbers.
    let again = rebuilt.assign(Def::new("NamespaceRwby.Hero", DefKind::Method));
    assert_eq!(again, Token(P + 1));

    // A genuinely new definition gets the next free number.
    let new = rebuilt.assign(Def::new("NamespaceRwby.Boss", DefKind::Method));
    assert_eq!(new, Token(P + 13));
    assert_eq!(rebuilt.next_token(), Token(P + 14));
}

#[test]
fn this_module_resolves_short_names_without_stealing_others() {
    let (index, _, _) = sketch();
    assert_eq!(
        index.globals.resolve_in_module("NamespaceRwby", "Hero"),
        Some(Token(P + 1))
    );
    assert_eq!(
        index
            .globals
            .resolve_in_module("NamespaceRwby.Hero", "HeroType0"),
        Some(Token(P + 2))
    );
    // Outside the owning module the short name is NOT resolved: module scope is
    // real, and the fully qualified name is still available globally.
    assert_eq!(index.globals.resolve_in_module("Elsewhere", "Hero"), None);
    assert_eq!(index.globals.resolve_in_module("Elsewhere", "Nope"), None);
    assert_eq!(
        index.globals.token_of("NamespaceRwby.Hero"),
        Some(Token(P + 1))
    );
    // System types win over a same-named project entry.
    assert_eq!(index.globals.resolve("System.String"), Some(Token(101)));
}

#[test]
fn comments_and_locals_use_their_own_spaces_and_condense() {
    let mut index = TokenIndex::new();
    index.index_defs(&[
        Def::new("NamespaceRwby", DefKind::Namespace),
        Def::new("NamespaceRwby.Hero", DefKind::Method)
            .access(Access::Public)
            .modifiers(["public", "static"]),
    ]);

    let c1 = index.comment_ref("/* a comment */");
    let c2 = index.comment_ref("/* a comment */");
    assert_eq!(c1, TokenRef::comment(1));
    assert_eq!(c1, c2, "identical comments share a token");

    let mut locals = Locals::new();
    let value = locals.intern("value");
    assert_eq!(value, 1);
    assert_eq!(local_decl(value, "value"), "lt(1,\"value\")");
    assert_eq!(
        parse_local_decl("lt(1,\"value\")"),
        Some((1, "value".into()))
    );

    let line = def_line(
        Token(P + 1),
        &[
            TokenRef::global(1),
            TokenRef::global(3),
            TokenRef::global(101),
            TokenRef::local(value),
            c1,
        ],
    );
    assert_eq!(
        line,
        format!("g({}) = [g(1), g(3), g(101), lt(1), c(1)]", P + 1)
    );
    let (token, refs) = parse_def_line(&line).expect("parse");
    assert_eq!(token, Token(P + 1));
    assert_eq!(refs.len(), 5);
    assert_eq!(refs[3], TokenRef::local(1));
    assert_eq!(refs[4], TokenRef::comment(1));
}

#[test]
fn the_dump_is_deterministic_and_self_describing() {
    let (a, _, _) = sketch();
    let (b, _, _) = sketch();
    assert_eq!(a.globals.dump(), b.globals.dump());
    assert!(a.globals.dump().starts_with("def2mcpfnnumber-globals 1\n"));
    assert!(a.globals.dump().contains("defaults 2\n"));
}

#[test]
fn a_project_cannot_shadow_a_keyword() {
    let mut d = Dictionary::new();
    d.assign(Def::new("public", DefKind::Method));
    // The project name exists, but `resolve` still answers the static token.
    assert_eq!(d.resolve("public"), Some(Token(1)));
    assert_eq!(d.token_of("public"), Some(Token(P)));
}

#[test]
fn every_definition_records_its_declared_type_token() {
    let (index, _, _) = sketch();

    // A system type is already a static token, before any project type exists.
    let hero = index.globals.get(Token(P + 1)).unwrap();
    assert_eq!(hero.type_name, "System.String");
    assert_eq!(hero.type_token, Some(Token(101)));

    // A project type resolves to the same global number as its declaration.
    let villan = index.globals.get(Token(P + 11)).unwrap();
    assert_eq!(villan.type_name, "NamespaceRwby.Hero");
    assert_eq!(villan.type_token, Some(Token(P + 1)));

    // An unknown/foreign type stays explicitly unresolved rather than inventing
    // a token.
    let mut d = Dictionary::new();
    d.assign(Def::new("A.M", DefKind::Method).type_ref("Some.External.Type"));
    assert_eq!(d.resolve_types(), 1);
    assert_eq!(d.get(Token(P)).unwrap().type_token, None);
}
