import os,json,hashlib,subprocess,tempfile,time
from pathlib import Path
root=Path('/home/bfadmin/Documents/datum-eda');out=Path(tempfile.mkdtemp(prefix='pm045-resource-campaign-'));print(out,flush=True)
paths=subprocess.check_output(['git','diff','--name-only'],cwd=root,text=True).splitlines()+subprocess.check_output(['git','ls-files','--others','--exclude-standard'],cwd=root,text=True).splitlines()
paths+=['crates/gui-render/src/render/frame_preparation.rs','crates/gui-render/src/render/preferences_scene.rs','crates/gui-render/src/render/new_project_scene.rs','crates/gui-render/src/render/measurement_owner.rs']
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
binary=root/'target/release/datum-gui'
r={'candidate_commit':subprocess.check_output(['git','rev-parse','HEAD'],cwd=root,text=True).strip(),'binary_sha256':sha(binary),'source_sha256':{p:sha(root/p) for p in sorted(set(paths))},'order':[['on',1],['off',1],['off',2],['on',2],['on',3],['off',3]],'controls':['overflow','existing'],'runs':[],'declaration_ns':time.time_ns(),'qualification_pass':False}
(out/'declaration.json').write_text(json.dumps(r,indent=2)+'\n')
env=os.environ.copy();env.update(PM045_CANDIDATE_SHA=r['binary_sha256'],PM045_RESOURCE_DECLARATION=str(out/'declaration.json'))
def save():(out/'result.json').write_text(json.dumps(r,indent=2)+'\n')
for mode,trial in r['order']:
 env.update(PM045_RESOURCE_MODE=mode,PM045_TRIAL=str(trial));log=out/f'{mode}-{trial}.log'
 with log.open('w') as f:p=subprocess.run(['python3','/tmp/pm045-resource-windows.py'],env=env,cwd=root,stdout=f,stderr=subprocess.STDOUT)
 lines=log.read_text().splitlines();path=Path(lines[0]);item={'mode':mode,'trial':trial,'path':str(path),'returncode':p.returncode};r['runs'].append(item);save();print(item,flush=True)
 assert p.returncode==0,log
 if mode=='on':
  with (out/f'{mode}-{trial}-analysis.log').open('w') as f:a=subprocess.run(['python3','/tmp/pm045-resource-snapshot-analyze.py',str(path)],cwd=root,stdout=f,stderr=subprocess.STDOUT)
  item['analysis_returncode']=a.returncode;save();assert a.returncode==0,('analysis failure',path)
for mode in r['controls']:
 log=out/f'{mode}.log'
 with log.open('w') as f:p=subprocess.run(['python3','/tmp/pm045-resource-negative.py',mode],env=env,cwd=root,stdout=f,stderr=subprocess.STDOUT)
 path=Path(log.read_text().splitlines()[0]);r['runs'].append({'mode':mode,'path':str(path),'returncode':p.returncode});save();print(mode,p.returncode,flush=True);assert p.returncode==0,log
assert sha(binary)==r['binary_sha256']
for p,h in r['source_sha256'].items():assert sha(root/p)==h,p
r['complete']=True;save()
