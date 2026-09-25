import os, sys, time, json, subprocess, hashlib, tempfile, re, uuid, ctypes, threading, socket
from pathlib import Path
root=Path('/home/bfadmin/Documents/datum-eda')
out=Path(tempfile.mkdtemp(prefix='pm045-final-handoff-')); print(out,flush=True)
board=Path('/tmp/datum-gui-candidate-boundary-v3/DOA2526.kicad_pcb')
assert hashlib.sha256(board.read_bytes()).hexdigest()=='7df67d426cc0cd808096824b0b7a05dcde351a1305fd41244d8c05a15801c08d'
binary=root/'target/release/datum-gui'; log=out/'native.log';log.touch()
assert hashlib.sha256(binary.read_bytes()).hexdigest()==os.environ['PM045_CANDIDATE_SHA']
env=os.environ.copy()
for k in list(env):
 if k.startswith(('DATUM_DIAGNOSTIC_','DATUM_GPU_DIAGNOSTIC_')):env.pop(k)
env.pop('WAYLAND_DISPLAY',None)
env.pop('DATUM_GUI_VERBOSE_LOG',None)
env.update(WINIT_UNIX_BACKEND='x11',WINIT_X11_SCALE_FACTOR='1',XDG_CONFIG_HOME=str(out/'config'),XDG_CACHE_HOME=str(out/'cache'),DATUM_GUI_LOG=str(log),DATUM_GPU_MEASUREMENTS='1',EDA_CLI_BIN=str(root/'target/release/datum-eda'))
cmd=[str(binary),'--project-root','/tmp/pm045-admission-hqugp169/project','--initial-layout','single','--window-size','1280x800','--visual-scale-factor','1']
report={'candidate':subprocess.check_output(['git','rev-parse','HEAD'],cwd=root,text=True).strip(),'binary_sha256':hashlib.sha256(binary.read_bytes()).hexdigest(),'command':cmd,'backend':'x11 on Xwayland','cycles':[],'limits':['Partial W-WINDOWS: CPU and attachment ownership; GPU duty/timing and complete memory accounting not sampled.','No endurance or independent replay acceptance.']}
report['source_state']='4d7cdd09 plus archived measurement shutdown source patch'
paths=['crates/gui-app/src/app_shell.rs','crates/gui-app/src/native_gpu_measurements.rs','crates/gui-app/src/native_measurement_shutdown.rs','crates/engine/src/substrate/mod.rs','crates/engine/src/substrate/tests.rs','crates/engine/src/substrate/project_read_cache.rs','crates/engine/src/substrate/tests/project_read_cache.rs','crates/gui-app/src/project_preferences_runtime.rs','crates/gui-app/src/project_preferences_runtime_tests.rs','crates/gui-app/src/project_preferences_read_cache.rs','crates/gui-app/src/project_preferences_read_cache_tests.rs']
report['source_sha256']={p:hashlib.sha256((root/p).read_bytes()).hexdigest() for p in paths}
report['limits']=['Final shutdown handoff tested; earlier client lifecycle and complete ACC-03 remain unqualified. GPU timestamps enabled; overhead not qualified. Polling may miss entire short GPU client lifetimes.','Method probe only:3Project cycles, first cold; not required30cycles/3trials.', 'Cgroup CPU includes in-process engine and all inherited children; external services remain unadmitted.', 'Window readback shows expected static content, not physical display/latency or complete input proof.', 'No GPU client/execution, observer-overhead, endurance or independent qualification.']
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
group=Path('/sys/fs/cgroup/user.slice')/f'user-{os.getuid()}.slice'/f'user@{os.getuid()}.service'/('datum-pm045-window-'+uuid.uuid4().hex)
group.mkdir()
from pm045_client_ledger import snapshot as drm_snapshot, interval as drm_interval
observer_stop=threading.Event(); observer_lock=threading.Lock(); drm_samples=[]
def observe_drm():
 with observer_lock:
  record=drm_snapshot((group/'cgroup.procs').read_text().split())
  drm_samples.append(record)
  return len(drm_samples)-1
def observe_loop():
 while not observer_stop.is_set():
  observe_drm();observer_stop.wait(.02)
observer=threading.Thread(target=observe_loop)
observer.start()

def group_counts():return {k:int(v) for k,v in (line.split() for line in (group/'cpu.stat').read_text().splitlines())}
report['cpu_observer']={'group':str(group),'before_spawn':group_counts(),'samples':[]}
libc=ctypes.CDLL(None,use_errno=True)
def cpu():
 clock=ctypes.c_int()
 rc=libc.clock_getcpuclockid(p.pid,ctypes.byref(clock));assert rc==0,rc
 stat=Path(f'/proc/{p.pid}/stat').read_text().rsplit(')',1)[1].split()
 sample={'monotonic_ns':time.monotonic_ns(),'group':group_counts(),'gui_cpu_ns':time.clock_gettime_ns(clock.value),'gui_pid':p.pid,'gui_start_ticks':int(stat[19]),'members':(group/'cgroup.procs').read_text().split()}
 sample['drm_sample']=observe_drm()
 report['cpu_observer']['samples'].append(sample)
 return sample['group']['usage_usec']/1000000

listener=socket.socket(socket.AF_UNIX,socket.SOCK_STREAM)
listener.bind(str(out/'observer.sock'));listener.listen(1);listener.settimeout(10)
env['DATUM_MEASUREMENT_SHUTDOWN_SOCKET']=str(out/'observer.sock')

def enter_group():(group/'cgroup.procs').write_text(str(os.getpid()))
def observe_pixels(wid,cycle):
 ref=root/'docs/reviews/gui-performance/implementation/S5/resolver-cache-ownership/native/PROJECT.png'
 attempts=[]
 for attempt in range(15):
  path=out/f'PROJECT-{cycle}-{attempt}.png'
  subprocess.run(['import','-window',str(wid),str(path)],check=True)
  r=subprocess.run(['compare','-metric','AE',str(ref),str(path),'null:'],capture_output=True,text=True)
  attempts.append({'monotonic_ns':time.monotonic_ns(),'capture':str(path),'pixel_difference':r.stderr,'returncode':r.returncode})
  if r.returncode==0:return attempts
  time.sleep(.025)
 raise AssertionError(('expected window pixels not observed',attempts))
def rss():return {s.split(':')[0]:s.split(':')[1].strip() for s in Path(f'/proc/{p.pid}/status').read_text().splitlines() if s.startswith(('VmRSS:','VmHWM:'))}
with (out/'stderr.log').open('w') as stream:
 p=subprocess.Popen(cmd,cwd=root,env=env,stdout=stream,stderr=stream,preexec_fn=enter_group)
 try:
  main=int(until(lambda:xd('search','--all','--onlyvisible','--pid',p.pid,'--name','Datum')) .splitlines()[0])
  xd('windowactivate','--sync',main);time.sleep(5)
  report['main_window']=main;report['rss_initial']=rss();report['action_log_start']=len(lines())
  for host in ('PROJECT',):
   for i in range(1):
    start=len(lines())
    # Native pointer opens menu; native keys select its actual model entries.
    xd('mousemove','--window',main,148 if host!='NEW' else 110,16,'click',1)
    if host=='NEW':xd('key','--delay',20,'Return')
    else:
     xd('key','--delay',20,'Up','Right','Down');time.sleep(.15)
     c0=cpu();t0=time.monotonic();xd('key','Return')
    wid=int(until(lambda:next((w for w in xd('search','--all','--onlyvisible','--pid',p.pid,'--name','^Project Preferences').splitlines() if int(w)!=main),None)))
    until(lambda:int(xd('getwindowfocus'))==wid)
    observed=observe_pixels(wid,i+1)
    title=xd('getwindowname',wid)
    assert {'GLOBAL':'Global Preferences','PROJECT':'Project Preferences','NEW':'New Project'}[host] in title, title
    opened={'cpu_ms':(cpu()-c0)*1000,'wall_ms':(time.monotonic()-t0)*1000,'pixel_observations':observed}
    if i==0:subprocess.run(['import','-window',str(wid),str(out/(host+'.png'))],check=True)
    close_start=len(lines());c1=cpu();t1=time.monotonic();xd('key','Escape')
    until(lambda:int(xd('getwindowfocus'))==main)
    until(lambda:subprocess.run(['xwininfo','-id',str(wid)],stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL).returncode!=0)
    report['cycles'].append({'host':host,'title':title,'cycle':i+1,'open':opened,'close':{'cpu_ms':(cpu()-c1)*1000,'wall_ms':(time.monotonic()-t1)*1000},'focus_restored':True,'rss':rss()});save()
   print(host,'one native cycle complete',flush=True)
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
  observer_stop.set();observer.join()
  report['drm_samples']=drm_samples
  start=report['cpu_observer']['samples'][0]['drm_sample'];end=report['cpu_observer']['samples'][-1]['drm_sample']
  report['drm_interval']=drm_interval(drm_samples[start:end+1])
  rows=log.read_text().splitlines()
  report['gpu_samples']=[json.loads(line.split('gpu_measurement ',1)[1]) for line in rows if 'gpu_measurement ' in line]
  report['gpu_incomplete']=[line for line in rows if 'gpu_measurement_incomplete ' in line]
  report['gpu_hosts']=[line for line in rows if 'gpu_measurement_host ' in line]
  report['action_gpu_incomplete']=[line for line in rows[report['action_log_start']:] if 'gpu_measurement_incomplete ' in line]
  assert not report['action_gpu_incomplete'],report['action_gpu_incomplete']
  assert report['gpu_samples'],'no GPU timestamp samples'

 except Exception as e:
  report['error']=repr(e);print(repr(e),flush=True)
 finally:
  observer_stop.set();observer.join()
  report.setdefault('drm_samples',drm_samples)
  report['rss_final']=rss() if p.poll() is None else None
  if p.poll() is None:
   p.terminate()
   try:p.wait(timeout=5)
   except subprocess.TimeoutExpired:p.kill();p.wait()
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

if report.get('error'):raise SystemExit(1)
