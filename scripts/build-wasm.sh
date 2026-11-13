#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."

WASM_BINDGEN_VERSION="0.2.130"

if ! command -v wasm-bindgen >/dev/null 2>&1; then
  echo "wasm-bindgen CLI not found. Install it with:" >&2
  echo "  cargo install wasm-bindgen-cli --version ${WASM_BINDGEN_VERSION} --locked" >&2
  exit 1
fi

installed_version=$(wasm-bindgen --version)
installed_version=${installed_version##* }
if [[ "$installed_version" != "$WASM_BINDGEN_VERSION" ]]; then
  echo "wasm-bindgen ${WASM_BINDGEN_VERSION} required, found ${installed_version}." >&2
  echo "  cargo install wasm-bindgen-cli --version ${WASM_BINDGEN_VERSION} --locked" >&2
  exit 1
fi

rm -rf public/wasm
mkdir -p public/wasm

BUILD_ARGS=()
if [[ "${WASM_BUILD_STD:-0}" == "1" ]]; then
  # Host toolchains that ship no matching wasm32 std (e.g. Termux) can build
  # std from rust-src instead.
  export RUSTC_BOOTSTRAP=1
  BUILD_ARGS+=(-Z build-std=std,panic_abort)
fi

for crate in simplex linalg complexnum graphnet series logic floating boolean; do
  cargo build \
    "${BUILD_ARGS[@]}" \
    --manifest-path rust/Cargo.toml \
    --target wasm32-unknown-unknown \
    --release \
    -p "$crate"

  out="public/wasm/$crate"
  rm -rf "$out"
  mkdir -p "$out"
  wasm-bindgen \
    --target web \
    --out-dir "$out" \
    --out-name "$crate" \
    "rust/target/wasm32-unknown-unknown/release/$crate.wasm"

  bytes=$(stat -c%s "$out/${crate}_bg.wasm")
  gzip_bytes=$(gzip -9c "$out/${crate}_bg.wasm" | wc -c)
  js_bytes=$(stat -c%s "$out/$crate.js")
  echo "$crate: ${bytes} bytes wasm (${gzip_bytes} gzipped), ${js_bytes} bytes glue"
done
