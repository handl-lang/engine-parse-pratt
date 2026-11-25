//! Keyword call-form (apply-forms unification, Route A —
//! `project/design/apply-forms-unification.md` in the host repo).
//!
//! Multi-argument keyword/comma forms (`trans P, Q`, `J p, motive, base`,
//! `glue A, φ, T̂`, `transp`, `hcomp`, `Comp`) also parse in call form
//! `f(a, b, c)`, producing the *same* node kinds — zero downstream changes.
//! These tests pin that against the canonical kit language, and that the
//! production does not disturb the surrounding syntax (ordinary calls,
//! paren/type forms, `forall`/`exists`, adjacency rules).

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

fn root_kind(ast: &Ast) -> &str {
    ast.nodes().last().expect("non-empty arena").kind.as_str()
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

fn child_kind(ast: &Ast, parent: usize, slot: usize) -> &str {
    let kids = children(ast, parent);
    ast.nodes()[kids[slot]].kind.as_str()
}

/// The six multi-argument comma-separated prefix rows, in both spellings.
#[test]
fn keyword_call_form_covers_all_comma_rows() {
    let cases: &[(&str, &str, &str, usize)] = &[
        // (keyword spelling, call spelling, expected node kind, arity)
        ("trans P, Q", "trans(P, Q)", "trans", 2),
        ("J p, motive, base", "J(p, motive, base)", "j", 3),
        ("glue A, phi, That", "glue(A, phi, That)", "glue", 3),
        ("transp A, i, phi, u", "transp(A, i, phi, u)", "transp", 4),
        (
            "hcomp A, i, phi, u, cap",
            "hcomp(A, i, phi, u, cap)",
            "hcomp",
            5,
        ),
        (
            "Comp F, i, phi, u, cap",
            "Comp(F, i, phi, u, cap)",
            "comp",
            5,
        ),
    ];
    for (keyword_src, call_src, kind, arity) in cases {
        let kw = parse(keyword_src).unwrap_or_else(|e| panic!("{keyword_src}: {e}"));
        let call = parse(call_src).unwrap_or_else(|e| panic!("{call_src}: {e}"));
        let kw_root = kw.nodes().len() - 1;
        let call_root = call.nodes().len() - 1;
        assert_eq!(root_kind(&kw), *kind, "{keyword_src}");
        assert_eq!(root_kind(&call), *kind, "{call_src}");
        assert_eq!(children(&kw, kw_root).len(), *arity, "{keyword_src} arity");
        assert_eq!(children(&call, call_root).len(), *arity, "{call_src} arity");
        // Same node kind + same child count + same child node kinds —
        // the call form is a pure alias, not a desugar.
        for (k, c) in children(&kw, kw_root)
            .iter()
            .zip(children(&call, call_root))
        {
            assert_eq!(
                kw.nodes()[*k].kind,
                call.nodes()[c].kind,
                "{keyword_src} vs {call_src} child kinds"
            );
        }
    }
}

/// Keyword forms keep parsing — the call form is an *added* spelling.
#[test]
fn keyword_forms_still_parse() {
    for src in [
        "trans P, Q",
        "J p, motive, base",
        "glue A, phi, That",
        "transp A, i, phi, u",
        "hcomp A, i, phi, u, cap",
        "Comp F, i, phi, u, cap",
    ] {
        assert!(parse(src).is_ok(), "{src} must still parse");
    }
}

/// Arity is enforced in call form, matching the keyword form.
#[test]
fn call_form_arity_mismatch_is_parse_error() {
    assert!(parse("trans(P)").is_err(), "trans(P) — one arg, needs two");
    assert!(
        parse("J(p, motive)").is_err(),
        "J(p, motive) — two args, needs three"
    );
    assert!(parse("trans()").is_err(), "trans() — empty call form");
}

/// The `(` must be byte-adjacent: `trans (P), Q` keeps the keyword
/// reading — `(P)` is the parenthesized first child, `, Q` the rest.
#[test]
fn spaced_paren_keeps_keyword_reading() {
    let ast = parse("trans (P), Q").expect("keyword form with paren child");
    let root = ast.nodes().len() - 1;
    assert_eq!(root_kind(&ast), "trans");
    assert_eq!(child_kind(&ast, root, 0), "paren");
    assert_eq!(child_kind(&ast, root, 1), "var");
}

/// Ordinary calls, unary keyword forms, and parenthesized/type forms are
/// untouched.
#[test]
fn surrounding_syntax_unaffected() {
    // Ordinary identifier call.
    let ast = parse("f(a, b)").unwrap();
    assert_eq!(root_kind(&ast), "call");

    // Unary keyword forms already accepted `f(x)` via transparent paren.
    let ast = parse("symm(p)").unwrap();
    assert_eq!(root_kind(&ast), "symm");

    // Parenthesized comma list is still a `paren` node, not a keyword form.
    let ast = parse("(x : Nat, y : Nat)").unwrap();
    assert_eq!(root_kind(&ast), "paren");
    assert_eq!(children(&ast, ast.nodes().len() - 1).len(), 2);

    // `forall`/`exists` declare per-child binding powers — they are NOT
    // comma-application shapes and `forall(...)` stays a parse error.
    assert!(
        parse("forall(x : Nat, P x)").is_err(),
        "forall keeps its keyword-only spelling"
    );
}

/// A keyword call-form node is a normal operand: infix operators bind
/// around it exactly as they do around the keyword form.
#[test]
fn call_form_node_binds_infix_operators() {
    let ast = parse("trans(P, Q) # v").expect("transport over call-form trans");
    let root = ast.nodes().len() - 1;
    assert_eq!(root_kind(&ast), "transport");
    assert_eq!(child_kind(&ast, root, 0), "trans");
}

/// Nesting works: call forms inside call forms.
#[test]
fn call_forms_nest() {
    let ast = parse("trans(J(p, motive, base), Q)").expect("nested call forms");
    let root = ast.nodes().len() - 1;
    assert_eq!(root_kind(&ast), "trans");
    assert_eq!(child_kind(&ast, root, 0), "j");
}
