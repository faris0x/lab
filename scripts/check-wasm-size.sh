#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."

LIMIT_BYTES=${WASM_GZIP_LIMIT:-131072}
status=0

for crate in simplex linalg complexnum graphnet series logic floating boolean; do
  file="public/wasm/$crate/${crate}_bg.wasm"
  if [[ ! -f "$file" ]]; then
    echo "missing $file" >&2
    status=1
    continue
  fi
  size=$(gzip -9c "$file" | wc -c)
  if (( size > LIMIT_BYTES )); then
    echo "$crate gzip size ${size} exceeds ${LIMIT_BYTES} bytes" >&2
    status=1
  else
    echo "$crate gzip size ${size} bytes (limit ${LIMIT_BYTES})"
  fi
done

exit "$status"
