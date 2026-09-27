"""Affected native warm-window CPU trial; preserves the existing fixture, pixel/focus and shutdown oracles."""
import ctypes
import hashlib
import importlib.util
import shutil
import json
import os
from pathlib import Path
import re
import socket
import subprocess
import sys
import tempfile
import time
import uuid

ROOT = Path(os.environ.get('PM045_REPLAY_ROOT', subprocess.check_output(['git', 'rev-parse', '--show-toplevel'], text=True).strip()))
TOOLS = ROOT / 'docs/reviews/gui-performance/implementation/S5/resource-snapshots/tools'

OUT = Path(tempfile.mkdtemp(prefix='pm045-startup-', dir=os.environ.get('PM045_REPLAY_OUTPUT_ROOT')))
print(OUT, flush=True)
ROLE = os.environ['PM045_STARTUP_ROLE']
assert ROLE in ('baseline', 'candidate')
TRIAL = int(os.environ['PM045_STARTUP_TRIAL'])
assert TRIAL in (1, 2, 3)
EXPECTED = {'baseline': '05fbc6bc8e9e29c439f901ee6107c5c778ddaee1b8690d33840d52ab71e54b1b',
            'candidate': '35d413855bc19cb12a8f69a2915003ed0b08403d92f4ccac8443c07149de1f6a'}[ROLE]
BINARY = ROOT/'target/pm045-independent-replay-artifacts'/(
    'candidate-ba24e35a-datum-gui' if ROLE == 'baseline' else 'candidate-startup-35d413855bc1-datum-gui')
PROJECT_SOURCE = Path(os.environ['PM045_REPLAY_PROJECT'])
PROJECT = OUT/'project'
def fixture_inventory(project):
    result = {}
    for p in project.rglob('*'):
        rel = str(p.relative_to(project))
        assert not p.is_symlink(), ('unsupported fixture symlink', rel)
        if p.is_file() and rel != '.datum/gui-terminal-context.json' and not rel.startswith(('.datum/tool-sessions/', '.datum/terminal-contexts/')):
            result[rel] = hashlib.sha256(p.read_bytes()).hexdigest()
    return result
fixture_hashes = fixture_inventory(PROJECT_SOURCE)
fixture_manifest = json.loads((ROOT/'docs/reviews/gui-performance/implementation/S5/independent-replay/fixture.json').read_text())
assert fixture_hashes == fixture_manifest['native_files_sha256'], 'fixture differs from replay packet'
for rel in fixture_hashes:
    dest = PROJECT/rel
    dest.parent.mkdir(parents=True, exist_ok=True)
    shutil.copy2(PROJECT_SOURCE/rel, dest)
assert fixture_inventory(PROJECT) == fixture_hashes
sha = lambda p: hashlib.sha256(Path(p).read_bytes()).hexdigest()
assert sha(BINARY) == EXPECTED
ENGINE = ROOT/'target/pm045-independent-replay-artifacts/datum-eda'
ENGINE_EXPECTED = '77b36ff02b6a09e3c42472e89c4631bc773fc7cb1a59677cd28ab3a8b2bf8284'
assert sha(ENGINE) == ENGINE_EXPECTED
assets = OUT/'pinned-assets'
assets.mkdir()
shutil.copy2(TOOLS/'pm045_wm_close.py', assets/'pm045_wm_close.py')
for host in ('GLOBAL', 'PROJECT', 'NEW'):
    shutil.copy2(ROOT/f'docs/reviews/gui-performance/implementation/S5/resolver-cache-ownership/native/{host}.png', assets/(host+'.png'))
asset_hashes = {p.name: sha(p) for p in assets.iterdir()}
spec = importlib.util.spec_from_file_location('pinned_close', assets/'pm045_wm_close.py')
helper = importlib.util.module_from_spec(spec)
spec.loader.exec_module(helper)
close_window = helper.close_window
model = json.loads((PROJECT / 'board/board.json').read_text())
model.pop('uuid', None)
model_hash = hashlib.sha256(json.dumps(model, sort_keys=True, separators=(',', ':')).encode()).hexdigest()
assert model_hash == '33e62de1c1da2020f0608444a4802eac23fb97a9f56cc8cf87c844b6077499ed'
source = json.loads((Path(__file__).resolve().parent/'startup-source-bindings.json').read_text())
for path, digest in source.items():
    assert sha(ROOT / path) == digest, path
report = {
    'frontier_step': 'GPI-S5', 'qualification_pass': False,
    'checkout_commit': subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip(),
    'binary_sha256': EXPECTED, 'engine_sha256': sha(ENGINE),
    'binary_role': ROLE, 'trial': TRIAL, 'binary_source_record': ('docs/reviews/gui-performance/implementation/S5/repeated-device-recovery/candidate.json' if ROLE == 'baseline' else 'docs/reviews/gui-performance/implementation/S5/lazy-terminal-pipeline/startup-candidate.json'),
    'normalized_model_sha256': model_hash, 'current_checkout_sha256': source, 'driver_sha256': sha(__file__),
    'fixture_file_sha256': fixture_hashes,
    'fixture_generated_runtime_exclusions': ['.datum/gui-terminal-context.json', '.datum/tool-sessions/', '.datum/terminal-contexts/'], 'pinned_asset_sha256': asset_hashes,
    'declaration': 'One of six predeclared alternating baseline/candidate X11/1x F-DOA trials: separately record first use of all3hosts, five-second warmup,30warm cycles per GLOBAL/PROJECT/NEW host, five-second idle tail and normal device-live drain. Diagnostic traces and GPU timestamps off. Exact pixels, focus, window destruction and GUI+descendant cgroup CPU retained. No automatic retry or replacement trial.',
    'limits': [
        'Affected warm-open/close CPU and static native correctness only, not full W-WINDOWS/S5 acceptance.',
        'Three before/after pairs are descriptive; no seven-pair relative claim or observer-overhead qualification.',
        'No GPU duty/timestamps/full resource, other backend/scale, cold10start, mixed/endurance, independent replay or owner UX acceptance.',
        'Pixel readiness is not calibrated presentation latency. CPU includes application work until observed readiness/destruction.',
        'Cgroup usage includes exiting descendants; sampled process records do not establish exhaustive short-lived PID identity.',
        'Baseline is the preserved ba24e35a producer binary, not a pre-PM045 baseline.',
        'Ten-minute warm-work check occurs between cycles; command timeouts are separate and this is not a hard whole-process wall-clock deadline.'
    ],
    'cycles': [], 'first_use_cycles': [], 'samples': []
}
def save():
    pending = OUT/'result.pending.json'
    pending.write_text(json.dumps(report, indent=2) + '\n')
    pending.replace(OUT/'result.json')
def preserve():
    # Persistence errors must not prevent process, cgroup or display cleanup.
    try:
        save()
    except BaseException as error:
        report.setdefault('persistence_errors', []).append(repr(error))
        try:
            print('PERSISTENCE FAILED: '+repr(error), file=sys.stderr, flush=True)
        except BaseException:
            pass
(OUT / 'declaration.json').write_text(json.dumps(report, indent=2) + '\n')
save()
log = OUT / 'native.log'
log.touch()
env = os.environ.copy()
for key in list(env):
    if key.startswith(('DATUM_DIAGNOSTIC_', 'DATUM_GPU_DIAGNOSTIC_', 'DATUM_RESOURCE_TRACE', 'DATUM_GPU_ALLOCATION_TRACE', 'DATUM_PRIVATE_TEXT_TRACE')):
        env.pop(key)
for key in ('WAYLAND_DISPLAY', 'LD_AUDIT', 'PM045_X11_AUDIT_PATH', 'DATUM_ACTION_EVIDENCE', 'DATUM_GUI_VERBOSE_LOG'):
    env.pop(key, None)
env.update(WINIT_UNIX_BACKEND='x11', WINIT_X11_SCALE_FACTOR='1',
    XDG_CONFIG_HOME=str(OUT/'config'), XDG_CACHE_HOME=str(OUT/'cache'),
    DATUM_GUI_LOG=str(log), DATUM_GPU_MEASUREMENTS='0',
    EDA_CLI_BIN=str(ENGINE),
    DATUM_MEASUREMENT_SHUTDOWN_SOCKET=str(OUT/'observer.sock'))
cmd = [str(BINARY), '--project-root', str(PROJECT), '--initial-layout', 'single',
       '--window-size', '1280x800', '--visual-scale-factor', '1']
report['command'] = cmd
report['environment'] = {k: env.get(k) for k in ('DISPLAY', 'XDG_SESSION_TYPE', 'WINIT_UNIX_BACKEND', 'WINIT_X11_SCALE_FACTOR', 'DATUM_GUI_VERBOSE_LOG', 'DATUM_GPU_MEASUREMENTS', 'DATUM_RESOURCE_TRACE', 'DATUM_GPU_ALLOCATION_TRACE', 'DATUM_PRIVATE_TEXT_TRACE', 'EDA_CLI_BIN')}
assert all(env.get(k) is None for k in ('DATUM_GUI_VERBOSE_LOG', 'DATUM_RESOURCE_TRACE', 'DATUM_GPU_ALLOCATION_TRACE', 'DATUM_PRIVATE_TEXT_TRACE'))
p = None
group = None
listener = None
stream = None
old_scale = None
libc = ctypes.CDLL(None, use_errno=True)
clock_id = ctypes.c_int()

def xd(*args):
    result = subprocess.run(['xdotool', *map(str, args)], capture_output=True, text=True, timeout=5)
    if result.returncode and args[0] != 'search':
        raise RuntimeError(result.stderr)
    return result.stdout.strip()

def until(fn, seconds=15):
    end = time.monotonic() + seconds
    while time.monotonic() < end:
        assert p.poll() is None, ('process exit', p.returncode)
        result = fn()
        if result:
            return result
        time.sleep(.025)
    raise AssertionError('native predicate timed out')

def group_counts():
    return dict((k, int(v)) for k, v in (line.split() for line in (group/'cpu.stat').read_text().splitlines()))

def sample(label):
    members = (group/'cgroup.procs').read_text().split()
    processes = []
    for pid in members:
        try:
            status = Path(f'/proc/{pid}/status').read_text()
            stat = Path(f'/proc/{pid}/stat').read_text().rsplit(')', 1)[1].split()
            processes.append({'pid': int(pid), 'start_ticks': int(stat[19]),
                'rss': dict(line.split(':', 1) for line in status.splitlines()
                            if line.startswith(('Name:', 'VmRSS:', 'VmHWM:')))})
        except FileNotFoundError:
            processes.append({'pid': int(pid), 'exited_during_sample': True})
    row = {'label': label, 'monotonic_ns': time.monotonic_ns(), 'group': group_counts(),
           'gui_cpu_ns': time.clock_gettime_ns(clock_id.value), 'processes': processes}
    report['samples'].append(row)
    return len(report['samples'])-1

def wait_to(deadline, label):
    while time.monotonic() < deadline:
        assert p.poll() is None, ('process exit', p.returncode)
        sample(label)
        save()
        time.sleep(min(5, max(0, deadline-time.monotonic())))

def pixels(wid, host, index):
    reference = assets/(host+'.png')
    assert sha(reference) == asset_hashes[reference.name]
    attempts = []
    for attempt in range(15):
        path = OUT/f'{index:03d}-{host}-{attempt}.png'
        subprocess.run(['import', '-window', str(wid), str(path)], check=True, timeout=10)
        result = subprocess.run(['compare', '-metric', 'AE', str(reference), str(path), 'null:'], capture_output=True, text=True, timeout=10)
        attempts.append({'path': path.name, 'ae': result.stderr, 'returncode': result.returncode, 'ns': time.monotonic_ns()})
        if result.returncode == 0:
            return attempts
        time.sleep(.025)
    report['failed_pixel_attempts'] = attempts
    raise AssertionError(('pixel oracle failed', host, index))

def cycle(host, index, due, first_use=False):
    assert int(xd('getwindowfocus')) == main
    row = {'host': host, 'index': index, 'scheduled_ns': int(due*1e9), 'begin_sample': sample('cycle-start')}
    report['first_use_cycles' if first_use else 'cycles'].append(row)
    save()
    x = 148 if host != 'NEW' else 110
    xd('mousemove', '--window', main, x+2, 16)
    xd('mousemove', '--window', main, x, 16)
    time.sleep(.15)  # quiet mode: no diagnostic cursor receipt; final native output remains required
    xd('click', 1)
    if host!='NEW':
        xd('key', '--delay', 20, 'Up', 'Right', *(['Down'] if host=='PROJECT' else []))
    time.sleep(.15)
    row['open_begin_sample'] = sample('open-begin')
    xd('key', 'Return')
    title = {'GLOBAL': 'Global Preferences', 'PROJECT': 'Project Preferences', 'NEW': 'New Project'}[host]
    wid = int(until(lambda: next((w for w in xd('search', '--all', '--onlyvisible', '--pid', p.pid, '--name', '^'+title).splitlines() if int(w)!=main), None)))
    until(lambda: int(xd('getwindowfocus')) == wid)
    row['window'] = wid
    row['geometry'] = xd('getwindowgeometry', '--shell', wid)
    row['pixels'] = pixels(wid, host, index)
    row['open_end_sample'] = sample('open-end')
    time.sleep(.1)
    row['close_begin_sample'] = sample('close-begin')
    xd('key', 'Escape')
    until(lambda: int(xd('getwindowfocus')) == main)
    until(lambda: subprocess.run(['xwininfo', '-id', str(wid)], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, timeout=3).returncode != 0)
    row['close_end_sample'] = sample('close-end')
    row['focus_restored'] = True
    row['completed'] = True
    row['finished_ns'] = time.monotonic_ns()
    assert time.monotonic() < due+30, ('window cycle exceeded bounded timeout', index)
    save()

try:
    before = subprocess.check_output(['kscreen-doctor', '-o'], text=True, timeout=10)
    (OUT/'display-before.txt').write_text(before)
    plain = re.sub(r'\x1b\[[0-9;]*m', '', before)
    panel = next(block for block in plain.split('Output:') if 'eDP-1' in block)
    old_scale = re.search(r'Scale:\s*([0-9.]+)', panel).group(1)
    subprocess.run(['kscreen-doctor', 'output.eDP-1.scale.1'], check=True, capture_output=True, timeout=10)
    time.sleep(2)
    during = subprocess.check_output(['kscreen-doctor', '-o'], text=True, timeout=10)
    (OUT/'display-during.txt').write_text(during)
    group = Path('/sys/fs/cgroup/user.slice')/f'user-{os.getuid()}.slice'/f'user@{os.getuid()}.service'/('datum-startup-'+uuid.uuid4().hex)
    group.mkdir()
    report['cgroup'] = str(group)
    report['group_before_spawn'] = group_counts()
    listener = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
    listener.bind(str(OUT/'observer.sock'))
    listener.listen(1)
    listener.settimeout(10)
    def enter_group():
        (group/'cgroup.procs').write_text(str(os.getpid()))
    stream = (OUT/'stderr.log').open('w')
    p = subprocess.Popen(cmd, cwd=ROOT, env=env, stdout=stream, stderr=stream, start_new_session=True, preexec_fn=enter_group)
    report['pid'] = p.pid
    assert libc.clock_getcpuclockid(p.pid, ctypes.byref(clock_id)) == 0
    main = int(until(lambda: xd('search', '--all', '--onlyvisible', '--pid', p.pid, '--name', '^Datum EDA')).splitlines()[0])
    report['main_window'] = main
    xd('windowactivate', '--sync', main)
    until(lambda: int(xd('getwindowfocus')) == main)
    time.sleep(5)
    report['initial_geometry'] = xd('getwindowgeometry', '--shell', main)
    assert 'WIDTH=1280' in report['initial_geometry'] and 'HEIGHT=800' in report['initial_geometry']
    for index, host in enumerate(('GLOBAL', 'PROJECT', 'NEW')):
        cycle(host, -3+index, time.monotonic(), first_use=True)
    report['warmup_start_ns'] = time.monotonic_ns()
    print('Native first use verified; five-second warmup', flush=True)
    wait_to(time.monotonic()+5, 'warmup')
    start = time.monotonic()
    report['workload_start_ns'] = int(start*1e9)
    for index in range(90):
        assert time.monotonic()-start < 600, 'warm trial exceeded ten-minute bound'
        cycle(('GLOBAL', 'PROJECT', 'NEW')[index%3], index, time.monotonic())
        if index%30 == 29:
            print(f'{index+1}/90 warm cycles complete', flush=True)
    report['workload_end_ns'] = time.monotonic_ns()
    wait_to(time.monotonic()+5, 'idle-tail')
    report['final_before_close_sample'] = sample('final-before-close')
    close_window(main)
    connection, _ = listener.accept()
    with connection:
        connection.settimeout(2)
        receipt = b''
        while not receipt.endswith(b'\n'):
            part = connection.recv(1)
            assert part and len(receipt)<4096
            receipt += part
        report['shutdown_receipt'] = json.loads(receipt)
        assert report['shutdown_receipt']['pid'] == p.pid
        assert report['shutdown_receipt']['phase'] == 'drained_device_live'
        sample('drained-device-live')
        connection.sendall(b'!')
    p.wait(timeout=15)
    report['normal_exit_code'] = p.returncode
    assert p.returncode == 0
except BaseException as error:
    report['error'] = repr(error)
    preserve()
    print('FAILED: '+repr(error), flush=True)
    if p and p.poll() is None:
        for name in ('status', 'wchan', 'stack'):
            try:
                (OUT/('failure-proc-'+name+'.txt')).write_text(Path(f'/proc/{p.pid}/{name}').read_text())
            except OSError as capture_error:
                report.setdefault('failure_capture_errors', []).append(repr(capture_error))
finally:
    # Each cleanup result is saved independently; one failure cannot erase others.
    def cleanup(label, operation):
        try:
            operation()
        except BaseException as error:
            report.setdefault('cleanup_errors', {})[label] = repr(error)
        preserve()
    def stop_owned():
        if p and p.poll() is None:
            report['forced_cleanup'] = True
            preserve()
            p.terminate()
            try:
                p.wait(timeout=5)
            except subprocess.TimeoutExpired:
                p.kill()
                p.wait(timeout=5)
    cleanup('stop-owned', stop_owned)
    def close_group():
        if group and group.exists():
            report['group_after_root_exit'] = {'begin_ns': time.monotonic_ns(), 'counts': group_counts(), 'end_ns': time.monotonic_ns()}
            remaining = (group/'cgroup.procs').read_text().strip()
            report['remaining_before_cleanup'] = remaining
            preserve()
            if remaining:
                (group/'cgroup.kill').write_text('1')
                time.sleep(.5)
            report['group_after_cleanup'] = {'begin_ns': time.monotonic_ns(), 'counts': group_counts(), 'end_ns': time.monotonic_ns()}
            preserve()
            if not (group/'cgroup.procs').read_text().strip():
                group.rmdir()
    cleanup('close-group', close_group)
    if stream:
        cleanup('close-stream', stream.close)
    if listener:
        cleanup('close-listener', listener.close)
        cleanup('unlink-socket', lambda: (OUT/'observer.sock').unlink(missing_ok=True))
    def restore_display():
        if old_scale:
            result = subprocess.run(['kscreen-doctor', 'output.eDP-1.scale.'+old_scale], capture_output=True, text=True, timeout=10)
            report['display_restore_returncode'] = result.returncode
            assert result.returncode == 0, result.stderr
            after = subprocess.check_output(['kscreen-doctor', '-o'], text=True, timeout=10)
            (OUT/'display-after.txt').write_text(after)
            plain = re.sub(r'\x1b\[[0-9;]*m', '', after)
            panel = next(block for block in plain.split('Output:') if 'eDP-1' in block)
            assert re.search(r'Scale:\s*([0-9.]+)', panel).group(1) == old_scale
    cleanup('restore-display', restore_display)
    def verify_artifacts():
        report['binary_unchanged'] = sha(BINARY) == EXPECTED
        report['engine_unchanged'] = sha(ENGINE) == ENGINE_EXPECTED
        report['fixture_file_sha256_final'] = fixture_inventory(PROJECT)
        report['fixture_unchanged'] = report['fixture_file_sha256_final'] == fixture_hashes
        final_model = json.loads((PROJECT/'board/board.json').read_text())
        final_model.pop('uuid', None)
        report['normalized_model_sha256_final'] = hashlib.sha256(json.dumps(final_model, sort_keys=True, separators=(',', ':')).encode()).hexdigest()
        report['pinned_assets_unchanged'] = all(sha(assets/name) == digest for name, digest in asset_hashes.items())
        report['production_source_unchanged'] = all(sha(ROOT/path) == digest for path, digest in source.items())
        assert report['binary_unchanged'] and report['engine_unchanged'] and report['fixture_unchanged'] and report['pinned_assets_unchanged'] and report['production_source_unchanged']
        assert report['normalized_model_sha256_final'] == model_hash
    cleanup('verify-artifacts', verify_artifacts)
    preserve()
if report.get('error') or report.get('cleanup_errors') or report.get('persistence_errors'):
    raise SystemExit(1)
