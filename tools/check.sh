#!/usr/bin/env bash
# Runs every check of this repository (the same script runs in CI):
#   1. the WIT package parses and encodes
#   2. the JSON schemas accept the valid samples and reject the invalid ones
#   3. the conformance plugins build into components
#   4. each component has exactly the expected types/permissions, and the negative
#      components are rejected
#   5. each plugin is packaged as dist/<id>.epplugin
# Needs: cargo with the wasm32-unknown-unknown target, wasm-tools, python3 + jsonschema
# (all provided by `nix develop`).
set -euo pipefail
cd "$(dirname "$0")/.."

TARGET=wasm32-unknown-unknown
PLUGINS=(about importer database-source password-generator store-feed network)
rm -rf dist
mkdir -p dist/negative

echo "== 1. WIT"
wasm-tools component wit wit > /dev/null
wasm-tools component wit wit --wasm -o dist/easepass-plugin.wit.wasm
wasm-tools validate dist/easepass-plugin.wit.wasm

echo "== 2. JSON schemas"
python3 tools/validate_schemas.py

echo "== 3. Conformance plugins"
cargo build --manifest-path conformance/Cargo.toml --release --locked --target "$TARGET"

echo "== 4. Types and permissions"
for plugin in "${PLUGINS[@]}"; do
    core="conformance/target/$TARGET/release/conformance_${plugin//-/_}.wasm"
    component="dist/conformance-$plugin.wasm"
    wasm-tools component new "$core" -o "$component"
    wasm-tools validate "$component"
    python3 tools/check_component.py "$component" \
        --expected "conformance/$plugin/expected.json" \
        --manifest "conformance/$plugin/manifest.json" > /dev/null
    echo "ok   $plugin"
done
for wat in conformance/negative/*.wat; do
    component="dist/negative/$(basename "$wat" .wat).wasm"
    wasm-tools parse "$wat" -o "$component"
    wasm-tools validate "$component"
    python3 tools/check_component.py "$component" --expect-rejected > /dev/null
    echo "ok   $(basename "$wat") (rejected)"
done

echo "== 5. Packages"
for plugin in "${PLUGINS[@]}"; do
    python3 tools/package.py --manifest "conformance/$plugin/manifest.json" \
        --wasm "dist/conformance-$plugin.wasm" --out-dir dist
done

echo "All checks passed."
