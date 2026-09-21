import os,subprocess,time,json,hashlib
from pathlib import Path
root=Path('/home/bfadmin/Documents/datum-eda');binary=root/'target/release/datum-gui';case=Path('/tmp/pm045-readiness-x11/MAIN');log=case/'interaction.log';log.write_text('')
env=os.environ.copy()
for k in list(env):
 if k.startswith('DATUM_DIAGNOSTIC_') or k.startswith('DATUM_GPU_DIAGNOSTIC_'):env.pop(k)
env.pop('WAYLAND_DISPLAY',None)
env.update(DATUM_GPU_MEASUREMENTS='0',DATUM_GUI_VERBOSE_LOG='1',DATUM_GUI_LOG=str(log),WINIT_UNIX_BACKEND='x11',XDG_CONFIG_HOME=str(case/'config'),XDG_CACHE_HOME=str(case/'cache'),TMPDIR=str(case))
cmd=[str(binary),'--board',str(case/'board.kicad_pcb'),'--window-size','1280x800','--interaction-smoke']
with (case/'interaction-stderr.log').open('w') as stream:
 p=subprocess.Popen(cmd,cwd=root,env=env,stdout=stream,stderr=stream)
 try:
  deadline=time.monotonic()+30
  while time.monotonic()<deadline:
   data=log.read_text()
   if 'redraw handler end' in data or p.poll() is not None:break
   time.sleep(.1)
  ok=p.poll() is None and 'redraw handler end' in data and data.count('render acquire end')>=3
  Path('/tmp/pm045-queue-interaction.json').write_text(json.dumps(dict(passed=ok,command=cmd,binary_sha256=hashlib.sha256(binary.read_bytes()).hexdigest(),acquisitions=data.count('render acquire end'),unexpected_exit=p.poll()),indent=2)+'\n')
  print('interaction smoke',ok,flush=True)
  if not ok:raise SystemExit(1)
 finally:
  p.terminate()
  try:p.wait(timeout=5)
  except subprocess.TimeoutExpired:p.kill();p.wait()
