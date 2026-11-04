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
