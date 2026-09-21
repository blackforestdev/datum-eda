import ctypes as C,os,subprocess,time,json,re,hashlib
from pathlib import Path
root=Path('/home/bfadmin/Documents/datum-eda');binary=root/'target/release/datum-gui';out=Path('/tmp/pm045-native-dock-cancel');out.mkdir(exist_ok=False)
def lines(log):
 s=log.read_text();return s.splitlines() if s.endswith('\n') else s.splitlines()[:-1]
def records(log):return [json.loads(s.split('native_surface_lifecycle ',1)[1]) for s in lines(log) if 'native_surface_lifecycle ' in s]
def until(fn,p):
 end=time.monotonic()+15
 while time.monotonic()<end and p.poll() is None:
  value=fn()
  if value:return value
  time.sleep(.05)
 raise AssertionError('predicate timeout or exit '+str(p.poll()))
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
x=C.CDLL('libX11.so.6');x.XOpenDisplay.restype=C.c_void_p;x.XOpenDisplay.argtypes=[C.c_char_p];x.XDefaultRootWindow.restype=C.c_ulong;x.XDefaultRootWindow.argtypes=[C.c_void_p];x.XCreateSimpleWindow.restype=C.c_ulong;x.XCreateSimpleWindow.argtypes=[C.c_void_p,C.c_ulong,C.c_int,C.c_int,C.c_uint,C.c_uint,C.c_uint,C.c_ulong,C.c_ulong];x.XMapWindow.argtypes=[C.c_void_p,C.c_ulong];x.XSetInputFocus.argtypes=[C.c_void_p,C.c_ulong,C.c_int,C.c_ulong];x.XDestroyWindow.argtypes=[C.c_void_p,C.c_ulong];x.XFlush.argtypes=[C.c_void_p];x.XCloseDisplay.argtypes=[C.c_void_p]
d=x.XOpenDisplay(None);assert d;helper=x.XCreateSimpleWindow(d,x.XDefaultRootWindow(d),20,20,100,80,0,0,0);x.XMapWindow(d,helper);x.XFlush(d)
case=out;log=out/'native.log';log.write_text('');env=os.environ.copy()
for k in list(env):
 if k.startswith(('DATUM_DIAGNOSTIC_','DATUM_GPU_DIAGNOSTIC_')) or k=='DATUM_TRACE_TIMING':env.pop(k)
env.pop('WAYLAND_DISPLAY',None);env.update(WINIT_UNIX_BACKEND='x11',EDA_CLI_BIN=str(root/'target/release/datum-eda'),CARGO_NET_OFFLINE='true',DATUM_GUI_VERBOSE_LOG='1',DATUM_GUI_LOG=str(log),DATUM_GPU_MEASUREMENTS='0',XDG_CONFIG_HOME=str(out/'config'),XDG_CACHE_HOME=str(out/'cache'))
cmd=[str(binary),'--board','/tmp/pm045-readiness-x11/MAIN/board.kicad_pcb','--window-size','1280x800','--visual-scale-factor','1']
report={'command':cmd,'binary_sha256':hashlib.sha256(binary.read_bytes()).hexdigest()};held=False
with (out/'stderr.log').open('w') as stream:
 p=subprocess.Popen(cmd,cwd=root,env=env,stdout=stream,stderr=stream)
 try:
  wid=until(lambda:ids[0] if (ids:=re.findall(r'surface identity window=WindowId\((\d+)\)',log.read_text())) else None,p);key='WindowId('+wid+')'
  until(lambda:any(v['reason']=='present' for v in records(log)),p);time.sleep(.5)
  geometry={k:int(v) for k,v in (line.split('=',1) for line in subprocess.check_output(['xdotool','getwindowgeometry','--shell',wid],text=True).splitlines())};h=geometry['HEIGHT']
  subprocess.run(['xdotool','windowfocus','--sync',wid],check=True)
  start=len(lines(log));subprocess.run(['xdotool','mousemove','--window',wid,'50',str(h-40),'click','1'],check=True)
  until(lambda:any('terminal resize begin' in s for s in lines(log)[start:]),p);time.sleep(.5)
  sizes=lambda:[tuple(map(int,v)) for v in re.findall(r'terminal resize begin (\d+)x(\d+)',log.read_text())]
  before=sizes()[-1];start=len(lines(log));count=len(sizes());y=h-246+2
  subprocess.run(['xdotool','mousemove','--window',wid,'500',str(y)],check=True);time.sleep(.1)
  subprocess.run(['xdotool','mousedown','1'],check=True);held=True;time.sleep(.1)
  subprocess.run(['xdotool','mousemove','--window',wid,'500',str(y-90)],check=True)
  until(lambda:any('cursor moved 500.00,'+format(y-90,'.2f') in s for s in lines(log)[start:]),p);time.sleep(.2)
  assert len(sizes())==count,'preview resized terminal before cancellation'
  focus_start=len(lines(log));x.XSetInputFocus(d,helper,2,0);x.XFlush(d)
  until(lambda:any('window event '+key+' focused false' in s for s in lines(log)[focus_start:]),p)
  after=until(lambda:sizes()[-1] if len(sizes())>count else None,p)
  assert after[1]>before[1];assert len(sizes())==count+1
  subprocess.run(['xdotool','windowfocus','--sync',wid],check=True)
  subprocess.run(['xdotool','mouseup','1'],check=True);held=False
  barrier=len(lines(log));subprocess.run(['xdotool','mousemove','--window',wid,'501',str(y-90)],check=True)
  until(lambda:any('cursor moved 501.00,'+format(y-90,'.2f') in s for s in lines(log)[barrier:]),p)
  time.sleep(.3);assert len(sizes())==count+1,'trailing release resized again'
  report.update(before=before,after=after,preview_resize_count=0,cancel_resize_count=1,trailing_release_resize_count=0,native_focus_loss=True,later_pointer_barrier=True)
  close_native(p,'Datum EDA',case);p.wait(timeout=20);assert p.returncode==0;report['exit']=0;report['passed']=True
 except Exception as e:report.update(passed=False,error=str(e));raise
 finally:
  if held:subprocess.run(['xdotool','mouseup','1'],check=False)
  if p.poll() is None:p.terminate();p.wait(timeout=5)
  x.XDestroyWindow(d,helper);x.XFlush(d);x.XCloseDisplay(d)
  (out/'result.json').write_text(json.dumps(report,indent=2)+'\n')
