# Installation

No release is available. Build from audited source following BUILDING.md.
The alpha requires Linux GTK3/WebKitGTK 4.1 and the official Steam client for
launching. Omarchy is optional. No root access is needed for the user installer.

After `npm run release`, run `bash scripts/install.sh`. It installs the binary
to ~/.local/bin, the desktop entry and icon under XDG_DATA_HOME (default
~/.local/share), and records checksums. Add ~/.local/bin to PATH if needed.
The installer refuses collisions and symlinks. Run
`bash scripts/uninstall.sh` to remove unchanged owned files; modified files
are retained. It does not remove Steam, games or personal preferences.

The Arch recipe is preparation only: build from a reviewed sanitized archive
using makepkg in an isolated build environment. It has not been published to
AUR or validated in a clean Arch chroot. Never run makepkg as root.
