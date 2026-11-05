"""Owned fixture cleanup only. Never discover or signal host applications."""
import os
from pathlib import Path
import signal
import subprocess


def request_close(config):
    colors = Path(config) / 'omarchy/current/theme/colors.toml'
    colors.parent.mkdir(parents=True, exist_ok=True)
    pending = colors.with_suffix('.tmp')
    pending.write_text("background='#10151b'\nforeground='#ffffff'\naccent='#654321'\n")
    pending.replace(colors)


def stop_owned(process, timeout=5, signal_group=os.killpg):
    """Caller MUST create process with start_new_session=True.

    Reap normal completion first; TERM/KILL only the verified private session.
    Return whether forced cleanup was needed, so callers cannot call it PASS.
    """
    if process is None:
        return False
    try:
        process.wait(timeout=timeout)
        return False
    except subprocess.TimeoutExpired:
        pass
    for sig in (signal.SIGTERM, signal.SIGKILL):
        if process.poll() is not None:
            return False
        try:
            if os.getsid(process.pid) != process.pid or os.getpgid(process.pid) != process.pid:
                raise RuntimeError('Refusing cleanup outside owned fixture session')
            signal_group(process.pid, sig)
        except ProcessLookupError:
            pass
        try:
            process.wait(timeout=timeout)
            return True
        except subprocess.TimeoutExpired:
            continue
    raise RuntimeError('Owned fixture did not exit after bounded cleanup')


def close_fixture(app, daemon, config):
    """Ask GTK to close before stopping its display; always reap the display."""
    try:
        if app is not None and app.poll() is None:
            request_close(config)
        forced = stop_owned(app)
        if app is not None and (forced or app.returncode != 0):
            raise RuntimeError(f'Native fixture shutdown failed: forced={forced}, status={app.returncode}')
    finally:
        stop_owned(daemon, timeout=1)
