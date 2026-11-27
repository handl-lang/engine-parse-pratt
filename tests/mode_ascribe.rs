//! `T @ μ` — ambient-mode ascription (PL-0004 E1;
//! design/effects-ambient-modes.md §1). `@` is an infix operator on a
//! type/subject expression minting a `mode_ascribe` node whose right
//! operand is the mode expression: the bracketed set
//! `@ [Algebra(payload), …]`, a bare `Algebra(payload)` call, a mode
//! variable (`@ m`), or `[]` (pure).
//!
//! Runs against the canonical kit language (real lexer + grammar data),
//! so `token.at`, `grammar.mode_ascribe.infix`, and `ast.mode_ascribe`
//! are exercised end to end.

use handl_kit_data_prelude::language_from_yaml;
use handl_language::{Ast, Language};
use hnl_tool_lex_pattern::lex;
use hnl_tool_parse::{ParserEngine, ParserRequest};
use hnl_tool_parse_pratt::PrattEngine;

fn lang() -> Language {
    language_from_yaml().expect("canonical language assembles")
}

fn parse(src: &str) -> Ast {
    let lang = lang();
    let tokens = lex(&lang, src).expect("lex");
    PrattEngine::new()
        .parse(&lang, &ParserRequest::new(tokens))
        .expect("parse")
}

fn kinds(ast: &Ast) -> Vec<&str> {
    ast.nodes().iter().map(|n| n.kind.as_str()).collect()
}

/// Children of the (single) `mode_ascribe` node.
fn ascribe_children(ast: &Ast) -> (usize, usize) {
    let node = ast
        .nodes()
        .iter()
        .find(|n| n.kind == "mode_ascribe")
        .expect("a mode_ascribe node");
    let kids = node
        .payload
        .get("children")
        .and_then(|c| c.as_array())
        .expect("children");
    assert_eq!(kids.len(), 2, "mode_ascribe arity");
    (
        kids[0].as_u64().expect("type child idx") as usize,
        kids[1].as_u64().expect("mode child idx") as usize,
    )
}

#[test]
fn bracketed_set_mints_mode_ascribe() {
    let ast = parse("printLn : (Str ~> Str) @ [Effects({ puts: Str ~> Unit })]");
    let kinds = kinds(&ast);
    assert!(
        kinds.contains(&"mode_ascribe"),
        "expected mode_ascribe, got {kinds:?}"
    );
    assert!(kinds.contains(&"type-annot"), "got {kinds:?}");
    let (ty, mode) = ascribe_children(&ast);
    assert_eq!(kind(&ast, ty), "paren");
    assert_eq!(kind(&ast, mode), "array");
}

#[test]
fn empty_set_is_pure() {
    let ast = parse("handled : (Str ~> Str) @ [] := f");
    let kinds = kinds(&ast);
    assert!(kinds.contains(&"mode_ascribe"), "got {kinds:?}");
    let (_, mode) = ascribe_children(&ast);
    assert_eq!(kind(&ast, mode), "array");
    // `[]` has no element children.
    let elems = ast.nodes()[mode]
        .payload
        .get("children")
        .and_then(|c| c.as_array())
        .map(|a| a.len())
        .unwrap_or(0);
    assert_eq!(elems, 0, "empty mode set");
}

#[test]
fn bare_call_mode_operand() {
    // `@ Effects(S)` without brackets — a single algebra application.
    let ast = parse("f : Nat @ Cost(3) := x");
    let (_, mode) = ascribe_children(&ast);
    assert_eq!(kind(&ast, mode), "call");
}

#[test]
fn mode_variable_operand() {
    // `@ m` — a mode variable (E3 mode polymorphism spelling).
    let ast = parse("f : Nat @ m := x");
    let (_, mode) = ascribe_children(&ast);
    assert_eq!(kind(&ast, mode), "var");
}

#[test]
fn ascription_binds_inside_type_annotation() {
    // `x : T @ [μ] := e` — the `@` operand is the annotation's type
    // child, not the whole `type-annot`: the assign's value stays
    // outside the mode_ascribe subtree.
    let ast = parse("greet : Str @ [Effects(puts)] := perform(puts, \"hi\")");
    let kinds = kinds(&ast);
    assert!(kinds.contains(&"mode_ascribe"), "got {kinds:?}");
    assert!(kinds.contains(&"assign"), "got {kinds:?}");
    let (ty, _) = ascribe_children(&ast);
    assert_eq!(kind(&ast, ty), "var");
}

#[test]
fn at_does_not_steal_wiskers() {
    // `<@`/`@>` keep their own tokens — `@` must not shadow them.
    let lang = lang();
    for (src, want) in [("a <@ b", "token.at_left"), ("a @> b", "token.at_right")] {
        let tokens = lex(&lang, src).expect("lex");
        let kinds: Vec<&str> = tokens.0.iter().map(|t| t.kind.as_str()).collect();
        assert!(
            kinds.contains(&want),
            "{src}: expected {want} in {kinds:?}"
        );
    }
}

fn kind(ast: &Ast, idx: usize) -> &str {
    ast.nodes()[idx].kind.as_str()
}
