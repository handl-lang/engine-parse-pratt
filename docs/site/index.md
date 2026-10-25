# engine-parse-pratt

The Pratt parser: top-down operator precedence driven by grammar data.

## What it is

The parse engine used by the canonical kit. It looks up `GrammarRow`s by
`(token, position)` and drives a classic top-down operator-precedence loop. It
never calls a Rust `rule.prefix(token, ctx)`: prefix/infix kind, binding powers and
child shape are read from the descriptor and executed (ADR-0044 §3).

## Start here

- **Reference** — [public API](/engine-parse-pratt/reference/)
- **How-to** — [task recipes](/engine-parse-pratt/how-to/)
- **Explanation** — [design and rationale](/engine-parse-pratt/explanation/)

No tutorials: this repo is a building block used from Rust. End-to-end
walkthroughs live with the product that composes it — see the
[Handl platform](/handl/).
