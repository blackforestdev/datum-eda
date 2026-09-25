import os, sys, time, json, subprocess, hashlib, tempfile, re, uuid, ctypes, threading, socket
from pathlib import Path
root=Path('/home/bfadmin/Documents/datum-eda')
out=Path(tempfile.mkdtemp(prefix='pm045-idle-native-')); print(out,flush=True)
board=Path('/tmp/datum-gui-candidate-boundary-v3/DOA2526.kicad_pcb')
assert hashlib.sha256(board.read_bytes()).hexdigest()=='7df67d426cc0cd808096824b0b7a05dcde351a1305fd41244d8c05a15801c08d'
binary=Path(os.environ['PM045_IDLE_BINARY']); log=out/'native.log';log.touch()
assert hashlib.sha256(binary.read_bytes()).hexdigest()==os.environ['PM045_CANDIDATE_SHA']
mode='plain'
resource_mode=os.environ['PM045_RESOURCE_MODE'];assert resource_mode in ('on','off')
trial=int(os.environ['PM045_TRIAL']);assert trial in (1,2,3)
env=os.environ.copy()
for k in list(env):
 if k.startswith(('DATUM_DIAGNOSTIC_','DATUM_GPU_DIAGNOSTIC_','DATUM_RESOURCE_TRACE','DATUM_GPU_ALLOCATION_TRACE','DATUM_PRIVATE_TEXT_TRACE')):env.pop(k)
env.pop('WAYLAND_DISPLAY',None)
env.pop('DATUM_GUI_VERBOSE_LOG',None)
env.update(WINIT_UNIX_BACKEND='x11',WINIT_X11_SCALE_FACTOR='1',XDG_CONFIG_HOME=str(out/'config'),XDG_CACHE_HOME=str(out/'cache'),DATUM_GUI_LOG=str(log),DATUM_GPU_MEASUREMENTS='1' if mode=='kernel' else '0',EDA_CLI_BIN=str(root/'target/release/datum-eda'))
env['DATUM_GUI_VERBOSE_LOG']='1'
if resource_mode=='on':
 env.update(DATUM_RESOURCE_TRACE=str(out/'resources.jsonl'),DATUM_GPU_ALLOCATION_TRACE=str(out/'gpu.jsonl'),DATUM_PRIVATE_TEXT_TRACE=str(out/'private.jsonl'))
cmd=[str(binary),'--project-root','/tmp/pm045-admission-hqugp169/project','--initial-layout','single','--window-size','1280x800','--visual-scale-factor','1']
report={'candidate':subprocess.check_output(['git','rev-parse','HEAD'],cwd=root,text=True).strip(),'binary_sha256':hashlib.sha256(binary.read_bytes()).hexdigest(),'command':cmd,'backend':'x11 on Xwayland','cycles':[],'limits':['Partial W-WINDOWS: CPU and attachment ownership; GPU duty/timing and complete memory accounting not sampled.','No endurance or independent replay acceptance.']}
report['mode']=mode;report['trial']=trial
report['limits']=['Three predeclared matched mode pairs; descriptive overhead only, not seven-pair relative inference.', 'CPU resource mode has timestamp/ptrace diagnostics off. GPU diagnostic mode needs overhead and client lifetime reconciliation before numerical admission.', 'Exact window readback verifies static readiness, not calibrated physical display latency.', 'No complete memory/endurance/backend-scale/independent acceptance.']
report['source_state']='Pinned idle baseline/candidate; declaration records binary/source provenance'
report['binary_role']=os.environ['PM045_IDLE_ROLE']
report['resource_trace_mode']=resource_mode
report['source_sha256']=json.loads(Path(os.environ['PM045_RESOURCE_DECLARATION']).read_text())['source_sha256']
for path,digest in report['source_sha256'].items():assert hashlib.sha256((root/path).read_bytes()).hexdigest()==digest,path
report['limits']=['Three declared baseline/candidate pairs only; no formal seven-pair statistical or observer-overhead qualification.', 'GPU timestamp/driver-client duty counters not collected here. Existing unsupported NVIDIA counter boundary remains.', 'Exact final window pixels/focus are endpoints, not calibrated physical presentation latency.', 'Sampled overlapping resource views and conservative reservation peaks do not establish complete memory/subcap/endurance/backend/scale/independent acceptance. CPU includes GUI and cooperating descendants through owned cgroup; snapshot RSS is GUI PID only.']
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
  xd('windowactivate','--sync',main);time.sleep(5)
  report['main_window']=main;report['rss_initial']=rss();report['action_log_start']=len(lines())
  report['idle_trials']=[]
  def capture(window,name):
   path=out/(name+'.png');subprocess.run(['import','-window',str(window),str(path)],check=True);return path
  for host in ('MAIN','GLOBAL','PROJECT','NEW'):
   wid=main
   if host!='MAIN':
    title={'GLOBAL':'Global Preferences','PROJECT':'Project Preferences','NEW':'New Project'}[host]
    xd('mousemove','--window',main,148 if host!='NEW' else 110,16,'click',1)
    if host!='NEW':xd('key','--delay',20,'Up','Right',*(['Down'] if host=='PROJECT' else []))
    xd('key','Return');wid=int(until(lambda:next((w for w in xd('search','--all','--onlyvisible','--pid',p.pid,'--name','^'+title).splitlines() if int(w)!=main),None)))
    until(lambda:int(xd('getwindowfocus'))==wid)
    observe_pixels(wid,0,host,'ready')
   xd('mousemove',1800,1000)
   time.sleep(5)
   before=capture(wid,host+'-before')
   time.sleep(.1)
   start_offset=log.stat().st_size;start_cpu=cpu();start_ns=time.monotonic_ns();start_sample=len(report['cpu_observer']['samples'])-1
   start_rss=rss()
   deadline=time.monotonic()+60
   while time.monotonic()<deadline:
    time.sleep(min(20,max(0,deadline-time.monotonic())))
    assert p.poll() is None,('idle process exited',p.returncode)
    print(report['binary_role'],trial,host,'idle elapsed',round((time.monotonic_ns()-start_ns)/1e9),flush=True)
   end_cpu=cpu();end_ns=time.monotonic_ns();end_offset=log.stat().st_size;end_sample=len(report['cpu_observer']['samples'])-1
   after=capture(wid,host+'-after')
   comparison=subprocess.run(['compare','-metric','AE',str(before),str(after),'null:'],capture_output=True,text=True)
   with log.open('rb') as source:source.seek(start_offset);timed=source.read(end_offset-start_offset)
   (out/(host+'-timed.log')).write_bytes(timed)
   elapsed=(end_ns-start_ns)/1e9;cpu_seconds=end_cpu-start_cpu
   row={'host':host,'window':wid,'started_ns':start_ns,'finished_ns':end_ns,'elapsed_s':elapsed,'cpu_seconds':cpu_seconds,'cpu_duty_percent':100*cpu_seconds/elapsed,'start_sample':start_sample,'end_sample':end_sample,'start_log_offset':start_offset,'end_log_offset':end_offset,'rss_before':start_rss,'rss_after':rss(),'pixels_returncode':comparison.returncode,'pixels_difference':comparison.stderr,'timed_presentations':timed.count(b'frame present begin'),'timed_submission_receipts':timed.count(b'"reason":"submit"')+timed.count(b'"reason":"submit_upload"'),'timed_redraw_requests':timed.count(b'reason":"requested"')}
   report['idle_trials'].append(row);save()
   assert comparison.returncode==0,('idle content changed',row)
   if host!='MAIN':
    xd('key','Escape');until(lambda:int(xd('getwindowfocus'))==main);until(lambda:subprocess.run(['xwininfo','-id',str(wid)],stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL).returncode!=0)
   print(report['binary_role'],trial,host,'complete',row['cpu_duty_percent'],row['timed_presentations'],flush=True)
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
  if mode=='kernel':assert report['gpu_samples'],'no GPU timestamp samples'
  else:assert not report['gpu_samples'],'unexpected GPU timestamps in resource mode'

 except Exception as e:
  report['error']=repr(e);print(repr(e),flush=True)
  if isinstance(e,TimeoutError) and p.poll() is None:
   threads=[]
   for task in sorted(Path(f'/proc/{p.pid}/task').iterdir()):
    item={'tid':task.name}
    for name in ('comm','wchan','syscall','status'):
     try:item[name]=(task/name).read_text()
     except OSError as error:item[name]=repr(error)
    threads.append(item)
   (out/'timeout-threads.json').write_text(json.dumps(threads,indent=2)+'\n')
   with (out/'timeout-gdb.txt').open('w') as diagnostic:
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
