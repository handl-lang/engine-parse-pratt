//! `X := new T(args) { members }` — the mode/algebra declaration form
//! (graded-arrows.md §3 "Algebra declaration syntax", MTT migration).
//!
//! `new` is a parse-level alias (the `forall`→`Pi` precedent): the
//! grammar row mints a `new` node with children `[type_expr, dict]` —
//! `type_expr` is the algebra former application `T(args)` (`Algebra(Nat)`
//! parses as a `call`), `dict` the member record (`object`). The
//! checker-side expansion (mode family + instance registration) lives
//! in `synth_assign`'s `new` arm; the desugar pass lowers the node for
//! eval. These tests pin the surface parse only.

use handl_kit_data_prelude::language_from_yaml;
use handl_language::Ast;
use hnl_tool_lex_pattern::lex;
use hnl_tool_parse::{ParserEngine, ParserRequest};
use hnl_tool_parse_pratt::PrattEngine;

fn parse(src: &str) -> Result<Ast, handl_language::LangError> {
    let lang = language_from_yaml().expect("canonical language assembles");
    let tokens = lex(&lang, src).expect("lex");
    PrattEngine::new().parse(&lang, &ParserRequest::new(tokens))
}

fn kind(ast: &Ast, idx: usize) -> &str {
    ast.nodes()[idx].kind.as_str()
}

fn children(ast: &Ast, idx: usize) -> Vec<usize> {
    ast.nodes()[idx]
        .payload
        .get("children")
        .and_then(|c| c.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_u64().map(|n| n as usize))
                .collect()
        })
        .unwrap_or_default()
}

fn payload_str(ast: &Ast, idx: usize) -> Option<String> {
    ast.nodes()[idx].payload.as_str().map(str::to_string)
}

#[test]
fn new_decl_parses_assign_new_call_object() {
    let ast = parse(
        "Cost := new Algebra(Nat) {\n  compose: (a, b) ~> a + b,\n  unit: 0,\n  order: (a, b) ~> a <= b\n}\n",
    )
    .expect("new declaration parses");
    // A single top-level statement is the arena root itself.
    let root = ast.len() - 1;
    assert_eq!(kind(&ast, root), "assign");
    let pair = children(&ast, root);
    assert_eq!(pair.len(), 2, "assign(target, value)");
    assert_eq!(kind(&ast, pair[0]), "var");
    assert_eq!(payload_str(&ast, pair[0]).as_deref(), Some("Cost"));
    let new_node = pair[1];
    assert_eq!(kind(&ast, new_node), "new");
    let kids = children(&ast, new_node);
    assert_eq!(kids.len(), 2, "new[type_expr, dict]");
    // child 0 — `Algebra(Nat)` is a `call`.
    assert_eq!(kind(&ast, kids[0]), "call");
    let call_kids = children(&ast, kids[0]);
    assert_eq!(kind(&ast, call_kids[0]), "var");
    assert_eq!(payload_str(&ast, call_kids[0]).as_deref(), Some("Algebra"));
    assert_eq!(kind(&ast, call_kids[1]), "var");
    assert_eq!(payload_str(&ast, call_kids[1]).as_deref(), Some("Nat"));
    // child 1 — the member dictionary is an `object` of entries.
    assert_eq!(kind(&ast, kids[1]), "object");
    let entries = children(&ast, kids[1]);
    assert_eq!(entries.len(), 3, "compose/unit/order entries");
    assert!(entries.iter().all(|&e| kind(&ast, e) == "entry"));
}

#[test]
fn new_decl_bare_former_and_single_member() {
    // `new T { … }` — no argument list — still parses: child 0 is a `var`.
    let ast =
        parse("Session := new Proto {\n  open: () ~> ()\n}\n").expect("bare-former new parses");
    let root = ast.len() - 1;
    assert_eq!(kind(&ast, root), "assign");
    let new_node = children(&ast, root)[1];
    assert_eq!(kind(&ast, new_node), "new");
    let kids = children(&ast, new_node);
    assert_eq!(kind(&ast, kids[0]), "var");
    assert_eq!(payload_str(&ast, kids[0]).as_deref(), Some("Proto"));
    assert_eq!(kind(&ast, kids[1]), "object");
}

#[test]
fn new_is_a_keyword_not_an_identifier() {
    // `new` lexes as `token.keyword.new` — a bare `new` expression still
    // parses (the node is meaningful only as an assign RHS; the checker
    // warns), but `new` can no longer name a variable.
    let lang = language_from_yaml().expect("canonical language assembles");
    let toks = lex(&lang, "new").expect("lex");
    assert!(
        toks.0.iter().any(|t| t.kind == "token.keyword.new"),
        "`new` lexes as the keyword token"
    );
}

#[test]
fn instance_decl_still_parses_unchanged() {
    // `new` is additive sugar — the `instance` keyword path is untouched.
    let ast = parse("instance Eq(Nat) := { eq: (x, y) ~> x == y }\n")
        .expect("instance declaration still parses");
    let root = ast.len() - 1;
    assert_eq!(kind(&ast, root), "instance");
    let kids = children(&ast, root);
    assert_eq!(kids.len(), 2);
    assert_eq!(kind(&ast, kids[0]), "call");
    assert_eq!(kind(&ast, kids[1]), "object");
}
