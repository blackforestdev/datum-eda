import os,subprocess,time,json,re,hashlib,ctypes as C
from pathlib import Path
root=Path('/home/bfadmin/Documents/datum-eda');binary=root/'target/release/datum-gui';out=Path('/tmp/pm045-native-dialog-input');out.mkdir(exist_ok=True)
class Data(C.Union):_fields_=[('l',C.c_long*5)]
class Client(C.Structure):_fields_=[('type',C.c_int),('serial',C.c_ulong),('send_event',C.c_int),('display',C.c_void_p),('window',C.c_ulong),('message_type',C.c_ulong),('format',C.c_int),('data',Data)]
class Event(C.Union):_fields_=[('client',Client),('pad',C.c_long*24)]
x=C.CDLL('libX11.so.6');x.XOpenDisplay.restype=C.c_void_p;x.XOpenDisplay.argtypes=[C.c_char_p];x.XDefaultRootWindow.restype=C.c_ulong;x.XDefaultRootWindow.argtypes=[C.c_void_p];x.XInternAtom.restype=C.c_ulong;x.XInternAtom.argtypes=[C.c_void_p,C.c_char_p,C.c_int];x.XSendEvent.argtypes=[C.c_void_p,C.c_ulong,C.c_int,C.c_long,C.POINTER(Event)];x.XFlush.argtypes=[C.c_void_p];x.XCloseDisplay.argtypes=[C.c_void_p]
def maximize(wid,on):
 d=x.XOpenDisplay(None);assert d;e=Event();e.client.type=33;e.client.display=d;e.client.window=int(wid);e.client.message_type=x.XInternAtom(d,b'_NET_WM_STATE',0);e.client.format=32;e.client.data.l[:]=[int(on),x.XInternAtom(d,b'_NET_WM_STATE_MAXIMIZED_VERT',0),x.XInternAtom(d,b'_NET_WM_STATE_MAXIMIZED_HORZ',0),1,0];assert x.XSendEvent(d,x.XDefaultRootWindow(d),0,(1<<20)|(1<<19),C.byref(e));x.XFlush(d);x.XCloseDisplay(d)
def records(log):return [json.loads(s.split('native_surface_lifecycle ',1)[1]) for s in log.read_text().splitlines() if 'native_surface_lifecycle ' in s]
def until(fn,p):
 deadline=time.monotonic()+12
 while time.monotonic()<deadline and p.poll() is None:
  val=fn()
  if val:return val
  time.sleep(.05)
 raise AssertionError('native predicate timeout or process exit '+str(p.poll()))
report={'binary_sha256':hashlib.sha256(binary.read_bytes()).hexdigest(),'runs':[]}
for host,flag in [('GLOBAL','--open-global-preferences'),('PROJECT','--open-project-preferences'),('NEW','--open-new-project')]:
 case=out/host;case.mkdir(exist_ok=True);fixture=Path('/tmp/pm045-readiness-x11')/host;log=case/'native.log';log.write_text('');env=os.environ.copy()
 for k in list(env):
  if k.startswith('DATUM_DIAGNOSTIC_') or k.startswith('DATUM_GPU_DIAGNOSTIC_'):env.pop(k)
 env.pop('WAYLAND_DISPLAY',None);env.update(EDA_CLI_BIN=str(root/'target/release/datum-eda'),CARGO_NET_OFFLINE='true',DATUM_GPU_MEASUREMENTS='0',DATUM_GUI_VERBOSE_LOG='1',DATUM_GUI_LOG=str(log),WINIT_UNIX_BACKEND='x11',XDG_CONFIG_HOME=str(fixture/'config'),XDG_CACHE_HOME=str(fixture/'cache'),TMPDIR=str(fixture))
 cmd=[str(binary),'--board',str(fixture/'board.kicad_pcb'),'--window-size','1280x800']+([flag] if flag else [])
 with (case/'stderr.log').open('w') as stream:
  p=subprocess.Popen(cmd,cwd=root,env=env,stdout=stream,stderr=stream)
  try:
   count=2 if flag else 1
   ids=until(lambda:(v if len(v)==count else None) if (v:=re.findall(r'surface identity window=WindowId\((\d+)\)',log.read_text())) else None,p);wid=ids[-1];key='WindowId('+wid+')'
   until(lambda:'Map State: IsViewable' in subprocess.check_output(['xwininfo','-id',wid],text=True),p)
   subprocess.run(['xdotool','windowactivate',wid],check=True)
   def shown():return [r for r in records(log) if r['window']==key and r['reason']=='present']
   initial=until(lambda:shown()[-1] if shown() else None,p);original=initial['configured_extent']
   def counts():
    rows=records(log)
    return {i:{reason:sum(r['window']=='WindowId('+i+')' and r['reason']==reason for r in rows) for reason in ['submit','present']} for i in ids}
   stable=counts();since=time.monotonic();deadline=since+6
   while time.monotonic()-since<.6:
    assert time.monotonic()<deadline,'startup did not settle'
    time.sleep(.05);latest=counts()
    if latest!=stable:stable=latest;since=time.monotonic()
   assert subprocess.check_output(['xdotool','getwindowfocus'],text=True).strip()==wid
   before=counts();start=len(records(log))
   subprocess.run(['xdotool','mousemove','--window',wid,'400','70','click','1'],check=True)
   query={'GLOBAL':'units','PROJECT':'precision','NEW':'PM045 Input'}[host]
   subprocess.run(['xdotool','type','--delay','15',query],check=True)
   until(lambda:counts()[wid]['present']>before[wid]['present'],p);time.sleep(.7)
   after=counts();assert after[ids[0]]==before[ids[0]],'dialog input induced Main submit/present'
   assert after[wid]['present']>before[wid]['present']
   subprocess.run(['import','-window',wid,str(case/'typed.png')],check=True)
   quiet=counts();subprocess.run(['xdotool','key','F6'],check=True);time.sleep(.5)
   no_op=counts();assert quiet==no_op,'unmatched consumed key induced a frame'
   report['runs'].append({'host':host,'passed':True,'command':cmd,'window':key,'query':query,'before':before,'after':after,'unmatched_key':'F6','before_unmatched':quiet,'after_unmatched':no_op,'scope':'native input to own dialog; Main submission/presentation unchanged; screenshot final text requires visual review'})
   (out/'result.json').write_text(json.dumps(report,indent=2)+'\n');print(host,'local input and consumed-key no-frame pass',flush=True)
  except Exception as e:
   report['runs'].append({'host':host,'passed':False,'error':str(e)});(out/'result.json').write_text(json.dumps(report,indent=2)+'\n');raise
  finally:
   p.terminate()
   try:p.wait(timeout=5)
   except subprocess.TimeoutExpired:p.kill();p.wait()
assert hashlib.sha256(binary.read_bytes()).hexdigest()==report['binary_sha256']
