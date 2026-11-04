#!/usr/bin/env python3
"""Small safety check against the committed exporter and allowlist."""
import importlib.util
from pathlib import Path
import tempfile

spec = importlib.util.spec_from_file_location('export_public', Path(__file__).with_name('export-public.py'))
m = importlib.util.module_from_spec(spec)
spec.loader.exec_module(m)
with tempfile.TemporaryDirectory() as tmp:
    existing = Path(tmp)/'existing'
    existing.mkdir()
    (existing/'sentinel').write_text('keep')
    for dest in [existing, m.ROOT/'forbidden-export']:
        try:
            m.export(dest, 'HEAD')
        except ValueError:
            pass
        else:
            raise AssertionError('unsafe destination accepted')
    assert (existing/'sentinel').read_text() == 'keep'
    a = m.export(Path(tmp)/'a', 'HEAD')
    b = m.export(Path(tmp)/'b', 'HEAD')
    assert a == b
    assert not (Path(tmp)/'a/.git').exists()
    assert not any(p['path'].startswith(('design/', 'docs/publication/', 'docs/requirements/')) for p in a['files'])
print('export safety and deterministic manifest checks passed')
