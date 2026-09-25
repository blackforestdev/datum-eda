import importlib.util,subprocess,json
from pathlib import Path
spec=importlib.util.spec_from_file_location('conformance','scripts/check_gui_conformance.py')
m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)
results=[]
for name,cmd in m.GATES[7:]:
 print('START',name,flush=True)
 rc=subprocess.run(cmd).returncode
 results.append({'name':name,'command':cmd,'returncode':rc})
 print('END',name,rc,flush=True)
rc=subprocess.run(['bash','/tmp/pm045-read-cache-drift-remainder.sh']).returncode
results.append({'name':'Remaining drift commands after failed conformance aggregate','returncode':rc})
Path('/tmp/pm045-cache-owner-remaining-checks.json').write_text(json.dumps(results,indent=2)+'\n')
raise SystemExit(any(r['returncode'] for r in results))
