import os,subprocess,time,json,re,hashlib,ctypes as C
from pathlib import Path
root=Path('/home/bfadmin/Documents/datum-eda');binary=root/'target/release/datum-gui';out=Path(__import__('sys').argv[1]);out.mkdir(exist_ok=True)
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

log=out/'native.log';log.write_text('');env=os.environ.copy()
for k in list(env):
 if k.startswith('DATUM_DIAGNOSTIC_') or k.startswith('DATUM_GPU_DIAGNOSTIC_'):env.pop(k)
env.pop('WAYLAND_DISPLAY',None)
env.update(EDA_CLI_BIN=str(root/'target/release/datum-eda'),DATUM_GPU_MEASUREMENTS='0',DATUM_DIAGNOSTIC_QUEUE_COMPLETION='hold-until-retry',DATUM_GUI_VERBOSE_LOG='1',DATUM_GUI_LOG=str(log),WINIT_UNIX_BACKEND='x11',XDG_CONFIG_HOME=str(out/'config'),XDG_CACHE_HOME=str(out/'cache'))
report={'binary_sha256':hashlib.sha256(binary.read_bytes()).hexdigest()}
other=subprocess.Popen(['xmessage','-title','PM045 focus receiver','-geometry','200x100+0+0','Native focus test'],stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL)
p=None
try:
 time.sleep(.3);oid=subprocess.check_output(['xdotool','search','--name','^PM045 focus receiver$'],text=True).strip().splitlines()[-1]
 with (out/'stderr.log').open('w') as stream:
  cmd=[str(binary),'--board','/tmp/pm045-readiness-x11/MAIN/board.kicad_pcb','--window-size','1280x800'];report['command']=cmd
  p=subprocess.Popen(cmd,cwd=root,env=env,stdout=stream,stderr=stream)
  first=until(lambda:next((r for r in records(log) if r['reason']=='present'),None),p)
  wid=re.search(r'WindowId\((\d+)\)',first['window'])[1]
  subprocess.run(['xdotool','windowactivate','--sync',oid],check=True)
  until(lambda:'window event '+first['window']+' focused false' in log.read_text(),p)
  time.sleep(.3);offset=len(complete_lines(log))
  subprocess.run(['xdotool','windowminimize',wid],check=True);time.sleep(.1)
  state=subprocess.check_output(['xprop','-id',wid,'_NET_WM_STATE','WM_STATE'],text=True);assert '_NET_WM_STATE_HIDDEN' in state
  report['hidden_state']=state;report['before_hidden']=records(log)[-1];time.sleep(2.5)
  lines=complete_lines(log);report['events_during_hidden']=[s for s in lines[offset:] if 'window event ' in s];report['after_hidden']=records(log)[-1]
  report['failed_while_hidden']='Rendering paused for shared native queue:' in log.read_text()
  report['minimized_observed']='native host '+first['window']+' minimized true' in log.read_text()
  print(json.dumps(report,indent=2),flush=True)
  if not report['failed_while_hidden']:
   subprocess.run(['xdotool','windowactivate','--sync',wid],check=True)
   until(lambda:'Rendering paused for shared native queue:' in log.read_text(),p)
   report['failed_after_restore']=True
   subprocess.run(['xdotool','key','F5'],check=True)
   until(lambda:'native queue held completion released by Retry' in log.read_text(),p)
   until(lambda:any(r['reason']=='present' and r['last_present_receipt']>first['last_present_receipt'] for r in records(log)),p)
   report['retry_presented']=True
  close_window(wid);p.wait(timeout=20);report['exit']=p.returncode
except Exception as error:
 report['error']=str(error);raise
finally:
 (out/'result.json').write_text(json.dumps(report,indent=2)+'\n')
 if p and p.poll() is None:p.terminate();p.wait(timeout=5)
 other.terminate();other.wait(timeout=5)
