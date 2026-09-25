import json,hashlib,subprocess,tempfile
from pathlib import Path
out=Path(tempfile.mkdtemp(prefix='pm045-close-campaign-'));print(out,flush=True)
r={'purpose':'Diagnose missing WM_DELETE_WINDOW delivery, not qualification; stop first failure, maximum6nine-cycle runs','binary_sha256':hashlib.sha256(Path('target/release/datum-gui').read_bytes()).hexdigest(),'attempt_limit':6,'runs':[]}
(out/'declaration.json').write_text(json.dumps(r,indent=2)+'\n')
for i in range(1,7):
 log=out/f'run-{i}.log'
 with log.open('w') as f:p=subprocess.run(['python3','/tmp/pm045-close-diagnostic.py','off'],stdout=f,stderr=subprocess.STDOUT)
 path=log.read_text().splitlines()[0];r['runs'].append({'index':i,'path':path,'returncode':p.returncode});print(r['runs'][-1],flush=True);(out/'result.json').write_text(json.dumps(r,indent=2)+'\n')
 if p.returncode:break
