# engine-parse-pratt

`hnl-engine-parse-pratt` — The **Pratt engine** for the parse operation (ADR-0044 §3).

## Development

```bash
cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test
```

`ci.sh` is the gate (fmt → clippy → test → doc). Deps resolve from git, so no
sibling checkouts are needed; a plain clone test-runs as-is.

## License

MIT.

## Documentation

Published site (Diátaxis): <https://handl-lang.github.io/docs/engine-parse-pratt/>

- [Reference](https://handl-lang.github.io/docs/engine-parse-pratt/reference/) — public API surface
- [How-to](https://handl-lang.github.io/docs/engine-parse-pratt/how-to/) — task recipes
- [Explanation](https://handl-lang.github.io/docs/engine-parse-pratt/explanation/) — design and rationale

Decision records live on the `project` branch under `project/adrs/`.
