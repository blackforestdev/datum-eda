import os, sys, time, json, subprocess, hashlib, tempfile, re, uuid, ctypes, threading, socket
from pathlib import Path
root=Path('/home/bfadmin/Documents/datum-eda')
out=Path(tempfile.mkdtemp(prefix='pm045-callback-native-')); print(out,flush=True)
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
if os.environ['PM045_LOG_VERBOSE']=='1':env['DATUM_GUI_VERBOSE_LOG']='1'
env['DATUM_GPU_MEASUREMENTS']=os.environ.get('PM045_POINTER_GPU','0')
env['PM045_CALLBACK_CPU_PATH']=str(out/'callback-cpu.json')
if resource_mode=='on':
 env.update(DATUM_RESOURCE_TRACE=str(out/'resources.jsonl'),DATUM_GPU_ALLOCATION_TRACE=str(out/'gpu.jsonl'),DATUM_PRIVATE_TEXT_TRACE=str(out/'private.jsonl'))
cmd=[str(binary),'--project-root','/tmp/pm045-admission-hqugp169/project','--initial-layout','single','--window-size','1280x800','--visual-scale-factor','1']
report={'candidate':subprocess.check_output(['git','rev-parse','HEAD'],cwd=root,text=True).strip(),'binary_sha256':hashlib.sha256(binary.read_bytes()).hexdigest(),'command':cmd,'backend':'x11 on Xwayland','cycles':[],'limits':['Partial W-WINDOWS: CPU and attachment ownership; GPU duty/timing and complete memory accounting not sampled.','No endurance or independent replay acceptance.']}
report['mode']=mode;report['trial']=trial;report['verbose_log']=os.environ['PM045_LOG_VERBOSE']
report['limits']=['Three predeclared matched mode pairs; descriptive overhead only, not seven-pair relative inference.', 'CPU resource mode has timestamp/ptrace diagnostics off. GPU diagnostic mode needs overhead and client lifetime reconciliation before numerical admission.', 'Exact window readback verifies static readiness, not calibrated physical display latency.', 'No complete memory/endurance/backend-scale/independent acceptance.']
report['source_state']='Pinned cold-start baseline/candidate provenance in declaration'
report['binary_role']=os.environ['PM045_POINTER_ROLE']
report['resource_trace_mode']=resource_mode
report['source_sha256']=json.loads(Path(os.environ['PM045_RESOURCE_DECLARATION']).read_text())['source_sha256']
for path,digest in report['source_sha256'].items():assert hashlib.sha256((root/path).read_bytes()).hexdigest()==digest,path
report['limits']=['Temporary callback instrumentation candidate, not production cap acceptance. Main-thread CPU/wall scopes cover ApplicationHandler window_event and about_to_wait; record cost excluded from scope remains in family CPU. No sampled symbol attribution. Scope coverage/overlap and exact output must be checked.', 'W-CLAMP eachbound100untimedoutwarddetents,5swarmup,1800 alternating press/release transitions60Hz30s,5sstill,one inward press positive control; release separately recorded after timing. Exactcamera numericstate/continuouspresentation notexported;no fullqualification.', 'GPU timestamp/driver-client duty counters not collected here. Existing unsupported NVIDIA counter boundary remains.', 'Exact final window pixels/focus are endpoints, not calibrated physical presentation latency.', 'Sampled overlapping resource views and conservative reservation peaks do not establish complete memory/subcap/endurance/backend/scale/independent acceptance. CPU includes GUI and cooperating descendants through owned cgroup; snapshot RSS is GUI PID only.']
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
  main=int(until(lambda:xd('search','--all','--onlyvisible','--pid',p.pid,'--name','Datum')) .splitlines()[0])
  report['main_window']=main
  xd('windowactivate','--sync',main);time.sleep(.5)
  xd('mousemove','--window',main,300,150)
  def ready_pixels():
   q=out/'warm.png';subprocess.run(['import','-window',str(main),str(q)],check=True)
   z=subprocess.run(['compare','-metric','AE','/tmp/pm045-pointer-native-emp4nxo2/final.png',str(q),'null:'],capture_output=True,text=True)
   assert z.returncode in (0,1);return z.returncode==0
  until(ready_pixels)
  geometry=dict(line.split('=',1) for line in xd('getwindowgeometry','--shell',main).splitlines() if '=' in line)
  assert int(geometry['WIDTH'])==1280 and int(geometry['HEIGHT'])==800
  report['geometry']=geometry;report['clamp_trials']=[];report['action_log_start']=len(lines())
  xd('mousemove','--window',main,600,385)
  for label,button,reverse in [('minimum',5,4),('maximum',4,5)]:
   subprocess.run(['python3','/tmp/pm045-wheel-stream.py',str(button),'100','200',str(out/(label+'-setup-input.json'))],check=True)
   time.sleep(5)
   before=out/(label+'-before.png');subprocess.run(['import','-window',str(main),str(before)],check=True)
   start_offset=log.stat().st_size;s0=len(report['cpu_observer']['samples']);c0=cpu();t0=time.monotonic_ns()
   subprocess.run(['python3','/tmp/pm045-wheel-edges.py',str(button),'1800','60',str(out/(label+'-input.json'))],check=True)
   c1=cpu();t1=time.monotonic_ns();s1=len(report['cpu_observer']['samples'])-1;active_end_offset=log.stat().st_size
   time.sleep(5);c2=cpu();t2=time.monotonic_ns();s2=len(report['cpu_observer']['samples'])-1;end_offset=log.stat().st_size
   with log.open('rb') as source:source.seek(start_offset);(out/(label+'-active-tail.log')).write_bytes(source.read(end_offset-start_offset))
   after=out/(label+'-after.png');subprocess.run(['import','-window',str(main),str(after)],check=True)
   cmp=subprocess.run(['compare','-metric','AE',str(before),str(after),'null:'],capture_output=True,text=True);assert cmp.returncode in (0,1)
   positive_start=log.stat().st_size;pc0=cpu();pt0=time.monotonic_ns();ps0=len(report['cpu_observer']['samples'])-1
   subprocess.run(['python3','/tmp/pm045-wheel-edges.py',str(reverse),'1','60',str(out/(label+'-reversal-input.json')),'press'],check=True)
   deadline=time.monotonic()+2;polls=[];changed=False
   while time.monotonic()<deadline:
    capture=out/(label+'-reversal.png');subprocess.run(['import','-window',str(main),str(capture)],check=True)
    cmp2=subprocess.run(['compare','-metric','AE',str(after),str(capture),'null:'],capture_output=True,text=True)
    assert cmp2.returncode in (0,1);polls.append({'monotonic_ns':time.monotonic_ns(),'pixel_AE':cmp2.stderr.strip()})
    if cmp2.returncode==1:changed=True;break
    time.sleep(.025)
   pc1=cpu();pt1=time.monotonic_ns();ps1=len(report['cpu_observer']['samples'])-1
   with log.open('rb') as source:source.seek(positive_start);(out/(label+'-positive.log')).write_bytes(source.read())
   row={'bound':label,'button':button,'started_ns':t0,'active_finished_ns':t1,'tail_finished_ns':t2,'active_cpu_ms':(c1-c0)*1000,'tail_cpu_ms':(c2-c1)*1000,'start_sample':s0,'active_end_sample':s1,'tail_end_sample':s2,'start_log_offset':start_offset,'active_end_log_offset':active_end_offset,'end_log_offset':end_offset,'endpoint_AE':cmp.stderr.strip(),'positive':{'start_sample':ps0,'end_sample':ps1,'started_ns':pt0,'finished_ns':pt1,'cpu_ms':(pc1-pc0)*1000,'pixel_polls':polls,'changed':changed}}
   report['clamp_trials'].append(row);save();print(label,row['endpoint_AE'],row['active_cpu_ms'],changed,flush=True)
   assert cmp.returncode==0,'outward clamp changed final pixels'
   assert changed,'inward positive control did not change pixels'
   row['pilot_endpoint_comparisons']={}
   for phase in ['before','after','reversal']:
    actual=out/(label+'-'+phase+'.png');reference=Path('/tmp/pm045-clamp-edges-native-y2pegrfz')/actual.name
    comparison=subprocess.run(['compare','-metric','AE',str(reference),str(actual),'null:'],capture_output=True,text=True)
    row['pilot_endpoint_comparisons'][phase]=comparison.stderr.strip();assert comparison.returncode==0,(label,phase,comparison.stderr)
   save()
   active=log.read_bytes()[start_offset:active_end_offset].decode();positive=(out/(label+'-positive.log')).read_text()
   assert active.count('mouse wheel delta=')==(1800 if report['verbose_log']=='1' else 0),active.count('mouse wheel delta=')
   assert positive.count('mouse wheel delta=')==(1 if report['verbose_log']=='1' else 0),positive.count('mouse wheel delta=')
   cleanup_offset=log.stat().st_size
   subprocess.run(['python3','/tmp/pm045-wheel-edges.py',str(reverse),'1','60',str(out/(label+'-release-cleanup-input.json')),'release'],check=True)
   time.sleep(.5)
   (out/(label+'-release-cleanup.log')).write_bytes(log.read_bytes()[cleanup_offset:])
  assert int(xd('getwindowfocus'))==main
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
