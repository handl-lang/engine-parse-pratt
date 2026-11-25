# Agent guide — tool-parse-pratt

`hnl-tool-parse-pratt` — the Pratt engine for the parse operation:
a concrete parser engine driven by precedence/associativity data from
the `Language`, not by per-kind `match` arms. `publish = false`
(git-install only). Rust 2024.

## Layout

| path | role |
| --- | --- |
| `src/lib.rs` | the Pratt engine implementing `hnl-tool-parse` traits |
| `tests/` | `family_record`, `let_identifier`, `parse_conformance` |
| `project/`, `docs/` | project data + engine doc |
| `xtask/`, `workflows/gate.yaml`, `bupha.toml`, `.cargo/` | vendored repo gate + CI wiring |

## Contract

- Deps resolve from git — `handl-language` → `handl-lang/kernel`,
  `hnl-tool-parse`/`hnl-tool-lex-pattern` → sibling `tool-*` repos,
  `handl-test-utils`/`handl-kit-data-prelude` → `handl-lang/host`, all
  `branch = "main"`.
- Precedence/associativity come from `Language` data — a new operator
  is a data change, never an engine edit.

## Verification

```bash
cargo xtask gate   # fmt --check → clippy -D warnings → test → doc
```


<!-- myco:agent-awareness -->

## Agent awareness

- Session start / any work discussion: run `myco status --prompt`
  before proposing next steps — know the board first.
- User describes new work → offer to capture it (`myco todo add`,
  `issue new`, `proposal new`). Don't wait to be asked.
- Picking up a task: `myco resume <ID>` is the briefing.
- Wrapping up: `myco handoff <ID>` anything resumable; keep statuses
  honest (claimed → in-progress → blocked/done).

<!-- /myco:agent-awareness -->
