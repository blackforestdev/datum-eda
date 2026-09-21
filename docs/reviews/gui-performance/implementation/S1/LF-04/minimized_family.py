import os,subprocess,time,json,re,hashlib,ctypes as C
from pathlib import Path
root=Path('/home/bfadmin/Documents/datum-eda');binary=root/'target/release/datum-gui';out=Path('/tmp/pm045-minimized-family');out.mkdir(exist_ok=True)
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
def stamp(line):
 m=re.search(r'tv_sec: (\d+), tv_nsec: (\d+)',line);return int(m[1])+int(m[2])/1e9
report={'binary_sha256':hashlib.sha256(binary.read_bytes()).hexdigest(),'runs':[]}
for host,flag in [('MAIN',None),('GLOBAL','--open-global-preferences'),('PROJECT','--open-project-preferences'),('NEW','--open-new-project')]:
 case=out/host;case.mkdir(exist_ok=True);log=case/'native.log';log.write_text('');env=os.environ.copy()
 for k in list(env):
  if k.startswith('DATUM_DIAGNOSTIC_') or k.startswith('DATUM_GPU_DIAGNOSTIC_'):env.pop(k)
 env.pop('WAYLAND_DISPLAY',None)
 env.update(EDA_CLI_BIN=str(root/'target/release/datum-eda'),CARGO_NET_OFFLINE='true',DATUM_GPU_MEASUREMENTS='0',DATUM_DIAGNOSTIC_SURFACE_FAULT='timeout',DATUM_GUI_VERBOSE_LOG='1',DATUM_GUI_LOG=str(log),WINIT_UNIX_BACKEND='x11',XDG_CONFIG_HOME=str(case/'config'),XDG_CACHE_HOME=str(case/'cache'))
 cmd=[str(binary),'--board',str(Path('/tmp/pm045-readiness-x11')/host/'board.kicad_pcb'),'--window-size','1280x800']+([flag] if flag else [])
 with (case/'stderr.log').open('w') as stream:
  p=subprocess.Popen(cmd,cwd=root,env=env,stdout=stream,stderr=stream);result={'host':host,'command':cmd}
  try:
   ids=until(lambda:(v if len(v)==(2 if flag else 1) else None) if (v:=re.findall(r'surface identity window=WindowId\((\d+)\)',log.read_text())) else None,p);wid=ids[-1];key='WindowId('+wid+')'
   def rows():return [r for r in records(log) if r['window']==key]
   first=until(lambda:next((r for r in rows() if r['reason']=='acquire_timeout'),None),p)
   firstline=next(s for s in complete_lines(log) if 'native_surface_lifecycle ' in s and json.loads(s.split('native_surface_lifecycle ',1)[1])==first)
   cycles=[]
   for cycle in range(2):
    time.sleep(.55)
    offset=len(complete_lines(log));subprocess.run(['xdotool','windowminimize',ids[0]],check=True)
    time.sleep(.15)
    result['minimized_window_state']=subprocess.check_output(['xprop','-id',wid,'_NET_WM_STATE','WM_STATE'],text=True);print(result['minimized_window_state'],flush=True)
    hidden=until(lambda:next((s for s in complete_lines(log)[offset:] if 'native host '+key+' minimized true' in s),None),p)
    before=rows()[-1];time.sleep(.6);after=rows()[-1]
    for count in ['configure_attempts','acquire_attempts','acquired','frame_submissions','presented']:assert before[count]==after[count],(count,before,after)
    assert not any('Rendering paused for native host '+key in s for s in complete_lines(log))
    offset=len(complete_lines(log));subprocess.run(['xdotool','windowactivate','--sync',ids[0]],check=True)
    restored=until(lambda:next((s for s in complete_lines(log)[offset:] if 'native host '+key+' minimized false' in s),None),p)
    until(lambda:any(r['acquire_attempts']>before['acquire_attempts'] for r in rows()),p)
    cycles.append({'hidden_at':stamp(hidden),'restored_at':stamp(restored),'before':before,'after':after})
   failure=until(lambda:next((s for s in complete_lines(log) if 'Rendering paused for native host '+key in s),None),p)
   active=stamp(failure)-stamp(firstline)-sum(c['restored_at']-c['hidden_at'] for c in cycles)
   assert 1.85<=active<=2.3,active
   remaining=stamp(failure)-cycles[-1]['restored_at'];assert remaining<1.3,remaining
   count=rows()[-1]['acquire_attempts'];time.sleep(.2);assert rows()[-1]['acquire_attempts']==count
   result.update({'cycles':cycles,'observed_drawable_active_seconds':active,'failure_after_last_restore_seconds':remaining,'attempts_at_exhaustion':count,'bounded_pause_and_no_reset':True})
   close_window(wid)
   if flag:
    until(lambda:subprocess.check_output(['xdotool','getwindowfocus'],text=True).strip()==ids[0],p);close_window(ids[0])
   p.wait(timeout=20);assert p.returncode==0;result.update({'passed':True,'exit':0});print(host,'hidden retry pass',active,remaining,flush=True)
  except Exception as error:result.update({'passed':False,'error':str(error)});print(host,type(error).__name__,str(error),flush=True);raise
  finally:
   report['runs'].append(result);(out/'result.json').write_text(json.dumps(report,indent=2)+'\n')
   if p.poll() is None:p.terminate();p.wait(timeout=5)
