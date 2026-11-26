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
use handl_language::{Ast, AstNode, AstRef};
use hnl_tool_lex_pattern::lex;
use hnl_tool_parse::{ParserEngine, ParserRequest};
use hnl_tool_parse_pratt::PrattEngine;

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
    let ast =
        parse_family("Vec<n : Nat, a : Type> := {\n  VNil : Vec(0, a),\n  VHead : Vec(n, a)\n}\n");
    let data = ast.nodes().last().expect("arena root");
    assert_eq!(data.kind, "data");
    let mut ctors = Vec::new();
    ctor_leaves(&ast, children_of(data)[1], &mut ctors);
    assert_eq!(ctors.len(), 2);
    // Both ctors here carry a result index vector, so both are wrapped.
    for c in &ctors {
        assert_eq!(c.kind, "type-annot", "ctor {:?} wrapped once", c.payload);
        let subject = ast.get(AstRef::new(children_of(c)[0])).expect("subject");
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

// ---------------------------------------------------------------------------
// PL-0003 I2 — `X := new Inductive { … }` lowers to the same `data` node
// (the `Inductive` schema of the unified `new` registry).
// ---------------------------------------------------------------------------

/// The lowered `data` node for `src` — panics unless the declaration
/// lowered (single-statement chunk: the `data` node is the arena root).
fn lowered_data(ast: &Ast) -> &AstNode {
    let data = ast.nodes().last().expect("arena root");
    assert_eq!(data.kind, "data", "new Inductive lowers to `data`");
    data
}

#[test]
fn new_inductive_lowers_to_data_nullary_and_arged_ctors() {
    // `Nat := new Inductive { zero : Nat, succ : (pred : Nat) ~> Nat }` —
    // the graded-arrows.md §3 surface. Must produce the identical `data`
    // node the record spelling `Nat := { … }` produces.
    let ast = parse_family(
        "Nat := new Inductive {\n  zero : Nat,\n  succ : (pred : Nat) ~> Nat\n}\n",
    );
    let data = lowered_data(&ast);
    assert_eq!(data.payload.get("name").and_then(|v| v.as_str()), Some("Nat"));
    let data_children = children_of(data);
    assert_eq!(data_children.len(), 2, "data children [type, ctors]");
    // The family head is the assign target `var Nat`.
    let head = ast.get(AstRef::new(data_children[0])).expect("head");
    assert_eq!(head.kind, "var");
    assert_eq!(head.payload.as_str(), Some("Nat"));

    let mut ctors = Vec::new();
    ctor_leaves(&ast, data_children[1], &mut ctors);
    assert_eq!(ctors.len(), 2, "two constructors: {ctors:?}");
    // Ctor identity lives either in the `type-annot` payload's `name`
    // (wrapped ctors) or as the leaf's own string payload (bare ctors).
    let ctor_name = |n: &AstNode| {
        n.payload
            .get("name")
            .and_then(|v| v.as_str())
            .or_else(|| n.payload.as_str())
            .map(str::to_string)
    };
    let zero = ctors
        .iter()
        .find(|n| ctor_name(n).as_deref() == Some("zero"))
        .expect("zero ctor");
    // `zero : Nat` — bare nullary ctor: unwrapped `var` leaf.
    assert_eq!(zero.kind, "var", "bare ctor stays a var leaf");
    assert_eq!(zero.payload.as_str(), Some("zero"));
    let succ = ctors
        .iter()
        .find(|n| ctor_name(n).as_deref() == Some("succ"))
        .expect("succ ctor");
    // `succ : (pred : Nat) ~> Nat` — `call` leaf under one `type-annot`.
    assert_eq!(succ.kind, "type-annot");
    let subject = ast
        .get(AstRef::new(children_of(succ)[0]))
        .expect("succ subject");
    assert_eq!(subject.kind, "call", "succ leaf is the arg `call`");
}

#[test]
fn new_inductive_lowers_indexed_family_with_params() {
    // Params ride the assign target exactly like the record spelling:
    // `Vec(n : Nat, a : Type) := new Inductive { … }`.
    let ast = parse_family(
        "Vec(n : Nat, a : Type) := new Inductive {\n  vnil  : Vec(0, a),\n  vcons : (x : a, xs : Vec(n, a)) ~> Vec(n + 1, a)\n}\n",
    );
    let data = lowered_data(&ast);
    assert_eq!(data.payload.get("name").and_then(|v| v.as_str()), Some("Vec"));
    let data_children = children_of(data);
    let head = ast.get(AstRef::new(data_children[0])).expect("head");
    assert_eq!(head.kind, "call", "parameterized head stays a call");
    let mut ctors = Vec::new();
    ctor_leaves(&ast, data_children[1], &mut ctors);
    assert_eq!(ctors.len(), 2, "two constructors: {ctors:?}");
}

#[test]
fn new_inductive_non_family_codomain_stays_new_node() {
    // A field whose codomain does not name the declared family is not a
    // constructor — the `new` node survives for `synth_new_decl` (which
    // reports `Inductive` honestly) rather than mislowering.
    let ast = parse_family("Nat := new Inductive {\n  zero : Nat,\n  bad  : String\n}\n");
    // The assign survives (no `data` node was emitted for it): find the
    // `new` node still present in the arena.
    assert!(
        ast.nodes().iter().any(|n| n.kind == "new"),
        "unlowered `new Inductive` keeps its new node"
    );
    assert!(
        !ast.nodes().iter().any(|n| n.kind == "data"),
        "no data node for a non-ctor field"
    );
}

#[test]
fn new_inductive_inside_block_lowers_in_place() {
    let ast = parse_family(
        "do {\n  Nat := new Inductive { zero : Nat, succ : (pred : Nat) ~> Nat }\n  succ(zero)\n}\n",
    );
    let root = ast.nodes().last().expect("arena root");
    assert_eq!(root.kind, "block", "root stays the enclosing block");
    let kids = children_of(root);
    let first = ast.get(AstRef::new(kids[0])).expect("first statement");
    assert_eq!(first.kind, "data", "new Inductive lowered in place");
}
