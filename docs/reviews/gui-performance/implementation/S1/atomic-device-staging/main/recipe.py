import os,subprocess,time,json,re,hashlib,ctypes as C
from pathlib import Path
root=Path('/home/bfadmin/Documents/datum-eda');binary=root/'target/release/datum-gui';out=Path('/tmp/pm045-device-staging-main');out.mkdir(exist_ok=True)
class Data(C.Union):_fields_=[('l',C.c_long*5)]
class Client(C.Structure):_fields_=[('type',C.c_int),('serial',C.c_ulong),('send_event',C.c_int),('display',C.c_void_p),('window',C.c_ulong),('message_type',C.c_ulong),('format',C.c_int),('data',Data)]
class Event(C.Union):_fields_=[('client',Client),('pad',C.c_long*24)]
x=C.CDLL('libX11.so.6');x.XOpenDisplay.restype=C.c_void_p;x.XOpenDisplay.argtypes=[C.c_char_p];x.XDefaultRootWindow.restype=C.c_ulong;x.XDefaultRootWindow.argtypes=[C.c_void_p];x.XInternAtom.restype=C.c_ulong;x.XInternAtom.argtypes=[C.c_void_p,C.c_char_p,C.c_int];x.XSendEvent.argtypes=[C.c_void_p,C.c_ulong,C.c_int,C.c_long,C.POINTER(Event)];x.XFlush.argtypes=[C.c_void_p];x.XCloseDisplay.argtypes=[C.c_void_p]
def maximize(wid,on):
 d=x.XOpenDisplay(None);assert d;e=Event();e.client.type=33;e.client.display=d;e.client.window=int(wid);e.client.message_type=x.XInternAtom(d,b'_NET_WM_STATE',0);e.client.format=32;e.client.data.l[:]=[int(on),x.XInternAtom(d,b'_NET_WM_STATE_MAXIMIZED_VERT',0),x.XInternAtom(d,b'_NET_WM_STATE_MAXIMIZED_HORZ',0),1,0];assert x.XSendEvent(d,x.XDefaultRootWindow(d),0,(1<<20)|(1<<19),C.byref(e));x.XFlush(d);x.XCloseDisplay(d)
def records(log):
 data=log.read_text();lines=data.splitlines() if data.endswith('\n') else data.splitlines()[:-1]
 return [json.loads(s.split('native_surface_lifecycle ',1)[1]) for s in lines if 'native_surface_lifecycle ' in s]
def until(fn,p):
 deadline=time.monotonic()+12
 while time.monotonic()<deadline and p.poll() is None:
  val=fn()
  if val:return val
  time.sleep(.05)
 raise AssertionError('native predicate timeout or process exit '+str(p.poll()))
def close_window(wid):
 d=x.XOpenDisplay(None);assert d;e=Event();e.client.type=33;e.client.display=d;e.client.window=int(wid);e.client.message_type=x.XInternAtom(d,b'WM_PROTOCOLS',0);e.client.format=32;e.client.data.l[:]=[x.XInternAtom(d,b'WM_DELETE_WINDOW',0),0,0,0,0];assert x.XSendEvent(d,int(wid),0,0,C.byref(e));x.XFlush(d);x.XCloseDisplay(d)

report={'binary_sha256':hashlib.sha256(binary.read_bytes()).hexdigest(),'runs':[]}
for host,flag in [('MAIN',None)]:
 case=out/host;case.mkdir(exist_ok=True);log=case/'native.log';log.write_text('');env=os.environ.copy()
 for k in list(env):
  if k.startswith('DATUM_DIAGNOSTIC_') or k.startswith('DATUM_GPU_DIAGNOSTIC_'):env.pop(k)
 env.pop('WAYLAND_DISPLAY',None)
 env.update(EDA_CLI_BIN=str(root/'target/release/datum-eda'),CARGO_NET_OFFLINE='true',DATUM_GPU_MEASUREMENTS='0',DATUM_DIAGNOSTIC_DEVICE_LOSS='once',DATUM_GUI_VERBOSE_LOG='1',DATUM_GUI_LOG=str(log),WINIT_UNIX_BACKEND='x11',XDG_CONFIG_HOME=str(case/'config'),XDG_CACHE_HOME=str(case/'cache'))
 cmd=[str(binary),'--board',str(Path('/tmp/pm045-readiness-x11')/host/'board.kicad_pcb'),'--window-size','1280x800']+([flag] if flag else [])
 with (case/'stderr.log').open('w') as stream:
  p=subprocess.Popen(cmd,cwd=root,env=env,stdout=stream,stderr=stream);result={'host':host,'command':cmd}
  try:
   until(lambda:'native device replacement committed' in log.read_text(),p)
   data=log.read_text();ids=list(dict.fromkeys(re.findall(r'surface identity window=WindowId\((\d+)\)',data)))
   assert len(ids)==(2 if flag else 1),ids
   target='WindowId('+ids[-1]+')'
   old=next(r for r in records(log) if r['reason']=='present')
   shown=until(lambda:next((r for r in reversed(records(log)) if r['window']==target and r['reason']=='present' and r['queue_epoch']!=old['queue_epoch']),None),p)
   time.sleep(.2);data=log.read_text()
   assert data.count('native device replacement begin')==1
   assert data.count('native device replacement committed')==1
   assert data.count('native device state preserved workspace=true terminal_registry=true')==1
   assert 'native device replacement failed' not in data
   result.update({'attempts':1,'commits':1,'workspace_and_terminal_registry_preserved':True,'old_queue_epoch':old['queue_epoch'],'new_queue_epoch':shown['queue_epoch'],'presented_window':target})
   close_window(ids[-1])
   if flag:
    until(lambda:subprocess.check_output(['xdotool','getwindowfocus'],text=True).strip()==ids[0],p)
    close_window(ids[0])
   p.wait(timeout=20);assert p.returncode==0
   assert 'native Main window ownership released=true' in log.read_text()
   result.update({'passed':True,'exit':p.returncode});print(host,'native device recovery pass',flush=True)
  except Exception as error:
   result.update({'passed':False,'error':str(error)});raise
  finally:
   report['runs'].append(result);(out/'result.json').write_text(json.dumps(report,indent=2)+'\n')
   if p.poll() is None:p.terminate();p.wait(timeout=5)
assert report['binary_sha256']==hashlib.sha256(binary.read_bytes()).hexdigest()
