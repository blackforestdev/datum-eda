import hashlib,json,os,subprocess,time
from pathlib import Path
out=Path('/tmp/pm045-window-campaign-o1yw7lsn');root=Path('/home/bfadmin/Documents/datum-eda')
record=json.loads((out/'result.json').read_text())
order=[('kernel',1),('kernel',2),('plain',2),('plain',3),('kernel',3)]
amendment={'reason':'Kernel1 stopped during startup before any workload cycle because NVIDIA DRM fdinfo lacks engine/client counters. Preserve that attempt. Collector now records opaque GPU descriptor lifetimes explicitly; any such observations remain unsupported, never zero/pass. No production/backend policy change. Keep completed plain1; execute only pending schedule. No numerical selection or new formal inference.','remaining_order':order,'observer_source_sha256':hashlib.sha256(Path('/tmp/pm045_fd_trace.c').read_bytes()).hexdigest(),'observer_binary_sha256':hashlib.sha256(Path('/tmp/pm045_fd_trace').read_bytes()).hexdigest(),'plain1_reuse':'Changes affect traced fd classification only; plain branch never invokes it. CPU resource candidate and off-mode path unchanged.'}
(out/'continuation-declaration.json').write_text(json.dumps(amendment,indent=2)+'\n')
for mode,trial in order:
 env=os.environ.copy();env.update(PM045_MODE=mode,PM045_TRIAL=str(trial),PM045_CANDIDATE_SHA=record['binary_sha256'])
 logpath=out/f'{mode}-{trial}-continued.log';begin=time.monotonic_ns()
 with logpath.open('w') as log:rc=subprocess.run(['python3','/tmp/pm045-window-batch.py'],cwd=root,env=env,stdout=log,stderr=subprocess.STDOUT).returncode
 text=logpath.read_text();print(mode,trial,rc,text[-700:],flush=True)
 location=next((line for line in text.splitlines() if line.startswith('/tmp/pm045-window-batch-')),None)
 result=json.loads((Path(location)/'result.json').read_text()) if location and (Path(location)/'result.json').exists() else {}
 record['runs'].append({'mode':mode,'trial':trial,'continuation':True,'begin_ns':begin,'end_ns':time.monotonic_ns(),'returncode':rc,'path':location,'error':result.get('error')})
 (out/'result.json').write_text(json.dumps(record,indent=2)+'\n')
 if rc or result.get('error'):raise SystemExit(1)
record['status']='Six completed scheduled runs plus retained unsupported startup attempt; analysis pending'
(out/'result.json').write_text(json.dumps(record,indent=2)+'\n')
