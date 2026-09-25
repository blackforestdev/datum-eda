import subprocess,json
from pathlib import Path
commands=[('negative-restored','cargo test -p datum-gui-render --features visual --lib terminal_last_layer_matches_invisible_later_passes --offline -- --ignored --test-threads=1'),('clippy','cargo clippy -p datum-gui-render -p datum-gui-app --all-targets --features datum-gui-render/visual --offline -- -D warnings'),('release','cargo build --release -p datum-gui-app --bin datum-gui --offline')]
results=[]
for name,cmd in commands:
 with open('/tmp/pm045-msaa-'+name+'.log','w') as log:rc=subprocess.run(['python3','scripts/run_cargo_guarded.py','--workload','proof','--',*cmd.split()],stdout=log,stderr=subprocess.STDOUT).returncode
 results.append({'name':name,'command':cmd,'returncode':rc});Path('/tmp/pm045-msaa-build-proof.json').write_text(json.dumps(results,indent=2)+'\n');print(name,rc,flush=True)
 if rc:raise SystemExit(rc)
