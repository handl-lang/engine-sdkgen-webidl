# The `webidl` backend — WebIDL specs for the `js` client

`handl-plugin-codegen-backend-webidl` parses a WebIDL spec (e.g. a
webref `*.idl`) into `SdkgenModule` IR for the `js` client; the
`hnl-file` emitter (`hnl-file`) renders port + adapter. Framework
context: [`host/docs/codegen/`](https://github.com/handl-lang/host/blob/main/docs/codegen/index.md).

```bash
hnl codegen webidl fetch.idl --output handl-packages/generated/fetch
hnl codegen webidl --from-url https://…/fetch.idl --output …/generated/fetch
hnl codegen webidl fetch.idl --output … --config fetch.sdkgen.yaml
```

`--target` is a free-form slug (default `browser`) that names the
adapter package (`<pkg>-webidl-<slug>`); `--include`/`--exclude` are
**not** accepted (`source_filters: false`).

## What the parser accepts

`parser.rs` is a hand-written WebIDL subset tuned to webref specs:

- **Definitions**: `interface` (with `: base`), `interface mixin`,
  `dictionary` (with `: base`), `namespace`, `enum`, `typedef`,
  `callback`/`callback interface`, `T includes M;`, `partial` on all of
  the container kinds. Partials of the same kind+name merge into one
  member set (a doc on the non-partial wins; a partial-only doc still
  lands).
- **Members**: `const`, `attribute` (`readonly` flagged), dictionary
  fields (`required` flagged, `= default` skipped), operations
  (`optional`/variadic args, `=` defaults — `optional` is parsed but
  emitted as an ordinary port arg; `T...` sets `variadic`),
  `constructor(…)`. `static` marks the operation (`Operation.
  is_static`) so the backend can keep it receiver-less;
  `getter`/`setter`/`deleter`/`legacycaller`/`stringifier`
  qualifiers parse but don't mark the output; `iterable<K,V>`/
  `maplike`/`setlike`/`async_iterable` members are container semantics —
  nothing to emit.
- **Extended attributes** `[…]` are skipped (bracket-depth balanced);
  comments wedged inside still count as docs.
- **Comments become docs**: `//` and `/* */` text attaches to the next
  def/member/enum value; same-line comments document the item they
  trail; file-provenance comment blocks (followed by a blank line) are
  not docs.
- Unrecognized definitions/members produce parser diagnostics, not
  failures — they surface as `module.diagnostics`.

## AST → IR rules

- `interface`/`mixin`/`dictionary` → `Record` (attribute/dictionary
  members become fields; `readonly` → `is_const`) **plus** one
  `Function` per operation/constructor (`<intf>_<op>` snake name,
  `<Intf><Op>` port name, `c_decl = "Interface.member"`).
- **JS dispatch markers** on interface/mixin functions (dictionaries
  carry data members only):
  - non-static operations get a leading `self: Handle` port arg whose
    descriptor is `recv:js_object` — the emitter peels it into
    `invoke(self, "member", …)`.
  - `static` operations and `namespace` members stay receiver-less —
    `call("Interface"/"namespace", "member", …)`.
  - `constructor(…)` returns `Handle`/`js_object` (the constructed
    instance) and maps to `construct("", "Interface", …)`.
  - `attribute EventHandler on<event>` (any `*EventHandler`/
    `*EventHandlerNonNull` typedef, nullable or not) becomes an
    `(self, handler)` function — `recv:js_object` +
    `handler:<event>` descriptors — which the emitter turns into
    `on(self, "<event>", handler)`. The attribute also remains a
    record field; ordinary data attributes have no accessor yet (no
    `get`/`set` op exists).
- `namespace` → functions only (no record).
- `callback` → a `Function` with `c_decl = "callback Name(args…)"` (no
  doc-source lookup — MDN has no page for callbacks).
- `enum` → `EnumConst`s named `<Enum>_<PascalValue>` valued 0, 1, 2…
- `typedef` → `Typedef` via `map_type`.
- `T includes M;` grafts the mixin's members onto `T`'s interface
  (dedup by member name). An `includes` whose *target* lives in a
  supplementary spec pulls that interface in so the augmented record
  emits in this package — see `includes:` below.
- `module.per_record_files = true` — WebIDL records never reference
  each other by name (foreign types are `Handle`), so the file emitter
  writes one `<Record>.hnl` pair per record plus `types.hnl` for
  typedefs/constants/loose functions.
- `client_package = "js"`, `backend_id = "webidl"`, `library`/`module`
  from the spec file stem.

## Type mapping (`map_type`)

| WebIDL | Port | Descriptor |
| --- | --- | --- |
| `void`, `undefined` | `()` | `void` |
| `boolean` | `Bool` | `bool` |
| `byte`/`short`/`long`/`long long` | `Int` | `i8`/`i16`/`i32`/`i64` |
| `octet`/unsigned variants | `Nat` | `u8`/`u16`/`u32`/`u64` |
| `float`/`double`/`unrestricted …` | `Float` | `f32`/`f64` |
| `DOMString`/`USVString`/`ByteString` | `String` | `js_string` |
| `ArrayBuffer`, typed arrays, `BufferSource` | `Bytes` | `js_bytes` |
| `any`, `object`, `Promise<T>`, `sequence<T>`, `record<K,V>`, `FrozenArray`/`ObservableArray`, `T?`, unions, named foreign types | `Handle` | `js_object` |

The `hnl-file` emitter remaps descriptors to `js_*` spellings for the
`js` client (`i32` → `js_number`, `void` → `js_void`, `bool` →
`js_boolean`, `promise*` → `js_promise`); `js_string`/`js_bytes`/
`js_object` pass through. Adapters dispatch per function: `call`
(namespace/static/global ops), `invoke` (`recv:` receiver),
`construct` (`Interface.constructor`), `on` (`handler:<event>`
listener registration — its result convention is `null` + recorded
`error()` = failure, since `on` does not clear `error()` on
success). See `tool-codegen-file/docs/index.md` for the emitted
shapes.

## Config — `WebIdlConfig`

```yaml
docs: { "SubtleCrypto.digest": "…" }   # override doc comments
external_docs: false                 # disable the MDN layer
includes: [streams.idl]              # supplementary specs, relative to this file
```

| Key | Effect |
| --- | --- |
| `docs` | Doc overrides keyed by `Interface.member` (snake `interface_member` and `InterfaceMember` spellings also match), `Interface`, `Interface.field`, typedef/constant names |
| `external_docs` | `false` disables MDN fetch entirely |
| `includes` | Extra spec files contributing `interface mixin` definitions for cross-spec `X includes Y` (e.g. encoding.idl's `GenericTransformStream` lives in streams.idl); their own interfaces/records/typedefs/constants are **not** emitted unless an `includes` pulls one in as a foreign graft target |

## Doc layering

`docs` pass = `WebIdlBackend::apply_docs`: `docs:` override → spec
comment → `MdnDocSource` (fetches `developer.mozilla.org/…/Web/API/
<path>/index.json`, takes `doc.summary`). Key → MDN path: `URL` → `URL`;
`URL.href` → `URL/href` then `URL/href_static` (static members live at
`*_static`); `URL.constructor` → `URL/URL`. Lookups are cached in
`.mdn-cache.json` beside the output dir — misses are cached too, so
404s don't refetch on regen.
