import copy
import datetime as dt
import json
from pathlib import Path
import shutil
import tempfile
import unittest
from unittest import mock
import check


class GateTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.policy = json.loads((check.ROOT / 'scripts/security/source-alpha.json').read_text())
        for name in self.policy['files']:
            dest = self.root / name
            dest.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(check.ROOT / name, dest)
        self.db = self.root / 'db'
        # Fixture hashes avoid requiring network/a local DB for unit tests.
        for name in self.policy['advisories']:
            dest = self.db / name
            dest.parent.mkdir(parents=True, exist_ok=True)
            dest.write_text(name)
            self.policy['advisories'][name] = check.digest(dest)
        self.report = {'database': {'advisory-count': 1403, 'last-commit': None, 'last-updated': None}, 'lockfile': {'dependency-count': 418}, 'settings': {'target_arch': [], 'target_os': [], 'severity': None, 'ignore': [], 'informational_warnings': ['unmaintained','unsound','notice']}, 'vulnerabilities': {'found': False, 'count': 0, 'list': []}, 'warnings': {'unmaintained': [copy.deepcopy(self.policy['warning'])]}}
        self.now = dt.datetime(2026,11,5,tzinfo=dt.timezone.utc)

    def validate(self, code=1):
        check.validate(self.policy, self.report, code, self.now, self.root, self.db)

    def test_mandatory_prerequisites_fail_closed(self):
        verifier = [check.sys.executable, '-I', 'scripts/verify-glib-vendor.py', '--upstream']
        regression = ['cargo', 'test', '--locked', '--release', '--manifest-path', 'src-tauri/Cargo.toml', '--test', 'glib_variant_str_iter']
        for results, expected in [([RuntimeError('upstream failed')], [mock.call(verifier)]), ([None, RuntimeError('regression failed')], [mock.call(verifier), mock.call(regression)])]:
            with self.subTest(results=results), mock.patch.object(check.sys, 'argv', ['check.py']), mock.patch.object(check.subprocess, 'check_output', return_value='cargo-audit 0.22.2'), mock.patch.object(check.subprocess, 'run') as audit, mock.patch.object(check, 'run', side_effect=results) as prerequisite:
                with self.assertRaises(RuntimeError):
                    check.main()
                self.assertEqual(prerequisite.call_args_list, expected)
                audit.assert_not_called()

    def test_accept(self):
        self.validate()

    def test_expiry_boundary(self):
        self.now = dt.datetime(2026,12,5,tzinfo=dt.timezone.utc)
        with self.assertRaises(ValueError): self.validate()

    def test_tamper(self):
        for base, names in [(self.root,self.policy['files']), (self.db,self.policy['advisories'])]:
            for name in names:
                with self.subTest(name=name):
                    path = base / name
                    original = path.read_bytes()
                    path.write_bytes(original+b'x')
                    with self.assertRaises(ValueError): self.validate()
                    path.write_bytes(original)

    def test_tool_exit(self):
        for code in [0,2,101,-11]:
            with self.assertRaises(ValueError): self.validate(code)

    def test_report_changes(self):
        original = copy.deepcopy(self.report)
        changes = [lambda r:r.update(error='network'), lambda r:r['warnings'].update(notice=[]), lambda r:r['warnings']['unmaintained'].append({}), lambda r:r.update(vulnerabilities={'found':True,'count':1,'list':[{}]}), lambda r:r['warnings']['unmaintained'][0]['advisory'].update(cvss='9.8'), lambda r:r.update(warnings={}), lambda r:r['settings'].update(ignore=['RUSTSEC-2024-0370'])]
        for change in changes:
            self.report = copy.deepcopy(original)
            change(self.report)
            with self.assertRaises(ValueError): self.validate()

    def test_bad_json(self):
        for raw in ['', '{} trailing', '{"a":1,"a":2}', '{"a":NaN}']:
            with self.assertRaises(ValueError): check.decode(raw)


if __name__ == '__main__': unittest.main()
