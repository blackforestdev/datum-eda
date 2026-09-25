import subprocess,json
from pathlib import Path
commands=[('renderer-serial', 'cargo test -p datum-gui-render --features visual --lib --offline -- --test-threads=1'), ('observer', 'cargo test -p datum-gui-render --features visual --lib gpu_history_ --offline -- --ignored --test-threads=1'), ('gpu', 'cargo test -p datum-gui-render --features visual --lib gpu_overlay_tests --offline -- --ignored --test-threads=1'), ('app', 'cargo test -p datum-gui-app --offline -- --test-threads=1'), ('clippy', 'cargo clippy -p datum-gui-render -p datum-gui-app --all-targets --features datum-gui-render/visual --offline -- -D warnings'), ('release', 'cargo build --release -p datum-gui-app --bin datum-gui --offline')]
results=[]
for name,cmd in commands[3:]:
 with open('/tmp/pm045-gpu-history-'+name+'.log','w') as log:
  rc=subprocess.run(['python3','scripts/run_cargo_guarded.py','--workload','proof','--',*cmd.split()],stdout=log,stderr=subprocess.STDOUT).returncode
 results.append({'name':name,'command':cmd,'returncode':rc})
 Path('/tmp/pm045-gpu-history-proof.json').write_text(json.dumps(results,indent=2)+'\n')
 print(name,rc,flush=True)
 if rc:raise SystemExit(rc)
