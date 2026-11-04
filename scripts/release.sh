#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
export CARGO_BUILD_JOBS=2
npm run build
npm run tauri -- build --no-bundle
cargo_target=$(cargo metadata --manifest-path src-tauri/Cargo.toml --format-version 1 --no-deps | python -c 'import json,sys; print(json.load(sys.stdin)["target_directory"])')
install -Dm755 "$cargo_target/release/steam-omarchy" release/steam-omarchy
sha256sum release/steam-omarchy > release/SHA256SUMS
