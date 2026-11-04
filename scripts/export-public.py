#!/usr/bin/env python3
"""Export an exact committed allowlist, with no Git metadata or history."""
import argparse
import hashlib
import json
from pathlib import Path, PurePosixPath
import re
import subprocess

ROOT = Path(__file__).resolve().parents[1]

def require(condition, message='invalid export input'):
    if not condition:
        raise ValueError(message)

def git(*args):
    return subprocess.check_output(['git', '-C', str(ROOT), *args])

def export(destination, ref):
    dest = Path(destination).absolute()
    # Require a new directory outside every original checkout, including main.
    trees = [Path(line[9:]).resolve() for line in git('worktree', 'list', '--porcelain').decode().splitlines() if line.startswith('worktree ')]
    require(not dest.exists() and not dest.is_symlink(), 'destination must not exist')
    resolved = dest.resolve()
    require(all(resolved != p and p not in resolved.parents for p in trees), 'destination inside original checkout')
    sha = git('rev-parse', '--verify', ref+'^{commit}').decode().strip()
    names = json.loads(git('show', sha+':scripts/public-allowlist.json'))
    require(len(names) == len(set(names)), 'duplicate paths')
    files = []
    for name in names:
        p = PurePosixPath(name)
        require(not p.is_absolute() and '..' not in p.parts and '.git' not in p.parts)
        mode, kind, _ = git('ls-tree', sha, '--', name).decode().split(maxsplit=2)
        require(kind == 'blob' and mode in ('100644', '100755'), 'only regular files')
        data = git('show', sha+':'+name)
        if name != 'src-tauri/icons/icon.png':
            text = data.decode('utf-8')
            # Heuristic only; independent human review remains required.
            require(not re.search(r'/home/[A-Za-z0-9_.-]+/|/Users/[A-Za-z0-9_.-]+/|(?m:^-----BEGIN [A-Z ]*PRIVATE KEY-----$)|gh[pousr]_[A-Za-z0-9]{20,}|AKIA[A-Z0-9]{16}', text), 'sensitive pattern: '+name)
        files.append((name, mode, data))
    dest.mkdir(parents=True, exist_ok=False)
    manifest = {'source_commit': sha, 'status': 'NOT_READY_PENDING_INDEPENDENT_AUDIT', 'files': []}
    for name, mode, data in files:
        p = dest/name
        p.parent.mkdir(parents=True, exist_ok=True)
        p.write_bytes(data)
        p.chmod(0o755 if mode == '100755' else 0o644)
        digest = hashlib.sha256(data).hexdigest()
        manifest['files'].append({'path': name, 'source_sha256': digest, 'export_sha256': digest, 'transformation': 'none'})
    (dest/'EXPORT-MANIFEST.json').write_text(json.dumps(manifest, indent=2)+'\n')
    require({str(p.relative_to(dest)) for p in dest.rglob('*') if p.is_file()} == set(names)|{'EXPORT-MANIFEST.json'})
    return manifest

if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('destination')
    parser.add_argument('--ref', default='HEAD')
    args = parser.parse_args()
    result = export(args.destination, args.ref)
    print(f'Exported {len(result["files"])} files; NOT READY until independent audit. No Git history initialized.')
