"""Isolated release worker QA: never opens a window or reads host Steam/config."""
import json
import os
from pathlib import Path
import queue
import subprocess
import sys
import tempfile
import threading
import time

with tempfile.TemporaryDirectory(prefix='steam-theme-qa-') as root:
    root = Path(root)
    home, state, config = [root / name for name in ('home', 'state', 'config')]
    for p in (home, state, config):
        p.mkdir()
    env = dict(os.environ, HOME=str(home), XDG_STATE_HOME=str(state), XDG_CONFIG_HOME=str(config))
    process = subprocess.Popen([str(Path(sys.argv[1] if len(sys.argv) > 1 else 'release/steam-omarchy').resolve()), '--theme-watch-probe'], env=env, stdin=subprocess.PIPE, stdout=subprocess.PIPE, text=True)
    events = queue.Queue()
    threading.Thread(target=lambda: [events.put(json.loads(line)) for line in process.stdout], daemon=True).start()
    def wait(predicate):
        deadline = time.monotonic() + 8
        while time.monotonic() < deadline:
            t = events.get(timeout=max(.01, deadline-time.monotonic()))
            if predicate(t):
                print(json.dumps(t))
                return t
        raise AssertionError('snapshot timeout')
    def palette(bg='#10151b', accent='#123456'):
        return f"background='{bg}'\nforeground='#ffffff'\naccent='{accent}'\nblue='#abcdef'\nselection='#223344'\nmuted='#556677'\n"
    try:
        wait(lambda t: t['source'] == 'Palette de secours')  # no Omarchy / Quickshell
        real = home / '.local/state/omarchy/current/theme'
        real.mkdir(parents=True)
        (real/'colors.toml').write_text(palette())
        first = wait(lambda t: t['roles']['accent'] == '#123456')
        assert first['source'] == 'Omarchy HOME upstream'
        target = state / 'omarchy/current/theme'
        target.mkdir(parents=True)
        (target/'colors.toml').write_text(palette(accent='#654321'))
        second = wait(lambda t: t['roles']['accent'] == '#654321')
        assert second['generation'] > first['generation']
        (target/'colors.toml').write_text("background='")
        invalid = wait(lambda t: any('invalide' in w for w in t['warnings']))
        assert invalid['roles'] == second['roles']
        (target/'colors.toml').write_text(palette(accent='#654321'))
        restored = wait(lambda t: t['roles'] == second['roles'] and t['warnings'] == second['warnings'])
        (target/'colors.toml').write_text("background='#ffffff'")
        incomplete = wait(lambda t: any('invalide' in w for w in t['warnings']))
        assert incomplete['roles'] == restored['roles']
        (target/'colors.toml').write_text(palette(accent='#654321'))
        wait(lambda t: t['roles'] == restored['roles'] and t['warnings'] == second['warnings'])
        (target/'colors.toml').write_text(' ' * 65537)
        oversized = wait(lambda t: any('invalide' in w for w in t['warnings']))
        assert oversized['roles'] == restored['roles']
        staged = target.with_name('next-theme')
        staged.mkdir()
        (staged/'colors.toml').write_text(palette('#ffffff', '#112233'))
        (target/'colors.toml').unlink()
        target.rmdir()
        staged.rename(target)
        third = wait(lambda t: t['roles']['accent'] == '#112233')
        assert third['mode'] == 'light'
        for i in range(20):
            (target/'colors.toml').write_text(palette(accent=f'#{i:06x}'))
        final = wait(lambda t: t['roles']['accent'] == '#000013')
        assert final['generation'] > third['generation']
        time.sleep(.15)  # let the quiet debounce settle into the 30-second wait
        start = time.monotonic()
        process.stdin.close()
        assert process.wait(timeout=2) == 0
        elapsed = time.monotonic() - start
        assert elapsed < 1, elapsed
        print(f'Cooperative shutdown: {elapsed:.3f}s')
        print('PASS: absent parents, HOME/XDG divergence and priority, invalid/incomplete/oversize last-valid, atomic directory replacement, burst final state, monotonic generations')
    finally:
        if process.poll() is None:
            process.kill()
            process.wait(timeout=5)

    # Stop shortly after a burst, using the same real worker.
    process = subprocess.Popen([str(Path(sys.argv[1] if len(sys.argv) > 1 else 'release/steam-omarchy').resolve()), '--theme-watch-probe'], env=env, stdin=subprocess.PIPE, stdout=subprocess.PIPE, text=True)
    try:
        events = queue.Queue()
        threading.Thread(target=lambda: [events.put(json.loads(line)) for line in process.stdout], daemon=True).start()
        wait(lambda t: True)  # watcher installed and initial snapshot published
        for i in range(20):
            (target/'colors.toml').write_text(palette(accent=f'#{i:06x}'))
        start = time.monotonic()
        process.stdin.close()
        process.wait(timeout=2)
        elapsed = time.monotonic() - start
        assert process.returncode == 0 and elapsed < 1, (process.returncode, elapsed)
        print(f'PASS: burst cooperative shutdown {elapsed:.3f}s')
    finally:
        if process.poll() is None:
            process.kill()
            process.wait(timeout=5)
