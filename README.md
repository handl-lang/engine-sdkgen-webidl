# engine-sdkgen-webidl

`handl-plugin-sdkgen-backend-webidl` — WebIDL backend for the Handl SDK
generator. Part of the `engine-sdkgen-*` family: parses WebIDL specs and emits
SDK bindings.

`handl-packages/generated/` is the dogfood corpus — sdkgen output for a
dozen-plus pinned webref specs (port dirs like `webcrypto/`, adapter dirs
like `webcrypto-webidl-browser/`). It is a **gitignored build artifact**:
the committed inputs live in `specs/` (`.idl` files pinned to
w3c/webref@19e2457, two `.sdkgen.yaml` configs, and the MDN doc-cache
seed) and `scripts/regen-generated.sh` re-emits the whole tree:

```bash
HNL=../host-cli/target/debug/hnl scripts/regen-generated.sh
```

## Development

```bash
cargo xtask gate        # fmt --check → clippy -D warnings → test → doc
```

`cargo xtask` wraps the repo's `xtask/` crate; `ci.sh` is the gate entry.
Deps resolve from git, so no sibling checkouts are needed; a plain clone
test-runs as-is.

## Docs

- [docs/index.md](docs/index.md) — the `webidl` backend: grammar
  subset, mixin/`includes` resolution, type map, `WebIdlConfig`, MDN
  doc layering.
- Framework: [`host/docs/sdkgen/`](https://github.com/handl-lang/host/blob/main/docs/sdkgen/index.md).

## License

MIT.
