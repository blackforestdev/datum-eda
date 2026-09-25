import os,json,subprocess,hashlib
from pathlib import Path
out=Path('/tmp/pm045-private-campaign-_9v5nd5k');r=json.loads((out/'result.json').read_text())
order=[('baseline-off','off',str(Path.cwd()/'target/pm045-msaa-baseline/datum-gui')),('overflow','overflow',None),('existing','existing',None),('off-replay','off',None)]
(out/'continuation-declaration.json').write_text(json.dumps({'reason':'Disabled control completed9 exact-pixel cycles but final observer connection timed out; no close-request log before owned cleanup. Preserve rejected run; source audit finds disabled observer returns before lock/output. Compare identical driver on existing pre-observer baseline, finish remaining negatives, and one declared candidate off replay. No numerical resampling/overhead claim.','order':order,'driver_sha256':hashlib.sha256(Path('/tmp/pm045-private-native.py').read_bytes()).hexdigest()},indent=2)+'\n')
for label,mode,binary in order:
 env=os.environ.copy()
 if binary:env['PM045_PRIVATE_BINARY']=binary
 with (out/(label+'.log')).open('w') as f:rc=subprocess.run(['python3','/tmp/pm045-private-native.py',mode],env=env,stdout=f,stderr=subprocess.STDOUT).returncode
 text=(out/(label+'.log')).read_text();location=next((x for x in text.splitlines() if x.startswith('/tmp/pm045-private-native-')),None);r['runs'].append({'mode':label,'returncode':rc,'path':location});(out/'result.json').write_text(json.dumps(r,indent=2)+'\n');print(label,rc,location,flush=True)
 if rc and label in ('overflow','existing'):raise SystemExit(rc)
r['status']='Bounded declared continuation completed; all attempts retained, analysis pending';(out/'result.json').write_text(json.dumps(r,indent=2)+'\n')
