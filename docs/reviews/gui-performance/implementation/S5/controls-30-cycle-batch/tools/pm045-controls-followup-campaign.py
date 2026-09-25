import os,json,hashlib,subprocess,tempfile,time
from pathlib import Path
root=Path.cwd();out=Path(tempfile.mkdtemp(prefix='pm045-controls-followup-campaign-'));print(out,flush=True)
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest();binary=root/'target/release/datum-gui';expected='b8c1fa1bf390d0dd768554b7b86548a6dc9b9777d8f6fdb5a1c7107f8e7e2b5e';assert sha(binary)==expected
source=json.loads(Path('/tmp/pm045-focus-campaign-zaw4cz7n/declaration.json').read_text())['source_sha256']
for path,digest in source.items():assert sha(path)==digest,path
refs=Path('/tmp/pm045-controls-30-references');manifest=json.loads((refs/'manifest.json').read_text())
for path,item in manifest.items():assert sha(refs/path)==item['sha256'],path
first=json.loads(Path('/tmp/pm045-controls-30-campaign-c8fwtqnc/result.json').read_text());assert first['display_restored']
project=json.loads(Path('/tmp/pm045-controls-30-native-nwazawem/result.json').read_text());assert project['normal_exit_code']==0 and project['controls'][0]['completed_cycles']==30
jobs=[('GLOBAL','/tmp/pm045-controls-30-v2-native.py'),('PROJECT','/tmp/pm045-controls-clip-native.py')]
r={'purpose':'Retain completed Project30cycle workload. Correct Global repeated-announcement oracle and run fullGlobal30; supplementProject only30actualclipped-control/Search-overlap probes. No repeated completedProjectsequence. No performance resampling or fullS5acceptance.','candidate_commit':subprocess.check_output(['git','rev-parse','HEAD'],text=True).strip(),'order':jobs,'binary_sha256':expected,'source_sha256':source,'native_drivers':{p:sha(p) for _,p in jobs},'references':manifest,'visual_oracle':'Globalcycle1last-name uses initialreference withoutnotice; cycles2..30 use retainedcollapsed reference showing samebottomfocus withnotice. No notice mask. Other17actionimages matchfullclient; onlysearchfocuscrop210,42,960,100 becauseprecedingcycleoffsetdiffers. Projectclip supplement sameSearch-focusedimage for30overlapclicks.','clip_geometry':'Units8rows*86=688. Atlast-rowreveal firstrowtop32 regardlessnoticeheight, SingleChoicecontrol y46..76,width>=58 endingatright931 (900inside); viewportstarts100Project/148Global, so controlfullyclipped. Search54..89covers900,70. Nochoiceactivation/exactRGBunchangedrequired. Code/image derivation, notnative numerictelemetry.','failure_policy':'Stoponfirstfailure; allattemptsretained. No numericalresampling.','measurements':'Verbose/structuralnativeprofile, no CPU/GPU/memory caps,temporal,otherbackendscale,enduranceorindependentacceptance.','reuse_project':'/tmp/pm045-controls-30-native-nwazawem','prior_campaign':'/tmp/pm045-controls-30-campaign-c8fwtqnc','runs':[]}
(out/'declaration.json').write_text(json.dumps(r,indent=2)+'\n');(out/'display-before.txt').write_text(subprocess.check_output(['kscreen-doctor','-o'],text=True));assert '1.05' in (out/'display-before.txt').read_text()
try:
 subprocess.run(['kscreen-doctor','output.eDP-1.scale.1'],check=True,stdout=subprocess.PIPE);time.sleep(2);(out/'display-during.txt').write_text(subprocess.check_output(['kscreen-doctor','-o'],text=True))
 for host,driver in jobs:
  env=os.environ.copy();env.pop('LD_AUDIT',None);env.pop('PM045_X11_AUDIT_PATH',None);env.update(PM045_CONTROLS_HOST=host,PM045_CONTROLS_REFERENCE=str(refs),PM045_POINTER_BINARY=str(binary),PM045_POINTER_ROLE=host,PM045_POINTER_GPU='0',PM045_CANDIDATE_SHA=expected,PM045_RESOURCE_MODE='off',PM045_TRIAL='1',PM045_RESOURCE_DECLARATION=str(out/'declaration.json'))
  log=out/(host+'.log')
  with log.open('w') as f:p=subprocess.run(['python3',driver],env=env,stdout=f,stderr=subprocess.STDOUT)
  path=log.read_text().splitlines()[0];r['runs'].append({'host':host,'driver':driver,'path':path,'returncode':p.returncode});(out/'result.json').write_text(json.dumps(r,indent=2)+'\n');print(r['runs'][-1],flush=True)
  if p.returncode:break
finally:
 subprocess.run(['kscreen-doctor','output.eDP-1.scale.1.05'],check=True,stdout=subprocess.PIPE);(out/'display-after.txt').write_text(subprocess.check_output(['kscreen-doctor','-o'],text=True));r['display_restored']=True;(out/'result.json').write_text(json.dumps(r,indent=2)+'\n')
