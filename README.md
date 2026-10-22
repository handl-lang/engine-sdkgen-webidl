# sdkgen-webidl

`handl-plugin-sdkgen-backend-webidl` — WebIDL backend for the Handl SDK generator (ADR-0113).

Sibling-repo deps are git dependencies (`branch = "main"`); `[patch]` tables
resolve them from sibling checkouts.

`handl-packages/generated/` is the dogfood corpus — sdkgen output for a
dozen-plus pinned webref specs (port dirs like `webcrypto/`, adapter dirs
like `webcrypto-webidl-browser/`). It is a **gitignored build artifact**:
the committed inputs live in `specs/` (`.idl` files pinned to
w3c/webref@19e2457, two `.sdkgen.yaml` configs, and the MDN doc-cache
seed) and `scripts/regen-generated.sh` re-emits the whole tree:

```bash
HNL=../host-cli/target/debug/hnl scripts/regen-generated.sh
```

## Docs

- [docs/index.md](docs/index.md) — the `webidl` backend: grammar
  subset, mixin/`includes` resolution, type map, `WebIdlConfig`, MDN
  doc layering.
- Framework: [`host/docs/sdkgen/`](https://github.com/handl-lang/host/blob/main/docs/sdkgen/index.md).
