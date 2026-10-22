#!/usr/bin/env bash
# Regenerate the dogfood WebIDL corpus under handl-packages/generated/ —
# gitignored build artifacts produced by `hnl sdkgen webidl` from the
# committed specs in specs/ (w3c/webref @
# 19e2457df856a43ec749c0fd4e88ae1edc4f6cb8, ed/idl/*.idl — pinned so regen
# is deterministic and offline-capable; `--source-url` stamps the upstream
# pin as `--- source:` provenance without fetching; doc comments consult
# MDN via the seeded .mdn-cache.json).
#
# specs/html.idl is an include-only input — it feeds storage.idl's
# `X includes Y` mixin grafting via specs/storage.sdkgen.yaml and produces
# no package of its own.
#
# Usage:  HNL=../host-cli/target/debug/hnl scripts/regen-generated.sh
set -euo pipefail

HNL="${HNL:-../host-cli/target/debug/hnl}"
GEN_DIR="${GEN_DIR:-handl-packages/generated}"
SPEC_DIR="${SPEC_DIR:-specs}"
PIN="19e2457df856a43ec749c0fd4e88ae1edc4f6cb8"
if [ ! -x "$HNL" ]; then
  echo "error: no hnl binary at '$HNL'" >&2
  echo "       build it:   (cd ../host-cli && cargo build --bin hnl)" >&2
  exit 1
fi

mkdir -p "$GEN_DIR"

# Seed the persistent MDN doc cache next to the output dir (the backend
# resolves <output>/../.mdn-cache.json) so regen doesn't re-fetch the
# docstrings it already knows; genuine misses are tolerated and cached
# in place.
if [ -f "$SPEC_DIR/mdn-cache.json" ]; then
  cp "$SPEC_DIR/mdn-cache.json" "$GEN_DIR/.mdn-cache.json"
fi

gen() { # <spec-file> <pkg> [sdkgen-config]
  local spec="$1" pkg="$2" cfg="${3:-}" url
  url="https://raw.githubusercontent.com/w3c/webref/$PIN/ed/idl/$(basename "$spec")"
  echo "== hnl sdkgen webidl $spec --output $GEN_DIR/$pkg =="
  if [ -n "$cfg" ]; then
    "$HNL" sdkgen webidl "$spec" --output "$GEN_DIR/$pkg" --config "$cfg" --source-url "$url"
  else
    "$HNL" sdkgen webidl "$spec" --output "$GEN_DIR/$pkg" --source-url "$url"
  fi
}

gen "$SPEC_DIR/dom.idl"                  dom
gen "$SPEC_DIR/webcrypto.idl"            webcrypto
gen "$SPEC_DIR/streams.idl"              streams
gen "$SPEC_DIR/fetch.idl"                fetch
gen "$SPEC_DIR/FileAPI.idl"              fileapi
gen "$SPEC_DIR/encoding.idl"             encoding            "$SPEC_DIR/encoding.sdkgen.yaml"
gen "$SPEC_DIR/xhr.idl"                  xhr
gen "$SPEC_DIR/performance-timeline.idl" performance-timeline
gen "$SPEC_DIR/urlpattern.idl"           urlpattern
gen "$SPEC_DIR/user-timing.idl"          user-timing
gen "$SPEC_DIR/storage.idl"              storage             "$SPEC_DIR/storage.sdkgen.yaml"
gen "$SPEC_DIR/websockets.idl"           websockets
gen "$SPEC_DIR/hr-time.idl"              hr-time
gen "$SPEC_DIR/url.idl"                  url
# spec has no interfaces — emits a stub types.hnl only.
gen "$SPEC_DIR/console.idl"              console

echo
echo "regenerated $GEN_DIR ($(find "$GEN_DIR" -name '*.hnl' | wc -l) .hnl files)"
