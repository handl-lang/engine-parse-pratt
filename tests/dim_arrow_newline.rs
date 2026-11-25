//! Dimension-arrow newline continuation: both spellings of `A ~[n] B` —
//! the legacy `A ~[n]~> B` and the bracketed `A ~>[n] B` — must accept a
//! right operand that starts on the next line. The Pratt cursor skips
//! layout tokens transparently, so this holds for every `child_separators`
//! form; these tests pin it against the canonical kit language.
//!
//! (The reported "unexpected end of input" failure was *not* here — it
//! came from `plugin-ext-diag::split_statements`, which cut the chunk
//! after `~>[n]`'s `]` before the operand line. The parser-level
//! guarantee is still worth pinning.)

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

#[test]
fn dim_arrow_bare_accepts_newline_right_operand() {
    let ast = parse("x := f ~>[2]\n  g").expect("bare spelling parses");
    assert_eq!(root_kind(&ast), "assign");
}

#[test]
fn dim_arrow_legacy_accepts_newline_right_operand() {
    let ast = parse("x := f ~[2]~>\n  g").expect("legacy spelling parses");
    assert_eq!(root_kind(&ast), "assign");
}

#[test]
fn dim_arrow_bare_and_legacy_mint_same_node() {
    // Both spellings produce the same `dim_arrow` node with the dim
    // payload and the right operand as children.
    for src in ["x := f ~>[2]\n  g", "x := f ~[2]~>\n  g"] {
        let ast = parse(src).unwrap_or_else(|e| panic!("{src}: {e}"));
        let kinds: Vec<&str> = ast.nodes().iter().map(|n| n.kind.as_str()).collect();
        assert!(kinds.contains(&"dim_arrow"), "{src}: {kinds:?}");
    }
}

#[test]
fn dim_arrow_bare_multiline_chain() {
    // The lib-cli `argv_head` shape: the operator opens a continuation
    // line and the right operand sits on the following line.
    parse(
        "export argv_head := (argv: Argv) ~>\n  (p, s, pos, fs, o, r) ~>\n    argv(p, s, pos, fs, o, r).value[0]\n  ~>[2]\n  (p, s, pos, fs, o, r) ~> p",
    )
    .expect("chained site parses");
    // `>>>[k]` axis flow shares the `]` separator — same guarantee.
    parse("x := a >>>[k]\n  b").expect("axis_flow parses");
}
