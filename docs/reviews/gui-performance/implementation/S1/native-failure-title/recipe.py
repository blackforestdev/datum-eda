import os,subprocess,time,json,re,hashlib,sys
from pathlib import Path
root=Path('/home/bfadmin/Documents/datum-eda'); binary=root/'target/release/datum-gui'
out=Path(sys.argv[1]);out.mkdir(exist_ok=False); expected=sys.argv[2]=='candidate'
def wait(fn,p):
 end=time.monotonic()+15
 while time.monotonic()<end:
  assert p.poll() is None, p.returncode
  value=fn()
  if value:return value
  time.sleep(.05)
 raise AssertionError('native predicate timed out')
def title(w):return subprocess.check_output(['xdotool','getwindowname',w],text=True).strip()
def close(p,title,case):
 js=case/'close.js';js.write_text('for (const w of workspace.windowList()) { if (Number(w.pid) === '+str(p.pid)+' && w.caption === '+json.dumps(title)+') w.closeWindow(); }')
 name='pm045-title-'+str(p.pid)+'-'+str(time.monotonic_ns())
 script=subprocess.check_output(['qdbus6','org.kde.KWin','/Scripting','loadScript',str(js),name],text=True).strip()
 try:subprocess.run(['qdbus6','org.kde.KWin','/Scripting/Script'+script,'run'],check=True,stdout=subprocess.DEVNULL)
 finally:subprocess.run(['qdbus6','org.kde.KWin','/Scripting','unloadScript',name],check=True,stdout=subprocess.DEVNULL)
report={'source_commit':subprocess.check_output(['git','rev-parse','HEAD'],cwd=root,text=True).strip(),'binary_sha256':hashlib.sha256(binary.read_bytes()).hexdigest(),'expected_notice':expected,'runs':[]}
for host,flag,base in [('MAIN',None,'Datum EDA'),('GLOBAL','--open-global-preferences','Global Preferences — Datum'),('PROJECT','--open-project-preferences','Project Preferences — Datum'),('NEW','--open-new-project','New Project — Datum')]:
 if os.environ.get('PM045_HOST') and os.environ['PM045_HOST']!=host:continue
 case=out/host;case.mkdir();log=case/'native.log';log.write_text('');env=os.environ.copy()
 for key in list(env):
  if key.startswith(('DATUM_DIAGNOSTIC_','DATUM_GPU_DIAGNOSTIC_')):env.pop(key)
 env.pop('WAYLAND_DISPLAY',None);env.pop('DATUM_TRACE_TIMING',None)
 env.update(EDA_CLI_BIN=str(root/'target/release/datum-eda'),DATUM_GPU_MEASUREMENTS='0',DATUM_DIAGNOSTIC_QUEUE_COMPLETION='hold-until-retry',DATUM_GUI_VERBOSE_LOG='1',DATUM_GUI_LOG=str(log),WINIT_UNIX_BACKEND='x11',XDG_CONFIG_HOME=str(case/'config'),XDG_CACHE_HOME=str(case/'cache'))
 cmd=[str(binary),'--board','/tmp/pm045-readiness-x11/MAIN/board.kicad_pcb','--window-size','1280x800']+([flag] if flag else [])
 with (case/'stderr.log').open('w') as stream:
  p=subprocess.Popen(cmd,cwd=root,env=env,stdout=stream,stderr=stream); result={'host':host,'command':cmd}
  try:
   ids=wait(lambda: (v if len(v)==(2 if flag else 1) else None) if (v:=re.findall(r'surface identity window=WindowId\((\d+)\)',log.read_text())) else None,p)
   wait(lambda:'Rendering paused for shared native queue:' in log.read_text(),p)
   time.sleep(.3)
   observed=[title(w) for w in ids];result['failed_titles']=observed
   assert all(('Rendering paused; F5 to Retry or close window' in t)==expected for t in observed),observed
   subprocess.run(['xdotool','windowactivate','--sync',ids[-1]],check=True)
   subprocess.run(['xdotool','key','--delay','80','F5'],check=True)
   wait(lambda:'native queue held completion released by Retry' in log.read_text(),p)
   wait(lambda:title(ids[-1])==base and title(ids[0])=='Datum EDA',p)
   result['restored_titles']=[title(w) for w in ids]
   close(p,base,case)
   if flag:
    wait(lambda:'owner_drop' in log.read_text(),p);close(p,'Datum EDA',case)
   p.wait(timeout=20);assert p.returncode==0
   result.update(passed=True,exit=0);print(host,'pass',flush=True)
  except Exception as error:result.update(passed=False,error=str(error));print(host,repr(error),flush=True)
  finally:
   if p.poll() is None:p.terminate();p.wait(timeout=5)
   report['runs'].append(result);(out/'result.json').write_text(json.dumps(report,indent=2)+'\n')
assert all(r['passed'] for r in report['runs'])
