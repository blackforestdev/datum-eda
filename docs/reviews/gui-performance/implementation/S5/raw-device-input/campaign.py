"""Three fixed quiet clamp trials; no failed-run replacement or comparative claim."""
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile

ROOT = Path(subprocess.check_output(['git', 'rev-parse', '--show-toplevel'], text=True).strip())
sys.path.insert(0, str(ROOT/'scripts'))
import run_cargo_guarded as guard

raw_root = ROOT/'target/pm045-evidence-work/raw-device-input'
raw_root.mkdir(parents=True, exist_ok=True)
out = Path(tempfile.mkdtemp(prefix='campaign-', dir=raw_root))
print(out, flush=True)
env = os.environ.copy()
env['PM045_REPLAY_OUTPUT_ROOT'] = str(raw_root)
env['PM045_REPLAY_PROJECT'] = '/tmp/pm045-independent-replay-fixture/project'
results = []
policy = guard.load_policy()
with guard.acquire_lock(policy.lock_path, policy.lock_timeout_seconds):
    for trial in range(1, 4):
        with (out/f'trial-{trial}.log').open('w') as log:
            result = subprocess.run([sys.executable, str(Path(__file__).with_name('native_clamp.py'))], env=env, cwd=ROOT, stdout=log, stderr=subprocess.STDOUT)
        lines = (out/f'trial-{trial}.log').read_text().splitlines()
        row = {'trial':trial, 'returncode':result.returncode, 'raw':lines[0] if lines else None}
        results.append(row)
        (out/'result.json').write_text(json.dumps(results, indent=2)+'\n')
        print(json.dumps(row), flush=True)
        if result.returncode:
            break
raise SystemExit(0 if len(results)==3 and all(r['returncode']==0 for r in results) else 1)
