import subprocess,json
from pathlib import Path
commands=[('observer', 'cargo test -p datum-gui-render --features visual --lib local_observation_preserves_budget_identity_without_retaining_gpu_resources --offline -- --ignored --test-threads=1'), ('gpu', 'cargo test -p datum-gui-render --features visual --lib gpu_overlay_tests --offline -- --ignored --test-threads=1'), ('app', 'cargo test -p datum-gui-app --offline -- --test-threads=1'), ('clippy', 'cargo clippy -p datum-gui-render -p datum-gui-app --all-targets --features datum-gui-render/visual --offline -- -D warnings'), ('release', 'cargo build --release -p datum-gui-app --bin datum-gui --offline')]
results=[]
for name,cmd in commands:
 with open('/tmp/pm045-resource-snapshot-'+name+'.log','w') as log:
  rc=subprocess.run(['python3','scripts/run_cargo_guarded.py','--workload','proof','--',*cmd.split()],stdout=log,stderr=subprocess.STDOUT).returncode
 results.append({'name':name,'command':cmd,'returncode':rc})
 Path('/tmp/pm045-resource-snapshot-proof.json').write_text(json.dumps(results,indent=2)+'\n')
 print(name,rc,flush=True)
 if rc:raise SystemExit(rc)
