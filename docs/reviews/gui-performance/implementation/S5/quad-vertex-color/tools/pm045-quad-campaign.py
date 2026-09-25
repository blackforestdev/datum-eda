import os,json,hashlib,subprocess,tempfile,time
from pathlib import Path
root=Path.cwd();out=Path(tempfile.mkdtemp(prefix='pm045-quad-campaign-'));print(out,flush=True)
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest();binaries={'baseline':root/'target/pm045-quad-baseline/datum-gui','candidate':root/'target/release/datum-gui'}
assert sha(binaries['baseline'])=='b8c1fa1bf390d0dd768554b7b86548a6dc9b9777d8f6fdb5a1c7107f8e7e2b5e'
assert sha(binaries['candidate'])!=sha(binaries['baseline'])
source=json.loads(Path('/tmp/pm045-focus-campaign-zaw4cz7n/declaration.json').read_text())['source_sha256'];source.update({p:sha(root/p) for p in ['crates/gui-render/src/render/gpu_init.rs','crates/gui-render/src/render/gpu_data.rs']});r={'purpose':'QUAD CONVERSION DIAGNOSTIC: three fixed alternating matched baseline/candidate pairs, quiet with GPUtimestampsON, familyCPU, exactendpoint pixels, normaldrain. QuantizedXTest120Hzschedule has1280distinctinteger changes, not3600distinctpositions; no fullWPOINTER/nativeinput/caps/overhead or formal7pairSTAT acceptance. No selective replacement; stoponfailure. Candidate preserves8xMSAA, painter order, constantprimitivecolors and allresourcebounds.','order':[['baseline',1,1],['candidate',1,1],['candidate',2,1],['baseline',2,1],['baseline',3,1],['candidate',3,1]],'source_sha256':source,'binary_sha256':{k:sha(v) for k,v in binaries.items()},'input_driver_sha256':sha('/tmp/pm045-pointer-stream.py'),'native_driver_sha256':sha('/tmp/pm045-quad-native.py'),'runs':[]}
(out/'declaration.json').write_text(json.dumps(r,indent=2)+'\n');(out/'display-before.txt').write_text(subprocess.check_output(['kscreen-doctor','-o'],text=True));assert '1.05' in (out/'display-before.txt').read_text()
try:
 subprocess.run(['kscreen-doctor','output.eDP-1.scale.1'],check=True,stdout=subprocess.PIPE);time.sleep(2);(out/'display-during.txt').write_text(subprocess.check_output(['kscreen-doctor','-o'],text=True))
 for role,i,gpu in r['order']:
  env=os.environ.copy();env.pop('LD_AUDIT',None);env.pop('PM045_X11_AUDIT_PATH',None);env.update(PM045_LOG_VERBOSE='0',PM045_POINTER_BINARY=str(binaries[role]),PM045_POINTER_ROLE=role,PM045_POINTER_GPU=str(gpu),PM045_CANDIDATE_SHA=r['binary_sha256'][role],PM045_RESOURCE_MODE='off',PM045_TRIAL=str(i),PM045_RESOURCE_DECLARATION=str(out/'declaration.json'))
  log=out/f'{role}-{i}-gpu{gpu}.log'
  with log.open('w') as f:p=subprocess.run(['python3','/tmp/pm045-quad-native.py'],env=env,stdout=f,stderr=subprocess.STDOUT)
  path=log.read_text().splitlines()[0];r['runs'].append({'role':role,'trial':i,'gpu':gpu,'path':path,'returncode':p.returncode});(out/'result.json').write_text(json.dumps(r,indent=2)+'\n');print(r['runs'][-1],flush=True)
  if p.returncode:break
finally:
 subprocess.run(['kscreen-doctor','output.eDP-1.scale.1.05'],check=True,stdout=subprocess.PIPE);(out/'display-after.txt').write_text(subprocess.check_output(['kscreen-doctor','-o'],text=True));r['display_restored']=True;(out/'result.json').write_text(json.dumps(r,indent=2)+'\n')
