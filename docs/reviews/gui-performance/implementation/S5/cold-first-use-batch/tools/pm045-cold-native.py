import os, sys, time, json, subprocess, hashlib, tempfile, re, uuid, ctypes, threading, socket
from pathlib import Path
root=Path('/home/bfadmin/Documents/datum-eda')
out=Path(tempfile.mkdtemp(prefix='pm045-cold-native-')); print(out,flush=True)
board=Path('/tmp/datum-gui-candidate-boundary-v3/DOA2526.kicad_pcb')
assert hashlib.sha256(board.read_bytes()).hexdigest()=='7df67d426cc0cd808096824b0b7a05dcde351a1305fd41244d8c05a15801c08d'
binary=Path(os.environ['PM045_COLD_BINARY']); log=out/'native.log';log.touch()
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
if resource_mode=='on':
 env.update(DATUM_RESOURCE_TRACE=str(out/'resources.jsonl'),DATUM_GPU_ALLOCATION_TRACE=str(out/'gpu.jsonl'),DATUM_PRIVATE_TEXT_TRACE=str(out/'private.jsonl'))
cmd=[str(binary),'--project-root','/tmp/pm045-admission-hqugp169/project','--initial-layout','single','--window-size','1280x800','--visual-scale-factor','1']
report={'candidate':subprocess.check_output(['git','rev-parse','HEAD'],cwd=root,text=True).strip(),'binary_sha256':hashlib.sha256(binary.read_bytes()).hexdigest(),'command':cmd,'backend':'x11 on Xwayland','cycles':[],'limits':['Partial W-WINDOWS: CPU and attachment ownership; GPU duty/timing and complete memory accounting not sampled.','No endurance or independent replay acceptance.']}
report['mode']=mode;report['trial']=trial
report['limits']=['Three predeclared matched mode pairs; descriptive overhead only, not seven-pair relative inference.', 'CPU resource mode has timestamp/ptrace diagnostics off. GPU diagnostic mode needs overhead and client lifetime reconciliation before numerical admission.', 'Exact window readback verifies static readiness, not calibrated physical display latency.', 'No complete memory/endurance/backend-scale/independent acceptance.']
report['source_state']='Pinned cold-start baseline/candidate provenance in declaration'
report['binary_role']=os.environ['PM045_COLD_ROLE']
report['resource_trace_mode']=resource_mode
report['source_sha256']=json.loads(Path(os.environ['PM045_RESOURCE_DECLARATION']).read_text())['source_sha256']
for path,digest in report['source_sha256'].items():assert hashlib.sha256((root/path).read_bytes()).hexdigest()==digest,path
report['limits']=['Ten declared fresh-process starts per binary with three auxiliary first-use hosts; not a cold-machine/page-cache-flush or formal seven-pair inference.', 'GPU timestamp/driver-client duty counters not collected here. Existing unsupported NVIDIA counter boundary remains.', 'Exact final window pixels/focus are endpoints, not calibrated physical presentation latency.', 'Sampled overlapping resource views and conservative reservation peaks do not establish complete memory/subcap/endurance/backend/scale/independent acceptance. CPU includes GUI and cooperating descendants through owned cgroup; snapshot RSS is GUI PID only.']
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
  # No activation/input during initial readiness; pointer moved before launch.
  until(lambda:'frame present end' in log.read_text())
  attempts=[]
  ref=Path('/tmp/pm045-idle-native-qhyk1ecq/MAIN-before.png')
  for attempt in range(100):
   png=out/f'MAIN-cold-{attempt}.png';subprocess.run(['import','-window',str(main),str(png)],check=True)
   comparison=subprocess.run(['compare','-metric','AE',str(ref),str(png),'null:'],capture_output=True,text=True)
   attempts.append({'image':png.name,'returncode':comparison.returncode,'difference':comparison.stderr,'observed_ns':time.monotonic_ns()})
   if comparison.returncode==0:break
   time.sleep(.05)
  assert comparison.returncode==0,('cold endpoint never matched',attempts)
  total_cpu=cpu();report['cold_main']={'ready_ns':time.monotonic_ns(),'launch_ns':report['launch_ns'],'cpu_seconds_since_group_before_spawn':total_cpu-report['cpu_observer']['before_spawn']['usage_usec']/1e6,'end_sample':len(report['cpu_observer']['samples'])-1,'pixel_attempts':attempts,'scope':'Fresh process and empty private XDG cache/config; OS page cache not flushed. End-to-end application CPU includes engine/startup/import-resolution; static readback readiness, not physical display latency.'};save()
  xd('windowactivate','--sync',main)
  report['main_window']=main;report['rss_initial']=rss();report['action_log_start']=len(lines())
  def cycle(host,i,phase):
    title_expected={'GLOBAL':'Global Preferences','PROJECT':'Project Preferences','NEW':'New Project'}[host]
    xd('mousemove','--window',main,148 if host!='NEW' else 110,16,'click',1)
    if host!='NEW':xd('key','--delay',20,'Up','Right',*(['Down'] if host=='PROJECT' else []))
    time.sleep(.15)
    before_rss=rss()
    c0=cpu();t0=time.monotonic();start_sample=len(report['cpu_observer']['samples'])-1
    xd('key','Return')
    wid=int(until(lambda:next((w for w in xd('search','--all','--onlyvisible','--pid',p.pid,'--name','^'+title_expected).splitlines() if int(w)!=main),None)))
    until(lambda:int(xd('getwindowfocus'))==wid)
    observed=observe_pixels(wid,i,host,phase)
    assert title_expected in xd('getwindowname',wid)
    opened={'cpu_ms':(cpu()-c0)*1000,'wall_ms':(time.monotonic()-t0)*1000,'pixel_observations':observed,'start_cpu_sample':start_sample,'end_cpu_sample':len(report['cpu_observer']['samples'])-1}
    open_rss=rss()
    c1=cpu();t1=time.monotonic();close_sample=len(report['cpu_observer']['samples'])-1
    xd('key','Escape')
    until(lambda:int(xd('getwindowfocus'))==main)
    until(lambda:subprocess.run(['xwininfo','-id',str(wid)],stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL).returncode!=0)
    report['cycles'].append({'host':host,'title':title_expected,'phase':phase,'cycle':i,'open':opened,'close':{'cpu_ms':(cpu()-c1)*1000,'wall_ms':(time.monotonic()-t1)*1000,'start_cpu_sample':close_sample,'end_cpu_sample':len(report['cpu_observer']['samples'])-1},'focus_restored':True,'rss_before':before_rss,'rss_open':open_rss,'rss':rss()});save()
  for host in ('GLOBAL','PROJECT','NEW'):cycle(host,0,'first-use')
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
