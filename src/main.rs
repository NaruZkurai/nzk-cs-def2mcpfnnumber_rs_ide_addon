//! Demo for the token dictionary.
//!
//! Run with `cargo run`. It builds the owner's `NamespaceRwby` sketch, prints
//! the static/system table first, then the project table and the condensed
//! spelling, and proves the dump round-trips.

use def2mcpfnnumber::{
    Access, Def, DefKind, TokenIndex, TokenRef, def_line, local_decl, parse_def_line,
};

fn main() {
    println!("def2mcpfnnumber {}", def2mcpfnnumber::VERSION);

    // ------------------------------------------------------------------
    // 1. The static table: system types first, before anything indexed.
    // ------------------------------------------------------------------
    println!("\nstatic system types (first, fixed):");
    for entry in def2mcpfnnumber::defaults::all()
        .iter()
        .filter(|e| e.kind == def2mcpfnnumber::DefaultKind::Type)
        .take(8)
    {
        println!(
            "  g({:3}) {:<40} {}",
            entry.token,
            entry.name,
            entry.kind.as_str()
        );
    }
    println!(
        "  … {} system types total",
        def2mcpfnnumber::defaults::all()
            .iter()
            .filter(|e| e.kind == def2mcpfnnumber::DefaultKind::Type)
            .count()
    );

    println!("\nstatic words used by the sketch:");
    for name in ["public", "private", "static", "const", "void", "return"] {
        let t = def2mcpfnnumber::defaults::token(name).unwrap();
        println!("  g({t}) = {name}");
    }

    // ------------------------------------------------------------------
    // 2. The project: namespace, one method with nine nested types, another
    //    method. Tokens start after the pre-assigned system types, never at 0/1.
    // ------------------------------------------------------------------
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
            .modifiers(["public", "static"])
            .type_ref("NamespaceRwby.Hero"),
    );

    let mut index = TokenIndex::new();
    let tokens = index.index_defs(&defs);
    let unresolved_types = index.globals.resolve_types();

    println!("\nproject tokens (declared type resolved):");
    for (def, token) in defs.iter().zip(tokens.iter()) {
        let ty = index
            .globals
            .get(*token)
            .and_then(|e| e.type_token)
            .map(|t| format!("g({})", t.0))
            .unwrap_or_else(|| "-".to_string());
        println!(
            "  g({:3}) {:<10} {:<32} {}:{}  type={ty}",
            token.0,
            def.kind.as_str(),
            def.full,
            def.file,
            def.line
        );
    }
    println!("  unresolved declared types: {unresolved_types}");

    println!("\nsub-dictionary NamespaceRwby:");
    for e in index.globals.members("NamespaceRwby") {
        println!("  g({:3}) {}", e.token.0, e.name);
    }
    println!("\nsub-dictionary NamespaceRwby.Hero:");
    for e in index.globals.members("NamespaceRwby.Hero") {
        println!("  g({:3}) {}", e.token.0, e.name);
    }

    println!(
        "\nthismodule resolve: NamespaceRwby.Hero -> {:?}",
        index.globals.resolve_in_module("NamespaceRwby", "Hero")
    );

    // ------------------------------------------------------------------
    // 3. Condensed definitions: what the Scratch/IDE link would store.
    // ------------------------------------------------------------------
    let mut locals = index.function_locals();
    let value = locals.intern("value");
    let comment = index.comment_ref("/* Hero is public static. */");

    let hero = index
        .globals
        .token_of("NamespaceRwby.Hero")
        .expect("Hero was indexed");
    let hero_line = def_line(
        hero,
        &[
            TokenRef::global(def2mcpfnnumber::defaults::PUBLIC),
            TokenRef::global(def2mcpfnnumber::defaults::STATIC),
            TokenRef::global(def2mcpfnnumber::defaults::VOID),
            comment,
            TokenRef::local(value),
        ],
    );

    println!("\ncondensed:");
    println!("  {hero_line}");
    println!("  {}", local_decl(value, "value"));
    println!("  {comment} = {:?}", index.comments.text_of(1));
    let (back_token, back_refs) = parse_def_line(&hero_line).expect("round-trip");
    println!(
        "  parsed back: g({}) with {} refs",
        back_token.0,
        back_refs.len()
    );

    // ------------------------------------------------------------------
    // 4. Persistence: the token map survives a rebuild.
    // ------------------------------------------------------------------
    let dump = index.globals.dump();
    let reloaded = def2mcpfnnumber::Dictionary::load(&dump).expect("load dump");
    println!(
        "\ndump: {} bytes, {} entries, reload identical = {}",
        dump.len(),
        index.globals.len(),
        reloaded.dump() == dump
    );
    println!("next token after reload: g({})", reloaded.next_token().0);
}
