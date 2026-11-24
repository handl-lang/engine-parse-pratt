//! Regression: `let` is not a keyword — it lexes as `token.identifier`
//! and parses as an ordinary name (expression, binding target, function
//! name). There is no `token.keyword.let`, no `lex.keyword.let` rule, and
//! no `let`/`stmt.let` node kind in the canonical language.

use handl_kit_data_prelude::language_from_yaml;
use handl_language::{Ast, AstRef};
use hnl_tool_lex_pattern::lex;
use hnl_tool_parse::{ParserEngine, ParserRequest};
use hnl_tool_parse_pratt::PrattEngine;

fn parse(src: &str) -> Ast {
    let lang = language_from_yaml().expect("canonical language assembles");
    let tokens = lex(&lang, src).expect("lex");
    PrattEngine::new()
        .parse(&lang, &ParserRequest::new(tokens))
        .expect("parse")
}

fn children_of(node: &handl_language::AstNode) -> Vec<usize> {
    node.payload
        .get("children")
        .and_then(|c| c.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_u64().map(|n| n as usize))
                .collect()
        })
        .unwrap_or_default()
}

#[test]
fn let_parses_as_plain_var() {
    let ast = parse("let");
    let root = ast.nodes().last().expect("arena root");
    assert_eq!(root.kind, "var");
    assert_eq!(root.payload.as_str(), Some("let"));
}

#[test]
fn let_is_a_valid_binding_target() {
    // `let = 1` — `let` is the assign target, not a keyword.
    let ast = parse("let = 1");
    let root = ast.nodes().last().expect("arena root");
    assert_eq!(root.kind, "assign");
    let kids = children_of(root);
    assert_eq!(kids.len(), 2, "assign children [target, value]");
    let target = ast.get(AstRef::new(kids[0])).expect("target node");
    assert_eq!(target.kind, "var");
    assert_eq!(target.payload.as_str(), Some("let"));
}

#[test]
fn let_prefix_does_not_break_longer_identifiers() {
    // `letter`/`let_x` were always single identifiers; `let` joining them
    // must not change that.
    for src in ["letter", "let_x", "lets"] {
        let ast = parse(src);
        let root = ast.nodes().last().expect("arena root");
        assert_eq!(root.kind, "var", "{src} parses as a var");
        assert_eq!(root.payload.as_str(), Some(src));
    }
}
