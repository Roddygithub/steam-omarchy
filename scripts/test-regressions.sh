#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
fixture=$(mktemp -d)
trap 'rm -rf "$fixture"' EXIT
export HOME="$fixture/home" XDG_DATA_HOME="$fixture/data"
mkdir -p "$HOME/.local/bin"
echo collision > "$HOME/.local/bin/steam-omarchy"
if bash scripts/install.sh; then exit 1; fi
[[ $(cat "$HOME/.local/bin/steam-omarchy") == collision ]]
rm "$HOME/.local/bin/steam-omarchy"
echo sentinel > "$fixture/target"
ln -s "$fixture/target" "$HOME/.local/bin/steam-omarchy"
if bash scripts/install.sh; then exit 1; fi
[[ $(cat "$fixture/target") == sentinel ]]
rm "$HOME/.local/bin/steam-omarchy"
bash scripts/install.sh
bash scripts/install.sh
echo modified >> "$HOME/.local/bin/steam-omarchy"
if bash scripts/install.sh; then exit 1; fi
bash scripts/uninstall.sh
tail -c 9 "$HOME/.local/bin/steam-omarchy" | cmp - <(printf "modified\n")
[[ ! -e "$XDG_DATA_HOME/applications/steam-omarchy.desktop" ]]
# Compile and execute the actual frontend helper, without another test dependency.
./node_modules/.bin/tsc src/preferences.ts --target ES2022 --module ES2022 --moduleResolution bundler --skipLibCheck --outDir "$fixture/js"
cp "$fixture/js/preferences.js" "$fixture/preferences.mjs"
node --input-type=module - "$fixture/preferences.mjs" <<'JS'
import assert from 'node:assert/strict';
const {collectionIds}=await import(process.argv[2]);
for(const key of ['toString','constructor','__proto__']){
  const collections={};
  assert.deepEqual(collectionIds(collections,key),[]);
  const updated={...collections,[key]:[...new Set([...collectionIds(collections,key),'440'])]};
  assert.deepEqual(collectionIds(updated,key),['440']);
  assert.equal(Object.getPrototypeOf(updated),Object.prototype);
}
JS
