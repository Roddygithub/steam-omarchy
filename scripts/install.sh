#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
prefix="${XDG_DATA_HOME:-$HOME/.local/share}"
state="$prefix/steam-omarchy/install.sha256"
paths=("$HOME/.local/bin/steam-omarchy" "$prefix/icons/hicolor/128x128/apps/steam-omarchy.png" "$prefix/applications/steam-omarchy.desktop")
# Refuse symlinks in every destination component, including parents.
for p in "$state" "${paths[@]}"; do
  q="$p"
  while [[ "$q" != / && "$q" != . ]]; do
    [[ ! -L "$q" ]] || { echo "Symlink refused: $q" >&2; exit 1; }
    q=$(dirname "$q")
  done
done
if [[ -e "$state" ]]; then
  # Ownership record must describe exactly these three paths.
  [[ $(wc -l < "$state") == 3 ]] || exit 1
  for p in "${paths[@]}"; do
    expected=$(awk -v p="$p" 'substr($0,67)==p {print substr($0,1,64)}' "$state")
    [[ -n "$expected" && -f "$p" && $(sha256sum "$p" | cut -c1-64) == "$expected" ]] || { echo "Changed installation: $p" >&2; exit 1; }
  done
else
  for p in "${paths[@]}"; do
    [[ ! -e "$p" ]] || { echo "Collision: $p" >&2; exit 1; }
  done
fi
install -Dm755 release/steam-omarchy "${paths[0]}"
install -Dm644 src-tauri/icons/icon.png "${paths[1]}"
mkdir -p "$(dirname "${paths[2]}")" "$(dirname "$state")"
cat > "${paths[2]}" <<DESKTOP
[Desktop Entry]
Type=Application
Name=Steam Omarchy
Comment=Bibliothèque Steam locale
Exec="$HOME/.local/bin/steam-omarchy"
Icon=steam-omarchy
Categories=Game;
Terminal=false
DESKTOP
sha256sum "${paths[@]}" > "$state"
