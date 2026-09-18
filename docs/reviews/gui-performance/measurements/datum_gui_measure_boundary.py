import argparse,ctypes as c,hashlib,json,os,pathlib,re,shutil,statistics,subprocess,tempfile,time
P=argparse.ArgumentParser();P.add_argument('--mode',choices=['main','global','project','new'],required=True);P.add_argument('--seconds',type=float,default=10);P.add_argument('--trials',type=int,default=3);P.add_argument('--output',required=True);P.add_argument('--trace',action='store_true');a=P.parse_args()
root=pathlib.Path('/home/bfadmin/Documents/datum-eda');out=pathlib.Path(a.output);out.mkdir(parents=True,exist_ok=False)
board_source=pathlib.Path('/home/bfadmin/Documents/kicad_projects/DOA2526/hardware/DOA2526/DOA2526.kicad_pcb');board=out/'DOA2526.kicad_pcb';shutil.copy2(board_source,board)
binary=root/'target/release/datum-gui';env=dict(os.environ,TMPDIR=str(out),XDG_CONFIG_HOME=str(out/'config'),XDG_CACHE_HOME=str(out/'cache'),EDA_CLI_BIN=str(root/'target/debug/datum-eda'),WINIT_UNIX_BACKEND='x11')
for k in ['WAYLAND_DISPLAY','DATUM_ENGINE_SOCKET','EDA_ENGINE_SOCKET','DATUM_GUI_PREFERENCES_PATH','DATUM_TRACE_TIMING']:env.pop(k,None)
if a.trace:env['DATUM_TRACE_TIMING']='1'
x=c.CDLL('libX11.so.6');xt=c.CDLL('libXtst.so.6');x.XOpenDisplay.argtypes=[c.c_char_p];x.XOpenDisplay.restype=c.c_void_p;x.XWarpPointer.argtypes=[c.c_void_p,c.c_ulong,c.c_ulong,c.c_int,c.c_int,c.c_uint,c.c_uint,c.c_int,c.c_int];x.XFlush.argtypes=[c.c_void_p];x.XCloseDisplay.argtypes=[c.c_void_p];xt.XTestFakeButtonEvent.argtypes=[c.c_void_p,c.c_uint,c.c_int,c.c_ulong]
d=x.XOpenDisplay(env['DISPLAY'].encode());assert d
hz=os.sysconf('SC_CLK_TCK');logpath=out/'gui.log'
def cmd(*args):return subprocess.check_output(args,env=env,text=True,timeout=10).strip()
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
samples=[]
def sample(pid,window,name,rate=60):
 # Screenshots are output evidence, not compositor feedback.
 cmd('xdotool','windowactivate','--sync',str(window));time.sleep(.4)
 x.XWarpPointer(d,0,window,0,0,0,0,420,400);x.XFlush(d);time.sleep(.4)

 if name=='clamped_zoom':
  for _ in range(40):
   xt.XTestFakeButtonEvent(d,4,1,0);xt.XTestFakeButtonEvent(d,4,0,0);x.XFlush(d);time.sleep(.05)
  time.sleep(1)
  subprocess.run(['import','-window',str(window),str(out/f'before-{len(samples)}.png')],env=env,check=True,timeout=10)
 start_ticks=ticks(pid);start_gpu=drm(pid);peak=rss(pid);start_offset=logpath.stat().st_size;start=time.monotonic();sent=0;timeline=[]
 while time.monotonic()-start<a.seconds:
  if name=='pointer':x.XWarpPointer(d,0,window,0,0,0,0,350+(sent%100),350+(sent%50))
  elif name in ('zoom','scroll','clamped_zoom'):
   button=4 if name=='clamped_zoom' or (sent//10)%2==0 else 5;xt.XTestFakeButtonEvent(d,button,1,0);xt.XTestFakeButtonEvent(d,button,0,0)
  x.XFlush(d)
  if name!='idle':sent+=1
  t=time.monotonic()-start
  if not timeline or t-timeline[-1]['seconds']>=1:
   peak=max(peak,rss(pid));timeline.append({'seconds':t,'cpu_ticks':ticks(pid),'rss_kib':rss(pid),'gpu':drm(pid)})
  time.sleep(max(0,min(1/rate,a.seconds-(time.monotonic()-start))))
 elapsed=time.monotonic()-start;cpu=(ticks(pid)-start_ticks)/hz;gpu_end=drm(pid);log=logpath.read_bytes()[start_offset:].decode(errors='replace')
 frames=re.findall(r'\[datum-timing\] runtime render (.*)',log);dialogs=[int(v) for v in re.findall(r'dialog renderer=(\d+)us',log)]
 sample={'phase':name,'duration_s':elapsed,'input_events_sent':sent,'requested_hz':rate,'cpu_seconds':cpu,'cpu_percent_one_core':cpu/elapsed*100,'rss_peak_kib':peak,'gpu_engine_active_percent':{k:max(0,v-start_gpu[k])/(elapsed*1e9)*100 for k,v in gpu_end.items() if k in start_gpu},'main_submissions':len(frames) if a.trace else None,'dialog_submissions':len(dialogs) if a.trace else None,'prepared_miss_frames':sum('prepared_was_cached=false' in v for v in frames) if a.trace else None,'retained_miss_frames':sum('retained_was_cached=false' in v for v in frames) if a.trace else None,'main_renderer_ms':[int(re.search(r'renderer=(\d+)ms',v)[1]) for v in frames],'dialog_renderer_us':dialogs,'raw_counters':timeline}

 if name=='clamped_zoom':subprocess.run(['import','-window',str(window),str(out/f'after-{len(samples)}.png')],env=env,check=True,timeout=10)
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
  time.sleep(5)
  sample(p.pid,main,'idle',rate=10)
  for phase,window,rate in [('clamped_zoom',main,10)]:
   for trial in range(a.trials):sample(p.pid,window,phase,rate)
  if owned and a.mode in ('global','project'):
   # Project opens Units; Global Units can be selected without settings mutation.
   if a.mode=='global':
    cmd('xdotool','windowactivate','--sync',str(owned));x.XWarpPointer(d,0,owned,0,0,0,0,90,106);x.XFlush(d);xt.XTestFakeButtonEvent(d,1,1,0);xt.XTestFakeButtonEvent(d,1,0,0);x.XFlush(d);time.sleep(1)
   for trial in range(a.trials):sample(p.pid,owned,'scroll',20)
  for w,title in windows:subprocess.run(['import','-window',str(w),str(out/f'window-{w}.png')],env=env,check=True)
  sample(p.pid,main,'idle',rate=10)
 finally:
  if p.poll() is None:p.terminate();p.wait(timeout=10)
  x.XCloseDisplay(d)
report={'candidate':cmd('git','rev-parse','HEAD'),'binary':str(binary),'binary_sha256':hashlib.sha256(binary.read_bytes()).hexdigest(),'fixture_sha256':hashlib.sha256(board.read_bytes()).hexdigest(),'fixture_source':str(board_source),'mode':a.mode,'trace_enabled':a.trace,'backend':'X11/Xwayland on existing desktop; synthetic XTest input, not native Wayland qualification','platform':os.uname()._asdict() if hasattr(os.uname(),'_asdict') else list(os.uname()),'windows':windows,'samples':samples,'limitations':['Sent input is not acknowledged delivery; screenshots/traces are not compositor feedback.','GPU engine busy time is not frequency-normalized utilization or per-frame GPU timing.','CPU counters include all GUI threads but not independent engine processes.','Timing log overhead exists when trace_enabled; absolute acceptance not asserted.']};(out/'report.json').write_text(json.dumps(report,indent=2)+'\n')
print('REPORT',out/'report.json',flush=True)
