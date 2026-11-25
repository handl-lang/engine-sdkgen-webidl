# Agent guide — tool-codegen-webidl

`handl-plugin-codegen-backend-webidl` — WebIDL backend for the Handl
SDK generator (`tool-codegen-*` family): parses WebIDL specs and emits
SDK bindings (`SdkgenModule` IR). Rust 2024.

## Layout

| path | role |
| --- | --- |
| `src/lib.rs` | crate docs + public API |
| `src/parser.rs` | WebIDL grammar subset parser |
| `src/mdn.rs` | MDN metadata integration |
| `src/config.rs` | `WebIdlConfig` |
| `tests/webidl_backend.rs` + `tests/fixtures/` | backend contract tests |
| `specs/` | pinned input specs |
| `scripts/`, `docs/` | helpers + backend doc (grammar subset, mixin/`includes` resolution, type map) |
| `xtask/`, `workflows/gate.yaml`, `bupha.toml`, `.cargo/` | vendored repo gate + CI wiring |

## Contract

- Deps resolve from git (`handl-plugin-codegen` → `handl-lang/host`
  `branch = "main"`); `[patch]` tables map engine/kernel/host siblings
  locally.
- `handl-packages/generated/` is the dogfood corpus — codegen output
  for a dozen-plus pinned webref specs (port dirs like `webcrypto/`,
  adapter dirs).

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
