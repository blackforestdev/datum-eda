"""One diagnostic pointer stream localizing GPU scene time; not numerical qualification."""
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

OUT = Path(tempfile.mkdtemp(prefix='pm045-scene-', dir=os.environ.get('PM045_REPLAY_OUTPUT_ROOT')))
print(OUT, flush=True)
CANDIDATE = json.loads((Path(__file__).resolve().parent/'candidate.json').read_text())
EXPECTED = CANDIDATE['binary_sha256']
BINARY = ROOT/CANDIDATE['binary']
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
for host in ():
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
source = json.loads((Path(__file__).resolve().parent/'source-bindings.json').read_text())
for path, digest in source.items():
    assert sha(ROOT / path) == digest, path
report = {
    'frontier_step': 'GPI-S5', 'qualification_pass': False,
    'checkout_commit': subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip(),
    'binary_sha256': EXPECTED, 'engine_sha256': sha(ENGINE),
    'normalized_model_sha256': model_hash, 'current_checkout_sha256': source, 'driver_sha256': sha(__file__),
    'fixture_file_sha256': fixture_hashes,
    'fixture_generated_runtime_exclusions': ['.datum/gui-terminal-context.json', '.datum/tool-sessions/', '.datum/terminal-contexts/'], 'pinned_asset_sha256': asset_hashes,
    'declaration': 'Exactly one X11/1x diagnostic30s pointer rectangle at120scheduledpositions/s, after five-second warmup and followed by five-second still tail. Intra-pass GPU markers around grid and composed world; unchanged draws/8xMSAA/pass structure. No replacement run after failure. Original scene/text endpoints retained.',
    'limits': ['Diagnostic localization, not CPU/GPU resource-cap or relative performance acceptance.',
        'Markers include pipeline scheduling/synchronization/marker overhead, not isolated shader execution.',
        'Every expected composed-scene frame must have three valid markers; missing samples fail this method.',
        'Exact before/after pixels/focus and changed-position receipts are not continuous path/hit, display latency or owner UX acceptance.',
        'No full resource/accounting/endurance/otherbackend-scale or independent replay acceptance.'],
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
    DATUM_GUI_LOG=str(log), DATUM_GUI_VERBOSE_LOG='1', DATUM_GPU_MEASUREMENTS='1', DATUM_GPU_SCENE_MARKERS='1',
    EDA_CLI_BIN=str(ENGINE),
    DATUM_MEASUREMENT_SHUTDOWN_SOCKET=str(OUT/'observer.sock'))
cmd = [str(BINARY), '--project-root', str(PROJECT), '--initial-layout', 'single',
       '--window-size', '1280x800', '--visual-scale-factor', '1']
report['command'] = cmd
report['environment'] = {k: env.get(k) for k in ('DISPLAY', 'XDG_SESSION_TYPE', 'WINIT_UNIX_BACKEND', 'WINIT_X11_SCALE_FACTOR', 'DATUM_GUI_VERBOSE_LOG', 'DATUM_GPU_MEASUREMENTS', 'DATUM_GPU_SCENE_MARKERS', 'DATUM_RESOURCE_TRACE', 'DATUM_GPU_ALLOCATION_TRACE', 'DATUM_PRIVATE_TEXT_TRACE', 'EDA_CLI_BIN')}
assert all(env.get(k) is None for k in ('DATUM_RESOURCE_TRACE', 'DATUM_GPU_ALLOCATION_TRACE', 'DATUM_PRIVATE_TEXT_TRACE'))
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

def move_and_ack(wid, x, y):
    for target_x in (x+2, x):
        offset = log.stat().st_size
        xd('mousemove', '--window', wid, target_x, y)
        expected = f'window event WindowId({wid}) cursor moved {target_x:.2f},{y:.2f}'
        def acknowledged():
            with log.open() as stream:
                stream.seek(offset)
                return expected in stream.read()
        until(acknowledged)

def capture(name):
    path = OUT/name
    subprocess.run(['import', '-window', str(main), str(path)], check=True, timeout=10)
    return path

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
    group = Path('/sys/fs/cgroup/user.slice')/f'user-{os.getuid()}.slice'/f'user@{os.getuid()}.service'/('datum-scene-'+uuid.uuid4().hex)
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
    move_and_ack(main, 300, 150)
    wait_to(time.monotonic()+5, 'warmup')
    before_pixels = capture('before.png')
    report['action_log_offset'] = log.stat().st_size
    geometry = dict(line.split('=', 1) for line in report['initial_geometry'].splitlines() if '=' in line)
    report['workload_start_ns'] = time.monotonic_ns()
    pointer_tool = ROOT/'docs/reviews/gui-performance/implementation/S5/pointer-native-batch/tools/pm045-pointer-stream.py'
    shutil.copy2(pointer_tool, assets/'pointer-stream.py')
    asset_hashes['pointer-stream.py'] = sha(assets/'pointer-stream.py')
    subprocess.run([sys.executable, str(assets/'pointer-stream.py'), geometry['X'], geometry['Y'], str(OUT/'pointer.json')], check=True, timeout=40)
    report['workload_end_ns'] = time.monotonic_ns()
    wait_to(time.monotonic()+5, 'idle-tail')
    after_pixels = capture('after.png')
    comparison = subprocess.run(['compare', '-metric', 'AE', str(before_pixels), str(after_pixels), 'null:'], capture_output=True, text=True, timeout=10)
    report['pixel_comparison'] = {'returncode': comparison.returncode, 'ae': comparison.stderr}
    assert comparison.returncode == 0, report['pixel_comparison']
    assert int(xd('getwindowfocus')) == main
    with log.open() as stream:
        stream.seek(report['action_log_offset'])
        action_lines = stream.read().splitlines()
    schedule = json.loads((OUT/'pointer.json').read_text())
    assert schedule['scheduled_count'] == 3600 and len(schedule['rows']) == 3600
    changes = []
    previous = [300, 150]
    for row in schedule['rows']:
        position = row['physical_position']
        if position != previous:
            changes.append(position)
            previous = position
    pattern = re.compile(r'window event WindowId\(' + str(main) + r'\) cursor moved ([0-9.]+),([0-9.]+)')
    received = [[round(float(x)), round(float(y))] for line in action_lines for x,y in pattern.findall(line)]
    remaining = iter(changes)
    for position in received:
        assert any(expected == position for expected in remaining), 'unexpected or out-of-order native cursor position'
    assert received and received[-1] == previous == [300, 150], 'final pointer state missing'
    report['pointer_receipts'] = {'scheduled':3600, 'changed':len(changes), 'received':len(received), 'final':previous, 'all_changed_received':received == changes, 'coalesced_or_missing':len(changes)-len(received)}
    print('One pointer stream complete; final pixels verified; draining GPU records', flush=True)
    report['final_before_close_sample'] = sample('final-before-close')
    report['pre_close_log_offset'] = log.stat().st_size
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
    with log.open() as final_log:
        final_log.seek(report['action_log_offset'])
        drained_lines = final_log.read().splitlines()
    assert not any('gpu_measurement_incomplete ' in line for line in drained_lines), 'incomplete GPU observation through final drain'
    samples = [json.loads(line.split('gpu_measurement ', 1)[1]) for line in drained_lines if 'gpu_measurement ' in line]
    assert samples and all(len(row.get('scene_marker_ticks') or []) == 3 for row in samples), 'missing expected scene markers'
    report['gpu_samples'] = samples
    report['gpu_sample_scope'] = 'All records received from action start through final drain; may include delayed warmup and shutdown work. Diagnostic only.'
    with log.open() as final_log:
        final_log.seek(report['pre_close_log_offset'])
        report['post_close_gpu_samples'] = [json.loads(line.split('gpu_measurement ', 1)[1]) for line in final_log if 'gpu_measurement ' in line]
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
                report['forced_descendant_cleanup'] = True
                (group/'cgroup.kill').write_text('1')
                time.sleep(.5)
            report['group_after_cleanup'] = {'begin_ns': time.monotonic_ns(), 'counts': group_counts(), 'end_ns': time.monotonic_ns()}
            preserve()
            report['remaining_after_cleanup'] = (group/'cgroup.procs').read_text().strip()
            assert not report['remaining_after_cleanup'], 'descendants remain after cleanup'
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
if any(report.get(key) for key in ('error', 'cleanup_errors', 'persistence_errors', 'forced_cleanup', 'forced_descendant_cleanup', 'remaining_before_cleanup', 'remaining_after_cleanup')):
    raise SystemExit(1)
