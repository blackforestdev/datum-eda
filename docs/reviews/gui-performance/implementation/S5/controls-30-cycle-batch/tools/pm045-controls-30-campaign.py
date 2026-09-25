import os,json,hashlib,subprocess,tempfile,time
from pathlib import Path
root=Path.cwd();out=Path(tempfile.mkdtemp(prefix='pm045-controls-30-campaign-'));print(out,flush=True)
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest()
binary=root/'target/release/datum-gui';expected='b8c1fa1bf390d0dd768554b7b86548a6dc9b9777d8f6fdb5a1c7107f8e7e2b5e';assert sha(binary)==expected
source=json.loads(Path('/tmp/pm045-focus-campaign-zaw4cz7n/declaration.json').read_text())['source_sha256']
for path,digest in source.items():assert sha(path)==digest,path
refs=Path('/tmp/pm045-controls-30-references');manifest=json.loads((refs/'manifest.json').read_text())
for path,item in manifest.items():assert sha(refs/path)==item['sha256'],path
r={'purpose':'W-CONTROLS30cycles each Preferences host, X11/1x, after5second warmup. Current b8c1fa1b with fixed native ownership. Same window repeated cycles, seeded launch for eachhost. No performance/resource/other backend-scale acceptance.','candidate_commit':subprocess.check_output(['git','rev-parse','HEAD'],text=True).strip(),'order':['PROJECT','GLOBAL'],'cycles_per_host':30,'binary_sha256':expected,'source_sha256':source,'native_driver_sha256':sha('/tmp/pm045-controls-30-native.py'),'references':manifest,'visual_oracle':'Every action compared to previously retained compositor pilot reference. Only Global notice interior x211..947,y101..145 masked because announcement text carries across cycles; search-focus compares pinnedsearch rect210,42,960,100 because prior cycle leaves different scroll offset. All other action images fullclient. Exact pixels establish endpoint agreement, not physical latency, numerical scroll state or complete hidden-control oracle. Added pinnedclip probes x900,y70Global/y7Project require unchangedimage. They sample pinned input with clipped content; do not prove exact overlap with every hidden control.','failure_policy':'Stop on first structural/visual/input failure; retain every attempted cycle and all raw evidence. No selective replacement.','measurements':'Verbose native receipts; cgroupCPU diagnostic only includes input wait; compositor readback peraction. No GPU/private/resource instrumentation. No caps or observer-overhead inference.','runs':[]}
(out/'declaration.json').write_text(json.dumps(r,indent=2)+'\n')
(out/'display-before.txt').write_text(subprocess.check_output(['kscreen-doctor','-o'],text=True));assert '1.05' in (out/'display-before.txt').read_text()
try:
 subprocess.run(['kscreen-doctor','output.eDP-1.scale.1'],check=True,stdout=subprocess.PIPE);time.sleep(2);(out/'display-during.txt').write_text(subprocess.check_output(['kscreen-doctor','-o'],text=True))
 for host in r['order']:
  env=os.environ.copy();env.pop('LD_AUDIT',None);env.pop('PM045_X11_AUDIT_PATH',None);env.update(PM045_CONTROLS_HOST=host,PM045_CONTROLS_REFERENCE=str(refs),PM045_POINTER_BINARY=str(binary),PM045_POINTER_ROLE=host,PM045_POINTER_GPU='0',PM045_CANDIDATE_SHA=expected,PM045_RESOURCE_MODE='off',PM045_TRIAL='1',PM045_RESOURCE_DECLARATION=str(out/'declaration.json'))
  log=out/(host+'.log')
  with log.open('w') as f:p=subprocess.run(['python3','/tmp/pm045-controls-30-native.py'],env=env,stdout=f,stderr=subprocess.STDOUT)
  path=log.read_text().splitlines()[0];r['runs'].append({'host':host,'path':path,'returncode':p.returncode});(out/'result.json').write_text(json.dumps(r,indent=2)+'\n');print(r['runs'][-1],flush=True)
  if p.returncode:break
finally:
 subprocess.run(['kscreen-doctor','output.eDP-1.scale.1.05'],check=True,stdout=subprocess.PIPE);(out/'display-after.txt').write_text(subprocess.check_output(['kscreen-doctor','-o'],text=True));r['display_restored']=True;(out/'result.json').write_text(json.dumps(r,indent=2)+'\n')
