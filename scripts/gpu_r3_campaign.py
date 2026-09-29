#!/usr/bin/env python3
"""Spend exactly the next declared r3 pointer trial; a stopped campaign stays stopped."""
import fcntl
import hashlib
import json
from pathlib import Path
import subprocess
import sys
import time

ROOT = Path(__file__).resolve().parents[1]
DIRECTORY = ROOT / 'target/gpu-regional-r4-native-correction'
STATE = DIRECTORY / 'campaign-state.json'
ORDER = [('candidate','gpu',1),('baseline','gpu',1),('baseline','quiet',1),('candidate','quiet',1),
         ('baseline','gpu',2),('candidate','gpu',2),('candidate','quiet',2),('baseline','quiet',2),
         ('candidate','gpu',3),('baseline','gpu',3),('baseline','quiet',3),('candidate','quiet',3)]


def next_index(state, declaration, digest):
    assert declaration['run_cap'] == 12 and len(declaration['runs']) == 12
    assert [(r['role'],r['mode'],r['pair']) for r in declaration['runs']] == ORDER
    assert state['declaration_sha256'] == digest, 'declaration changed'
    assert state['status'] == 'ready', 'campaign is stopped, running or complete'
    index = len(state['results'])
    assert index < 12, 'pointer run cap exhausted'
    assert all(row['status'] == 'valid_descriptive_run' for row in state['results'])
    return index


def main():
    declaration_path = DIRECTORY / 'declaration.json'
    payload = declaration_path.read_bytes()
    declaration = json.loads(payload)
    digest = hashlib.sha256(payload).hexdigest()
    with (DIRECTORY / 'campaign.lock').open('a') as lock:
        fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
        if not STATE.exists():
            with STATE.open('x') as stream:
                json.dump(dict(status='ready', declaration_sha256=digest, results=[]), stream)
        state = json.loads(STATE.read_text())
        index = next_index(state, declaration, digest)
        state.update(status='running', reserved_index=index, reserved_ns=time.time_ns())
        STATE.write_text(json.dumps(state, indent=2)+'\n')
        # A process interruption or missing receipt leaves running/stopped state;
        # neither another directory nor unspent quota authorizes a replacement.
        launch = DIRECTORY / f'run-{index}-launch.log'
        timeout = False
        try:
            with launch.open('x') as log:
                completed = subprocess.run([sys.executable, str(ROOT/'scripts/gpu_r3_native_trial.py'),
                                        str(declaration_path), str(index)], stdout=log,
                                           stderr=subprocess.STDOUT, timeout=150)
        except subprocess.TimeoutExpired:
            timeout = True
        lines = launch.read_text().splitlines()
        receipt = Path(lines[0])/'result.json' if lines else None
        result = json.loads(receipt.read_text()) if receipt and receipt.is_file() else {'status':'invalid'}
        if timeout:
            group=Path(result.get('owned_cgroup','/nonexistent'))
            expected=Path('/sys/fs/cgroup/user.slice')/f'user-{__import__("os").getuid()}.slice'/f'user@{__import__("os").getuid()}.service'
            if group.parent == expected and group.name.startswith('datum-gpu-r3-') and group.is_dir():
                (group/'cgroup.kill').write_text('1')
            result['status']='invalid'
        exit_code = 124 if timeout else completed.returncode
        state['results'].append(dict(index=index, status=result['status'], exit_code=exit_code,
                                     receipt=str(receipt)))
        valid = exit_code == 0 and result['status'] == 'valid_descriptive_run'
        state['status'] = ('complete' if len(state['results']) == 12 else 'ready') if valid else 'stopped'
        STATE.write_text(json.dumps(state,indent=2)+'\n')
        print(json.dumps(state['results'][-1]))
        return 0 if valid else 1


if __name__ == '__main__':
    raise SystemExit(main())
