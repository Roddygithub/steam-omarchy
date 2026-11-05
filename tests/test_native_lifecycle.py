"""Offline lifecycle checks: fake processes only, no compositor or signals."""
import subprocess
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch
from native_lifecycle import close_fixture, stop_owned


class FakeProcess:
    pid = 987654
    returncode = None

    def __init__(self, name, events, hung=False):
        self.name, self.events, self.hung = name, events, hung

    def poll(self):
        return self.returncode

    def wait(self, timeout):
        self.events.append(self.name)
        if self.hung:
            raise subprocess.TimeoutExpired(self.name, timeout)
        self.returncode = 0
        return 0


class LifecycleTests(unittest.TestCase):
    def test_client_closes_before_display(self):
        events = []
        with tempfile.TemporaryDirectory() as root:
            close_fixture(FakeProcess('client', events), FakeProcess('display', events), root)
            self.assertIn('#654321', (Path(root)/'omarchy/current/theme/colors.toml').read_text())
        self.assertEqual(events, ['client', 'display'])

    def test_timeout_escalates_only_owned_session_and_reports_failure(self):
        events, signals = [], []
        app = FakeProcess('client', events, hung=True)
        def signal(pid, sig):
            signals.append((pid, sig))
            app.hung = False
        with patch('native_lifecycle.os.getsid', return_value=app.pid), patch('native_lifecycle.os.getpgid', return_value=app.pid):
            self.assertTrue(stop_owned(app, signal_group=signal))
        self.assertEqual(len(signals), 1)
        self.assertEqual(signals[0][0], app.pid)

    def test_refuses_host_session(self):
        app = FakeProcess('client', [], hung=True)
        with patch('native_lifecycle.os.getsid', return_value=1):
            with self.assertRaisesRegex(RuntimeError, 'Refusing'):
                stop_owned(app, signal_group=lambda *_: self.fail('host signal'))

    def test_display_cleanup_after_client_error(self):
        events = []
        with tempfile.TemporaryDirectory() as root, patch('native_lifecycle.stop_owned', side_effect=[RuntimeError('timeout'), False]) as stop:
            app, display = FakeProcess('client', events), FakeProcess('display', events)
            with self.assertRaisesRegex(RuntimeError, 'timeout'):
                close_fixture(app, display, root)
            self.assertIs(stop.call_args.args[0], display)

    def test_kill_timeout_is_bounded(self):
        app, signals = FakeProcess('client', [], hung=True), []
        with patch('native_lifecycle.os.getsid', return_value=app.pid), patch('native_lifecycle.os.getpgid', return_value=app.pid):
            with self.assertRaisesRegex(RuntimeError, 'bounded'):
                stop_owned(app, signal_group=lambda *args: signals.append(args))
        self.assertEqual(len(signals), 2)


if __name__ == '__main__':
    unittest.main()
