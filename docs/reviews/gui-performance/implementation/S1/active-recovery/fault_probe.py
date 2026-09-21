import os,subprocess,time,json,hashlib
from pathlib import Path
root=Path('/home/bfadmin/Documents/datum-eda');binary=root/'target/release/datum-gui';base=Path('/tmp/pm045-readiness-x11');out=Path('/tmp/pm045-recovery-fault');out.mkdir(exist_ok=True)
report={'binary_sha256':hashlib.sha256(binary.read_bytes()).hexdigest(),'runs':[]}
for host,flag in [('MAIN',None),('GLOBAL','--open-global-preferences'),('PROJECT','--open-project-preferences'),('NEW','--open-new-project')]:
 case=base/host;log=out/(host+'-lost.log');log.write_text('');env=os.environ.copy()
 for k in list(env):
  if k.startswith('DATUM_DIAGNOSTIC_') or k.startswith('DATUM_GPU_DIAGNOSTIC_'):env.pop(k)
 env.pop('WAYLAND_DISPLAY',None);env.update(DATUM_GPU_MEASUREMENTS='0',DATUM_DIAGNOSTIC_SURFACE_FAULT='lost-once',DATUM_GUI_VERBOSE_LOG='1',DATUM_GUI_LOG=str(log),WINIT_UNIX_BACKEND='x11',XDG_CONFIG_HOME=str(case/'config'),XDG_CACHE_HOME=str(case/'cache'),TMPDIR=str(case))
 cmd=[str(binary),'--board',str(case/'board.kicad_pcb'),'--window-size','1280x800','--resize-torture-smoke']+([flag] if flag else [])
 with (out/(host+'-stderr.log')).open('w') as stream:
  p=subprocess.Popen(cmd,cwd=root,env=env,stdout=stream,stderr=stream)
  try:
   deadline=time.monotonic()+30
   while time.monotonic()<deadline:
    data=log.read_text()
    if 'native resize smoke end' in data or p.poll() is not None:break
    time.sleep(.1)
   count=data.count('kind=LostOnce');ok=p.poll() is None and 'native resize smoke end' in data and count==(1 if host=='MAIN' else 2)
   report['runs'].append(dict(host=host,command=cmd,injected=count,passed=ok));print(host,'lost',ok,flush=True)
   (out/'result.json').write_text(json.dumps(report,indent=2)+'\n')
   if not ok:raise SystemExit(1)
  finally:
   p.terminate()
   try:p.wait(timeout=5)
   except subprocess.TimeoutExpired:p.kill();p.wait()
# One main-host persistent fault verifies exhaustion and actual native Retry input.
case=base/'MAIN';log=out/'timeout.log';log.write_text('');env.update(DATUM_DIAGNOSTIC_SURFACE_FAULT='timeout',DATUM_GUI_LOG=str(log),XDG_CONFIG_HOME=str(case/'config'),XDG_CACHE_HOME=str(case/'cache'),TMPDIR=str(case))
cmd=[str(binary),'--board',str(case/'board.kicad_pcb'),'--window-size','1280x800']
with (out/'timeout-stderr.log').open('w') as stream:
 p=subprocess.Popen(cmd,cwd=root,env=env,stdout=stream,stderr=stream)
 try:
  deadline=time.monotonic()+20
  while 'Rendering paused' not in log.read_text() and p.poll() is None and time.monotonic()<deadline:time.sleep(.1)
  data=log.read_text();before=data.count('kind=Timeout');assert 'Rendering paused' in data and p.poll() is None
  time.sleep(.5);assert log.read_text().count('kind=Timeout')==before
  ids=subprocess.check_output(['xdotool','search','--pid',str(p.pid)],text=True).splitlines();assert ids
  subprocess.run(['xdotool','key','--window',ids[-1],'F5'],check=True)
  deadline=time.monotonic()+5
  while log.read_text().count('Rendering paused')<2 and p.poll() is None and time.monotonic()<deadline:time.sleep(.1)
  data=log.read_text();ok=p.poll() is None and data.count('Rendering paused')==2 and data.count('kind=Timeout')>before
  report['runs'].append(dict(host='MAIN',fault='persistent timeout',passed=ok,first_episode_attempts=before,total_attempts=data.count('kind=Timeout'),native_F5_retry=True,quiet_after_exhaustion=True));print('timeout and native Retry',ok,flush=True)
  (out/'result.json').write_text(json.dumps(report,indent=2)+'\n');assert ok
 finally:
  p.terminate()
  try:p.wait(timeout=5)
  except subprocess.TimeoutExpired:p.kill();p.wait()

assert hashlib.sha256(binary.read_bytes()).hexdigest()==report["binary_sha256"], "binary changed during trial"
