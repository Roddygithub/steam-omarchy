# Binary obligations — release blocked

The validation ELF is dynamically linked. LINUX-DYNAMIC-LINKAGE.txt records only
its direct readelf NEEDED entries; it has no personal filesystem paths. This
existing local artifact is not a release built from this commit. Resolve the
actual release's transitive DSOs in a clean build environment and record package
versions, source URLs, license texts and linkage before binary distribution.
System-provided GTK/WebKitGTK/GLib and related DSOs are not in the source export.
Do not mistake dynamic linkage for permission to bundle those libraries. Any
LGPL library redistribution needs its own applicable source/relinking compliance.
This application uses Tauri/GTK/WebKitGTK; no Qt dependency is asserted.

MPL covered crates in the Linux graph (no local modifications to registry source):

| Component | Version | Exact-version covered source |
|---|---|---|
| cssparser | 0.37.0 | https://static.crates.io/crates/cssparser/cssparser-0.37.0.crate |
| cssparser-macros | 0.7.1 | https://static.crates.io/crates/cssparser-macros/cssparser-macros-0.7.1.crate |
| dtoa-short | 0.3.5 | https://static.crates.io/crates/dtoa-short/dtoa-short-0.3.5.crate |
| option-ext | 0.2.0 | https://static.crates.io/crates/option-ext/option-ext-0.2.0.crate |
| selectors | 0.38.0 | https://static.crates.io/crates/selectors/selectors-0.38.0.crate |

Before shipping executables, establish which are embedded, provide recipients
clear covered-source availability instructions under MPL-2.0, and retain exact
source archives/checksums and any future patches. Links alone are preparation,
not a completed source delivery mechanism. Preserve other Apache NOTICE,
Unicode, LLVM exceptions and compound-license terms; document alternatives
selected per distributed component. Full binary attribution review is pending.
