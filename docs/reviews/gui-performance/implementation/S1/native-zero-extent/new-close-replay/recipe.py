import os,subprocess,time,json,re,hashlib,ctypes as C
from pathlib import Path
root=Path('/home/bfadmin/Documents/datum-eda');binary=root/'target/release/datum-gui';out=Path('/tmp/pm045-native-zero-extent-new-replay');out.mkdir(exist_ok=True)
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

class Configure(C.Structure):
 _fields_=[('type',C.c_int),('serial',C.c_ulong),('send_event',C.c_int),('display',C.c_void_p),('event',C.c_ulong),('window',C.c_ulong),('x',C.c_int),('y',C.c_int),('width',C.c_int),('height',C.c_int),('border_width',C.c_int),('above',C.c_ulong),('override_redirect',C.c_int)]
class Expose(C.Structure):
 _fields_=[('type',C.c_int),('serial',C.c_ulong),('send_event',C.c_int),('display',C.c_void_p),('window',C.c_ulong),('x',C.c_int),('y',C.c_int),('width',C.c_int),('height',C.c_int),('count',C.c_int)]
class Event(C.Union):_fields_=[('client',Client),('configure',Configure),('expose',Expose),('pad',C.c_long*24)]
x.XSendEvent.argtypes=[C.c_void_p,C.c_ulong,C.c_int,C.c_long,C.POINTER(Event)]
def notify_size(wid,geometry,width,height):
 d=x.XOpenDisplay(None);assert d;e=Event();v=e.configure;v.type=22;v.display=d;v.event=v.window=int(wid);v.x=geometry['X'];v.y=geometry['Y'];v.width=width;v.height=height
 assert x.XSendEvent(d,int(wid),0,0,C.byref(e));x.XFlush(d);x.XCloseDisplay(d)
def expose(wid,width,height):
 d=x.XOpenDisplay(None);assert d;e=Event();v=e.expose;v.type=12;v.display=d;v.window=int(wid);v.width=width;v.height=height
 assert x.XSendEvent(d,int(wid),0,0,C.byref(e));x.XFlush(d);x.XCloseDisplay(d)

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
for host,flag in [('NEW','--open-new-project')]:
 case=out/host;case.mkdir(exist_ok=True);log=case/'native.log';log.write_text('');env=os.environ.copy()
 for k in list(env):
  if k.startswith('DATUM_DIAGNOSTIC_') or k.startswith('DATUM_GPU_DIAGNOSTIC_'):env.pop(k)
 env.pop('WAYLAND_DISPLAY',None)
 env.update(EDA_CLI_BIN=str(root/'target/release/datum-eda'),CARGO_NET_OFFLINE='true',DATUM_GPU_MEASUREMENTS='0',DATUM_GUI_VERBOSE_LOG='1',DATUM_GUI_LOG=str(log),WINIT_UNIX_BACKEND='x11',XDG_CONFIG_HOME=str(case/'config'),XDG_CACHE_HOME=str(case/'cache'))
 cmd=[str(binary),'--board',str(Path('/tmp/pm045-readiness-x11')/host/'board.kicad_pcb'),'--window-size','1280x800']+([flag] if flag else [])
 with (case/'stderr.log').open('w') as stream:
  p=subprocess.Popen(cmd,cwd=root,env=env,stdout=stream,stderr=stream);result={'host':host,'command':cmd}
  try:
   ids=until(lambda:(v if len(v)==(2 if flag else 1) else None) if (v:=re.findall(r'surface identity window=WindowId\((\d+)\)',log.read_text())) else None,p);wid=ids[-1];key='WindowId('+wid+')'
   until(lambda:next((r for r in records(log) if r['window']==key and r['reason']=='present'),None),p)
   time.sleep(1)
   geometry={k:int(v) for k,v in (line.split('=',1) for line in subprocess.check_output(['xdotool','getwindowgeometry','--shell',wid],text=True).splitlines())}
   before=next(r for r in reversed(records(log)) if r['window']==key)
   start=len(complete_lines(log));notify_size(wid,geometry,0,0)
   until(lambda:any('window event '+key+' resized 0x0' in s for s in complete_lines(log)[start:]),p)
   exposure_start=len(complete_lines(log));expose(wid,geometry['WIDTH'],geometry['HEIGHT'])
   until(lambda:any('window event '+key+' redraw requested' in s for s in complete_lines(log)[exposure_start:]),p)
   time.sleep(.3)
   zero_rows=[r for r in records(log) if r['window']==key]
   for field in ['configure_attempts','acquire_attempts','frame_submissions','presented']:
    assert max(r[field] for r in zero_rows)==before[field], field+' advanced at zero extent'
   notify_size(wid,geometry,geometry['WIDTH'],geometry['HEIGHT'])
   restored=until(lambda:next((r for r in reversed(records(log)) if r['window']==key and r['reason']=='present' and r['presented']>before['presented']),None),p)
   assert restored['configured_extent']==[geometry['WIDTH'],geometry['HEIGHT']]
   assert restored['configure_attempts']==before['configure_attempts']
   result.update({'native_zero_notification':True,'native_expose_delivered_while_zero':True,'zero_interval_seconds':.3,'before':before,'restored':restored})
   close_native(p,'New Project — Datum',case)
   if flag:
    until(lambda:subprocess.check_output(['xdotool','getwindowfocus'],text=True).strip()==ids[0],p)
    close_native(p,'Datum EDA',case)
   p.wait(timeout=20);assert p.returncode==0;result.update({'passed':True,'exit':0});print(host,'native zero/expose/restore pass',flush=True)
  except Exception as error:result.update({'passed':False,'error':str(error)});raise
  finally:
   report['runs'].append(result);(out/'result.json').write_text(json.dumps(report,indent=2)+'\n')
   if p.poll() is None:p.terminate();p.wait(timeout=5)
assert report['binary_sha256']==hashlib.sha256(binary.read_bytes()).hexdigest()
