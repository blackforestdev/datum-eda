import os,subprocess,time,json,hashlib
from pathlib import Path
root=Path('/home/bfadmin/Documents/datum-eda');binary=root/'target/release/datum-gui';base=Path('/tmp/pm045-readiness-x11');out=Path('/tmp/pm045-frame-lease-device-proof');out.mkdir(exist_ok=True)
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
report={'binary_sha256':sha(binary),'runs':[]}
for host,flag in [('GLOBAL','--open-global-preferences')]:
 case=base/host;log=out/(host+'.log');log.write_text('');env=os.environ.copy()
 for k in list(env):
  if k.startswith('DATUM_DIAGNOSTIC_') or k.startswith('DATUM_GPU_DIAGNOSTIC_'):env.pop(k)
 env.pop('WAYLAND_DISPLAY',None);env.update(EDA_CLI_BIN=str(root/'target/release/datum-eda'),CARGO_NET_OFFLINE='true');env.update(DATUM_GPU_MEASUREMENTS='0',DATUM_DIAGNOSTIC_DEVICE_LOSS='once',DATUM_GUI_VERBOSE_LOG='1',DATUM_GUI_LOG=str(log),WINIT_UNIX_BACKEND='x11',XDG_CONFIG_HOME=str(case/'config'),XDG_CACHE_HOME=str(case/'cache'),TMPDIR=str(case))
 cmd=[str(binary),'--board',str(case/'board.kicad_pcb'),'--window-size','1280x800','--resize-torture-smoke']+([flag] if flag else [])
 with (out/(host+'-stderr.log')).open('w') as stream:
  p=subprocess.Popen(cmd,cwd=root,env=env,stdout=stream,stderr=stream)
  try:
   deadline=time.monotonic()+30
   while time.monotonic()<deadline:
    data=log.read_text()
    if 'native resize smoke end' in data or p.poll() is not None:break
    time.sleep(.1)
   ok=p.poll() is None and 'native resize smoke end' in data and data.count('native device replacement begin')==1 and data.count('native device replacement committed')==1 and data.count('native device state preserved')==1 and any('native device replacement begin fault_code='+str(code) in data for code in [1,2,3,4])
   report['runs'].append(dict(host=host,command=cmd,passed=ok,attempts=data.count('native device replacement begin'),commits=data.count('native device replacement committed'),state_receipts=data.count('native device state preserved'),unexpected_exit=p.poll(),backend_fault_receipts=[r for r in data.splitlines() if 'native device replacement begin fault_code=' in r]));print(host,'device replacement',ok,flush=True)
   (out/'result.json').write_text(json.dumps(report,indent=2)+'\n')
   if not ok:raise SystemExit(1)
  finally:
   p.terminate()
   try:p.wait(timeout=5)
   except subprocess.TimeoutExpired:p.kill();p.wait()
# Persistent allocation failure must preserve process and stop automatic recreation.
case=base/'MAIN';log=out/'oom.log';log.write_text('');env.pop('DATUM_DIAGNOSTIC_DEVICE_LOSS');env.update(DATUM_DIAGNOSTIC_SURFACE_FAULT='oom',DATUM_GUI_LOG=str(log),XDG_CONFIG_HOME=str(case/'config'),XDG_CACHE_HOME=str(case/'cache'),TMPDIR=str(case))
cmd=[str(binary),'--board',str(case/'board.kicad_pcb'),'--window-size','1280x800']
with (out/'oom-stderr.log').open('w') as stream:
 p=subprocess.Popen(cmd,cwd=root,env=env,stdout=stream,stderr=stream)
 try:
  deadline=time.monotonic()+20
  while 'Rendering paused' not in log.read_text() and p.poll() is None and time.monotonic()<deadline:time.sleep(.1)
  data=log.read_text();assert p.poll() is None and 'Rendering paused' in data and data.count('native device replacement begin')==1
  time.sleep(.5);assert log.read_text().count('native device replacement begin')==1
  ids=subprocess.check_output(['xdotool','search','--pid',str(p.pid)],text=True).splitlines();assert ids
  subprocess.run(['xdotool','key','--window',ids[-1],'F5'],check=True)
  deadline=time.monotonic()+8
  while log.read_text().count('Rendering paused')<2 and p.poll() is None and time.monotonic()<deadline:time.sleep(.1)
  data=log.read_text();ok=p.poll() is None and data.count('native device replacement begin')==2 and data.count('Rendering paused')>=2
  time.sleep(.5);ok=ok and log.read_text().count('native device replacement begin')==2
  report['runs'].append(dict(host='MAIN',fault='persistent surface OutOfMemory',passed=ok,automatic_attempts=1,after_native_F5_attempts=data.count('native device replacement begin'),process_retained=p.poll() is None));print('OOM bounded retry',ok,flush=True)
  (out/'result.json').write_text(json.dumps(report,indent=2)+'\n');assert ok
 finally:
  p.terminate()
  try:p.wait(timeout=5)
  except subprocess.TimeoutExpired:p.kill();p.wait()
assert sha(binary)==report['binary_sha256'],'binary changed during trial'
