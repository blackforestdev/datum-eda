import os,json,subprocess,tempfile,struct,hashlib
from pathlib import Path
out=Path(tempfile.mkdtemp(prefix='pm045-cpu-calibration-'));print(out,flush=True)
results=[]
for i,mode in enumerate([0,1,1,0,0,1,2]):
 env=os.environ.copy();env.pop('LD_PRELOAD',None)
 path=out/(str(i)+'.bin')
 if mode:env.update(LD_PRELOAD='/tmp/pm045_cpu_sampler.so',PM045_CPU_SAMPLE_PATH=str(path),PM045_CPU_SAMPLE_EXE='/tmp/pm045_cpu_calibrate')
 r=subprocess.run(['/tmp/pm045_cpu_calibrate',str(mode)],env=env,capture_output=True,text=True)
 row={'index':i,'mode':mode,'returncode':r.returncode,'stdout':r.stdout,'stderr':r.stderr}
 if r.returncode==0:row['measurement']=json.loads(r.stdout)
 if path.exists():
  data=path.read_bytes();h=struct.unpack_from('11Q',data);assert h[0]==0x504d303435435055 and len(data)==88+24*h[3]
  samples=[struct.unpack_from('QQq',data,88+24*j) for j in range(h[3])]
  row['samples']={'delivered':h[3],'dropped':h[4],'cpu_ns':h[6]-h[5],'period_ns':h[9],'overruns':sum(x[2] for x in samples),'raw_path':str(path),'max_overrun':max(x[2] for x in samples),'weighted_coverage':(h[3]+sum(x[2] for x in samples))*h[9]/(h[6]-h[5])}
 results.append(row);print(row,flush=True)
(out/'result.json').write_text(json.dumps({'results':results,'sources':{p:hashlib.sha256(Path(p).read_bytes()).hexdigest() for p in ['/tmp/pm045_cpu_sampler.c','/tmp/pm045_cpu_calibrate.c']},'limit':'Three pairs descriptive calibration only; synthetic work cannot establish native overhead or universal profiling accuracy. Blocked-signal negative control must show missing distinct samples/overruns, not invent repeated program counters.'},indent=2)+'\n')
