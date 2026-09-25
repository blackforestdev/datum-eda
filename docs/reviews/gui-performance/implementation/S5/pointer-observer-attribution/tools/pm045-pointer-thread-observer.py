import os,json,time,subprocess,sys
from pathlib import Path
pid=int(sys.argv[1]);out=Path(sys.argv[2]);rows=[];start=time.monotonic();sampled=False
while time.monotonic()-start<31 and Path(f'/proc/{pid}').exists():
 row={'monotonic_ns':time.monotonic_ns(),'threads':[]}
 for p in Path(f'/proc/{pid}/task').iterdir():
  try:
   s=(p/'stat').read_text();name=s[s.index('(')+1:s.rindex(')')];v=s[s.rindex(')')+2:].split();row['threads'].append({'tid':int(p.name),'name':name,'state':v[0],'user_ticks':int(v[11]),'system_ticks':int(v[12]),'start_ticks':int(v[19])})
  except (OSError,ValueError):pass
 rows.append(row)
 if not sampled and time.monotonic()-start>=15:
  sampled=True;begin=time.monotonic_ns();r=subprocess.run(['gdb','-q','-n','-batch','-ex','set pagination off','-ex','thread apply all bt 18','-p',str(pid)],capture_output=True,text=True,timeout=15);(out/'thread-gdb.txt').write_text(r.stdout+r.stderr);(out/'thread-gdb.json').write_text(json.dumps({'begin_ns':begin,'end_ns':time.monotonic_ns(),'returncode':r.returncode})+'\n')
 time.sleep(.1)
(out/'thread-cpu.json').write_text(json.dumps({'clock_ticks_per_second':os.sysconf('SC_CLK_TCK'),'pid':pid,'samples':rows},indent=2)+'\n')
