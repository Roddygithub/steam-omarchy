# Lead implementation decision — 2026-11-05

The integrated independent APPROVE WITH CONDITIONS review
(RUSTSEC-REVIEW.md, integrated in 4499d77) supports the exact two-line GLib backport and only the
source-alpha maintenance exception for RUSTSEC-2024-0370. The lead accepts
that bounded risk until **2026-12-05 00:00:00 UTC**, or a pinned lock,
manifest, vendor/verifier/regression or relevant advisory content change.
The machine-readable policy is scripts/security/source-alpha.json.

The gate runs the complete manual vendor verifier and optimized iterator
regression, verifies resolved dependency sources, fetches the official RustSec
database and scans with cargo-audit 0.22.2 --deny warnings with no ignores.
Raw exit 1 remains a strict audit failure; only its exact reviewed maintenance
warning may receive a separate source-alpha acceptance. Missing or changed
warnings also fail. Relevant advisory file bytes are pinned, not the whole DB:
unrelated additions remain scanned and any new applicable finding blocks.

No runtime vulnerability waiver exists for GLib. Path dependency scanner
omission is not clearance. The independently reproduced unpatched optimized
SIGSEGV and exact two-line provenance remain recorded in RUSTSEC-REVIEW.md.
Binary packaging/distribution remains blocked by BINARY-OBLIGATIONS.md,
including license and qualification conditions. No export or push is authorized
before independent review of this new implementation tree.

Historical investigation follows; its pending-decision language is superseded
by this bounded implementation decision, not by publication approval.

# RustSec triage — 2026-11-05

Status: **source alpha publication blocked pending independent patch/exception
review; binary distribution separately blocked** (see
`../licenses/BINARY-OBLIGATIONS.md`). No exception is enabled by this change.
The existing `cargo audit --deny warnings` gate remains intact. A successful
build is not audit clearance. Do not publish based only on zero vulnerabilities.

## Evidence and compatible upgrade decision

The actual cargo-audit 0.22.2 report (advisory database commit
965bf9ad083b58e6340d65829d85217b18be9290) has zero vulnerabilities and two
informational warnings, both denied by policy. The older 0.21.2 CVSS 4 parsing
failure did not establish a clean graph. CI now pins 0.22.2.

Official sources checked 2026-11-05:

- https://rustsec.org/advisories/RUSTSEC-2024-0429.html
- https://rustsec.org/advisories/RUSTSEC-2024-0370.html
- https://github.com/gtk-rs/gtk-rs-core/pull/1343
- https://github.com/gtk-rs/gtk-rs-core/commit/05dff0ee696f9bcd8617cd48c4b812d046d440cb
- https://crates.io/api/v1/crates/glib (latest 0.18 release: 0.18.5)
- https://crates.io/api/v1/crates/tauri (latest: 2.12.5)

Current locked Tauri 2.12.5 uses GTK 0.18 on Linux, which uses GLib 0.18.
There is no published compatible 0.18 fix. The PR's backported label does not
prove a 0.18 release exists. GLib >=0.20 is RustSec's fixed range; forcing that
minor into GTK 0.18 mixes incompatible Rust types and is not a supported upgrade.
A future Tauri/GTK migration should remove this vendor, but is outside this fix.

## RUSTSEC-2024-0429 — corrected locally, exception NOT approved

`VariantStrIter::impl_get` passed an immutable reference to a pointer as a C
variadic output parameter. Optimized code can discard the write, then pass NULL
to `CStr::from_ptr`, causing undefined behavior/crashes. This is a real soundness
bug, not a scanner false positive. App code does not call this iterator directly;
GTK/Tauri use the library transitively. Absence of an application call is not
proof of transitive unreachability, so no unreachable-code waiver is claimed.

`vendor/glib-0.18.5` contains the complete published crate, including LICENSE,
COPYRIGHT, original manifests, tests and registry VCS metadata. Version remains
0.18.5. Its only source modifications are the two upstream lines making `p`
mutable and passing `&mut p`. The merge commit above is immutable patch evidence.
Archive: https://static.crates.io/crates/glib/glib-0.18.5.crate
SHA256: `233daaf6e83ae6a12a52055f568f9d7cf4671dabb78ff9560ab6da230ce00ee5`.
`glib-vendor-sha256.json` covers every extracted file after patching.

Reproduce proof:

```
python3 scripts/verify-glib-vendor.py --upstream
cargo test --locked --release --manifest-path src-tauri/Cargo.toml --test glib_variant_str_iter
```

Local negative control: the same regression against the registry GLib 0.18.5
without the path patch terminates with SIGSEGV in release mode; the patched
regression passes.

The offline verifier and optimized regression run in CI. The regression covers
next, next_back, nth, nth_back, last and empty iteration, using the same patched
package as the application. It is not a general proof of memory safety.
RustSec fixed-version metadata still lists 0.18.5 as affected. In the actual
0.22.2 rerun, cargo-audit omits the GLib warning once the package is a local
path dependency (no registry source). That scanner omission is not clearance;
the local verifier and optimized test are essential. The rerun reports zero
vulnerabilities and RUSTSEC-2024-0370, and still exits 1 under --deny warnings. Any
future ID-specific exception must follow independent review of the exact patch,
lockfile resolution, provenance and optimized test results. Never bump the
version to impersonate 0.20 or ignore all warnings.

## RUSTSEC-2024-0370 — unresolved maintenance risk

`glib-macros 0.18.5` uses `proc-macro-error 1.0.4` for build-time diagnostics
(abort/abort_call_site and attribute handling). RustSec reports no patched
version. This is a proc macro/build dependency rather than a runtime parser;
compiling untrusted source is still outside the trust boundary. Unmaintained
diagnostic code can acquire undiscovered bugs; no safety guarantee is claimed.

Replacing it with proc-macro-error2 is not a Cargo-compatible version update:
the package name and major version differ. It requires vendoring glib-macros,
changing its manifest and validating macro expansion and compile-fail diagnostics.
This patch does not expand the vendor surface for an unproven replacement.
The minimal alternative is a transparent maintenance exception after review.

## Required independent decision before publication

Security/release maintainer (role; no individual approval claimed) must review
and sign a change before enabling either exception. A bounded proposed scope is
these two IDs only, this exact lockfile/vendor hash set, source alpha only,
expiring 2026-12-05 or on any affected dependency/advisory change, whichever comes
first. This is a proposal, not an active approval or ignore configuration.
All other warnings and vulnerabilities must remain denied. If approval is
withheld, keep source publication blocked and pursue an upstream 0.18 release,
a supported Tauri migration, or the separately tested glib-macros replacement.
Binary/DSO and license obligations remain independent gates even if approved.

## Local validation

On Rust 1.97.0: upstream archive/complete-tree verification, cargo fmt --check,
Clippy with -D warnings (default and native-qa), cargo test (default and native-qa:
10 app tests plus the iterator regression), optimized iterator regression,
frontend lint/typecheck/build, release binary build, installer regressions and
deterministic committed-export checks passed. The unchanged vendored crate emits
34 upstream compiler warnings; application Clippy checks still pass. Notice
collection reports 418 components, zero missing evidence, one pre-existing
@humanfs/types scope review. No binary license clearance is implied.
The unpatched optimized negative control fails with SIGSEGV as expected.
Cargo-audit 0.22.2 still fails --deny warnings for RUSTSEC-2024-0370; no ignore
configuration was added. Independent review/approval remains outstanding.

## Implementation validation — 2026-11-05

Six stdlib checker tests pass, including exact expiry, lock/manifest/vendor-map/
verifier/regression/advisory tampering, unexpected warning/vulnerability,
classification/settings changes, unexpected tool exits and malformed JSON.
The actual checker passes its compensating gates and accepts only the known
maintenance warning: raw strict cargo-audit exit **1**, zero vulnerabilities,
one RUSTSEC-2024-0370 warning, no ignores. Audit JSON/stderr/exit are retained
locally and uploaded by CI even on failure. Offline audit JSON deliberately has
null Git metadata; the separate mandatory official Git fetch supplies provenance.

Upstream full-tree verification, optimized GLib regression, frontend lint,
typecheck/build, Rust fmt, default Clippy, default and native-qa Rust tests,
installer regressions and Node bounded native-QA/default-off checks pass.
npm audit --audit-level=high exits 0 with seven moderate findings; these still
require release triage. Existing 34 upstream GLib compiler warnings remain visible.
NOT TESTED here: hosted CI, new-tree independent review, sanitized export,
binary release rebuild, or desktop/game execution. Dependencies and application
code are unchanged; a redundant release rebuild was omitted. No export or push
was performed. Binary licensing/qualification blocks remain in force.
