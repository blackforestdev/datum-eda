import os,pathlib,subprocess,time,re
root=pathlib.Path('/tmp/datum-resize-h2-stacks-v2');env=dict(os.environ,DATUM_RESIZE_IDLE_REPEATS='0',DATUM_MEASUREMENT_BOARD='/tmp/datum-gui-candidate-boundary-v3/DOA2526.kicad_pcb')
with open('/tmp/datum-resize-h2-stacks-v2-driver.log','w') as log:
 p=subprocess.Popen(['python3','/tmp/datum_resize_stacks_driver.py','--mode','main','--phases','width','--seconds','15','--trials','1','--output',str(root)],env=env,stdout=log,stderr=log)
 deadline=time.monotonic()+90
 while time.monotonic()<deadline and p.poll() is None:
  f=root/'datum-gui-last.log';s=f.read_text() if f.exists() else ''
  if s.count('resize apply')>40:break
  time.sleep(.2)
 else:raise RuntimeError('No active diagnostic resize run')
 pid=int(re.search(r'pid=(\d+)',s)[1])
 with (root/'stacks.log').open('w') as f:
  for n in range(12):
   f.write(f'\nSAMPLE {n}\n');f.flush()
   subprocess.run(['gdb','-q','-batch','-p',str(pid),'-ex','set pagination off','-ex','thread 1','-ex','bt 22','-ex','detach'],stdout=f,stderr=f,timeout=10)
   time.sleep(.1)
 print('driver exit',p.wait(timeout=90))
