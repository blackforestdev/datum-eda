import hashlib,json,os,subprocess,tempfile,time
from pathlib import Path
root=Path('/home/bfadmin/Documents/datum-eda')
out=Path(tempfile.mkdtemp(prefix='pm045-window-campaign-'));print(out,flush=True)
order=[('plain',1),('kernel',1),('kernel',2),('plain',2),('plain',3),('kernel',3)]
record={'status':'predeclared before execution','order':order,'warm_cycles_per_host':30,'hosts':['GLOBAL','PROJECT','NEW'],'first_use_cycles_per_host_separate':1,'warmup':'5seconds after initial Main focus and5seconds after first use of all3hosts','candidate':subprocess.check_output(['git','rev-parse','HEAD'],cwd=root,text=True).strip(),'binary_sha256':hashlib.sha256((root/'target/release/datum-gui').read_bytes()).hexdigest(),'driver_sha256':hashlib.sha256(Path('/tmp/pm045-window-batch.py').read_bytes()).hexdigest(),'observer_source_sha256':hashlib.sha256(Path('/tmp/pm045_fd_trace.c').read_bytes()).hexdigest(),'observer_binary_sha256':hashlib.sha256(Path('/tmp/pm045_fd_trace').read_bytes()).hexdigest(),'inference':'Three paired descriptive observer-on/off trials only. No seven-pair relative claim. Absolute CPU limits evaluated separately for every plain trial. Kernel/GPU duty not admitted without complete ledger and overhead review. No temporal-display, cold10start, full memory, endurance or independent acceptance.','failures':'Stop batch on structural/fixture/driver failure; retain every attempt. Numerical excess is retained and does not trigger selective resampling.','runs':[]}
(out/'declaration.json').write_text(json.dumps(record,indent=2)+'\n')
for mode,trial in order:
 env=os.environ.copy();env.update(PM045_MODE=mode,PM045_TRIAL=str(trial),PM045_CANDIDATE_SHA=record['binary_sha256'])
 logpath=out/f'{mode}-{trial}.log';begin=time.monotonic_ns()
 with logpath.open('w') as log:
  rc=subprocess.run(['python3','/tmp/pm045-window-batch.py'],cwd=root,env=env,stdout=log,stderr=subprocess.STDOUT).returncode
 text=logpath.read_text();print(mode,trial,rc,text[-700:],flush=True)
 location=next((line for line in text.splitlines() if line.startswith('/tmp/pm045-window-batch-')),None)
 result=json.loads((Path(location)/'result.json').read_text()) if location and (Path(location)/'result.json').exists() else {}
 record['runs'].append({'mode':mode,'trial':trial,'begin_ns':begin,'end_ns':time.monotonic_ns(),'returncode':rc,'path':location,'error':result.get('error')})
 (out/'result.json').write_text(json.dumps(record,indent=2)+'\n')
 if rc or result.get('error'):raise SystemExit(1)
record['status']='Six predeclared runs completed; analysis pending'
(out/'result.json').write_text(json.dumps(record,indent=2)+'\n')
