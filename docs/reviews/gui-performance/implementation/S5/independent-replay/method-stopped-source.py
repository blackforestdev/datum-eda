"""Sustained native window/resource trial using preserved S5 observers and oracles."""
import ctypes
import hashlib
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

ROOT = Path('/home/bfadmin/Documents/datum-eda')
TOOLS = ROOT / 'docs/reviews/gui-performance/implementation/S5/resource-snapshots/tools'
sys.path.insert(0, str(TOOLS))
from pm045_wm_close import close_window

OUT = Path(tempfile.mkdtemp(prefix='dev-agent-s5-endurance-'))
print(OUT, flush=True)
BINARY = ROOT / 'target/release/datum-gui'
PROJECT = Path('/tmp/pm045-admission-hqugp169/project')
EXPECTED = 'b8c1fa1bf390d0dd768554b7b86548a6dc9b9777d8f6fdb5a1c7107f8e7e2b5e'
sha = lambda p: hashlib.sha256(Path(p).read_bytes()).hexdigest()
assert sha(BINARY) == EXPECTED
model = json.loads((PROJECT / 'board/board.json').read_text())
model.pop('uuid', None)
model_hash = hashlib.sha256(json.dumps(model, sort_keys=True, separators=(',', ':')).encode()).hexdigest()
assert model_hash == '33e62de1c1da2020f0608444a4802eac23fb97a9f56cc8cf87c844b6077499ed'
source = json.loads(Path('/tmp/pm045-focus-campaign-zaw4cz7n/declaration.json').read_text())['source_sha256']
for path, digest in source.items():
    assert sha(ROOT / path) == digest, path
report = {
    'frontier_step': 'GPI-S5', 'qualification_pass': False,
    'candidate_commit': subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip(),
    'binary_sha256': EXPECTED, 'engine_sha256': sha(ROOT / 'target/release/datum-eda'),
    'normalized_model_sha256': model_hash, 'source_sha256': source, 'driver_sha256': sha(__file__),
    'declaration': 'One X11/1x F-DOA sustained run; 300s warmup, then 600 cycles starting every6s for3600s, round-robin GLOBAL/PROJECT/NEW (200 each). No replacements after failure. Native pixel/focus/closure oracle each cycle. Group CPU and process RSS throughout; existing resource/private/GPU observations retained. No fault injection in this run.',
    'limits': [
        'Diagnostics-on structural/resource endurance only; no diagnostics-off numerical CPU/GPU or observer-overhead acceptance.',
        'Sampled overlapping ownership views and reservation peaks do not establish full instantaneous memory or driver residency.',
        'No injected recovery, other backend/scale, mixed terminal workload, resolved schematic, independent replay or owner UX acceptance.',
        'Exact X11 window readback is static readiness, not calibrated physical presentation.',
        'Cgroup counts include exiting descendants, but sampled process identities do not establish exhaustive short-lived process identity.'
    ],
    'cycles': [], 'samples': []
}
def save():
    (OUT / 'result.json').write_text(json.dumps(report, indent=2) + '\n')
(OUT / 'declaration.json').write_text(json.dumps(report, indent=2) + '\n')
save()
log = OUT / 'native.log'
log.touch()
env = os.environ.copy()
for key in list(env):
    if key.startswith(('DATUM_DIAGNOSTIC_', 'DATUM_GPU_DIAGNOSTIC_', 'DATUM_RESOURCE_TRACE', 'DATUM_GPU_ALLOCATION_TRACE', 'DATUM_PRIVATE_TEXT_TRACE')):
        env.pop(key)
for key in ('WAYLAND_DISPLAY', 'LD_AUDIT', 'PM045_X11_AUDIT_PATH', 'DATUM_ACTION_EVIDENCE'):
    env.pop(key, None)
env.update(WINIT_UNIX_BACKEND='x11', WINIT_X11_SCALE_FACTOR='1',
    XDG_CONFIG_HOME=str(OUT/'config'), XDG_CACHE_HOME=str(OUT/'cache'),
    DATUM_GUI_LOG=str(log), DATUM_GUI_VERBOSE_LOG='1', DATUM_GPU_MEASUREMENTS='0',
    EDA_CLI_BIN=str(ROOT/'target/release/datum-eda'),
    DATUM_RESOURCE_TRACE=str(OUT/'resources.jsonl'), DATUM_RESOURCE_TRACE_INTERVAL_MS='1000',
    DATUM_GPU_ALLOCATION_TRACE=str(OUT/'gpu.jsonl'), DATUM_PRIVATE_TEXT_TRACE=str(OUT/'private.jsonl'),
    DATUM_MEASUREMENT_SHUTDOWN_SOCKET=str(OUT/'observer.sock'))
cmd = [str(BINARY), '--project-root', str(PROJECT), '--initial-layout', 'single',
       '--window-size', '1280x800', '--visual-scale-factor', '1']
report['command'] = cmd
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
    reference = ROOT/f'docs/reviews/gui-performance/implementation/S5/resolver-cache-ownership/native/{host}.png'
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

def cycle(host, index, due):
    assert int(xd('getwindowfocus')) == main
    row = {'host': host, 'index': index, 'scheduled_ns': int(due*1e9), 'begin_sample': sample('cycle-start')}
    report['cycles'].append(row)
    save()
    xd('mousemove', '--window', main, 148 if host!='NEW' else 110, 16, 'click', 1)
    if host!='NEW':
        xd('key', '--delay', 20, 'Up', 'Right', *(['Down'] if host=='PROJECT' else []))
    time.sleep(.15)
    row['open_begin_sample'] = sample('open-begin')
    xd('key', 'Return')
    title = {'GLOBAL': 'Global Preferences', 'PROJECT': 'Project Preferences', 'NEW': 'New Project'}[host]
    wid = int(until(lambda: next((w for w in xd('search', '--all', '--onlyvisible', '--pid', p.pid, '--name', '^'+title).splitlines() if int(w)!=main), None)))
    until(lambda: int(xd('getwindowfocus')) == wid)
    row['window'] = wid
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
    assert time.monotonic() < due+6, ('cycle overran next scheduled cycle', index)
    save()

try:
    before = subprocess.check_output(['kscreen-doctor', '-o'], text=True)
    (OUT/'display-before.txt').write_text(before)
    plain = re.sub(r'\x1b\[[0-9;]*m', '', before)
    panel = next(block for block in plain.split('Output:') if 'eDP-1' in block)
    old_scale = re.search(r'Scale:\s*([0-9.]+)', panel).group(1)
    subprocess.run(['kscreen-doctor', 'output.eDP-1.scale.1'], check=True, capture_output=True)
    time.sleep(2)
    during = subprocess.check_output(['kscreen-doctor', '-o'], text=True)
    (OUT/'display-during.txt').write_text(during)
    group = Path('/sys/fs/cgroup/user.slice')/f'user-{os.getuid()}.slice'/f'user@{os.getuid()}.service'/('datum-dev-agent-endurance-'+uuid.uuid4().hex)
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
    until(lambda: 'frame present end' in log.read_text())
    report['initial_geometry'] = xd('getwindowgeometry', '--shell', main)
    assert 'WIDTH=1280' in report['initial_geometry'] and 'HEIGHT=800' in report['initial_geometry']
    report['warmup_start_ns'] = time.monotonic_ns()
    print('Native startup verified; five-minute warmup started', flush=True)
    wait_to(time.monotonic()+300, 'warmup')
    start = time.monotonic()
    report['workload_start_ns'] = int(start*1e9)
    for index in range(600):
        due = start + index*6
        wait_to(due, 'between-cycles')
        cycle(('GLOBAL', 'PROJECT', 'NEW')[index%3], index, due)
        if index%10 == 9:
            print(f'{index+1}/600 cycles complete; elapsed={time.monotonic()-start:.1f}s', flush=True)
    wait_to(start+3600, 'final-interval')
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
    print('FAILED: '+repr(error), flush=True)
    if p and p.poll() is None:
        for name in ('status', 'wchan', 'stack'):
            try:
                (OUT/('failure-proc-'+name+'.txt')).write_text(Path(f'/proc/{p.pid}/{name}').read_text())
            except OSError as capture_error:
                report.setdefault('failure_capture_errors', []).append(repr(capture_error))
finally:
    if p and p.poll() is None:
        p.terminate()
        try:
            p.wait(timeout=5)
        except subprocess.TimeoutExpired:
            p.kill()
            p.wait(timeout=5)
        report['forced_cleanup'] = True
    if group and group.exists():
        report['group_after_root_exit'] = group_counts()
        remaining = (group/'cgroup.procs').read_text().strip()
        report['remaining_before_cleanup'] = remaining
        if remaining:
            (group/'cgroup.kill').write_text('1')
            time.sleep(.5)
        report['group_after_cleanup'] = group_counts()
        if not (group/'cgroup.procs').read_text().strip():
            group.rmdir()
    if stream:
        stream.close()
    if listener:
        listener.close()
        (OUT/'observer.sock').unlink(missing_ok=True)
    if old_scale:
        result = subprocess.run(['kscreen-doctor', 'output.eDP-1.scale.'+old_scale], capture_output=True, text=True)
        report['display_restore_returncode'] = result.returncode
        (OUT/'display-after.txt').write_text(subprocess.check_output(['kscreen-doctor', '-o'], text=True))
    report['binary_unchanged'] = sha(BINARY) == EXPECTED
    report['model_file_sha256_final'] = sha(PROJECT/'board/board.json')
    save()
if report.get('error'):
    raise SystemExit(1)
