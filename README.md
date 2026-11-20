# engine-parse-pratt

`hnl-engine-parse-pratt` — the Pratt engine for the parse operation.

An `engine-*` crate: a concrete parser engine driven by precedence/associativity
data from the `Language`, not by per-kind `match` arms.

## Development

```bash
cargo xtask gate        # fmt --check → clippy -D warnings → test → doc
```

`cargo xtask` wraps the repo's `xtask/` crate (`fmt`, `clippy`, `test`, `doc`,
`check`, `gate`); `ci.sh` is the gate entry. Deps resolve from git, so no
sibling checkouts are needed; a plain clone test-runs as-is.

## License

MIT.

## Documentation

<https://handl-lang.github.io/docs/engine-parse-pratt/>
