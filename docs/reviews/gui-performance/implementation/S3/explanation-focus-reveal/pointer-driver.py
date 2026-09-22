import os,subprocess,time,re,json,sys,hashlib
from pathlib import Path
root=Path('/home/bfadmin/Documents/datum-eda');out=Path(sys.argv[1]);out.mkdir();reports=[]
for host,flag in [('GLOBAL','--open-global-preferences'),('PROJECT','--open-project-preferences')]:
 d=out/host;d.mkdir();log=d/'native.log';log.write_text('');actions=[]
 env=os.environ.copy();env.pop('WAYLAND_DISPLAY',None);env.update(WINIT_UNIX_BACKEND='x11',DATUM_GUI_VERBOSE_LOG='1',DATUM_GUI_LOG=str(log),DATUM_GPU_MEASUREMENTS='0',XDG_CONFIG_HOME=str(d/'config'),XDG_CACHE_HOME=str(d/'cache'))
 cmd=[str(root/'target/release/datum-gui'),'--board','/tmp/pm045-readiness-x11/MAIN/board.kicad_pcb','--window-size','1280x800','--initial-layout','single',flag,'--visual-scale-factor','1']
 result={'host':host,'command':cmd,'actions':actions,'binary_sha256':hashlib.sha256((root/'target/release/datum-gui').read_bytes()).hexdigest()}
 with (d/'stderr.log').open('w') as err:
  p=subprocess.Popen(cmd,cwd=root,env=env,stdout=err,stderr=err)
  def wait(fn):
   end=time.monotonic()+20
   while time.monotonic()<end:
    assert p.poll() is None
    v=fn()
    if v:return v
    time.sleep(.04)
   raise AssertionError('predicate timeout')
  def xdo(*args):
   actions.append({'args':args,'time':time.time()});subprocess.run(['xdotool',*args],env=env,check=True,timeout=8)
  def click(x,y):
   g=dict(s.split('=',1) for s in subprocess.check_output(['xdotool','getwindowgeometry','--shell',wid],text=True,env=env).splitlines())
   xdo('mousemove',str(int(g['X'])+x),str(int(g['Y'])+y));time.sleep(.15);xdo('click','1')
  def capture(name):subprocess.run(['import','-window',wid,str(d/(name+'.png'))],env=env,check=True,timeout=8)
  try:
   ids=wait(lambda:v if len(v:=re.findall(r'surface identity window=WindowId\((\d+)\)',log.read_text()))==2 else None);wid=ids[-1];key='WindowId('+wid+')'
   def count():
    return sum(json.loads(s.split('native_surface_lifecycle ',1)[1]).get('reason')=='present' and json.loads(s.split('native_surface_lifecycle ',1)[1]).get('window')==key for s in log.read_text().splitlines() if 'native_surface_lifecycle ' in s and s.endswith('}'))
   wait(lambda:count()>0);xdo('windowactivate','--sync',wid);xdo('windowsize','--sync',wid,'1000','540');time.sleep(.6)
   if host=='GLOBAL':
    n=count();click(50,107);wait(lambda:count()>n);time.sleep(.2)
   g={k:int(v) for k,v in (line.split('=',1) for line in subprocess.check_output(['xdotool','getwindowgeometry','--shell',wid],text=True,env=env).splitlines())}
   result['geometry']=g;w,h=g['WIDTH'],g['HEIGHT']
   xdo('mousemove',str(g['X']+w-6),str(g['Y']+130));time.sleep(.2)
   n=count();xdo('mousedown','1');time.sleep(.2);xdo('mousemove',str(g['X']+w-6),str(g['Y']+h-8));wait(lambda:count()>n);time.sleep(.25);xdo('mouseup','1');time.sleep(.25)
   capture('last-row');n=count();click(300,h-67);wait(lambda:count()>n);time.sleep(.4);capture('explanation')
   if os.environ.get('PM045_CLICK_CLOSE')=='1':
    n=count();click(w-62,h-10);wait(lambda:count()>n);time.sleep(.4);capture('closed-explanation');result['close_click_frames']=count()-n
   result['passed_recipe']=True
  except Exception as e:result.update(passed_recipe=False,error=repr(e));print(host,repr(e),flush=True)
  finally:
   subprocess.run(['xdotool','mouseup','1'],env=env,timeout=8)
   if p.poll() is None:p.terminate();p.wait(timeout=10)
   reports.append(result);(out/'result.json').write_text(json.dumps(reports,indent=2)+'\n')
assert all(r['passed_recipe'] for r in reports)
