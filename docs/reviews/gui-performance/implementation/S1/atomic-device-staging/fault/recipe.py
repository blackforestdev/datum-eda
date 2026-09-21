import os,subprocess,time,json,re,hashlib,ctypes as C
from pathlib import Path
root=Path('/home/bfadmin/Documents/datum-eda');binary=root/'target/release/datum-gui';out=Path('/tmp/pm045-device-staging-native');out.mkdir(exist_ok=True)
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
for host,flag in [('GLOBAL','--open-global-preferences'),('PROJECT','--open-project-preferences'),('NEW','--open-new-project')]:
 case=out/host;case.mkdir(exist_ok=True);log=case/'native.log';log.write_text('');env=os.environ.copy()
 for k in list(env):
  if k.startswith('DATUM_DIAGNOSTIC_') or k.startswith('DATUM_GPU_DIAGNOSTIC_'):env.pop(k)
 env.pop('WAYLAND_DISPLAY',None)
 env.update(EDA_CLI_BIN=str(root/'target/release/datum-eda'),DATUM_GPU_MEASUREMENTS='0',DATUM_DIAGNOSTIC_DEVICE_LOSS='once-fail-staging',DATUM_GUI_VERBOSE_LOG='1',DATUM_GUI_LOG=str(log),WINIT_UNIX_BACKEND='x11',XDG_CONFIG_HOME=str(case/'config'),XDG_CACHE_HOME=str(case/'cache'))
 cmd=[str(binary),'--board',str(Path('/tmp/pm045-readiness-x11')/host/'board.kicad_pcb'),flag];result={'host':host,'command':cmd}
 with (case/'stderr.log').open('w') as stream:
  p=subprocess.Popen(cmd,cwd=root,env=env,stdout=stream,stderr=stream)
  try:
   until(lambda:'native device replacement failed:' in log.read_text(),p)
   data=log.read_text();old,staged=map(int,re.search(r'failure injected old_epoch=(\d+) staged_epoch=(\d+)',data).groups());retained=int(re.search(r'replacement retained epoch=(\d+)',data)[1]);assert retained==old and staged!=old
   assert 'native device replacement committed' not in data
   ids=list(dict.fromkeys(re.findall(r'surface identity window=WindowId\((\d+)\)',data)));assert len(ids)==2;main,wid=ids
   dropped=[r for r in records(log) if r['reason']=='owner_drop'];assert len(dropped)==2 and len({r['queue_epoch'] for r in dropped})==1
   assert all(r['frame_submissions']==r['acquire_attempts']==0 for r in dropped)
   time.sleep(.25);assert log.read_text().count('native device replacement begin fault_code=')==1
   subprocess.run(['xdotool','windowsize',wid,'1200','800'],check=True)
   until(lambda:'resized 1200x800' in log.read_text(),p)
   subprocess.run(['xdotool','windowactivate','--sync',wid,'key','F5'],check=True)
   until(lambda:'native device replacement committed' in log.read_text(),p)
   current=int(re.search(r'native device epoch replaced epoch=(\d+)',log.read_text())[1]);assert current not in [old,staged]
   final=until(lambda:next((r for r in reversed(records(log)) if r['window']=='WindowId('+wid+')' and r['reason']=='present' and r['configured_extent']==[1200,800]),None),p)
   assert 'native device state preserved workspace=true terminal_registry=true' in log.read_text()
   result.update(old_epoch=old,staged_epoch=staged,retained_epoch=retained,committed_epoch=current,staged_surface_drops=dropped,final_present=final,recovery_passed=True)
   close_window(wid);until(lambda:subprocess.check_output(['xdotool','getwindowfocus'],text=True).strip()==main,p);close_window(main);p.wait(timeout=20);assert p.returncode==0;result.update(passed=True,exit=0)
   print(host,'atomic staging / Retry passed',flush=True)
  except Exception as error:result.update(passed=False,error=str(error));print(host,type(error).__name__,str(error),flush=True)
  finally:
   report['runs'].append(result);(out/'result.json').write_text(json.dumps(report,indent=2)+'\n')
   if p.poll() is None:p.terminate();p.wait(timeout=5)
assert report['binary_sha256']==hashlib.sha256(binary.read_bytes()).hexdigest()
