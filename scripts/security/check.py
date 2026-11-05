#!/usr/bin/env python3
"""Bounded source-alpha gate. No clock, report, scope or prerequisite CLI overrides."""
import datetime as dt
import hashlib
import json
from pathlib import Path
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[2]
DB = Path.home() / '.cargo/advisory-db'


def require(condition, message):
    if not condition:
        raise ValueError(message)


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def unique(pairs):
    result = {}
    for key, value in pairs:
        require(key not in result, 'duplicate JSON key')
        result[key] = value
    return result


def decode(raw):
    return json.loads(raw, object_pairs_hook=unique,
                      parse_constant=lambda x: require(False, 'invalid JSON constant'))


def validate(policy, report, code, now, root=ROOT, db=DB):
    require(policy['scope'] == 'source-alpha', 'unsupported scope')
    require(now < dt.datetime.fromisoformat(policy['expires']), 'exception expired')
    for name, expected in policy['files'].items():
        require(digest(root / name) == expected, 'reviewed file changed: ' + name)
    for name, expected in policy['advisories'].items():
        require(digest(db / name) == expected, 'advisory changed: ' + name)
    require(code == 1, 'expected strict audit exit 1 only')
    require(set(report) == {'database', 'lockfile', 'settings', 'vulnerabilities', 'warnings'}, 'unknown report schema')
    require(set(report['database']) == {'advisory-count', 'last-commit', 'last-updated'}, 'database schema')
    require(type(report['database']['advisory-count']) is int and report['database']['advisory-count'] > 0, 'empty database')
    # cargo-audit 0.22.2 --no-fetch reports null Git metadata. Fetch is checked above.
    require(report['database']['last-commit'] is None and report['database']['last-updated'] is None, 'unexpected offline database metadata')
    require(report['lockfile'] == {'dependency-count': 418}, 'lockfile report changed')
    require(report['settings'] == {'target_arch': [], 'target_os': [], 'severity': None, 'ignore': [], 'informational_warnings': ['unmaintained', 'unsound', 'notice']}, 'audit settings changed')
    require(report['vulnerabilities'] == {'found': False, 'count': 0, 'list': []}, 'vulnerabilities or unknown format')
    require(report['warnings'] == {'unmaintained': [policy['warning']]}, 'unexpected warning or classification')
    warning = policy['warning']
    require(warning['advisory']['informational'] == 'unmaintained' and warning['advisory']['cvss'] is None, 'not maintenance-only')


def run(args):
    subprocess.run(args, cwd=ROOT, check=True)


def main():
    require(len(sys.argv) == 1, 'no CLI overrides supported')
    policy = decode((ROOT / 'scripts/security/source-alpha.json').read_text())
    version = subprocess.check_output(['cargo-audit', '--version'], text=True).strip()
    require(version == 'cargo-audit 0.22.2', 'cargo-audit 0.22.2 required')
    # No -O: the existing verifier uses assertions.
    run([sys.executable, '-I', 'scripts/verify-glib-vendor.py', '--upstream'])
    run(['cargo', 'test', '--locked', '--release', '--manifest-path', 'src-tauri/Cargo.toml', '--test', 'glib_variant_str_iter'])
    graph = decode(subprocess.check_output(['cargo', 'metadata', '--locked', '--format-version', '1', '--manifest-path', 'src-tauri/Cargo.toml'], cwd=ROOT, text=True))
    for package in graph['packages']:
        if package['name'] in ('steam-omarchy', 'glib'):
            expected = ROOT / ('src-tauri/Cargo.toml' if package['name'] == 'steam-omarchy' else 'vendor/glib-0.18.5/Cargo.toml')
            require(Path(package['manifest_path']).resolve() == expected and package['source'] is None, 'unexpected path resolution')
        else:
            require(package['source'] == 'registry+https://github.com/rust-lang/crates.io-index', 'alternate dependency source')
    # Fetch must succeed separately; audit reads this exact official database offline.
    if not DB.exists():
        run(['git', 'clone', 'https://github.com/RustSec/advisory-db.git', str(DB)])
    require(subprocess.check_output(['git', '-C', str(DB), 'remote', 'get-url', 'origin'], text=True).strip() == 'https://github.com/RustSec/advisory-db.git', 'unofficial database')
    run(['git', '-C', str(DB), 'fetch', 'origin', 'main'])
    run(['git', '-C', str(DB), 'reset', '--hard', 'FETCH_HEAD'])
    require(not subprocess.check_output(['git', '-C', str(DB), 'status', '--porcelain'], text=True).strip(), 'dirty advisory database')
    args = ['cargo', 'audit', '--db', str(DB), '--no-fetch', '--file', 'src-tauri/Cargo.lock', '--deny', 'warnings', '--json']
    result = subprocess.run(args, cwd=ROOT, capture_output=True, text=True)
    logs = ROOT / 'target/security'
    logs.mkdir(parents=True, exist_ok=True)
    (logs / 'audit.json').write_text(result.stdout)
    (logs / 'audit.stderr').write_text(result.stderr)
    (logs / 'audit.exit').write_text(str(result.returncode) + '\n')
    print('Strict cargo-audit exit:', result.returncode, flush=True)
    validate(policy, decode(result.stdout), result.returncode, dt.datetime.now(dt.timezone.utc))
    print('ACCEPTED source-alpha ONLY: strict audit exit 1, known maintenance warning; NOT audit clean.')
    print('Expires ' + policy['expires'] + '; lock/vendor/advisory changes revoke acceptance. Binary publication remains blocked.')


if __name__ == '__main__':
    try:
        main()
    except (ValueError, KeyError, TypeError, OSError, subprocess.SubprocessError) as error:
        print('BLOCKED:', error, file=sys.stderr)
        sys.exit(1)
