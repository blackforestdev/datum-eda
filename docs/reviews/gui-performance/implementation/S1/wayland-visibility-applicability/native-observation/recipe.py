import os,subprocess,time,json,re,hashlib
from pathlib import Path
root=Path('/home/bfadmin/Documents/datum-eda');binary=root/'target/release/datum-gui';out=Path('/tmp/pm045-wayland-hidden-queue-replay');out.mkdir(exist_ok=False)
def lines(log):
 s=log.read_text();return s.splitlines() if s.endswith('\n') else s.splitlines()[:-1]
def records(log):return [json.loads(s.split('native_surface_lifecycle ',1)[1]) for s in lines(log) if 'native_surface_lifecycle ' in s]
def until(fn,p):
 end=time.monotonic()+15
 while time.monotonic()<end and p.poll() is None:
  value=fn()
  if value:return value
  time.sleep(.05)
 raise AssertionError('predicate timeout or exit '+str(p.poll()))
def close_native(p,title,case,action="w.closeWindow();"):
 name='pm045-close-'+str(p.pid)+'-'+str(time.monotonic_ns());script=case/(name+'.js')
 script.write_text('for (const w of workspace.windowList()) { if (Number(w.pid) === '+str(p.pid)+' && w.caption === '+json.dumps(title)+') { '+action+' } }\n')
 def dbus(*args):return subprocess.check_output(['qdbus6','org.kde.KWin',*args],text=True,stderr=subprocess.STDOUT,timeout=10)
 sid=dbus('/Scripting','org.kde.kwin.Scripting.loadScript',str(script),name).strip();assert int(sid)>=0
 try:
  info=dbus('/Scripting/Script'+sid)
  (case/'kwin-interface.txt').write_text(info)
  dbus('/Scripting/Script'+sid,'org.kde.kwin.Script.run')
 finally:dbus('/Scripting','org.kde.kwin.Scripting.unloadScript',name)
report={'binary_sha256':hashlib.sha256(binary.read_bytes()).hexdigest(),'runs':[]}
for host,flag,title in [('MAIN',None,'Datum EDA')]:
 case=out/host;case.mkdir();log=case/'native.log';log.write_text('');env=os.environ.copy()
 for k in list(env):
  if k.startswith(('DATUM_DIAGNOSTIC_','DATUM_GPU_DIAGNOSTIC_')) or k=='DATUM_TRACE_TIMING':env.pop(k)
 env.update(DATUM_DIAGNOSTIC_QUEUE_COMPLETION='hold-until-retry',WINIT_UNIX_BACKEND='wayland',EDA_CLI_BIN=str(root/'target/release/datum-eda'),CARGO_NET_OFFLINE='true',DATUM_GUI_VERBOSE_LOG='1',DATUM_GUI_LOG=str(log),DATUM_GPU_MEASUREMENTS='0',XDG_CONFIG_HOME=str(case/'config'),XDG_CACHE_HOME=str(case/'cache'))
 cmd=[str(binary),'--board',str(Path('/tmp/pm045-readiness-x11')/host/'board.kicad_pcb'),'--window-size','1280x800']+([flag] if flag else [])
 with (case/'stderr.log').open('w') as stream:
  p=subprocess.Popen(cmd,cwd=root,env=env,stdout=stream,stderr=stream);r={'host':host,'pid':p.pid,'command':cmd}
  try:
   ids=until(lambda:re.findall(r'surface identity window=WindowId\((\d+)\) window_backend=wayland',log.read_text()) if log.read_text().count('window_backend=wayland')==(2 if flag else 1) else None,p)
   key='WindowId('+ids[-1]+')'
   until(lambda:any(x['window']==key and x['reason']=='present' for x in records(log)),p)
   matches=json.loads(subprocess.check_output(['busctl','--user','--json=short','call','org.kde.KWin','/WindowsRunner','org.kde.krunner1','Match','s','Datum EDA'],text=True))
   entries=matches['data'][0]
   def info(uuid):
    return json.loads(subprocess.check_output(['busctl','--user','--json=short','call','org.kde.KWin','/KWin','org.kde.KWin','getWindowInfo','s',uuid],text=True))['data'][0]
   found=[]
   for entry in entries:
    uuid=entry[0].split('_',1)[1];state=info(uuid)
    if entry[1]==title:found.append((uuid,state))
   assert len(found)==1, str(found)
   uuid,initial=found[0];r['kwin_initial']=initial;r['kwin_matches']=matches
   close_native(p,title,case,'w.minimized = true;')
   minimized=until(lambda:state if (state:=info(uuid)).get('minimized',{}).get('data') else None,p)
   r['kwin_minimized']=minimized
   before=[x for x in records(log) if x['reason']=='present'];time.sleep(3)
   after=[x for x in records(log) if x['reason']=='present']
   r['queue_exhausted_while_minimized']='Rendering paused for shared native queue' in log.read_text()
   r['reported_visibility_events']=[line for line in lines(log) if 'occluded ' in line or 'minimized ' in line]
   assert info(uuid)['minimized']['data']
   assert r['queue_exhausted_while_minimized']
   assert not r['reported_visibility_events']
   assert len(before)==len(after)
   close_native(p,title,case)
   if flag:
    until(lambda:any(x['window']==key and x['reason']=='owner_drop' for x in records(log)),p)
    close_native(p,'Datum EDA',case)
   p.wait(timeout=20);r["exit"]=p.returncode;assert p.returncode==0, "native exit="+str(p.returncode)
   text=log.read_text();assert text.count('close requested')>=(2 if flag else 1)
   assert 'native Main window ownership released=true' in text
   r.update(passed=True,exit=0,idle_presentations_before=len(before),idle_presentations_after=len(after),final_present=next(x for x in reversed(after) if x['window']==key),native_close_requests=text.count('close requested'))
   print(host,'Wayland idle/native-close pass',flush=True)
  except Exception as error:r.update(passed=False,error=str(error));raise
  finally:
   report['runs'].append(r);(out/'result.json').write_text(json.dumps(report,indent=2)+'\n')
   if p.poll() is None:p.terminate();p.wait(timeout=5)
