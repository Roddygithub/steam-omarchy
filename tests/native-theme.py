"""Run after VITE_NATIVE_QA=1 npm run build; cargo build --release --features native-qa,tauri/custom-protocol.
Pass the copied worktree binary. Broadway only; no host display fallback.
"""
import json
import os
from pathlib import Path
import queue
import subprocess
import sys
import tempfile
import threading
import time
from native_lifecycle import close_fixture

ROLES = {'surface', 'foreground', 'accent', 'muted', 'border', 'focus'}
LAST_VALID = 'Thème invalide : dernière palette conservée'


def validate_assertions(measurements):
    assert len(measurements) == 6
    assert measurements[0]['theme']['fallback']
    assert [v['roles']['accent'] for v in measurements[1:]] == [
        '#245b72', '#a5d5ef', '#ab42ce', '#ab42ce', '#654321']
    for value in measurements:
        assert value['url'].split('/', 3)[:3] == ['tauri:', '', 'localhost']
        assert set(value['roles']) == ROLES
        assert all(value['roles'][k] == value['theme']['roles'][k] for k in ROLES)
        assert value['scheme'] == value['theme']['mode']
        assert sum(r['command'] == 'read-ok' for r in value['records']) == 1
        assert sum(r['command'] == 'listen-ok' for r in value['records']) == 1
    assert all(b['theme']['generation'] > a['theme']['generation']
               for a, b in zip(measurements, measurements[1:]))
    assert LAST_VALID in measurements[4]['theme']['warnings']
    assert measurements[4]['roles'] == measurements[3]['roles']
    final = measurements[-1]
    assert final['roles']['accent'] == '#654321'
    assert final['children'] == 0
    assert sum(r['command'] == 'unlisten-ok' for r in final['records']) == 1


if len(sys.argv) == 3 and sys.argv[1] == '--validate':
    validate_assertions(json.loads(Path(sys.argv[2]).read_text()))
    print('PASS offline retained-record assertions (no native execution)')
    sys.exit(0)

out = Path('docs/evidence/native-theme')
out.mkdir(parents=True, exist_ok=True)
with tempfile.TemporaryDirectory(prefix='steam-native-qa-') as tmp:
    root = Path(tmp)
    env = dict(os.environ)
    for key in ('DISPLAY', 'WAYLAND_DISPLAY', 'WAYLAND_SOCKET', 'DBUS_SESSION_BUS_ADDRESS', 'XAUTHORITY'):
        env.pop(key, None)
    for key, name in [('HOME','home'), ('XDG_CONFIG_HOME','config'), ('XDG_STATE_HOME','state'), ('XDG_CACHE_HOME','cache'), ('XDG_DATA_HOME','data'), ('XDG_RUNTIME_DIR','runtime')]:
        (root/name).mkdir(mode=0o700)
        env[key] = str(root/name)
    env.update(GDK_BACKEND='broadway', BROADWAY_DISPLAY=':73', NO_AT_BRIDGE='1')
    events = queue.Queue()
    with (out/'broadway.log').open('w') as log, (out/'app.log').open('w') as app_log:
        daemon = subprocess.Popen(['/usr/bin/broadwayd','--address=127.0.0.1','--port=8153',':73'], env=env, stdout=log, stderr=log, start_new_session=True)
        app = None
        try:
            time.sleep(.5)
            assert daemon.poll() is None, 'Broadway bind failed'
            app = subprocess.Popen([str(Path(sys.argv[1]).resolve())], env=env, stdout=subprocess.PIPE, stderr=app_log, text=True, start_new_session=True)
            def collect():
                for line in app.stdout:
                    app_log.write(line); app_log.flush()
                    if line.startswith('NATIVE_QA '):
                        events.put(json.loads(line[len('NATIVE_QA '):]))
            threading.Thread(target=collect, daemon=True).start()
            def wait(predicate):
                end=time.monotonic()+15
                while time.monotonic()<end:
                    value=events.get(timeout=max(.01,end-time.monotonic()))
                    if predicate(value): return value
                raise AssertionError('native DOM acknowledgment timeout')
            print('Open hidden Broadway preview http://127.0.0.1:8153 now', flush=True)
            initial=wait(lambda v:v['theme'] and v['roles']['accent']==v['theme']['roles']['accent'])
            assert sum(r['command']=='listen-ok' for r in initial['records'])==1
            colors=root/'config/omarchy/current/theme/colors.toml'
            colors.parent.mkdir(parents=True)
            measurements=[initial]
            def palette(bg, fg, accent):
                return f"background='{bg}'\nforeground='{fg}'\naccent='{accent}'\nselection='#556677'\nmuted='{fg}'\n"
            def replace(text):
                pending=colors.with_suffix('.tmp'); pending.write_text(text);pending.replace(colors)
            for bg,fg,accent in [('#f4f2ec','#18252c','#245b72'),('#10151b','#f0f4f7','#a5d5ef'),('#201830','#ffffff','#ab42ce')]:
                replace(palette(bg,fg,accent))
                value=wait(lambda v:v['theme']['roles']['accent']==accent and v['roles']['accent']==accent and v['roles']['surface']==v['theme']['roles']['surface'])
                assert value['theme']['generation']>measurements[-1]['theme']['generation']
                assert all(value['theme']['roles'][k]==v for k,v in value['roles'].items())
                assert value['scheme']==value['theme']['mode']
                measurements.append(value)
                print('PASS native CSS',accent, flush=True)
                time.sleep(float(os.environ.get("NATIVE_QA_CAPTURE_PAUSE", "2")))
            replace('{malformed')
            invalid=wait(lambda v:LAST_VALID in v['theme']['warnings'] and v['theme']['generation']>measurements[-1]['theme']['generation'])
            assert invalid['roles']==measurements[-1]['roles']
            measurements.append(invalid)
            replace(palette('#10151b','#ffffff','#654321'))
            cleanup=wait(lambda v:v['children']==0 and any(r['command']=='unlisten-ok' for r in v['records']))
            measurements.append(cleanup)
            validate_assertions(measurements)
            app.wait(timeout=5)
            assert app.returncode==0
            (out/'measurements.json').write_text(json.dumps(measurements,indent=2))
            print('PASS actual React unmount/unlisten and native window close',flush=True)
        finally:
            close_fixture(app, daemon, root/'config')
