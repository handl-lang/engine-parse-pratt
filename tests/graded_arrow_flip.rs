//! Payload-kind flip for the `~[…]~>` spelling (graded-arrows.md §9,
//! post-codemod): the Pratt engine mints `graded_arrow` for every
//! payload EXCEPT a bare numeric literal, which keeps the legacy
//! `dim_arrow` reading. The `~>[x]` spelling always mints `dim_arrow`.
//!
//! Runs against the canonical kit language (real lexer + grammar data),
//! so the `token.dim_open`/`token.dim_close`/`token.dim_arrow_open`
//! spellings are exercised end to end.

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

/// The node kinds present in the arena.
fn kinds(ast: &Ast) -> Vec<&str> {
    ast.nodes().iter().map(|n| n.kind.as_str()).collect()
}

#[test]
fn call_payload_mints_graded_arrow() {
    // `~[Cost(3)]~>` — a call payload is a grade, not a dimension.
    let ast = parse("f ~[Cost(3)]~> g");
    let kinds = kinds(&ast);
    assert!(
        kinds.contains(&"graded_arrow"),
        "expected graded_arrow, got {kinds:?}"
    );
    assert!(
        !kinds.contains(&"dim_arrow"),
        "call payload must not mint dim_arrow, got {kinds:?}"
    );
}

#[test]
fn record_payload_mints_graded_arrow() {
    let ast = parse("f ~[{puts: Str}]~> g");
    let kinds = kinds(&ast);
    assert!(
        kinds.contains(&"graded_arrow"),
        "expected graded_arrow, got {kinds:?}"
    );
}

#[test]
fn named_payload_mints_graded_arrow() {
    let ast = parse("dedupe<N> := (g >>> g) ~[N]~> g");
    let kinds = kinds(&ast);
    assert!(
        kinds.contains(&"graded_arrow"),
        "expected graded_arrow, got {kinds:?}"
    );
}

#[test]
fn numeric_payload_stays_dim_arrow() {
    for src in ["f ~[2]~> g", "f ~[0]~> g"] {
        let ast = parse(src);
        let kinds = kinds(&ast);
        assert!(
            kinds.contains(&"dim_arrow"),
            "{src}: expected dim_arrow, got {kinds:?}"
        );
        assert!(
            !kinds.contains(&"graded_arrow"),
            "{src}: numeric payload must not mint graded_arrow, got {kinds:?}"
        );
    }
}

#[test]
fn bare_spelling_stays_dim_arrow_for_any_payload() {
    for src in ["f ~>[2] g", "f ~>[N] g"] {
        let ast = parse(src);
        let kinds = kinds(&ast);
        assert!(
            kinds.contains(&"dim_arrow"),
            "{src}: expected dim_arrow, got {kinds:?}"
        );
        assert!(
            !kinds.contains(&"graded_arrow"),
            "{src}: ~>[…] must never mint graded_arrow, got {kinds:?}"
        );
    }
}
