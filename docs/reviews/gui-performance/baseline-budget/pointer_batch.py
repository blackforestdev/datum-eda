"""Execute the frozen five-slot baseline packet once; never replace a failed slot."""
import hashlib
import json
import os
from pathlib import Path
import signal
import subprocess
import sys
import time

ROOT = Path(__file__).resolve().parents[4]
BASE = ROOT / 'docs/reviews/gui-performance/baseline-budget'
DECL = BASE / 'pointer-declaration.json'
STATE = ROOT / 'target/gui-baseline-budget/pointer-state.json'
d = json.loads(DECL.read_text())
assert len(d['runs']) == d['run_cap'] == 5
assert not STATE.exists(), 'preserve existing packet; no relaunch'
assert os.environ.get('DISPLAY') and Path(os.environ['XAUTHORITY']).is_file()
group_parent = Path(f'/sys/fs/cgroup/user.slice/user-{os.getuid()}.slice/user@{os.getuid()}.service')
assert os.access(group_parent, os.W_OK), 'owned cgroup unavailable'
for spec in [d['cli'], d['reference_png'], d['binaries']['candidate']]:
    assert hashlib.sha256(Path(spec['path']).read_bytes()).hexdigest() == spec['sha256']
assert hashlib.sha256((Path(d['project']['path'])/'board/board.json').read_bytes()).hexdigest() == d['project']['board_file_sha256']
for path, digest in d['method_sha256'].items():
    assert hashlib.sha256(Path(path).read_bytes()).hexdigest() == digest, path
STATE.parent.mkdir(parents=True, exist_ok=True)
state = {'status': 'sealed', 'declaration_sha256': hashlib.sha256(DECL.read_bytes()).hexdigest(), 'attempts': []}
def save():
    STATE.write_text(json.dumps(state, indent=2)+'\n')
save()
for index in range(5):
    state.update(status='running', reserved_index=index)
    save()
    started = time.monotonic()
    attempt = {'index': index, 'spec': d['runs'][index], 'started_unix_ns': time.time_ns()}
    state['attempts'].append(attempt)
    save()
    p = subprocess.Popen([sys.executable, str(BASE/'pointer_trial.py'), str(DECL), str(index)], cwd=ROOT, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True)
    first = p.stdout.readline().strip()
    print(first, flush=True)
    out = Path(first)
    attempt['output'] = first
    save()
    failure = None
    while p.poll() is None:
        if time.monotonic()-started > d['max_process_seconds']:
            failure = 'outer process deadline'
        if out.is_dir():
            size = sum(f.stat().st_size for f in out.rglob('*') if f.is_file())
            if size > d['max_run_evidence_bytes']:
                failure = 'run evidence capacity'
        total = sum(f.stat().st_size for a in state['attempts'] for f in Path(a['output']).rglob('*') if f.is_file())
        if total > d['max_packet_evidence_bytes']:
            failure = 'packet evidence capacity'
        if failure:
            p.send_signal(signal.SIGINT)
            try:
                p.wait(timeout=15)
            except subprocess.TimeoutExpired:
                p.kill()
                p.wait()
            break
        time.sleep(.25)
    rest = p.stdout.read()
    print(rest[-1800:], flush=True)
    attempt.update(returncode=p.returncode, elapsed_seconds=time.monotonic()-started, outer_failure=failure)
    if out.is_dir():
        (out/'launcher-output.txt').write_text(first+'\n'+rest)
    save()
    if p.returncode or failure:
        state['status'] = 'stopped_failure'
        save()
        sys.exit(1)
state['status'] = 'complete'
save()
