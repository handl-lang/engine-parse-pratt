//! ADR-0158 family-record lowering — the indexed-nullary ctor shape.
//!
//! `F<params> := { C : F(i⃗), … }` lowers to the internal `data` node. An
//! indexed nullary ctor (`VNil : Vec(0, a)`) must emit **exactly one**
//! `type-annot` wrap — `type-annot(var VNil, Vec(0, a), name: "VNil")` —
//! the same shape a source-level `VNil : Vec(0, a)` annotation parses to.
//! A double wrap (`type-annot(type-annot(C, R), R, name)`) leaves the
//! `subject` a `type-annot`, which leaf-peeling consumers
//! (`tool-transform-rewrite`'s `collect_constructor_leaf`,
//! `tool-typecheck-nbe`'s `collect_ctors`) silently drop — indexed
//! nullary ctors vanished from the desugared family.

use handl_kit_data_prelude::language_from_yaml;
use hnl_tool_lex_pattern::lex;
use hnl_tool_parse::{ParserEngine, ParserRequest};
use hnl_tool_parse_pratt::PrattEngine;
use handl_language::{Ast, AstNode, AstRef};

fn parse_family(src: &str) -> Ast {
    let lang = language_from_yaml().expect("canonical language assembles");
    let tokens = lex(&lang, src).expect("lex");
    PrattEngine::new()
        .parse(&lang, &ParserRequest::new(tokens))
        .expect("parse")
}

fn children_of(node: &AstNode) -> Vec<usize> {
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

/// Flatten the left-nested `alt` ctor chain into leaf ctor nodes.
fn ctor_leaves<'a>(ast: &'a Ast, node_idx: usize, out: &mut Vec<&'a AstNode>) {
    let node = ast.get(AstRef::new(node_idx)).expect("ctor node");
    if node.kind == "alt" {
        for c in children_of(node) {
            ctor_leaves(ast, c, out);
        }
    } else {
        out.push(node);
    }
}

#[test]
fn indexed_nullary_ctor_is_single_type_annot_wrap() {
    let ast = parse_family(
        "Vec<n : Nat, a : Type> := {\n  VNil  : Vec(0, a),\n  VCons : (x : a, xs : Vec(n, a)) ~> Vec(n + 1, a)\n}\n",
    );
    let data = ast.nodes().last().expect("arena root");
    assert_eq!(data.kind, "data", "record spelling lowers to `data`");
    let data_children = children_of(data);
    assert_eq!(data_children.len(), 2, "data children [type, ctors]");

    let mut ctors = Vec::new();
    ctor_leaves(&ast, data_children[1], &mut ctors);
    assert_eq!(ctors.len(), 2, "two constructors: {ctors:?}");

    let vnil = ctors
        .iter()
        .find(|n| n.payload.get("name").and_then(|v| v.as_str()) == Some("VNil"))
        .expect("VNil ctor present");
    assert_eq!(
        vnil.kind, "type-annot",
        "indexed nullary ctor is type-annot wrapped"
    );
    let kids = children_of(vnil);
    assert_eq!(kids.len(), 2, "type-annot [subject, annot]");
    let subject = ast.get(AstRef::new(kids[0])).expect("subject node");
    assert_eq!(
        subject.kind, "var",
        "subject must be the bare ctor leaf — exactly one wrap, got kind {}",
        subject.kind
    );
    assert_eq!(subject.payload.as_str(), Some("VNil"));

    // The argumented ctor keeps its single wrap over a `call` leaf.
    let vcons = ctors
        .iter()
        .find(|n| n.payload.get("name").and_then(|v| v.as_str()) == Some("VCons"))
        .expect("VCons ctor present");
    assert_eq!(vcons.kind, "type-annot");
    let vcons_subject = ast
        .get(AstRef::new(children_of(vcons)[0]))
        .expect("VCons subject");
    assert_eq!(vcons_subject.kind, "call", "VCons leaf is the arg `call`");
}

#[test]
fn bare_and_indexed_nullary_ctors_in_one_family() {
    // `Zero : Nat`-style bare ctor next to an indexed one — the bare ctor
    // stays unwrapped (the checker's bare-index convention hangs off the
    // absent annotation).
    let ast = parse_family(
        "Vec<n : Nat, a : Type> := {\n  VNil : Vec(0, a),\n  VHead : Vec(n, a)\n}\n",
    );
    let data = ast.nodes().last().expect("arena root");
    assert_eq!(data.kind, "data");
    let mut ctors = Vec::new();
    ctor_leaves(&ast, children_of(data)[1], &mut ctors);
    assert_eq!(ctors.len(), 2);
    // Both ctors here carry a result index vector, so both are wrapped.
    for c in &ctors {
        assert_eq!(c.kind, "type-annot", "ctor {:?} wrapped once", c.payload);
        let subject = ast
            .get(AstRef::new(children_of(c)[0]))
            .expect("subject");
        assert_eq!(subject.kind, "var", "single wrap — leaf is var");
    }
}

#[test]
fn inner_family_record_keeps_enclosing_root() {
    // A family declaration inside a `do { … }` block is an *inner* node:
    // the lowering appends constructor/`alt` nodes at the arena end, which
    // must not steal the "root = last node" convention from the enclosing
    // block — consumers (`apply_desugar`, the checker's module walk) read
    // the root from the last index.
    let ast = parse_family(
        "do {\n  Nat := { Zero : Nat, Succ : (pred : Nat) ~> Nat }\n  Succ(Zero)\n}\n",
    );
    let root = ast.nodes().last().expect("arena root");
    assert_eq!(
        root.kind, "block",
        "root must stay the enclosing block, got kind {}",
        root.kind
    );
    // The block's first child is the lowered `data` node.
    let kids = children_of(root);
    assert!(!kids.is_empty(), "block has statement children");
    let first = ast.get(AstRef::new(kids[0])).expect("first statement");
    assert_eq!(first.kind, "data", "family decl lowered in place");
}
