from pathlib import Path
import subprocess,json,tempfile,hashlib
root=Path.cwd();out=Path(tempfile.mkdtemp(prefix='pm045-gpu-history-campaign-'));print(out,flush=True)
paths=['crates/gui-render/src/lib.rs','crates/gui-render/src/cpu_alloc.rs','crates/gui-render/src/text_gpu/budget.rs','crates/gui-render/src/text_gpu/lifetime.rs','crates/gui-render/src/text_gpu/lifetime_observation.rs','crates/gui-render/src/text_gpu/lifetime_observation_tests.rs','crates/gui-app/src/native_gpu_allocation_measurements.rs','crates/gui-app/src/app_shell.rs','crates/gui-app/src/app_native_events.rs']
r={'order':['on','off','overflow','existing'],'native_cycles_each_success':9,'source_sha256':{p:hashlib.sha256((root/p).read_bytes()).hexdigest() for p in paths},'binary_sha256':hashlib.sha256((root/'target/release/datum-gui').read_bytes()).hexdigest(),'status':'Declared before execution; bounded observer conformance only; no resource budget/overhead/endurance qualification','runs':[]}
def fixture_hash():
 model=json.loads(Path('/tmp/pm045-admission-hqugp169/project/board/board.json').read_text());model.pop('uuid',None)
 return hashlib.sha256(json.dumps(model,sort_keys=True,separators=(',',':')).encode()).hexdigest()
assert fixture_hash()=='33e62de1c1da2020f0608444a4802eac23fb97a9f56cc8cf87c844b6077499ed'
r['normalized_fixture_sha256']=fixture_hash()
(out/'declaration.json').write_text(json.dumps(r,indent=2)+'\n')
for mode in r['order']:
 with (out/(mode+'.log')).open('w') as log:
  rc=subprocess.run(['python3','/tmp/pm045-gpu-history-native.py',mode],stdout=log,stderr=subprocess.STDOUT).returncode
 text=(out/(mode+'.log')).read_text();location=next((x for x in text.splitlines() if x.startswith('/tmp/pm045-gpu-history-native-')),None)
 r['runs'].append({'mode':mode,'returncode':rc,'path':location});(out/'result.json').write_text(json.dumps(r,indent=2)+'\n');print(mode,rc,location,flush=True)
 if rc:raise SystemExit(rc)
 if mode=='on':
  with (out/'analysis.log').open('w') as log:
   rc=subprocess.run(['python3','/tmp/pm045-gpu-history-analyze.py',location],stdout=log,stderr=subprocess.STDOUT).returncode
  if rc:raise SystemExit(rc)
  (out/'analysis.json').write_bytes((Path(location)/'analysis.json').read_bytes())
r['status']='Four declared native modes completed; raw data retained';(out/'result.json').write_text(json.dumps(r,indent=2)+'\n')

assert fixture_hash()==r["normalized_fixture_sha256"]
