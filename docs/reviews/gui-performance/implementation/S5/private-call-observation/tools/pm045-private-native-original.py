import os,sys,time,json,hashlib,tempfile,subprocess,socket,uuid,signal
from pathlib import Path
from pm045_wm_close import close_window
root=Path('/home/bfadmin/Documents/datum-eda');mode=sys.argv[1];assert mode in ('on','off','overflow','existing')
out=Path(tempfile.mkdtemp(prefix='pm045-private-native-'+mode+'-'));print(out,flush=True)
binary=root/'target/release/datum-gui';trace=out/'private.jsonl';log=out/'native.log';log.touch()
r={'mode':mode,'binary_sha256':hashlib.sha256(binary.read_bytes()).hexdigest(),'cycles':[],'scope':'Bounded native private-call delivery conformance; no timing/RSS/full memory/endurance acceptance.'}
env=os.environ.copy()
for k in list(env):
 if k.startswith(('DATUM_DIAGNOSTIC_','DATUM_GPU_DIAGNOSTIC_','DATUM_PRIVATE_TEXT_')):env.pop(k)
env.pop('WAYLAND_DISPLAY',None);env.pop('DATUM_GUI_VERBOSE_LOG',None)
env.update(WINIT_UNIX_BACKEND='x11',WINIT_X11_SCALE_FACTOR='1',XDG_CONFIG_HOME=str(out/'config'),XDG_CACHE_HOME=str(out/'cache'),DATUM_GUI_LOG=str(log),DATUM_GPU_MEASUREMENTS='0',EDA_CLI_BIN=str(root/'target/release/datum-eda'))
if mode!='off':env['DATUM_PRIVATE_TEXT_TRACE']=str(trace)
if mode=='overflow':env['DATUM_PRIVATE_TEXT_TRACE_CAPACITY']='1'
if mode=='existing':trace.write_text('preserved existing file\n')
group=Path('/sys/fs/cgroup/user.slice')/f'user-{os.getuid()}.slice'/f'user@{os.getuid()}.service'/('datum-pm045-private-'+uuid.uuid4().hex);group.mkdir()
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
with (out/'stderr.log').open('w') as f:
 p=subprocess.Popen(['/tmp/pm045_fd_trace',str(out/'process.jsonl'),*cmd],cwd=root,env=env,stdout=f,stderr=f)
 try:
  if mode in ('overflow','existing'):
   r['exit_code']=p.wait(timeout=30);assert r['exit_code']!=0
   if mode=='existing':assert trace.read_text()=='preserved existing file\n'
   else:
    rows=[json.loads(x) for x in trace.read_text().splitlines()];assert any(x.get('dropped_events',0)>0 and x['incomplete'] for x in rows);assert not any(x.get('complete_delivery') for x in rows)
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
   close_window(main);conn,_=listener.accept();conn.settimeout(2);receipt=b''
   while not receipt.endswith(b'\n'):
    part=conn.recv(1);assert part;receipt+=part
   r['shutdown']=json.loads(receipt);assert r['shutdown']['phase']=='drained_device_live';conn.sendall(b'!');conn.close();r['exit_code']=p.wait(timeout=10);assert r['exit_code']==0
   if mode=='on':
    rows=[json.loads(x) for x in trace.read_text().splitlines()];assert rows[-1]['phase']=='end' and rows[-1]['complete_delivery'];assert all(x.get('dropped_events',0)==0 for x in rows)
   else:assert not trace.exists()
 except BaseException as e:r['error']=repr(e);raise
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
