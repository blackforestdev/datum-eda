import json,hashlib,subprocess,tempfile,os,time
from pathlib import Path
root=Path.cwd();out=Path(tempfile.mkdtemp(prefix='pm045-cold-campaign-'));print(out,flush=True)
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest()
binaries={'baseline':root/'target/pm045-msaa-baseline/datum-gui','candidate':root/'target/release/datum-gui'}
assert sha(binaries['baseline'])=='c425e0893d6853e17c47534a09fcc7be89e74e996bfa7b4448bd9d3bf464d447';assert sha(binaries['candidate'])=='d0777db0135a322bf6a9d9c576fd4a34a6a663e77133276254fca4b0a4ca28d3'
order=[(role,i) for i in range(1,11) for role in (('baseline','candidate') if i%2 else ('candidate','baseline'))]
source=json.loads(Path('/tmp/pm045-resource-campaign-jksuuoyg/declaration.json').read_text())['source_sha256'];r={'purpose':'MET01 cold-process and auxiliary first-use distributions:10startsperbinary alternatingpairorder;stopstructuralfailure,no replacement. Physical60Hz1xX11; no auditing/timestamp/eventtraces. FD/kernelqueue/gdbfailurecaptureoutside measurements.','order':order,'binary_sha256':{k:sha(v) for k,v in binaries.items()},'engine_binary_sha256':sha(root/'target/release/datum-eda'),'source_sha256':source,'main_reference_sha256':sha('/tmp/pm045-idle-native-qhyk1ecq/MAIN-before.png'),'runs':[]}
(out/'declaration.json').write_text(json.dumps(r,indent=2)+'\n');(out/'display-before.txt').write_text(subprocess.check_output(['kscreen-doctor','-o'],text=True));assert '1.05' in (out/'display-before.txt').read_text()
try:
 subprocess.run(['kscreen-doctor','output.eDP-1.scale.1'],check=True,stdout=subprocess.PIPE);time.sleep(2);(out/'display-during.txt').write_text(subprocess.check_output(['kscreen-doctor','-o'],text=True))
 for role,i in order:
  subprocess.run(['xdotool','mousemove','1800','1000'],check=True)
  env=os.environ.copy();env.pop('LD_AUDIT',None);env.pop('PM045_X11_AUDIT_PATH',None);env.update(PM045_COLD_BINARY=str(binaries[role]),PM045_COLD_ROLE=role,PM045_CANDIDATE_SHA=r['binary_sha256'][role],PM045_RESOURCE_MODE='off',PM045_TRIAL=str(i),PM045_RESOURCE_DECLARATION=str(out/'declaration.json'))
  log=out/f'{role}-{i}.log'
  with log.open('w') as f:p=subprocess.run(['python3','/tmp/pm045-cold-native.py'],env=env,stdout=f,stderr=subprocess.STDOUT)
  path=log.read_text().splitlines()[0];r['runs'].append({'role':role,'trial':i,'path':path,'returncode':p.returncode});(out/'result.json').write_text(json.dumps(r,indent=2)+'\n');print(r['runs'][-1],flush=True)
  if p.returncode:break
finally:
 subprocess.run(['kscreen-doctor','output.eDP-1.scale.1.05'],check=True,stdout=subprocess.PIPE);(out/'display-after.txt').write_text(subprocess.check_output(['kscreen-doctor','-o'],text=True));r['display_restored']=True;(out/'result.json').write_text(json.dumps(r,indent=2)+'\n')
