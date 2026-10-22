# sdkgen-webidl

`handl-plugin-sdkgen-backend-webidl` — WebIDL backend for the Handl SDK generator (ADR-0113).

Sibling-repo deps are git dependencies (`branch = "main"`); `[patch]` tables
resolve them from sibling checkouts.

`handl-packages/generated/` tracks committed generated SDKs (port dirs
like `webcrypto/`, adapter dirs like `webcrypto-webidl-browser/`, plus
the fetched `.idl` inputs and `.sdkgen.yaml` configs) — the recipes are
reproducible from the `.sdkgen.yaml` + spec pin recorded in each file's
`--- source:` line.

## Docs

- [docs/index.md](docs/index.md) — the `webidl` backend: grammar
  subset, mixin/`includes` resolution, type map, `WebIdlConfig`, MDN
  doc layering.
- Framework: [`host/docs/sdkgen/`](https://github.com/handl-lang/host/blob/main/docs/sdkgen/index.md).
