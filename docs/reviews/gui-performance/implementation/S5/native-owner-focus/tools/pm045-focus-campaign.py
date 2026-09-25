import os,json,hashlib,subprocess,tempfile,time
from pathlib import Path
root=Path.cwd();out=Path(tempfile.mkdtemp(prefix='pm045-focus-campaign-'));print(out,flush=True)
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest();binaries={'baseline':root/'target/pm045-msaa-baseline/datum-gui','candidate':root/'target/release/datum-gui','GLOBAL':root/'target/release/datum-gui','PROJECT':root/'target/release/datum-gui','NEW':root/'target/release/datum-gui'}
assert sha(binaries['baseline'])=='c425e0893d6853e17c47534a09fcc7be89e74e996bfa7b4448bd9d3bf464d447';assert sha(binaries['candidate'])!='d0777db0135a322bf6a9d9c576fd4a34a6a663e77133276254fca4b0a4ca28d3'
source=json.loads(Path('/tmp/pm045-resource-campaign-jksuuoyg/declaration.json').read_text())['source_sha256'];source.update({p:sha(p) for p in ['crates/gui-app/src/owned_window_policy.rs','crates/gui-app/src/owned_window_policy/x11.rs']});r={'purpose':'Bounded production native X11 transient-owner correction. Fixed negative then positive per PROJECT/GLOBAL/NEW. Negative removes only WM_TRANSIENT_FOR from owned window. Focus-loss/helper activation and close should fail Main focus only with hint removed. Static compositor reference and normal drain required. No cap/full workload/Wayland/scale/endurance acceptance.','order':[[h,c,0] for h in ['PROJECT','GLOBAL','NEW'] for c in ['negative','positive']],'source_sha256':source,'binary_sha256':{k:sha(v) for k,v in binaries.items()},'input_driver_sha256':sha('/tmp/pm045-wheel-edges.py'),'native_driver_sha256':sha('/tmp/pm045-focus-native.py'),'runs':[]}
(out/'declaration.json').write_text(json.dumps(r,indent=2)+'\n');(out/'display-before.txt').write_text(subprocess.check_output(['kscreen-doctor','-o'],text=True));assert '1.05' in (out/'display-before.txt').read_text()
try:
 subprocess.run(['kscreen-doctor','output.eDP-1.scale.1'],check=True,stdout=subprocess.PIPE);time.sleep(2);(out/'display-during.txt').write_text(subprocess.check_output(['kscreen-doctor','-o'],text=True))
 for role,case,gpu in r['order']:
  i=1
  env=os.environ.copy();env.pop('LD_AUDIT',None);env.pop('PM045_X11_AUDIT_PATH',None);env.update(PM045_FOCUS_CASE=case,PM045_CONTROLS_HOST=role,PM045_POINTER_BINARY=str(binaries[role]),PM045_POINTER_ROLE=role,PM045_POINTER_GPU=str(gpu),PM045_CANDIDATE_SHA=r['binary_sha256'][role],PM045_RESOURCE_MODE='off',PM045_TRIAL=str(i),PM045_RESOURCE_DECLARATION=str(out/'declaration.json'))
  log=out/f'{role}-{case}.log'
  with log.open('w') as f:p=subprocess.run(['python3','/tmp/pm045-focus-native.py'],env=env,stdout=f,stderr=subprocess.STDOUT)
  path=log.read_text().splitlines()[0];r['runs'].append({'role':role,'case':case,'trial':i,'gpu':gpu,'path':path,'returncode':p.returncode});(out/'result.json').write_text(json.dumps(r,indent=2)+'\n');print(r['runs'][-1],flush=True)
  if p.returncode:break
finally:
 subprocess.run(['kscreen-doctor','output.eDP-1.scale.1.05'],check=True,stdout=subprocess.PIPE);(out/'display-after.txt').write_text(subprocess.check_output(['kscreen-doctor','-o'],text=True));r['display_restored']=True;(out/'result.json').write_text(json.dumps(r,indent=2)+'\n')
