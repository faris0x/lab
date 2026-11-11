#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
mkdir -p public/wasm

for crate in simplex linalg complexnum; do
  cargo build \
    --manifest-path rust/Cargo.toml \
    --target wasm32-unknown-unknown \
    --release \
    -p "$crate"
  cp "rust/target/wasm32-unknown-unknown/release/$crate.wasm" "public/wasm/$crate.wasm"
  bytes=$(stat -c%s "public/wasm/$crate.wasm")
  gzip_bytes=$(gzip -9c "public/wasm/$crate.wasm" | wc -c)
  echo "$crate.wasm: ${bytes} bytes (${gzip_bytes} bytes gzipped)"
done
