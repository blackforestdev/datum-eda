import subprocess,json
from pathlib import Path
commands=[('renderer','cargo test -p datum-gui-render --features visual --lib --offline -- --test-threads=1'),('observer','cargo test -p datum-gui-render --features visual --lib private_call_delivery --offline -- --ignored --test-threads=1'),('app','cargo test -p datum-gui-app --offline -- --test-threads=1'),('clippy','cargo clippy -p datum-gui-render -p datum-gui-app --all-targets --features datum-gui-render/visual --offline -- -D warnings'),('release','cargo build --release -p datum-gui-app --bin datum-gui --offline')]
results=[]
for name,cmd in commands:
 with open('/tmp/pm045-private-'+name+'.log','w') as f:rc=subprocess.run(['python3','scripts/run_cargo_guarded.py','--workload','proof','--',*cmd.split()],stdout=f,stderr=subprocess.STDOUT).returncode
 results.append({'name':name,'command':cmd,'returncode':rc});Path('/tmp/pm045-private-proof.json').write_text(json.dumps(results,indent=2)+'\n');print(name,rc,flush=True)
 if rc:raise SystemExit(rc)
