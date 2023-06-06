#!/bin/bash
set -e

# Generate optimized wasm files and verify generated wasm with cosmwasm-check
mkdir -p artifacts
RUSTFLAGS='-C link-arg=-s' cargo wasm
for WASM in ./target/wasm32-unknown-unknown/release/*.wasm; do
  NAME=$(basename "$WASM" .wasm)${SUFFIX}.wasm
  echo "########Creating intermediate hash for $NAME ...########"
  sha256sum -- "$WASM" | tee -a artifacts/checksums_intermediate.txt
  echo "########Optimizing $NAME ...########"
  wasm-opt -Oz "$WASM" -o "artifacts/$NAME"
  echo "########Verifying $NAME file with cosmwasm-check ...########"
  cosmwasm-check "artifacts/$NAME"
done
