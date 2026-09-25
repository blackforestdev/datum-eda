import subprocess
from pathlib import Path
p=Path('/tmp/pm045-cache-owner-drift.log')
with p.open('w') as log:
 rc=subprocess.run(['bash','scripts/run_drift_gates.sh'],stdout=log,stderr=subprocess.STDOUT).returncode
print('Original drift result',rc,flush=True)
if rc and 'menu_model_csv.py' in p.read_text()[-1000:]:
 with open('/tmp/pm045-cache-owner-drift-remainder.log','w') as log:
  rc=subprocess.run(['python3','/tmp/pm045-cache-owner-remaining-checks.py'],stdout=log,stderr=subprocess.STDOUT).returncode
 print('Unexecuted remainder result',rc,flush=True)
else:
 print('No continuation selected; inspect original result',flush=True)
