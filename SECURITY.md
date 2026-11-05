# Security

No stable supported release exists. Security fixes target the current alpha.
Never include credentials, Steam account files or private logs in an issue.
Use the repository's private vulnerability reporting feature if enabled. No
private reporting address is established yet; if private reporting is absent,
open only a neutral request for a confidential channel without exploit details.

Treat local manifests, artwork and theme files as untrusted input. The app
must validate IPC, limit reads and delegate fixed Steam actions without shell
interpolation. It is not a replacement for Steam authentication or DRM.

CI blocks npm advisories at high/critical severity (including build tools) and
all cargo-audit vulnerabilities. Advisory database/network failures also fail.
Moderate/low npm findings still require triage before release. No blanket
exceptions: document advisory ID, reachability, owner and expiry in a reviewed
change. Audits do not prove absence of vulnerabilities. CI does not publish.

The source-alpha gate is `python3 scripts/security/check.py` (cargo-audit
0.22.2 required). The lead accepts only RUSTSEC-2024-0370 for the exact
reviewed proc-macro-error 1.0.4 registry package until 2026-12-05 00:00:00 UTC.
`scripts/security/source-alpha.json` pins the lock, manifest, complete vendor
map, verifier, optimized regression and both relevant official advisory files.
Any change requires renewed review. Unrelated database updates are scanned.
The checker fetches the official database, preserves strict audit JSON/stderr
and exit status in target/security, and requires exactly exit 1 with the known
maintenance warning. This is bounded acceptance, never an audit-clean result.
Every other warning, vulnerability, malformed report or tool/network error blocks.
Path GLib's scanner omission requires the mandatory complete vendor verifier
and optimized regression; neither can be skipped through checker CLI options.
Production expiry uses current UTC; clock injection exists only in unit tests.
Binary packaging/publication remains blocked by license and qualification
conditions. Independent review of the new tree and sanitized export is still
required before publication; CI does not grant publication clearance.
