import os,json,subprocess,tempfile,hashlib,time
from pathlib import Path
root=Path.cwd();out=Path(tempfile.mkdtemp(prefix='pm045-idle-campaign-'));print(out,flush=True)
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
binaries={'baseline':root/'target/pm045-msaa-baseline/datum-gui','candidate':root/'target/release/datum-gui'}
assert sha(binaries['baseline'])=='c425e0893d6853e17c47534a09fcc7be89e74e996bfa7b4448bd9d3bf464d447'
assert sha(binaries['candidate'])=='d0777db0135a322bf6a9d9c576fd4a34a6a663e77133276254fca4b0a4ca28d3'
source=json.loads(Path('/tmp/pm045-resource-campaign-jksuuoyg/declaration.json').read_text())['source_sha256']
r={'purpose':'W-IDLE Main/Global/Project/New, each60secondsafter5secondwarmup,three independent baseline/candidate pairs. X11 at actual60Hz1x. Not complete backend/scale/GPU/negative-control qualification.','order':[['baseline',1],['candidate',1],['candidate',2],['baseline',2],['baseline',3],['candidate',3]],'binary_sha256':{k:sha(v) for k,v in binaries.items()},'source_sha256':source,'candidate_commit':subprocess.check_output(['git','rev-parse','HEAD'],text=True).strip(),'baseline_provenance':'47eefbc1 release c425 retained at final-msaa-resolve baseline; same release visual-profile build provenance in existing evidence','runs':[],'declaration_ns':time.time_ns()}
for p,h in source.items():assert sha(Path(p))==h,p
(out/'declaration.json').write_text(json.dumps(r,indent=2)+'\n')
(out/'display-before.txt').write_text(subprocess.check_output(['kscreen-doctor','-o'],text=True))
assert '1.05' in (out/'display-before.txt').read_text()
try:
 subprocess.run(['kscreen-doctor','output.eDP-1.scale.1'],check=True,stdout=subprocess.PIPE,stderr=subprocess.STDOUT);time.sleep(2)
 display=subprocess.check_output(['kscreen-doctor','-o'],text=True);(out/'display-during.txt').write_text(display)
 assert '1920x1080@60*' in display and 'Scale:' in display
 for role,trial in r['order']:
  env=os.environ.copy();env.update(PM045_IDLE_BINARY=str(binaries[role]),PM045_IDLE_ROLE=role,PM045_CANDIDATE_SHA=r['binary_sha256'][role],PM045_RESOURCE_MODE='off',PM045_TRIAL=str(trial),PM045_RESOURCE_DECLARATION=str(out/'declaration.json'))
  log=out/f'{role}-{trial}.log'
  with log.open('w') as f:p=subprocess.run(['python3','/tmp/pm045-idle-native.py'],env=env,stdout=f,stderr=subprocess.STDOUT)
  path=log.read_text().splitlines()[0];r['runs'].append({'role':role,'trial':trial,'path':path,'returncode':p.returncode});(out/'result.json').write_text(json.dumps(r,indent=2)+'\n');print(r['runs'][-1],flush=True)
  if p.returncode:break
finally:
 subprocess.run(['kscreen-doctor','output.eDP-1.scale.1.05'],check=True,stdout=subprocess.PIPE,stderr=subprocess.STDOUT)
 (out/'display-after.txt').write_text(subprocess.check_output(['kscreen-doctor','-o'],text=True))
 r['display_restored']=True;(out/'result.json').write_text(json.dumps(r,indent=2)+'\n')
