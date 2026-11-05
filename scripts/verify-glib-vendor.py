#!/usr/bin/env python3
"""Verify the complete vendored tree; --upstream also proves the exact backport."""
import hashlib
import io
import json
from pathlib import Path
import sys
import tarfile
import urllib.request

root = Path(__file__).resolve().parents[1]
vendor = root / 'vendor/glib-0.18.5'
expected = json.loads((root / 'docs/security/glib-vendor-sha256.json').read_text())
actual = {str(p.relative_to(vendor)): hashlib.sha256(p.read_bytes()).hexdigest()
          for p in vendor.rglob('*') if p.is_file()}
assert actual == expected, 'vendored file set or hashes changed'
assert not any(p.is_symlink() for p in vendor.rglob('*')), 'unexpected symlink'
if '--upstream' in sys.argv:
    data = urllib.request.urlopen('https://static.crates.io/crates/glib/glib-0.18.5.crate', timeout=30).read()
    assert hashlib.sha256(data).hexdigest() == '233daaf6e83ae6a12a52055f568f9d7cf4671dabb78ff9560ab6da230ce00ee5'
    with tarfile.open(fileobj=io.BytesIO(data)) as archive:
        originals = {m.name.split('/', 1)[1]: archive.extractfile(m).read()
                     for m in archive.getmembers() if m.isfile()}
    assert set(originals) == set(actual)
    for name, original in originals.items():
        if name == 'src/variant_iter.rs':
            assert original.count(b'let p: *mut libc::c_char = std::ptr::null_mut();') == 1
            assert original.count(b'                &p,') == 1
            original = original.replace(b'let p: *mut libc::c_char = std::ptr::null_mut();',
                                        b'let mut p: *mut libc::c_char = std::ptr::null_mut();')
            original = original.replace(b'                &p,', b'                &mut p,')
        assert (vendor / name).read_bytes() == original, name
print('GLib vendor hashes verified')
