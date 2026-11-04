# Steam Omarchy

Experimental Linux desktop library browser and launcher using Rust, Tauri 2,
React and TypeScript. Reads local Steam library metadata and delegates game
launching and Steam features to the official client. Omarchy theme integration
has a built-in fallback for other desktops.

**Unreleased, source-only alpha.** Manifest version 0.1.0 is a development version, not a
published release. Features and formats may change before 1.0. No AUR package
or downloadable release is promised. The UI currently contains French labels.

Independent community project. Not affiliated with, endorsed by, or sponsored
by Valve Corporation or the Omarchy team. Steam and other trademarks belong
to their respective owners.

Start with [installation](docs/INSTALLATION.md), [building](docs/BUILDING.md),
[themes](docs/THEMING.md), [Steam integration](docs/STEAM-INTEGRATION.md), and
[architecture](docs/ARCHITECTURE.md). See [contributing](CONTRIBUTING.md) and
[security](SECURITY.md) for reporting guidance.

## Source and rights

Original project code, documentation and the geometric library icon are MIT
licensed; see LICENSE. Dependencies retain their own licenses; see
THIRD-PARTY-NOTICES.md. No Steam client, game artwork, account data, video
captures or proprietary assets are included in the intended public snapshot.
Local artwork displayed at runtime remains owned by its respective holders.

Publication preparation uses a sanitized initial public snapshot. The original
private repository and its history remain preserved locally and must never be
pushed as the public history. The export tool copies only explicitly listed
files from a committed revision; it never imports Git objects or author data.
A fresh independent audit of that export is required before publication.

No screenshots are included yet. Future screenshots must use fictitious data
and original artwork with documented provenance.
