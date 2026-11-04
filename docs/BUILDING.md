# Building on Linux

Use Node 22 (at least 22.12), npm and Rust 1.97.0 with rustfmt/clippy. CI uses
Ubuntu 24.04. Native packages: build-essential, pkg-config, libwebkit2gtk-4.1-dev,
libgtk-3-dev, libayatana-appindicator3-dev, librsvg2-dev, patchelf, libssl-dev.
See the [official Tauri prerequisites](https://v2.tauri.app/start/prerequisites/).
On Arch, see packaging/arch/PKGBUILD for the prepared dependency list.

```sh
npm ci
npm run lint
npm run typecheck
npm run build
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
cargo clippy --locked --manifest-path src-tauri/Cargo.toml -- -D warnings
cargo test --locked --manifest-path src-tauri/Cargo.toml
npm run release
bash scripts/test-regressions.sh
```

The release command builds locally, without bundling or publishing, into
release/. It does not establish redistribution readiness. Tests use temporary
fixtures; do not import local Steam artwork for public test evidence.

CI actions are pinned to upstream commit IDs (checkout v4.2.2 and setup-node
v4.4.0, verified against official Git refs). Rust is installed through rustup
with an exact toolchain version. OS packages and advisory databases track the
runner repositories; builds are lockfile controlled, not bit-for-bit reproducible.
Review pins and tool versions regularly. CI has read-only repository permission,
no deployment secrets and no pull_request_target execution.

Regenerate license evidence after npm ci using
`python3 scripts/collect-notices.py`. Missing notices remain release blockers.
For source export run `python3 scripts/export-public.py NEW_DIRECTORY --ref HEAD`
from the private preparation checkout. The destination must not exist; review
its manifest and run the full checks there before any separate publication.
