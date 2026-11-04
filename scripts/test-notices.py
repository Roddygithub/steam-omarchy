#!/usr/bin/env python3
"""Verify offline collection is stable and rejects damaged external evidence."""
import json
from pathlib import Path
import subprocess
root = Path(__file__).resolve().parents[1]
collector = root/'scripts/collect-notices.py'
subprocess.run(['python3', str(collector)], check=True)
matrix = root/'docs/licenses/matrix.json'
before = matrix.read_bytes()
subprocess.run(['python3', str(collector)], check=True)
assert matrix.read_bytes() == before
entry = json.loads((root/'docs/licenses/upstream-evidence.json').read_text())[0]
notice = root/'docs/licenses'/entry['notices'][0]['path']
original = notice.read_bytes()
try:
    notice.write_bytes(b'corrupt evidence')
    result = subprocess.run(['python3', str(collector)], capture_output=True)
    assert result.returncode != 0 and b'verified evidence' in result.stderr
    assert matrix.read_bytes() == before
finally:
    notice.write_bytes(original)
print('Offline stability and fail-closed evidence check passed')
