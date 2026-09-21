import os,subprocess,time,json,re,hashlib,ctypes as C
from pathlib import Path
root=Path('/home/bfadmin/Documents/datum-eda');binary=root/'target/release/datum-gui';out=Path('/tmp/pm045-native-wheel-narrow');out.mkdir(exist_ok=True)
class Data(C.Union):_fields_=[('l',C.c_long*5)]
class Client(C.Structure):_fields_=[('type',C.c_int),('serial',C.c_ulong),('send_event',C.c_int),('display',C.c_void_p),('window',C.c_ulong),('message_type',C.c_ulong),('format',C.c_int),('data',Data)]
class Event(C.Union):_fields_=[('client',Client),('pad',C.c_long*24)]
x=C.CDLL('libX11.so.6');x.XOpenDisplay.restype=C.c_void_p;x.XOpenDisplay.argtypes=[C.c_char_p];x.XDefaultRootWindow.restype=C.c_ulong;x.XDefaultRootWindow.argtypes=[C.c_void_p];x.XInternAtom.restype=C.c_ulong;x.XInternAtom.argtypes=[C.c_void_p,C.c_char_p,C.c_int];x.XSendEvent.argtypes=[C.c_void_p,C.c_ulong,C.c_int,C.c_long,C.POINTER(Event)];x.XFlush.argtypes=[C.c_void_p];x.XCloseDisplay.argtypes=[C.c_void_p]
def maximize(wid,on):
 d=x.XOpenDisplay(None);assert d;e=Event();e.client.type=33;e.client.display=d;e.client.window=int(wid);e.client.message_type=x.XInternAtom(d,b'_NET_WM_STATE',0);e.client.format=32;e.client.data.l[:]=[int(on),x.XInternAtom(d,b'_NET_WM_STATE_MAXIMIZED_VERT',0),x.XInternAtom(d,b'_NET_WM_STATE_MAXIMIZED_HORZ',0),1,0];assert x.XSendEvent(d,x.XDefaultRootWindow(d),0,(1<<20)|(1<<19),C.byref(e));x.XFlush(d);x.XCloseDisplay(d)
def complete_lines(path):
 data=path.read_text();return data.splitlines() if data.endswith('\n') else data.splitlines()[:-1]
def records(log):return [json.loads(s.split('native_surface_lifecycle ',1)[1]) for s in complete_lines(log) if 'native_surface_lifecycle ' in s]
def until(fn,p):
 deadline=time.monotonic()+12
 while time.monotonic()<deadline and p.poll() is None:
  val=fn()
  if val:return val
  time.sleep(.05)
 raise AssertionError('native predicate timeout or process exit '+str(p.poll()))
def close_window(wid):
 d=x.XOpenDisplay(None);assert d;e=Event();e.client.type=33;e.client.display=d;e.client.window=int(wid);e.client.message_type=x.XInternAtom(d,b'WM_PROTOCOLS',0);e.client.format=32;e.client.data.l[:]=[x.XInternAtom(d,b'WM_DELETE_WINDOW',0),0,0,0,0];assert x.XSendEvent(d,int(wid),0,0,C.byref(e));x.XFlush(d);x.XCloseDisplay(d)

def progress(log):
 return [tuple(map(int,m.groups())) for line in complete_lines(log) if (m:=re.search(r'native queue progress epoch=(\d+) submitted=(\d+) completed=(\d+)',line))]

report={'binary_sha256':hashlib.sha256(binary.read_bytes()).hexdigest(),'runs':[]}
for host,flag in [('GLOBAL','--open-global-preferences'),('PROJECT','--open-project-preferences')]:
 case=out/host;case.mkdir(exist_ok=True);log=case/'native.log';log.write_text('');env=os.environ.copy()
 for k in list(env):
  if k.startswith('DATUM_DIAGNOSTIC_') or k.startswith('DATUM_GPU_DIAGNOSTIC_'):env.pop(k)
 env.pop('WAYLAND_DISPLAY',None)
 env.update(EDA_CLI_BIN=str(root/'target/release/datum-eda'),DATUM_GPU_MEASUREMENTS='0',DATUM_GUI_VERBOSE_LOG='1',DATUM_GUI_LOG=str(log),WINIT_UNIX_BACKEND='x11',XDG_CONFIG_HOME=str(case/'config'),XDG_CACHE_HOME=str(case/'cache'))
 cmd=[str(binary),'--board',str(Path('/tmp/pm045-readiness-x11')/host/'board.kicad_pcb'),flag]
 with (case/'stderr.log').open('w') as stream:
  p=subprocess.Popen(cmd,cwd=root,env=env,stdout=stream,stderr=stream);result={'host':host,'command':cmd,'steps':[]}
  try:
   ids=until(lambda:(v if len(v)==2 else None) if (v:=re.findall(r'surface identity window=WindowId\((\d+)\)',log.read_text())) else None,p);main,wid=ids
   def counts():return {i:sum(r['window']=='WindowId('+i+')' and r['reason']=='present' for r in records(log)) for i in ids}
   until(lambda:counts()[wid]>0,p)
   subprocess.run(['xdotool','windowsize',wid,'760','570'],check=True)
   until(lambda:any(r['window']=='WindowId('+wid+')' and r['reason']=='present' and r['configured_extent']==[760,570] for r in records(log)),p)
   if host=='GLOBAL':
    subprocess.run(['xdotool','mousemove','--window',wid,'400','118','click','1','type','--delay','20','unit'],check=True)
   time.sleep(1)
   assert subprocess.check_output(['xdotool','getwindowfocus'],text=True).strip()==wid
   def step(name,button,repeat,point,changed):
    subprocess.run(['xdotool','mousemove','--window',wid,str(point[0]),str(point[1])],check=True);time.sleep(.1)
    before=counts();offset=len(complete_lines(log));subprocess.run(['xdotool','click','--repeat',str(repeat),'--delay','10',str(button)],check=True);time.sleep(.6);after=counts()
    result['steps'].append({'name':name,'button':button,'repeat':repeat,'point':point,'before':before,'after':after,'wheel_events':[s for s in complete_lines(log)[offset:] if 'wheel' in s.lower()]})
    assert before[main]==after[main],('unrelated Main presentation',name)
    assert (after[wid]>before[wid])==changed,(name,before,after)
   step('top boundary',4,3,(450,260),False)
   step('horizontal only',6,3,(450,260),False)
   step('outside viewport',5,3,(450,20),False)
   step('changing down',5,1,(450,260),True)
   step('down to bottom',5,40,(450,260),True)
   step('bottom boundary',5,3,(450,260),False)
   step('immediate reversal',4,1,(450,260),True)
   close_window(wid);until(lambda:subprocess.check_output(['xdotool','getwindowfocus'],text=True).strip()==main,p);close_window(main);p.wait(timeout=20);assert p.returncode==0
   result.update(passed=True,exit=0);print(host,'native wheel predicates passed',flush=True)
  except Exception as error:result.update(passed=False,error=str(error));print(host,type(error).__name__,str(error),flush=True)
  finally:
   report['runs'].append(result);(out/'result.json').write_text(json.dumps(report,indent=2)+'\n')
   if p.poll() is None:p.terminate();p.wait(timeout=5)
assert report['binary_sha256']==hashlib.sha256(binary.read_bytes()).hexdigest()
