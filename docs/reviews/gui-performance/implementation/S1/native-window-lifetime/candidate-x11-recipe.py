import os,subprocess,time,json,re,hashlib,ctypes as C
from pathlib import Path
root=Path('/home/bfadmin/Documents/datum-eda');binary=root/'target/release/datum-gui';out=Path('/tmp/pm045-wayland-teardown-x11');out.mkdir(exist_ok=True)
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
for host,flag in [('MAIN',None),('GLOBAL','--open-global-preferences'),('PROJECT','--open-project-preferences'),('NEW','--open-new-project')]:
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
   time.sleep(2)
   subprocess.run(['xdotool','windowsize',wid,'1200','800'],check=True)
   final=until(lambda:next((r for r in reversed(records(log)) if r['window']==key and r['reason']=='present' and r['configured_extent']==[1200,800]),None),p)
   def settled():
    rows=[r for r in records(log) if r['reason']=='present'];receipt=max(r['frame_submission_receipts_issued'] for r in rows)
    return next((r for r in reversed(progress(log)) if r[1]==r[2]==receipt),None)
   completed=until(settled,p)
   before=[r for r in records(log) if r['reason']=='present'];count=len(before)
   time.sleep(3)
   after=[r for r in records(log) if r['reason']=='present']
   assert len(after)==count, 'idle generated another frame'
   assert progress(log)[-1]==completed
   assert completed[2]>=final['last_present_receipt']
   assert 'Rendering paused' not in log.read_text()
   result.update({'final_present':final,'completion':completed,'idle_seconds':3,'presentations_before_idle':count,'presentations_after_idle':len(after),'main_presentations':sum(r['window']=='WindowId('+ids[0]+')' for r in after)})
   close_window(wid)
   if flag:
    until(lambda:subprocess.check_output(['xdotool','getwindowfocus'],text=True).strip()==ids[0],p)
    close_window(ids[0])
   p.wait(timeout=20);assert p.returncode==0;result.update({'passed':True,'exit':0});print(host,'native idle completion pass',flush=True)
  except Exception as error:result.update({'passed':False,'error':str(error)});raise
  finally:
   report['runs'].append(result);(out/'result.json').write_text(json.dumps(report,indent=2)+'\n')
   if p.poll() is None:p.terminate();p.wait(timeout=5)
assert report['binary_sha256']==hashlib.sha256(binary.read_bytes()).hexdigest()
