#!/usr/bin/env bash
set -euo pipefail
prefix="${XDG_DATA_HOME:-$HOME/.local/share}"
state="$prefix/steam-omarchy/install.sha256"
[[ -f "$state" && ! -L "$state" ]] || { echo 'No ownership record; files retained.'; exit 1; }
q="$state"
while [[ "$q" != / && "$q" != . ]]; do
  [[ ! -L "$q" ]] || { echo "Symlink ownership path refused"; exit 1; }
  q=$(dirname "$q")
done
changed=0
for p in "$HOME/.local/bin/steam-omarchy" "$prefix/icons/hicolor/128x128/apps/steam-omarchy.png" "$prefix/applications/steam-omarchy.desktop"; do
  safe=true
  q="$p"
  while [[ "$q" != / && "$q" != . ]]; do
    if [[ -L "$q" ]]; then safe=false; fi
    q=$(dirname "$q")
  done
  expected=$(awk -v p="$p" 'substr($0,67)==p {print substr($0,1,64)}' "$state")
  if $safe && [[ -f "$p" && -n "$expected" && $(sha256sum "$p" | cut -c1-64) == "$expected" ]]; then
    rm -- "$p"
  elif [[ -e "$p" || -L "$p" ]]; then
    echo "Changed or unowned file retained: $p"
    changed=1
  fi
done
if (( changed == 0 )); then rm -- "$state"; fi
printf 'Local preferences retained.\n'
