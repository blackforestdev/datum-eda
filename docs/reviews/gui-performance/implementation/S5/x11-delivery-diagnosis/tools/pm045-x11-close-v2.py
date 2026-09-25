import os,sys,time,json,hashlib,tempfile,subprocess,socket,uuid,signal
from pathlib import Path
from pm045_wm_close import close_window
root=Path('/home/bfadmin/Documents/datum-eda');mode=sys.argv[1];assert mode in ('on','off','overflow','existing')
out=Path(tempfile.mkdtemp(prefix='pm045-x11-close-v2-'));print(out,flush=True)
binary=Path(os.environ.get('PM045_RESOURCE_BINARY',str(root/'target/release/datum-gui')));trace=out/'resources.jsonl';log=out/'native.log';log.touch()
r={'mode':mode,'binary_sha256':hashlib.sha256(binary.read_bytes()).hexdigest(),'cycles':[],'scope':'Bounded X11 close delivery diagnosis; loader audit changes timing, no performance or qualification claim.'}
env=os.environ.copy()
for k in list(env):
 if k.startswith(('DATUM_DIAGNOSTIC_','DATUM_GPU_DIAGNOSTIC_','DATUM_PRIVATE_TEXT_','DATUM_GPU_ALLOCATION_','DATUM_RESOURCE_TRACE')):env.pop(k)
env.pop('WAYLAND_DISPLAY',None);env.pop('DATUM_GUI_VERBOSE_LOG',None)
env.update(WINIT_UNIX_BACKEND='x11',WINIT_X11_SCALE_FACTOR='1',XDG_CONFIG_HOME=str(out/'config'),XDG_CACHE_HOME=str(out/'cache'),DATUM_GUI_LOG=str(log),DATUM_GPU_MEASUREMENTS='0',EDA_CLI_BIN=str(root/'target/release/datum-eda'))
env['DATUM_GUI_VERBOSE_LOG']='1'
env.update(LD_AUDIT='/tmp/pm045_x11_audit_v2.so',PM045_X11_AUDIT_PATH=str(out/'x11'))
if mode=='on':env['DATUM_PRIVATE_TEXT_TRACE']=str(out/'private.jsonl')
if mode!='off':env['DATUM_RESOURCE_TRACE']=str(trace)
if mode=='overflow':env.update(DATUM_RESOURCE_TRACE_LIMIT='1',DATUM_RESOURCE_TRACE_INTERVAL_MS='1')
if mode=='existing':trace.write_text('preserved existing file\n')
group=Path('/sys/fs/cgroup/user.slice')/f'user-{os.getuid()}.slice'/f'user@{os.getuid()}.service'/('datum-pm045-gpu-history-'+uuid.uuid4().hex);group.mkdir()
env.update(DATUM_FD_TRACE_MODE='off',DATUM_TRACE_CGROUP=str(group/'cgroup.procs'))
listener=socket.socket(socket.AF_UNIX,socket.SOCK_STREAM);listener.bind(str(out/'observer.sock'));listener.listen(1);listener.settimeout(10)
if mode in ('on','off'):env['DATUM_MEASUREMENT_SHUTDOWN_SOCKET']=str(out/'observer.sock')
cmd=[str(binary),'--project-root','/tmp/pm045-admission-hqugp169/project','--initial-layout','single','--window-size','1280x800','--visual-scale-factor','1']
def save():(out/'result.json').write_text(json.dumps(r,indent=2)+'\n')
def xd(*args):
 x=subprocess.run(['xdotool',*map(str,args)],capture_output=True,text=True)
 if x.returncode and args[0]!='search':raise RuntimeError(x.stderr)
 return x.stdout.strip()
def until(fn):
 deadline=time.monotonic()+15
 while time.monotonic()<deadline:
  assert p.poll() is None,('native exited',p.returncode)
  value=fn()
  if value:return value
  time.sleep(.025)
 raise AssertionError('native endpoint timeout')
def diagnose():
  r['diagnostic_captured']=True;save()
  # Pure debugger memory reads: no calls into the inferior.
  audit=out/f'x11.{pid}.jsonl';events=[json.loads(x) for x in audit.read_text().splitlines()]
  displays=list(dict.fromkeys(x['display'] for x in events if x['kind']=='open_display' and x['result']))
  offsets=json.loads(subprocess.check_output(['/tmp/pm045_xlib_offsets'],text=True))
  script=out/'queue.gdb'
  script.write_text('set pagination off\nset debuginfod enabled off\npython\n'+
    'import gdb,json\n'+
    'displays='+repr(displays)+'\n'+'offsets='+repr(offsets)+'\n'+
    'def read(a,n): return int.from_bytes(bytes(gdb.selected_inferior().read_memory(a,n)),"little")\n'+
    'records=[]\n'+
    'for value in displays:\n'+
    ' d=int(value,16); row={"display":value,"events":[]}\n'+
    ' try:\n'+
    '  row["qlen"]=read(d+offsets["qlen"],4); node=read(d+offsets["head"],8); seen=set()\n'+
    '  while node and node not in seen and len(seen)<1024:\n'+
    '   seen.add(node); e=node+offsets["qevent_event"]; row["events"].append({"type":read(e+offsets["event_type"],4),"window":read(e+offsets["event_window"],8),"message_type":read(e+offsets["event_message_type"],8),"data0":read(e+offsets["event_data"],8)});node=read(node+offsets["qevent_next"],8)\n'+
    ' except Exception as error: row["error"]=repr(error)\n'+
    ' records.append(row)\n'+
    'open('+repr(str(out/'queue-memory.json'))+',"w").write(json.dumps(records,indent=2)+"\\n")\n'+
    'end\nthread apply all bt\ndetach\n')
  with (out/'gdb.txt').open('w') as f:
   try:r['gdb_returncode']=subprocess.run(['gdb','-q','-nx','-batch','-p',str(pid),'-x',str(script)],stdout=f,stderr=subprocess.STDOUT,timeout=25).returncode
   except subprocess.TimeoutExpired:r['gdb_timeout']=True
  # One unrelated motion; the original ten-second close failure remains failed.
  r['wake_ns']=time.monotonic_ns();xd('mousemove','--window',main,300,350);listener.settimeout(3)
  try:
   delayed,_=listener.accept();delayed.settimeout(2);data=b''
   while not data.endswith(b'\n'):
    part=delayed.recv(1);assert part;data+=part
   r['after_wake_receipt']=json.loads(data);delayed.sendall(b'!');delayed.close();r['after_wake_exit']=p.wait(timeout=10)
  except TimeoutError:r['after_wake_timeout']=True
  save()
with (out/'stderr.log').open('w') as f:
 p=subprocess.Popen(['/tmp/pm045_fd_trace',str(out/'process.jsonl'),*cmd],cwd=root,env=env,stdout=f,stderr=f)
 try:
  if mode in ('overflow','existing'):
   r['exit_code']=p.wait(timeout=30);assert r['exit_code']!=0
   if mode=='existing':assert trace.read_text()=='preserved existing file\n'
   else:
    rows=[json.loads(x) for x in trace.read_text().splitlines()];assert any(x.get('phase')=='invalid' for x in rows);assert not any(x.get('complete_delivery') for x in rows)
   r['expected_rejection']=True
  else:
   deadline=time.monotonic()+10
   while not (out/'process.jsonl').exists() or not (out/'process.jsonl').read_text().endswith('\n'):
    assert time.monotonic()<deadline;time.sleep(.02)
   pid=json.loads((out/'process.jsonl').read_text().splitlines()[0])['pid'];r['pid']=pid
   main=int(until(lambda:xd('search','--all','--onlyvisible','--pid',pid,'--name','Datum')).splitlines()[0]);xd('windowactivate','--sync',main);time.sleep(2)
   for host,title in [('GLOBAL','Global Preferences'),('PROJECT','Project Preferences'),('NEW','New Project')]:
    for cycle in range(3):
     xd('mousemove','--window',main,148 if host!='NEW' else 110,16,'click',1)
     if host!='NEW':xd('key','--delay',20,'Up','Right',*(['Down'] if host=='PROJECT' else []))
     xd('key','Return');wid=int(until(lambda:next((w for w in xd('search','--all','--onlyvisible','--pid',pid,'--name','^'+title).splitlines() if int(w)!=main),None)))
     until(lambda:int(xd('getwindowfocus'))==wid)
     attempts=[]
     for attempt in range(15):
      png=out/f'{host}-{cycle}-{attempt}.png';subprocess.run(['import','-window',str(wid),str(png)],check=True)
      diff=subprocess.run(['compare','-metric','AE',str(root/f'docs/reviews/gui-performance/implementation/S5/resolver-cache-ownership/native/{host}.png'),str(png),'null:'],capture_output=True,text=True)
      attempts.append({'image':png.name,'difference':diff.stderr,'returncode':diff.returncode})
      if diff.returncode==0:break
      time.sleep(.025)
     assert diff.returncode==0,attempts
     xd('key','Escape');until(lambda:int(xd('getwindowfocus'))==main);until(lambda:subprocess.run(['xwininfo','-id',str(wid)],stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL).returncode!=0)
     r['cycles'].append({'host':host,'cycle':cycle,'images':attempts,'focus_restored':True});save()
   r['close_window']=main;r['close_sent_ns']=time.monotonic_ns();close_window(main)
   try:conn,_=listener.accept()
   except TimeoutError:
    r['close_timeout']=True;diagnose();raise
   conn.settimeout(2);receipt=b''
   while not receipt.endswith(b'\n'):
    part=conn.recv(1);assert part;receipt+=part
   r['shutdown']=json.loads(receipt);assert r['shutdown']['phase']=='drained_device_live';conn.sendall(b'!');conn.close();r['exit_code']=p.wait(timeout=10);assert r['exit_code']==0
   if mode=='on':
    rows=[json.loads(x) for x in trace.read_text().splitlines()];assert rows[-1]['phase']=='end' and rows[-1]['complete_delivery'];assert all(x.get('dropped_events',0)==0 for x in rows)
   else:assert not trace.exists()
 except BaseException as e:
  r['error']=repr(e)
  if isinstance(e,AssertionError) and str(e)=='native endpoint timeout' and not r.get('diagnostic_captured'):diagnose()
  raise
 finally:
  if p.poll() is None:
   (group/'cgroup.kill').write_text('1');p.wait(timeout=10)
  r['remaining_before_cleanup']=(group/'cgroup.procs').read_text().split()
  if r['remaining_before_cleanup']:
   (group/'cgroup.kill').write_text('1')
   for _ in range(100):
    if not (group/'cgroup.procs').read_text().strip():break
    time.sleep(.02)
  assert not (group/'cgroup.procs').read_text().strip();group.rmdir();r['cleanup']='owned process cgroup empty and removed'
  listener.close();(out/'observer.sock').unlink(missing_ok=True);save()
