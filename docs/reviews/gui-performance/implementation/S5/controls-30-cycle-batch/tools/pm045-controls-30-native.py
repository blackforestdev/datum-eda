import os, sys, time, json, subprocess, hashlib, tempfile, re, uuid, ctypes, threading, socket
from pathlib import Path
root=Path('/home/bfadmin/Documents/datum-eda')
out=Path(tempfile.mkdtemp(prefix='pm045-controls-30-native-')); print(out,flush=True)
board=Path('/tmp/datum-gui-candidate-boundary-v3/DOA2526.kicad_pcb')
assert hashlib.sha256(board.read_bytes()).hexdigest()=='7df67d426cc0cd808096824b0b7a05dcde351a1305fd41244d8c05a15801c08d'
binary=Path(os.environ['PM045_POINTER_BINARY']); log=out/'native.log';log.touch()
assert hashlib.sha256(binary.read_bytes()).hexdigest()==os.environ['PM045_CANDIDATE_SHA']
mode='plain'
resource_mode=os.environ['PM045_RESOURCE_MODE'];assert resource_mode in ('on','off')
trial=int(os.environ['PM045_TRIAL']);assert trial in range(1,11)
env=os.environ.copy()
for k in list(env):
 if k.startswith(('DATUM_DIAGNOSTIC_','DATUM_GPU_DIAGNOSTIC_','DATUM_RESOURCE_TRACE','DATUM_GPU_ALLOCATION_TRACE','DATUM_PRIVATE_TEXT_TRACE')):env.pop(k)
env.pop('WAYLAND_DISPLAY',None)
env.pop('DATUM_GUI_VERBOSE_LOG',None)
env.update(WINIT_UNIX_BACKEND='x11',WINIT_X11_SCALE_FACTOR='1',XDG_CONFIG_HOME=str(out/'config'),XDG_CACHE_HOME=str(out/'cache'),DATUM_GUI_LOG=str(log),DATUM_GPU_MEASUREMENTS='1' if mode=='kernel' else '0',EDA_CLI_BIN=str(root/'target/release/datum-eda'))
env['DATUM_GUI_VERBOSE_LOG']='1'
env['DATUM_GPU_MEASUREMENTS']=os.environ.get('PM045_POINTER_GPU','0')
if resource_mode=='on':
 env.update(DATUM_RESOURCE_TRACE=str(out/'resources.jsonl'),DATUM_GPU_ALLOCATION_TRACE=str(out/'gpu.jsonl'),DATUM_PRIVATE_TEXT_TRACE=str(out/'private.jsonl'))
cmd=[str(binary),'--project-root','/tmp/pm045-admission-hqugp169/project','--initial-layout','single','--window-size','1280x800','--visual-scale-factor','1']
cmd.append('--open-global-preferences' if os.environ['PM045_CONTROLS_HOST']=='GLOBAL' else '--open-project-preferences')
report={'candidate':subprocess.check_output(['git','rev-parse','HEAD'],cwd=root,text=True).strip(),'binary_sha256':hashlib.sha256(binary.read_bytes()).hexdigest(),'command':cmd,'backend':'x11 on Xwayland','cycles':[],'limits':['Partial W-WINDOWS: CPU and attachment ownership; GPU duty/timing and complete memory accounting not sampled.','No endurance or independent replay acceptance.']}
report['mode']=mode;report['trial']=trial
report['limits']=['Three predeclared matched mode pairs; descriptive overhead only, not seven-pair relative inference.', 'CPU resource mode has timestamp/ptrace diagnostics off. GPU diagnostic mode needs overhead and client lifetime reconciliation before numerical admission.', 'Exact window readback verifies static readiness, not calibrated physical display latency.', 'No complete memory/endurance/backend-scale/independent acceptance.']
report['source_state']='Pinned cold-start baseline/candidate provenance in declaration'
report['binary_role']=os.environ['PM045_POINTER_ROLE']
report['resource_trace_mode']=resource_mode
report['source_sha256']=json.loads(Path(os.environ['PM045_RESOURCE_DECLARATION']).read_text())['source_sha256']
for path,digest in report['source_sha256'].items():assert hashlib.sha256((root/path).read_bytes()).hexdigest()==digest,path
report['limits']=['W-CONTROLS30cycle X11/1x sequence with compositor transition comparisons and bounded clip probes; not other backend/scale, numerical cap, resource or full qualification. Historical pilot context follows. W-CONTROLS pilot one host per fresh seeded launch; compositor captures authoritative, X11 window readbacks retained only for comparison. One cycle per GLOBAL/PROJECT, not30-cycle qualification. Verbose structural/readback method trial; no CPU/resource/latency acceptance. Historical inherited driver context follows:', 'W-CLAMP eachbound100untimedoutwarddetents,5swarmup,1800detents60Hz30s,5sstill,oneinwardpositivecontrol. Exactcamera numericstate/continuouspresentation notexported;no fullqualification.', 'GPU timestamp/driver-client duty counters not collected here. Existing unsupported NVIDIA counter boundary remains.', 'Exact final window pixels/focus are endpoints, not calibrated physical presentation latency.', 'Sampled overlapping resource views and conservative reservation peaks do not establish complete memory/subcap/endurance/backend/scale/independent acceptance. CPU includes GUI and cooperating descendants through owned cgroup; snapshot RSS is GUI PID only.']
model=json.loads(Path('/tmp/pm045-admission-hqugp169/project/board/board.json').read_text());model.pop('uuid',None)
assert hashlib.sha256(json.dumps(model,sort_keys=True,separators=(',',':')).encode()).hexdigest()=='33e62de1c1da2020f0608444a4802eac23fb97a9f56cc8cf87c844b6077499ed'
def save(): (out/'result.json').write_text(json.dumps(report,indent=2)+'\n')
def xd(*args):
 r=subprocess.run(['xdotool',*map(str,args)],capture_output=True,text=True)
 if r.returncode and (not args or args[0]!='search'):raise RuntimeError(r.stderr)
 return r.stdout.strip()
def lines():
 data=log.read_text()
 return data[:data.rfind('\n')+1].splitlines()
def parsed(rows,tag):return [json.loads(s.split(tag,1)[1]) for s in rows if tag in s]
def until(fn):
 end=time.monotonic()+15
 while time.monotonic()<end:
  assert p.poll() is None,('process exited',p.returncode)
  v=fn()
  if v:return v
  time.sleep(.025)
 raise AssertionError('native readiness/closure timed out')
group=Path('/sys/fs/cgroup/user.slice')/f'user-{os.getuid()}.slice'/f'user@{os.getuid()}.service'/('datum-pm045-resource-windows-'+uuid.uuid4().hex)
group.mkdir()

def group_counts():return {k:int(v) for k,v in (line.split() for line in (group/'cpu.stat').read_text().splitlines())}
report['cpu_observer']={'group':str(group),'before_spawn':group_counts(),'samples':[]}
libc=ctypes.CDLL(None,use_errno=True)
def cpu():
 clock=ctypes.c_int()
 rc=libc.clock_getcpuclockid(p.pid,ctypes.byref(clock));assert rc==0,rc
 stat=Path(f'/proc/{p.pid}/stat').read_text().rsplit(')',1)[1].split()
 sample={'monotonic_ns':time.monotonic_ns(),'group':group_counts(),'gui_cpu_ns':time.clock_gettime_ns(clock.value),'gui_pid':p.pid,'gui_start_ticks':int(stat[19]),'members':(group/'cgroup.procs').read_text().split()}
 report['cpu_observer']['samples'].append(sample)
 return sample['group']['usage_usec']/1000000

listener=socket.socket(socket.AF_UNIX,socket.SOCK_STREAM)
listener.bind(str(out/'observer.sock'));listener.listen(1);listener.settimeout(10)
env['DATUM_MEASUREMENT_SHUTDOWN_SOCKET']=str(out/'observer.sock')

def enter_group():(group/'cgroup.procs').write_text(str(os.getpid()))
def observe_pixels(wid,cycle,host,phase):
 ref=root/f'docs/reviews/gui-performance/implementation/S5/resolver-cache-ownership/native/{host}.png'
 attempts=[]
 for attempt in range(15):
  path=out/f'{host}-{phase}-{cycle}-{attempt}.png'
  subprocess.run(['import','-window',str(wid),str(path)],check=True)
  r=subprocess.run(['compare','-metric','AE',str(ref),str(path),'null:'],capture_output=True,text=True)
  attempts.append({'monotonic_ns':time.monotonic_ns(),'capture':str(path),'pixel_difference':r.stderr,'returncode':r.returncode})
  if r.returncode==0:return attempts
  time.sleep(.025)
 raise AssertionError(('expected window pixels not observed',attempts))
def rss():return {s.split(':')[0]:s.split(':')[1].strip() for s in Path(f'/proc/{p.pid}/status').read_text().splitlines() if s.startswith(('VmRSS:','VmHWM:'))}
with (out/'stderr.log').open('w') as stream:
 env['DATUM_TRACE_CGROUP']=str(group/'cgroup.procs')
 env['DATUM_FD_TRACE_MODE']='off' if mode=='plain' else 'on'
 trace_path=out/'process.jsonl'
 report['launch_ns']=time.monotonic_ns()
 monitor=subprocess.Popen(['/tmp/pm045_fd_trace',str(trace_path),*cmd],cwd=root,env=env,stdout=stream,stderr=stream)
 try:
  end=time.monotonic()+10
  while True:
   assert monitor.poll() is None,('process supervisor exited',monitor.returncode)
   first=trace_path.read_text().splitlines() if trace_path.exists() else []
   if first and first[0].endswith('}'):
    root_record=json.loads(first[0]);break
   assert time.monotonic()<end,'process supervisor did not identify child'
   time.sleep(.01)
 except BaseException as error:
  (group/'cgroup.kill').write_text('1');monitor.wait(timeout=10)
  if not (group/'cgroup.procs').read_text().strip():group.rmdir()
  listener.close();(out/'observer.sock').unlink(missing_ok=True)
  report['error']=repr(error);report['startup_failed']=True;save();raise
 from types import SimpleNamespace
 import signal
 def signal_root(sig):
  try:os.kill(root_record['pid'],sig)
  except ProcessLookupError:pass
 class TracedProcess:
  pid=root_record['pid']
  @property
  def returncode(self):return monitor.returncode
  def poll(self):return monitor.poll()
  def wait(self,timeout=None):return monitor.wait(timeout=timeout)
  def terminate(self):signal_root(signal.SIGTERM)
  def kill(self):signal_root(signal.SIGKILL)
 p=TracedProcess()
 report['process_supervisor']=root_record
 try:
  main=int(until(lambda:xd('search','--all','--onlyvisible','--pid',p.pid,'--name','^Datum EDA')) .splitlines()[0])
  report['main_window']=main
  until(lambda:'frame present end' in log.read_text())
  report['main_window']=main;report['action_log_start']=len(lines());report['controls']=[]
  from PIL import Image
  from pm045_wm_close import close_window
  import atexit
  x=ctypes.CDLL('libX11.so.6');x.XOpenDisplay.argtypes=[ctypes.c_char_p];x.XOpenDisplay.restype=ctypes.c_void_p
  x.XDefaultRootWindow.argtypes=[ctypes.c_void_p];x.XDefaultRootWindow.restype=ctypes.c_ulong
  x.XCreateSimpleWindow.argtypes=[ctypes.c_void_p,ctypes.c_ulong,ctypes.c_int,ctypes.c_int,ctypes.c_uint,ctypes.c_uint,ctypes.c_uint,ctypes.c_ulong,ctypes.c_ulong];x.XCreateSimpleWindow.restype=ctypes.c_ulong
  x.XMapWindow.argtypes=[ctypes.c_void_p,ctypes.c_ulong];x.XFlush.argtypes=[ctypes.c_void_p];x.XDestroyWindow.argtypes=[ctypes.c_void_p,ctypes.c_ulong];x.XCloseDisplay.argtypes=[ctypes.c_void_p]
  display=x.XOpenDisplay(None);assert display
  helper=x.XCreateSimpleWindow(display,x.XDefaultRootWindow(display),10,10,100,60,0,0,0);x.XMapWindow(display,helper);x.XFlush(display)
  def cleanup_helper():
   xd('mouseup',1);x.XDestroyWindow(display,helper);x.XCloseDisplay(display)
  atexit.register(cleanup_helper)
  for host in (os.environ['PM045_CONTROLS_HOST'],):
   title={'GLOBAL':'Global Preferences','PROJECT':'Project Preferences'}[host]
   wid=int(until(lambda:next((w for w in xd('search','--all','--onlyvisible','--pid',p.pid,'--name','^'+title).splitlines() if int(w)!=main),None)))
   xd('windowactivate','--sync',wid);until(lambda:int(xd('getwindowfocus'))==wid);observe_pixels(wid,0,host,'initial')
   reference=Path(os.environ['PM045_CONTROLS_REFERENCE'])/host
   case={'host':host,'window':wid,'actions':[]};report['controls'].append(case);save()
   if host=='GLOBAL':xd('mousemove','--window',wid,50,107,'click',1);time.sleep(.3)
   def capture(name):
    png=out/(host+'-'+str(cycle)+'-'+name+'.png')
    completed=subprocess.run(['spectacle','-b','-n','-a','-e','-S','-o',str(png)],capture_output=True,text=True,timeout=15)
    assert completed.returncode==0,(completed.returncode,completed.stderr)
    assert Image.open(png).size==(960,720),(name,Image.open(png).size)
    case.setdefault('compositor_captures',[]).append({'name':name,'path':png.name,'cycle':cycle,'stderr':completed.stderr});save()
    return png
   xt=ctypes.CDLL('libXtst.so.6');xt.XTestFakeMotionEvent.argtypes=[ctypes.c_void_p,ctypes.c_int,ctypes.c_int,ctypes.c_int,ctypes.c_ulong];xt.XTestFakeButtonEvent.argtypes=[ctypes.c_void_p,ctypes.c_uint,ctypes.c_int,ctypes.c_ulong];xt.XTestFakeKeyEvent.argtypes=[ctypes.c_void_p,ctypes.c_uint,ctypes.c_int,ctypes.c_ulong]
   x.XStringToKeysym.argtypes=[ctypes.c_char_p];x.XStringToKeysym.restype=ctypes.c_ulong;x.XKeysymToKeycode.argtypes=[ctypes.c_void_p,ctypes.c_ulong];x.XKeysymToKeycode.restype=ctypes.c_uint;x.XSync.argtypes=[ctypes.c_void_p,ctypes.c_int]
   def send_key(name,down):
    key=x.XKeysymToKeycode(display,x.XStringToKeysym(name.encode()));assert key;assert xt.XTestFakeKeyEvent(display,key,int(down),0)
   def drive(args,offset,expect_logged=True):
    args=list(args);expected=[];keyboard=False
    if args[0]=='mousemove':
     assert args[1]=='--window';geo=dict(line.split('=',1) for line in xd('getwindowgeometry','--shell',wid).splitlines() if '=' in line)
     assert xt.XTestFakeMotionEvent(display,-1,int(geo['X'])+int(args[3]),int(geo['Y'])+int(args[4]),0);args=args[5:]
    if args and args[0] in ('click','mousedown','mouseup'):
     button=int(args[1]);assert button==1
     if args[0] in ('click','mousedown'):assert xt.XTestFakeButtonEvent(display,button,1,0);expected.append('mouse input Left Pressed')
     if args[0] in ('click','mouseup'):assert xt.XTestFakeButtonEvent(display,button,0,0);expected.append('mouse input Left Released')
    elif args and args[0] in ('key','type'):
     keyboard=True;kind=args.pop(0);delay=.03
     if args[:1]==['--delay']:delay=int(args[1])/1000;args=args[2:]
     keys=list(args[0]) if kind=='type' else args
     for key in keys:
      shifted=key.startswith('shift+');name=key[6:] if shifted else key
      if shifted:send_key('Shift_L',True)
      send_key(name,True);send_key(name,False)
      if shifted:send_key('Shift_L',False)
      expected.append('keyboard physical=Code('+('Key'+name.upper() if len(name)==1 else {'Return':'Enter'}.get(name,name))+') ')
      x.XFlush(display);time.sleep(delay)
    x.XSync(display,0)
    from collections import Counter
    required=Counter(expected)
    def delivered():
     
     with log.open('rb') as f:f.seek(offset);rows=f.read().decode().splitlines()
     matched=[line for line in rows if f'window event WindowId({wid}) ' in line]
     return all(sum(tag in line and (not keyboard or 'state=Pressed' in line) for line in matched)>=count for tag,count in required.items())
    if expect_logged:until(delivered)
   def pixels_equal(actual,expected,name):
    from PIL import ImageChops
    a=Image.open(actual).convert('RGB');b=Image.open(expected).convert('RGB');assert a.size==b.size
    # Notice text is legitimate announcement state carried across cycles. Its
    # fixed rectangle alone is excluded; content, scrollbars and focus remain.
    if host=='GLOBAL':
     a.paste((0,0,0),(211,101,948,146));b.paste((0,0,0),(211,101,948,146))
    if name=='search-focus':a=a.crop((210,42,960,100));b=b.crop((210,42,960,100))
    bounds=ImageChops.difference(a,b).getbbox();case.setdefault('visual_checks',[]).append({'cycle':cycle,'phase':name,'reference':str(expected),'difference_bounds':bounds});save()
    assert bounds is None,(host,cycle,name,bounds)
   def action(name,*args):
    offset=log.stat().st_size;c0=cpu();t0=time.monotonic_ns();drive(args,offset,name!='cancel-release');time.sleep(.3);c1=cpu();t1=time.monotonic_ns()
    png=capture(name);case['actions'].append({'cycle':cycle,'name':name,'input':args,'cpu_ms':(c1-c0)*1000,'start_ns':t0,'end_ns':t1,'start_log_offset':offset,'end_log_offset':log.stat().st_size,'capture':png.name});save();pixels_equal(png,reference/(host+'-'+name+'.png'),name);return png
   time.sleep(5)
   for cycle in range(1,31):
    case['current_cycle']=cycle;save()
    action('search-focus','mousemove','--window',wid,400,70,'click',1)
    action('last-name','key','--delay',90,'shift+Tab','shift+Tab','shift+Tab')
    action('expanded','key','Return')
    action('collapsed','key','Escape')
    action('search-refocus','mousemove','--window',wid,400,70,'click',1)
    clip_before=out/(host+'-'+str(cycle)+'-search-refocus.png')
    clip_y=70 if host=='GLOBAL' else 7
    clip_offset=log.stat().st_size;drive(['mousemove','--window',wid,900,clip_y,'click',1],clip_offset);time.sleep(.3)
    clip_after=capture('clip-probe');pixels_equal(clip_after,clip_before,'clip-probe')
    case.setdefault('clip_probes',[]).append({'cycle':cycle,'x':900,'y':clip_y,'expected':'Pinned Search (Global) or header (Project); no choice activation, exact unchanged image','begin_log_offset':clip_offset,'end_log_offset':log.stat().st_size});save()
    action('search','type','--delay',30,'precision')
    action('search-cleared','key','Escape')
    scan_index=[0]
    def thumb():
     scan_index[0]+=1;png=capture('thumb-scan-'+str(scan_index[0]));im=Image.open(png).convert('RGB');assert im.size==(960,720)
     ys=[y for y in range(100,720) if im.getpixel((954,y))==(178,184,195)]
     runs=[]
     for y in ys:
      if not runs or y!=runs[-1][-1]+1:runs.append([])
      runs[-1].append(y)
     substantial=[r for r in runs if len(r)>=32];assert len(substantial)==1,[(min(r),max(r)) for r in runs]
     thumb_run=substantial[0];return min(thumb_run),max(thumb_run)+1
    for fraction,delta in [(.25,20),(.75,-20)]:
     top,bottom=thumb();grab=round(top+(bottom-top)*fraction);name='grab-'+str(fraction)
     action(name,'mousemove','--window',wid,954,grab,'mousedown',1)
     action(name+'-drag','mousemove','--window',wid,954,grab+delta)
     action(name+'-release','mouseup',1)
    top,bottom=thumb();page_y=min(718,bottom+12) if bottom<708 else max(101,top-12)
    assert page_y<top or page_y>=bottom
    action('track-page','mousemove','--window',wid,954,page_y,'click',1)
    top,bottom=thumb();grab=round(top+(bottom-top)*.5)
    action('cancel-grab','mousemove','--window',wid,954,grab,'mousedown',1)
    focus_offset=log.stat().st_size;xd('windowactivate','--sync',helper)
    until(lambda:f'window event WindowId({wid}) focused false' in log.read_bytes()[focus_offset:].decode())
    xd('windowactivate','--sync',wid);until(lambda:int(xd('getwindowfocus'))==wid);time.sleep(.3);before=capture('cancel-before-motion')
    after=action('cancel-after-motion','mousemove','--window',wid,954,min(710,grab+20))
    cmp=subprocess.run(['compare','-metric','AE',str(before),str(after),'null:'],capture_output=True,text=True);case['cancelled_drag_AE']=cmp.stderr.strip();save();assert cmp.returncode==0,cmp.stderr
    action('cancel-release','mouseup',1)
    case['completed_cycles']=cycle;save();print(json.dumps({'host':host,'completed_cycles':cycle}),flush=True)
   close_window(wid)
   until(lambda:subprocess.run(['xwininfo','-id',str(wid)],stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL).returncode!=0)
   time.sleep(.5);case['automatic_focus_after_close']=int(xd('getwindowfocus'));case['automatic_focus_restored']=case['automatic_focus_after_close']==main;save()
   assert case['automatic_focus_restored'],'automatic Main focus restoration failed'
   if not case['automatic_focus_restored']:
    case['cleanup_manual_focus_required']=True;xd('windowactivate','--sync',main);until(lambda:int(xd('getwindowfocus'))==main)
   case['closed']=True;save()
  report['passed_native_cycles']=True
  action_end=cpu();drain_start=time.monotonic_ns()
  assert int(xd('getwindowfocus'))==main
  from pm045_wm_close import close_window
  close_window(main)
  connection,_=listener.accept();connection.settimeout(2)
  receipt=b''
  while not receipt.endswith(b'\n'):
   part=connection.recv(1);assert part,'missing drained receipt';receipt+=part
  report['shutdown_receipt']=json.loads(receipt)
  assert report['shutdown_receipt']['pid']==p.pid
  assert report['shutdown_receipt']['phase']=='drained_device_live'
  drain_end=cpu()
  report['drain']={'begin_ns':drain_start,'end_ns':time.monotonic_ns(),'cpu_ms':(drain_end-action_end)*1000,'boundary':'Controlled terminal shutdown and bounded shared GPU queue drain; final counters while device live'}
  connection.sendall(b'!');connection.close()
  p.wait(timeout=10);report['normal_exit_code']=p.returncode
  assert p.returncode==0,p.returncode
  rows=log.read_text().splitlines()
  report['gpu_samples']=[json.loads(line.split('gpu_measurement ',1)[1]) for line in rows if 'gpu_measurement ' in line]
  report['gpu_incomplete']=[line for line in rows if 'gpu_measurement_incomplete ' in line]
  report['gpu_hosts']=[line for line in rows if 'gpu_measurement_host ' in line]
  report['action_gpu_incomplete']=[line for line in rows[report['action_log_start']:] if 'gpu_measurement_incomplete ' in line]
  assert not report['action_gpu_incomplete'],report['action_gpu_incomplete']
  if env['DATUM_GPU_MEASUREMENTS']=='1':assert report['gpu_samples'],'missing GPU diagnostic timestamps'
  else:assert not report['gpu_samples'],'unexpected GPU timestamps'

 except Exception as e:
  report['error']=repr(e);print(repr(e),flush=True)
  if p.poll() is None:
   fd_state={'started_ns':time.monotonic_ns(),'descriptors':[]}
   for entry in sorted(Path(f'/proc/{p.pid}/fd').iterdir()):
    row={'fd':entry.name}
    try:row.update(target=os.readlink(entry),fdinfo=Path(f'/proc/{p.pid}/fdinfo/{entry.name}').read_text())
    except OSError as error:row['error']=repr(error)
    fd_state['descriptors'].append(row)
   proc=subprocess.run(['ss','-xnp'],capture_output=True,text=True);fd_state['ss_returncode']=proc.returncode;fd_state['ss_stderr']=proc.stderr;fd_state['socket_rows']=[line for line in proc.stdout.splitlines() if f'pid={p.pid},' in line];fd_state['finished_ns']=time.monotonic_ns()
   (out/'fd-state.json').write_text(json.dumps(fd_state,indent=2)+'\n')
   with (out/'gdb.txt').open('w') as diagnostic:
    try:report['gdb_returncode']=subprocess.run(['gdb','-q','-nx','-batch','-ex','set pagination off','-ex','set debuginfod enabled off','-ex','thread apply all bt','-ex','detach','-p',str(p.pid)],stdout=diagnostic,stderr=subprocess.STDOUT,timeout=25).returncode
    except subprocess.TimeoutExpired:report['gdb_timeout']=True
 finally:
  report['rss_final']=rss() if p.poll() is None and Path(f'/proc/{p.pid}/status').exists() else None
  if p.poll() is None:
   p.terminate()
   try:p.wait(timeout=5)
   except subprocess.TimeoutExpired:
    (group/'cgroup.kill').write_text('1');monitor.wait(timeout=10)
  report['cpu_observer']['after_root_exit']=group_counts()
  remaining=(group/'cgroup.procs').read_text().strip()
  report['cpu_observer']['remaining_before_cleanup']=remaining
  if remaining:
   (group/'cgroup.kill').write_text('1')
   for _ in range(100):
    if not (group/'cgroup.procs').read_text().strip():break
    time.sleep(.01)
  report['cpu_observer']['after_cleanup']=group_counts()
  if not (group/'cgroup.procs').read_text().strip():group.rmdir()
  listener.close();(out/'observer.sock').unlink(missing_ok=True)
  report['cleanup']='Owned observer closed and process/group removed; normal exit recorded separately, forced cleanup is not qualification.';save()
  assert hashlib.sha256(binary.read_bytes()).hexdigest()==report['binary_sha256']
  report['process_ledger_lines']=len(trace_path.read_text().splitlines());save()

if report.get('error'):raise SystemExit(1)
