#!/usr/bin/env python3
"""Collect unmodified local license evidence; never infer missing terms."""
import argparse
import urllib.request
import hashlib
import json
from pathlib import Path
import subprocess

root = Path(__file__).resolve().parents[1]
out = root / 'docs/licenses'
out.mkdir(exist_ok=True)
meta = json.loads(subprocess.check_output(['cargo', 'metadata', '--offline', '--locked',
    '--filter-platform', 'x86_64-unknown-linux-gnu', '--manifest-path', str(root/'src-tauri/Cargo.toml'), '--format-version', '1']))
resolved = {n['id'] for n in meta['resolve']['nodes']}
rows = []
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--fetch-upstream', action='store_true', help='restore pinned upstream evidence; default is offline')
args = parser.parse_args()
evidence = json.loads((out/'upstream-evidence.json').read_text())
external = {(e['ecosystem'], e['name'], e['version']): e for e in evidence}
for entry in evidence:
    for notice in entry['notices']:
        target = out/notice['path']
        if args.fetch_upstream:
            data = urllib.request.urlopen(notice['url'], timeout=30).read()
            if hashlib.sha256(data).hexdigest() != notice['sha256']:
                raise ValueError('upstream evidence hash mismatch: '+notice['url'])
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_bytes(data)
        if not target.is_file() or hashlib.sha256(target.read_bytes()).hexdigest() != notice['sha256']:
            raise ValueError('missing or changed verified evidence: '+notice['path'])


def collect(ecosystem, name, version, license, source, directory, usage):
    notices = []
    # Include nested license directories (e.g. compiler third-party notices).
    for p in sorted(directory.rglob('*')):
        rel = p.relative_to(directory)
        if 'node_modules' in rel.parts or p.is_symlink() or not p.is_file():
            continue
        if not any(x.lower().startswith(('license', 'licence', 'notice', 'copying', 'copyright')) for x in rel.parts):
            continue
        data = p.read_bytes()
        dest = Path('texts')/ecosystem/(name.replace('/', '__')+'-'+version)/rel
        (out/dest).parent.mkdir(parents=True, exist_ok=True)
        (out/dest).write_bytes(data)
        notices.append({'path': str(dest), 'sha256': hashlib.sha256(data).hexdigest()})
    entry = external.get((ecosystem, name, version))
    if entry:
        if entry['license'] != license:
            raise ValueError('license declaration changed: '+name)
        notices.extend(entry['notices'])
    rows.append(dict(ecosystem=ecosystem, name=name, version=version, license=license or 'UNVERIFIED',
        source=source, usage=usage, notices=notices, status=('EVIDENCE_SCOPE_REVIEW_REQUIRED' if entry and entry.get('qualification') else 'COLLECTED_REVIEW_REQUIRED') if notices and license else 'MISSING'))

for p in meta['packages']:
    if p['id'] in resolved and p['source']:
        collect('cargo', p['name'], p['version'], p['license'], p['source'], Path(p['manifest_path']).parent,
                'Linux resolved graph; build/runtime classification pending binary review')
lock = json.loads((root/'package-lock.json').read_text())
for path, p in lock['packages'].items():
    if not path:
        continue
    directory = root/path
    if not directory.exists():
        continue  # noninstalled optional platform variants are not this Linux installation
    actual = json.loads((directory/'package.json').read_text())
    assert actual['version'] == p['version'], path
    collect('npm', actual['name'], actual['version'], actual.get('license'), p.get('resolved'), directory,
            'build tool' if p.get('dev') else 'frontend runtime')
(out/'matrix.json').write_text(json.dumps(rows, indent=2)+'\n')
print(f'{len(rows)} components; {sum(r["status"] == "MISSING" for r in rows)} missing evidence; {sum(r["status"] == "EVIDENCE_SCOPE_REVIEW_REQUIRED" for r in rows)} unresolved scope')
