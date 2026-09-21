import sys,os,subprocess,time,json,re,hashlib
from pathlib import Path
root=Path('/home/bfadmin/Documents/datum-eda');binary=root/'target/release/datum-gui';out=Path('/tmp/pm045-empty-search-'+sys.argv[1]);out.mkdir(exist_ok=False)
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
def close_native(p,title,case):
 name='pm045-close-'+str(p.pid)+'-'+str(time.monotonic_ns());script=case/(name+'.js')
 script.write_text('for (const w of workspace.windowList()) { if (Number(w.pid) === '+str(p.pid)+' && w.caption === '+json.dumps(title)+') { w.closeWindow(); } }\n')
 def dbus(*args):return subprocess.check_output(['qdbus6','org.kde.KWin',*args],text=True,stderr=subprocess.STDOUT,timeout=10)
 sid=dbus('/Scripting','org.kde.kwin.Scripting.loadScript',str(script),name).strip();assert int(sid)>=0
 try:
  info=dbus('/Scripting/Script'+sid)
  (case/'kwin-interface.txt').write_text(info)
  dbus('/Scripting/Script'+sid,'org.kde.kwin.Script.run')
 finally:dbus('/Scripting','org.kde.kwin.Scripting.unloadScript',name)
report={'binary_sha256':hashlib.sha256(binary.read_bytes()).hexdigest(),'runs':[]}
for host,flag,title in [('GLOBAL','--open-global-preferences','Global Preferences — Datum'),('PROJECT','--open-project-preferences','Project Preferences — Datum')]:
 case=out/host;case.mkdir();log=case/'native.log';log.write_text('');env=os.environ.copy()
 for k in list(env):
  if k.startswith(('DATUM_DIAGNOSTIC_','DATUM_GPU_DIAGNOSTIC_')) or k=='DATUM_TRACE_TIMING':env.pop(k)
 env.pop('WAYLAND_DISPLAY',None)
 env.update(WINIT_UNIX_BACKEND='x11',EDA_CLI_BIN=str(root/'target/release/datum-eda'),CARGO_NET_OFFLINE='true',DATUM_GUI_VERBOSE_LOG='1',DATUM_GUI_LOG=str(log),DATUM_GPU_MEASUREMENTS='0',XDG_CONFIG_HOME=str(case/'config'),XDG_CACHE_HOME=str(case/'cache'))
 cmd=[str(binary),'--board',str(Path('/tmp/pm045-readiness-x11')/host/'board.kicad_pcb'),'--window-size','1280x800',flag]
 with (case/'stderr.log').open('w') as stream:
  p=subprocess.Popen(cmd,cwd=root,env=env,stdout=stream,stderr=stream);r={'host':host,'command':cmd,'pid':p.pid}
  try:
   ids=until(lambda:ids if len(ids:=re.findall(r'surface identity window=WindowId\((\d+)\)',log.read_text()))==2 else None,p);wid=ids[-1];key='WindowId('+wid+')'
   until(lambda:any(v['window']==key and v['reason']=='present' for v in records(log)),p)
   subprocess.run(['xdotool','windowfocus','--sync',wid],check=True);time.sleep(.5)
   def count():return max(v['presented'] for v in records(log) if v['window']==key)
   def keypress(name,label):
    start=len(lines(log));subprocess.run(['xdotool','key',name],check=True)
    until(lambda:any('window event '+key+' keyboard ' in s and label in s and 'state=Released' in s for s in lines(log)[start:]),p)
    time.sleep(.5)
   keypress('Tab','Named(Tab)')
   before=count();keypress('BackSpace','Named(Backspace)');empty=count()-before
   before=count();keypress('a','Character("a")');insert=count()-before
   before=count();keypress('BackSpace','Named(Backspace)');delete=count()-before
   before=count();keypress('BackSpace','Named(Backspace)');empty_again=count()-before
   assert insert>0 and delete>0
   r.update(empty_backspace_presentations=empty,insert_presentations=insert,delete_presentations=delete,second_empty_backspace_presentations=empty_again)
   close_native(p,title,case)
   until(lambda:any(v['window']==key and v['reason']=='owner_drop' for v in records(log)),p)
   close_native(p,'Datum EDA',case);p.wait(timeout=20);assert p.returncode==0
   r['exit']=0;print(host,r,flush=True)
   if sys.argv[1]=='candidate':assert empty==empty_again==0
  except Exception as e:r['error']=str(e);raise
  finally:
   report['runs'].append(r);(out/'result.json').write_text(json.dumps(report,indent=2)+'\n')
   if p.poll() is None:p.terminate();p.wait(timeout=5)
