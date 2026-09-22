import os,subprocess,time,json,re,hashlib,sys
from pathlib import Path
scale=float(os.environ['PM045_TEST_SCALE'])
root=Path('/home/bfadmin/Documents/datum-eda');out=Path(sys.argv[1]);out.mkdir(exist_ok=False)
assert os.environ.get('DISPLAY')
# Release the button left down by the preceding failed harness before its SIGTERM.
subprocess.run(['xdotool','mouseup','1'],check=True,timeout=8)
report={'display':os.environ['DISPLAY'],'wayland_display':os.environ.get('WAYLAND_DISPLAY'),'topology':'Owner-confirmed idle desktop X11/Xwayland; native adapter functional trial','binary_sha256':hashlib.sha256((root/'target/release/datum-gui').read_bytes()).hexdigest(),'visual_scale_factor':scale,'runs':[]}
for host,flag in [('GLOBAL','--open-global-preferences'),('PROJECT','--open-project-preferences')]:
 case=out/host;case.mkdir();log=case/'native.log';log.write_text('');actions=[]
 env=os.environ.copy()
 for key in list(env):
  if key.startswith(('DATUM_DIAGNOSTIC_','DATUM_GPU_DIAGNOSTIC_')):env.pop(key)
 env.pop('WAYLAND_DISPLAY',None)
 env.update(DATUM_TRACE_TIMING='1',EDA_CLI_BIN=str(root/'target/release/datum-eda'),DATUM_GPU_MEASUREMENTS='0',DATUM_GUI_VERBOSE_LOG='1',DATUM_GUI_LOG=str(log),WINIT_UNIX_BACKEND='x11',XDG_CONFIG_HOME=str(case/'config'),XDG_CACHE_HOME=str(case/'cache'))
 cmd=[str(root/'target/release/datum-gui'),'--board','/tmp/pm045-readiness-x11/MAIN/board.kicad_pcb','--window-size','1280x800','--initial-layout','single','--visual-scale-factor',str(scale),flag]
 result={'host':host,'command':cmd,'actions':actions}
 with (case/'stderr.log').open('w') as stderr:
  p=subprocess.Popen(cmd,cwd=root,env=env,stdout=stderr,stderr=stderr)
  def wait(fn):
   end=time.monotonic()+25
   while time.monotonic()<end:
    assert p.poll() is None,('process exited',p.returncode)
    value=fn()
    if value:return value
    time.sleep(.03)
   raise AssertionError('predicate timeout')
  pointer=['0','0']
  def xdo(*args):
   if args[0]=='mousemove':
    pointer[:]=[args[3],args[4]]
   elif args[0] in ('mousedown','mouseup','click'):
    args=('mousemove','--window',wid,*pointer,*args)
   if args[0]=='mousemove' and args[1]=='--window':
    geo=subprocess.check_output(['xdotool','getwindowgeometry','--shell',wid],env=env,text=True)
    d=dict(line.split('=',1) for line in geo.splitlines() if '=' in line)
    args=('mousemove',str(int(d['X'])+int(args[3])),str(int(d['Y'])+int(args[4])),*args[5:])
   action={'xdotool':args,'monotonic':time.monotonic(),'wall_time':time.time()}
   actions.append(action)
   subprocess.run(['xdotool',*args],env=env,check=True,timeout=8)
   action['pointer_after']=subprocess.check_output(['xdotool','getmouselocation','--shell'],env=env,text=True)
  try:
   ids=wait(lambda: (v if len(v)==2 else None) if (v:=re.findall(r'surface identity window=WindowId\((\d+)\)',log.read_text())) else None)
   wid=ids[-1];key='WindowId('+wid+')'
   def count():
    data=log.read_text();lines=data.splitlines() if data.endswith('\n') else data.splitlines()[:-1]
    return sum(json.loads(s.split('native_surface_lifecycle ',1)[1]).get('reason')=='present' and json.loads(s.split('native_surface_lifecycle ',1)[1]).get('window')==key for s in lines if 'native_surface_lifecycle ' in s)
   def event_count(event):return log.read_text().count('window event '+key+' '+event)
   wait(lambda:count()>0);time.sleep(.4)
   xdo('windowactivate','--sync',wid)
   xdo('windowsize',wid,str(round(960*scale)),str(round(720*scale)));time.sleep(.4)
   if host=='GLOBAL':
    before=count();xdo('mousemove','--window',wid,str(round(50*scale)),str(round(107*scale)),'click','1');wait(lambda:count()>before)
   time.sleep(.4)
   geometry=subprocess.check_output(['xdotool','getwindowgeometry','--shell',wid],env=env,text=True)
   dims=dict(line.split('=',1) for line in geometry.splitlines() if '=' in line);width=int(dims['WIDTH']);height=int(dims['HEIGHT'])
   result['geometry']=dims
   xdo('mousemove','--window',wid,str(round(width-6*scale)),str(round(130*scale)));time.sleep(.2)
   subprocess.run(['import','-window',wid,str(case/'before.png')],env=env,check=True,timeout=8)
   before=count();events=event_count('mouse input Left Pressed');xdo('mousedown','1');wait(lambda:event_count('mouse input Left Pressed')>events);time.sleep(.25)
   assert count()==before,('thumb press redrew',before,count())
   result['thumb_press_frames']=0
   xdo('mousemove','--window',wid,str(round(width-6*scale)),str(round(height-12*scale)));wait(lambda:count()>before);time.sleep(.3)
   result['drag_frames']=count()-before
   before=count();events=event_count('mouse input Left Released');xdo('mouseup','1');wait(lambda:event_count('mouse input Left Released')>events);time.sleep(.2)
   assert count()==before,('release redrew',before,count())
   subprocess.run(['import','-window',wid,str(case/'bottom.png')],env=env,check=True,timeout=8)
   xdo('mousemove','--window',wid,str(round(300*scale)),str(round(300*scale)));time.sleep(.2)
   before=count();events=event_count('mouse wheel');xdo('click','5');wait(lambda:event_count('mouse wheel')>events);time.sleep(.25)
   assert count()==before,('bottom boundary redrew',before,count())
   result['boundary_frames']=0
   xdo('click','4');wait(lambda:count()>before);time.sleep(.25)
   result['reverse_frames']=count()-before
   subprocess.run(['import','-window',wid,str(case/'reverse.png')],env=env,check=True,timeout=8)
   result['warm_cache_samples']=[]
   for cycle in range(3):
    for button in ['5','4']:
     before=count();start=(case/'stderr.log').stat().st_size
     xdo('click',button);wait(lambda:count()>before)
     def new_stats():
      with (case/'stderr.log').open() as trace:trace.seek(start);data=trace.read()
      return re.findall(r'dialog renderer=\d+us passes=1 text_cache=(\d+)/(\d+)',data)
     stats=wait(new_stats)
     result['warm_cache_samples'].append({'cycle':cycle,'button':button,'stats':stats})
     assert all(int(hit)>0 and int(miss)==0 for hit,miss in stats), stats
   result['passed']=True;print(host,'PASS',flush=True)
  except Exception as e:
   result.update(passed=False,error=repr(e));print(host,repr(e),flush=True)
  finally:
   subprocess.run(['xdotool','mouseup','1'],env=env,check=True,timeout=8)
   if p.poll() is None:p.terminate();p.wait(timeout=10)
   result['cleanup']='Harness SIGTERM; no teardown qualification'
   report['runs'].append(result);(out/'result.json').write_text(json.dumps(report,indent=2)+'\n')
assert all(r['passed'] for r in report['runs'])
