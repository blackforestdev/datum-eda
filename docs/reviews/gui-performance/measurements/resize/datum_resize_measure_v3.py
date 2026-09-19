import argparse,ctypes as c,hashlib,json,os,pathlib,re,shutil,statistics,subprocess,tempfile,time
P=argparse.ArgumentParser();P.add_argument('--mode',choices=['main','global','project','new'],required=True);P.add_argument('--seconds',type=float,default=10);P.add_argument('--trials',type=int,default=3);P.add_argument('--output',required=True);P.add_argument('--trace',action='store_true');P.add_argument('--idle-only',action='store_true');P.add_argument('--action-evidence',action='store_true');P.add_argument('--phases',default='pan,focus,resize,pane_cycle,new_cycle');a=P.parse_args()
root=pathlib.Path('/home/bfadmin/Documents/datum-eda');out=pathlib.Path(a.output);out.mkdir(parents=True,exist_ok=False)
board_source=pathlib.Path('/home/bfadmin/Documents/kicad_projects/DOA2526/hardware/DOA2526/DOA2526.kicad_pcb');board=pathlib.Path(os.environ.get('DATUM_MEASUREMENT_BOARD',out/'DOA2526.kicad_pcb'))
if 'DATUM_MEASUREMENT_BOARD' not in os.environ:shutil.copy2(board_source,board)
binary=pathlib.Path(os.environ.get('DATUM_MEASUREMENT_BINARY',root/'target/release/datum-gui'));binary_hash_at_start=hashlib.sha256(binary.read_bytes()).hexdigest();env=dict(os.environ,TMPDIR=str(out),XDG_CONFIG_HOME=str(out/'config'),XDG_CACHE_HOME=str(out/'cache'),EDA_CLI_BIN=str(root/'target/debug/datum-eda'),WINIT_UNIX_BACKEND='x11')
for k in ['WAYLAND_DISPLAY','DATUM_ENGINE_SOCKET','EDA_ENGINE_SOCKET','DATUM_GUI_PREFERENCES_PATH','DATUM_TRACE_TIMING']:env.pop(k,None)
if a.trace:env['DATUM_TRACE_TIMING']='1'
if a.action_evidence:env['DATUM_ACTION_EVIDENCE']='1'
else:env.pop('DATUM_ACTION_EVIDENCE',None)
x=c.CDLL('libX11.so.6');xt=c.CDLL('libXtst.so.6');x.XOpenDisplay.argtypes=[c.c_char_p];x.XOpenDisplay.restype=c.c_void_p;x.XWarpPointer.argtypes=[c.c_void_p,c.c_ulong,c.c_ulong,c.c_int,c.c_int,c.c_uint,c.c_uint,c.c_int,c.c_int];x.XFlush.argtypes=[c.c_void_p];x.XCloseDisplay.argtypes=[c.c_void_p];x.XResizeWindow.argtypes=[c.c_void_p,c.c_ulong,c.c_uint,c.c_uint];xt.XTestFakeButtonEvent.argtypes=[c.c_void_p,c.c_uint,c.c_int,c.c_ulong]
d=x.XOpenDisplay(env['DISPLAY'].encode());assert d
class XIMask(c.Structure):
 _fields_=[('deviceid',c.c_int),('mask_len',c.c_int),('mask',c.POINTER(c.c_ubyte))]
class Cookie(c.Structure):
 _fields_=[('type',c.c_int),('serial',c.c_ulong),('send_event',c.c_int),('display',c.c_void_p),('extension',c.c_int),('evtype',c.c_int),('cookie',c.c_uint),('data',c.c_void_p)]
xi=c.CDLL('libXi.so.6');xi.XIQueryVersion.argtypes=[c.c_void_p,c.POINTER(c.c_int),c.POINTER(c.c_int)];xi.XISelectEvents.argtypes=[c.c_void_p,c.c_ulong,c.POINTER(XIMask),c.c_int]
x.XPending.argtypes=[c.c_void_p];x.XNextEvent.argtypes=[c.c_void_p,c.c_void_p];x.XSync.argtypes=[c.c_void_p,c.c_int]
major=c.c_int(2);minor=c.c_int(0);assert xi.XIQueryVersion(d,c.byref(major),c.byref(minor))==0
input_totals={str(i):0 for i in range(2,7)}
def observe(w):
 bits=(c.c_ubyte*1)(sum(1<<i for i in range(2,7)));mask=XIMask(1,1,bits)
 assert xi.XISelectEvents(d,w,c.byref(mask),1)==0
 x.XSync(d,0)
def received_input():
 event=(c.c_long*24)()
 while x.XPending(d):
  x.XNextEvent(d,c.byref(event));v=c.cast(c.byref(event),c.POINTER(Cookie)).contents
  if v.type==35 and str(v.evtype) in input_totals:input_totals[str(v.evtype)]+=1
 return dict(input_totals)
hz=os.sysconf('SC_CLK_TCK');logpath=out/'gui.log'
def cmd(*args):return subprocess.check_output(args,env=env,text=True,timeout=10).strip()
def capture(window,path):
 if int(cmd('xdotool','getwindowfocus'))!=window:raise RuntimeError('Owned test window lost focus; refusing capture')
 capture_env=dict(env,WAYLAND_DISPLAY=os.environ['WAYLAND_DISPLAY'])
 subprocess.run(['spectacle','--background','--nonotify','--activewindow','--no-decoration','--no-shadow','--output',str(path)],env=capture_env,check=True,timeout=15)
def pixels(path):
 return subprocess.check_output(['convert',str(path),'-depth','8','rgba:-'],env=env,timeout=10)
def ticks(pid):
 fields=pathlib.Path(f'/proc/{pid}/stat').read_text().rsplit(')',1)[1].split();return int(fields[11])+int(fields[12])
def rss(pid):
 return int(next(l.split()[1] for l in pathlib.Path(f'/proc/{pid}/status').read_text().splitlines() if l.startswith('VmRSS:')))
def drm(pid):
 counters={}
 for p in pathlib.Path(f'/proc/{pid}/fdinfo').iterdir():
  try:fields=dict(line.split(':',1) for line in p.read_text().splitlines() if ':' in line)
  except OSError:continue
  if 'drm-client-id' not in fields:continue
  for k,v in fields.items():
   if k.startswith('drm-engine-') and v.strip().endswith(' ns'):
    key=fields.get('drm-pdev','').strip()+'/'+fields['drm-client-id'].strip()+'/'+k;counters[key]=max(counters.get(key,0),int(v.split()[0]))
 return counters
def button(n):
 xt.XTestFakeButtonEvent(d,n,1,0);xt.XTestFakeButtonEvent(d,n,0,0);x.XFlush(d)
def move(w,px,py):
 x.XWarpPointer(d,0,w,0,0,0,0,px,py);x.XFlush(d)
def key(k):cmd('xdotool','key',k)
def click(w,px,py):
 move(w,px,py);time.sleep(.04);button(1);time.sleep(.08)
def menu(w,row):
 click(w,190,17);click(w,270,34+4+row*32+16)
def child_cpu(pid):
 # Linux parent stat retains waited-for child CPU; include live descendants.
 total=0;live=[];todo=[pid]
 while todo:
  current=todo.pop()
  try:
   f=pathlib.Path(f'/proc/{current}/stat').read_text().rsplit(')',1)[1].split()
   total+=int(f[13])+int(f[14])
   if current!=pid:total+=int(f[11])+int(f[12]);live.append(current)
   todo += [int(v) for v in pathlib.Path(f'/proc/{current}/task/{current}/children').read_text().split()]
  except OSError:continue
 return {'ticks':total,'live_pids':live}
def action(w,name,n):
 if name in ('height','width'):
  delta=abs((n%120)-60)*4
  x.XResizeWindow(d,w,1280+delta if name=='width' else 1280,800+delta if name=='height' else 800);x.XFlush(d)
 elif name=='pan':move(w,370+(n%60),370+(n%30))
 elif name=='focus':key('Tab')
 elif name=='resize':cmd('xdotool','windowsize',str(w),'1280' if n%2 else '1200','800' if n%2 else '760')
 elif name=='pane_cycle':menu(w,8 if n%2==0 else 10)
 elif name=='new_cycle':
  if n%2==0:
   click(w,110,17);click(w,165,54)
   for _ in range(50):
    found=cmd('xdotool','search','--onlyvisible','--pid',str(p.pid)).split()
    if len(found)>1:break
    time.sleep(.05)
   if len(found)<2:raise RuntimeError('New Project did not open')
  else:key('Escape');time.sleep(.1);cmd('xdotool','windowactivate','--sync',str(w))
 elif name=='scroll':button(5 if n%2==0 else 4)
samples=[]
def sample(pid,window,name,rate=60):
 # Screenshots are output evidence, not compositor feedback.
 cmd('xdotool','windowactivate','--sync',str(window));time.sleep(.4)
 x.XWarpPointer(d,0,window,0,0,0,0,420,400);x.XFlush(d);time.sleep(.4)

 if name=='pan':
  move(window,370,370);cmd('xdotool','keydown','space');xt.XTestFakeButtonEvent(d,1,1,0);x.XFlush(d);time.sleep(.1)
 diag_path=out/'datum-gui-last.log';diag_offset=diag_path.stat().st_size if diag_path.exists() else 0
 observed_start=received_input()
 child_start=child_cpu(pid)
 start_ticks=ticks(pid);start_gpu=drm(pid);peak=rss(pid);start_offset=logpath.stat().st_size;start=time.monotonic();sent=0;timeline=[]
 while time.monotonic()-start<a.seconds:
  if name!='idle':action(window,name,sent)
  x.XFlush(d)
  if name!='idle':sent+=1
  t=time.monotonic()-start
  if not timeline or t-timeline[-1]['seconds']>=.1:
   peak=max(peak,rss(pid));timeline.append({'seconds':t,'cpu_ticks':ticks(pid),'rss_kib':rss(pid),'gpu':drm(pid),'received_xi_events':received_input()})
  time.sleep(max(0,min(1/rate,a.seconds-(time.monotonic()-start))))
 elapsed=time.monotonic()-start;cpu=(ticks(pid)-start_ticks)/hz;gpu_end=drm(pid);log=logpath.read_bytes()[start_offset:].decode(errors='replace')
 frames=re.findall(r'\[datum-timing\] runtime render (.*)',log);dialogs=[int(v) for v in re.findall(r'dialog renderer=(\d+)us',log)]
 sample={'phase':name,'duration_s':elapsed,'input_events_sent':sent,'requested_hz':rate,'rate_semantics':'minimum delay between actions; actions may include explicit settling waits','cpu_seconds':cpu,'cpu_percent_one_core':cpu/elapsed*100,'rss_peak_kib':peak,'gpu_engine_active_percent':{k:max(0,v-start_gpu[k])/(elapsed*1e9)*100 for k,v in gpu_end.items() if k in start_gpu},'world_bundle_rebuilds':log.count('world bundle rebuilt') if a.trace else None,'main_submissions':len(frames) if a.trace else None,'dialog_submissions':len(dialogs) if a.trace else None,'prepared_miss_frames':sum('prepared_was_cached=false' in v for v in frames) if a.trace else None,'retained_miss_frames':sum('retained_was_cached=false' in v for v in frames) if a.trace else None,'control_reverse':None,'preflight':'separate compositor before/after capture, not per-event display acknowledgement','main_renderer_ms':[int(re.search(r'renderer=(\d+)ms',v)[1]) for v in frames],'dialog_renderer_us':dialogs,'raw_counters':timeline}

 diag=diag_path.read_bytes()[diag_offset:].decode(errors='replace') if diag_path.exists() else ''
 sample['prepared_builds']=diag.count('prepared scene build begin');sample['retained_builds']=diag.count('retained scene build begin');sample['surface_recoveries']=diag.count('surface acquire recovered');sample['surface_timeouts']=diag.count('surface acquire timeout')
 sample['surface_configurations']=diag.count('surface configure begin');sample['resize_applications']=diag.count('resize apply');sample['final_gui_size']=re.findall(r'resize apply .*? -> (\d+x\d+)',diag)[-1:]
 sample['cpu_sample_interval_s']=.1;sample['clock_ticks_per_second']=hz
 sample['received_xi_events']={k:v-observed_start[k] for k,v in received_input().items()}
 sample['gpu_counter_scope']='Surviving DRM clients only; window create/destroy may remove clients and is not complete GPU duty.'
 sample['engine_children_start']=child_start;sample['engine_children_end']=child_cpu(pid)
 sample['engine_cpu_seconds']=(sample['engine_children_end']['ticks']-child_start['ticks'])/hz
 if name=='pan':xt.XTestFakeButtonEvent(d,1,0,0);x.XFlush(d);cmd('xdotool','keyup','space')
 if name in ('pane_cycle','new_cycle') and sent%2:action(window,name,sent)
 samples.append(sample);(out/'samples.json').write_text(json.dumps(samples,indent=2)+'\n');print(json.dumps({k:v for k,v in sample.items() if k not in ('raw_counters','main_renderer_ms','dialog_renderer_us')}),flush=True)
flags={'main':[],'global':['--open-global-preferences'],'project':['--open-project-preferences'],'new':['--open-new-project']}[a.mode]
with logpath.open('w') as log:
 p=subprocess.Popen([str(binary),'--board',str(board),'--window-size','1280x800',*flags],cwd=root,env=env,stdout=log,stderr=log)
 try:
  windows=[]
  for _ in range(200):
   if p.poll() is not None:raise RuntimeError('GUI exited: '+logpath.read_text()[-2000:])
   found=subprocess.run(['xdotool','search','--onlyvisible','--pid',str(p.pid)],env=env,capture_output=True,text=True).stdout.split();windows=[(int(w),cmd('xdotool','getwindowname',w)) for w in found]
   if len(windows)>=(1 if a.mode=='main' else 2):break
   time.sleep(.1)
  assert windows
  main=next(w for w,title in windows if 'Preferences' not in title and 'New Project' not in title)
  owned=next((w for w,title in windows if w!=main),None)
  cmd('xdotool','windowmove',str(main),'20','40');cmd('xdotool','windowsize',str(main),'1280','800')
  if owned:cmd('xdotool','windowmove',str(owned),'1320','40')
  target=main if a.mode=='main' else owned
  assert target is not None
  if owned:
   cmd('xdotool','windowmove',str(owned),'100','100');cmd('xdotool','windowsize',str(owned),'700','540');cmd('xdotool','windowactivate','--sync',str(owned))
   time.sleep(2)
   if a.mode=='global':click(owned,180,65)
   time.sleep(.5)
   move(owned,420,400);button(5);button(5)
  observe(target)
  time.sleep(5)
  sample(p.pid,target,'idle',rate=10)
  phases=[] if a.idle_only else a.phases.split(',') if a.mode=='main' else ['scroll']
  # Preflight visible change outside timing; full capture is not latency proof.
  for phase in phases:
   cmd('xdotool','windowactivate','--sync',str(target));move(target,420,400);time.sleep(.4)
   before=out/(phase+'-before.png');capture(target,before)
   if phase=='pan':
    move(target,370,370);cmd('xdotool','keydown','space');xt.XTestFakeButtonEvent(d,1,1,0);x.XFlush(d);time.sleep(.1)
    move(target,420,400);time.sleep(.2);xt.XTestFakeButtonEvent(d,1,0,0);x.XFlush(d);cmd('xdotool','keyup','space')
   else:action(target,phase,0)
   time.sleep(.6)
   capture_target=target
   if phase=='new_cycle':capture_target=next(int(v) for v in cmd('xdotool','search','--onlyvisible','--pid',str(p.pid)).split() if int(v)!=main)
   after=out/(phase+'-after.png');capture(capture_target,after)
   changed=pixels(before)!=pixels(after)
   if not changed:raise RuntimeError('Visible preflight failed: '+phase)
   if phase in ('pane_cycle','new_cycle','scroll'):action(target,phase,1)
   time.sleep(5)
   for trial in range(a.trials):sample(p.pid,target,phase,60 if phase in ('pan','height','width') else 20 if phase=='scroll' else 2)
  cmd('xdotool','windowsize',str(target),'1280','800');time.sleep(3)
  cmd('xdotool','windowactivate','--sync',str(target));move(target,420,400);time.sleep(1)
  capture(target,out/'settled-final.png')
  (out/'final-window-geometry.txt').write_text(cmd('xdotool','getwindowgeometry','--shell',str(target))+'\n')
  for _ in range(int(os.environ.get('DATUM_RESIZE_IDLE_REPEATS','3'))):sample(p.pid,target,'idle',rate=10)

 finally:
  if p.poll() is None:p.terminate();p.wait(timeout=10)
  x.XCloseDisplay(d)
report={'candidate':cmd('git','rev-parse','HEAD'),'binary':str(binary),'binary_sha256':binary_hash_at_start,'binary_sha256_at_end':hashlib.sha256(binary.read_bytes()).hexdigest(),'fixture_sha256':hashlib.sha256(board.read_bytes()).hexdigest(),'fixture_source':str(board_source),'fixture_runtime_path':str(board),'mode':a.mode,'trace_enabled':a.trace,'input_observer':'XI2 events2 keypress/3 keyrelease/4 buttonpress/5 buttonrelease/6 motion, selected owned target only; counts without key content; observer delivery is not application acknowledgement','action_evidence':a.action_evidence,'backend':'X11/Xwayland on existing desktop; synthetic XTest input, not native Wayland qualification','platform':os.uname()._asdict() if hasattr(os.uname(),'_asdict') else list(os.uname()),'windows':windows,'samples':samples,'limitations':['Sent input is not acknowledged delivery; screenshots/traces are not compositor feedback.','GPU engine busy time is not frequency-normalized utilization or per-frame GPU timing.','GUI CPU includes all threads; child CPU includes waited children and observed descendants; unrelated daemons excluded; process exit races remain possible.','Timing log overhead exists when trace_enabled; absolute acceptance not asserted.']};(out/'report.json').write_text(json.dumps(report,indent=2)+'\n')
print('REPORT',out/'report.json',flush=True)
