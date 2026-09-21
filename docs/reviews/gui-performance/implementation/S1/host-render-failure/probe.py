import os,time,subprocess,json,re,hashlib
from pathlib import Path
root=Path('/home/bfadmin/Documents/datum-eda');binary=root/'target/release/datum-gui';out=Path('/tmp/pm045-render-failure-proof');out.mkdir(exist_ok=True)
report={'binary_sha256':hashlib.sha256(binary.read_bytes()).hexdigest(),'runs':[]}
def records(log):return [json.loads(s.split('native_surface_lifecycle ',1)[1]) for s in log.read_text().splitlines() if 'native_surface_lifecycle ' in s]
def until(predicate,p):
 deadline=time.monotonic()+20
 while time.monotonic()<deadline and p.poll() is None:
  if predicate():return
  time.sleep(.05)
 raise AssertionError('native predicate failed, process='+str(p.poll()))
for host,flag in [('GLOBAL','--open-global-preferences'),('PROJECT','--open-project-preferences'),('NEW','--open-new-project')]:
 case=out/host;case.mkdir(exist_ok=True);fixture=Path('/tmp/pm045-readiness-x11')/host;log=case/'native.log';log.write_text('');env=os.environ.copy()
 for k in list(env):
  if k.startswith('DATUM_DIAGNOSTIC_') or k.startswith('DATUM_GPU_DIAGNOSTIC_'):env.pop(k)
 env.pop('WAYLAND_DISPLAY',None);env.update(EDA_CLI_BIN=str(root/'target/release/datum-eda'),CARGO_NET_OFFLINE='true',DATUM_GPU_MEASUREMENTS='0',DATUM_DIAGNOSTIC_SURFACE_FAULT='other-once',DATUM_GUI_VERBOSE_LOG='1',DATUM_GUI_LOG=str(log),WINIT_UNIX_BACKEND='x11',XDG_CONFIG_HOME=str(fixture/'config'),XDG_CACHE_HOME=str(fixture/'cache'),TMPDIR=str(fixture))
 cmd=[str(binary),'--board',str(fixture/'board.kicad_pcb'),'--window-size','1280x800',flag]
 with (case/'stderr.log').open('w') as stream:
  p=subprocess.Popen(cmd,cwd=root,env=env,stdout=stream,stderr=stream)
  try:
   until(lambda:log.read_text().count('Rendering paused for native host')==2,p)
   ids=re.findall(r'surface identity window=WindowId\((\d+)\)',log.read_text());assert len(ids)==2
   initial=records(log);assert not any(r['reason']=='present' for r in initial)
   time.sleep(.25);assert log.read_text().count('render error:')==2
   subprocess.run(['xdotool','key','--window',ids[1],'F5'],check=True)
   until(lambda:any(r['reason']=='present' and r['window']=='WindowId('+ids[1]+')' for r in records(log)),p)
   assert not any(r['reason']=='present' and r['window']=='WindowId('+ids[0]+')' for r in records(log))
   subprocess.run(['xdotool','key','--window',ids[1],'Escape'],check=True)
   until(lambda:any(r['reason']=='owner_drop' and r['window']=='WindowId('+ids[1]+')' for r in records(log)),p)
   subprocess.run(['xdotool','key','--window',ids[0],'F5'],check=True)
   until(lambda:any(r['reason']=='present' and r['window']=='WindowId('+ids[0]+')' for r in records(log)),p)
   data=log.read_text();assert 'native device replacement begin' not in data
   report['runs'].append({'host':host,'command':cmd,'passed':True,'native_ids':ids,'initial_paused_hosts':2,'automatic_retries':0,'auxiliary_retry_isolated':True,'both_present_after_own_F5':True,'main_retried_after_modal_close':True,'device_replacements':0,'records':records(log)})
   (out/'result.json').write_text(json.dumps(report,indent=2)+'\n');print(host,'isolated pause and native F5 recovery passed',flush=True)
  finally:
   p.terminate()
   try:p.wait(timeout=5)
   except subprocess.TimeoutExpired:p.kill();p.wait()
assert hashlib.sha256(binary.read_bytes()).hexdigest()==report['binary_sha256']
