# Third-party notices — source preparation

The project's MIT license applies to original project work only. Dependencies,
trademarks and game artwork retain their own terms. This is an unreleased alpha;
no binary distribution is cleared by this document.

`docs/licenses/matrix.json` inventories 418 installed Linux npm / resolved Cargo
components, including build tools. License expressions are preserved verbatim;
AND is not OR. `texts/` contains unchanged upstream notice evidence and hashes.
`upstream-evidence.json` records exact revisions, registry metadata and immutable
file URLs for the 13 packages whose local package omitted license files.
Run `python3 scripts/collect-notices.py` offline after installing locked packages;
`--fetch-upstream` restores those pinned files and checks their hashes before
writing. Missing or changed external evidence fails rather than silently vanishing.
This graph is not a binary SBOM or clearance for other platforms.

No entries lack evidence, but **one scope issue remains unresolved**:
`@humanfs/types 0.15.0` declares Apache-2.0 but has no license file at its registry
commit. The sibling core LICENSE is contextual evidence, not proof of coverage
for types. Its matrix status is EVIDENCE_SCOPE_REVIEW_REQUIRED. Do not describe
that issue as cleared. An upstream scope confirmation remains needed if shipping
that package; the source export ships notice evidence, not npm dependencies.

Keyv 4.5.4 has no registry gitHead or version tag. Its recorded commit matches
both published source files byte for byte; this is source correspondence, not
proof of the exact publishing commit. Natural-compare's pinned README supplies
the author's copyright and links the author-hosted MIT text, separately hashed.
Selectors' pinned source header points to the Mozilla MPL-2.0 text, also hashed.
The registry/tag evidence and qualifications must travel with these texts.

The source export now includes the complete MIT-licensed GLib 0.18.5 crate
with the upstream soundness correction. Its LICENSE and COPYRIGHT travel in
`vendor/glib-0.18.5`; provenance and file hashes are in `docs/security/`.
This is third-party source redistribution and requires preserving those notices.
Independent security exception review remains pending.

Publication of original code and lockfiles alone does not redistribute
system DSOs or npm/Cargo package code. Preserve these notices when including the
notice collection. It does not require a complete runtime DSO redistribution
clearance. Independent review of the exact sanitized export remains required.

## Binary distribution — blocked

See `docs/licenses/BINARY-OBLIGATIONS.md`. No project MIT grant clears bundled
third-party binaries, source duties or system-library redistribution.
