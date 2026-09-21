import os,subprocess,time,json,re,hashlib,ctypes as C,sys
from pathlib import Path
root=Path('/home/bfadmin/Documents/datum-eda');binary=root/'target/release/datum-gui';out=Path('/tmp/pm045-focus-capture-'+sys.argv[1]);out.mkdir(exist_ok=False)
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

# Output belongs only to this bounded native recipe.
out=Path('/tmp/pm045-focus-capture-'+sys.argv[1])
x.XCreateSimpleWindow.restype=C.c_ulong;x.XCreateSimpleWindow.argtypes=[C.c_void_p,C.c_ulong,C.c_int,C.c_int,C.c_uint,C.c_uint,C.c_uint,C.c_ulong,C.c_ulong]
x.XMapWindow.argtypes=[C.c_void_p,C.c_ulong];x.XSetInputFocus.argtypes=[C.c_void_p,C.c_ulong,C.c_int,C.c_ulong];x.XDestroyWindow.argtypes=[C.c_void_p,C.c_ulong]
log=out/'native.log';log.write_text('');env=os.environ.copy()
for k in list(env):
 if k.startswith(('DATUM_DIAGNOSTIC_','DATUM_GPU_DIAGNOSTIC_')):env.pop(k)
env.pop('WAYLAND_DISPLAY',None);env.update(WINIT_UNIX_BACKEND='x11',DATUM_GUI_VERBOSE_LOG='1',DATUM_GUI_LOG=str(log),DATUM_GPU_MEASUREMENTS='0',EDA_CLI_BIN=str(root/'target/release/datum-eda'),CARGO_NET_OFFLINE='true',XDG_CONFIG_HOME=str(out/'config'),XDG_CACHE_HOME=str(out/'cache'))
cmd=[str(binary),'--board','/tmp/pm045-readiness-x11/NEW/board.kicad_pcb','--window-size','1280x800','--open-new-project','--visual-scale-factor','1']
report={'command':cmd,'binary_sha256':hashlib.sha256(binary.read_bytes()).hexdigest()};d=x.XOpenDisplay(None);assert d;helper=x.XCreateSimpleWindow(d,x.XDefaultRootWindow(d),20,20,100,80,0,0,0);x.XMapWindow(d,helper);x.XFlush(d);held=False
with (out/'stderr.log').open('w') as stream:
 p=subprocess.Popen(cmd,cwd=root,env=env,stdout=stream,stderr=stream)
 try:
  ids=until(lambda:ids if len(ids:=re.findall(r'surface identity window=WindowId\((\d+)\)',log.read_text()))==2 else None,p);main,wid=ids;key='WindowId('+wid+')'
  subprocess.run(['xdotool','windowsize',wid,'760','720'],check=True)
  until(lambda:any(r['window']==key and r['reason']=='present' and r['configured_extent']==[760,720] for r in records(log)),p)
  subprocess.run(['xdotool','windowfocus','--sync',wid],check=True)
  subprocess.run(['xdotool','mousemove','--window',wid,'600','528'],check=True);time.sleep(.2)
  subprocess.run(['xdotool','mousedown','1'],check=True);held=True;time.sleep(.1)
  start=len(complete_lines(log));x.XSetInputFocus(d,helper,2,0);x.XFlush(d)
  until(lambda:any('window event '+key+' focused false' in s for s in complete_lines(log)[start:]),p)
  subprocess.run(['xdotool','windowfocus','--sync',wid],check=True)
  until(lambda:any('window event '+key+' focused true' in s for s in complete_lines(log)[start:]),p)
  release_start=len(complete_lines(log))
  subprocess.run(['xdotool','mouseup','1'],check=True);held=False
  subprocess.run(['xdotool','mousemove','--window',wid,'600','529'],check=True)
  until(lambda:any('window event '+key+' cursor moved 600.00,529.00' in line for line in complete_lines(log)[release_start:]),p)
  report['later_native_pointer_event_observed']=True
  closed=any(r['window']==key and r['reason']=='owner_drop' for r in records(log));report['closed_by_cancelled_release']=closed
  assert not closed,'focus-cancelled release activated New Project Cancel'
  subprocess.run(['xdotool','click','1'],check=True)
  until(lambda:any(r['window']==key and r['reason']=='owner_drop' for r in records(log)),p)
  report['fresh_click_closes']=True;close_window(main);p.wait(timeout=20);assert p.returncode==0;report['exit']=0;report['passed']=True
 except Exception as error:report.update(passed=False,error=str(error));raise
 finally:
  if held:subprocess.run(['xdotool','mouseup','1'],check=False)
  if p.poll() is None:p.terminate();p.wait(timeout=5)
  x.XDestroyWindow(d,helper);x.XFlush(d);x.XCloseDisplay(d)
  (out/'result.json').write_text(json.dumps(report,indent=2)+'\n')
