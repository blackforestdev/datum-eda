import os,sys,json,hashlib,tempfile,subprocess,time,socket
from pathlib import Path
from pm045_wm_close import close_window
root=Path('/home/bfadmin/Documents/datum-eda')
binary=Path(sys.argv[1]);out=Path(sys.argv[2]);out.mkdir()
log=out/'native.log';log.touch()
env=os.environ.copy()
for k in list(env):
 if k.startswith(('DATUM_DIAGNOSTIC_','DATUM_GPU_DIAGNOSTIC_')):env.pop(k)
env.pop('WAYLAND_DISPLAY',None);env.pop('DATUM_GUI_VERBOSE_LOG',None)
env.update(WINIT_UNIX_BACKEND='x11',WINIT_X11_SCALE_FACTOR='1',XDG_CONFIG_HOME=str(out/'config'),XDG_CACHE_HOME=str(out/'cache'),DATUM_GUI_LOG=str(log),DATUM_GPU_MEASUREMENTS='1',EDA_CLI_BIN=str(root/'target/release/datum-eda'))
listener=socket.socket(socket.AF_UNIX,socket.SOCK_STREAM);listener.bind(str(out/'observer.sock'));listener.listen(1);listener.settimeout(10)
env['DATUM_MEASUREMENT_SHUTDOWN_SOCKET']=str(out/'observer.sock')
cmd=[str(binary),'--project-root','/tmp/pm045-admission-hqugp169/project','--initial-layout','single','--window-size','1280x800','--visual-scale-factor','1']
r={'binary_sha256':hashlib.sha256(binary.read_bytes()).hexdigest(),'command':cmd,'cycles':[],'scope':'Bounded main/menu resolve attribution, static pixels and native input only. Timestamp diagnostics on in both builds; no ptrace, total GPU duty, calibrated CPU or formal confidence/budget claim.'}
def save():(out/'result.json').write_text(json.dumps(r,indent=2)+'\n')
def xd(*args):
 x=subprocess.run(['xdotool',*map(str,args)],capture_output=True,text=True)
 if x.returncode and args[0]!='search':raise RuntimeError(x.stderr)
 return x.stdout.strip()
def until(fn):
 deadline=time.monotonic()+15
 while time.monotonic()<deadline:
  assert p.poll() is None,('native exited',p.returncode)
  result=fn()
  if result:return result
  time.sleep(.02)
 raise AssertionError('native endpoint timed out')
def samples():
 data=log.read_text();lines=data[:data.rfind('\n')+1].splitlines()
 return [json.loads(line.split('gpu_measurement ',1)[1]) for line in lines if 'gpu_measurement ' in line]
def capture(name):
 path=out/(name+'.png');subprocess.run(['import','-window',str(main),str(path)],check=True);return str(path)
with (out/'stderr.log').open('w') as f:
 p=subprocess.Popen(cmd,cwd=root,env=env,stdout=f,stderr=f)
 try:
  main=int(until(lambda:xd('search','--all','--onlyvisible','--pid',p.pid,'--name','Datum')).splitlines()[0]);xd('windowactivate','--sync',main);time.sleep(5)
  # Prime View menu without changing settings or the board.
  xd('mousemove','--window',main,196,16,'click',1);time.sleep(.3);xd('key','Escape');time.sleep(5)
  r['warm_start_line']=len(log.read_text().splitlines());r['warm_start_sample']=len(samples())
  for i in range(10):
   start=len(samples());xd('mousemove','--window',main,196,16,'click',1)
   def menu_ready():
    current=samples()[start:]
    return any(any(q[0]=='menu-text' for q in s['passes_ns']) for s in current)
   until(menu_ready);time.sleep(.05)
   if i==0:r['menu_capture']=capture('menu')
   opened=len(samples());xd('key','Escape')
   until(lambda:len(samples())>opened);time.sleep(.05)
   if i==0:r['closed_capture']=capture('closed')
   r['cycles'].append({'index':i,'start_sample':start,'opened_sample':opened,'end_sample':len(samples()),'focus_restored':int(xd('getwindowfocus'))==main});assert r['cycles'][-1]['focus_restored'];save()
  r['warm_end_sample']=len(samples());close_window(main)
  conn,_=listener.accept();conn.settimeout(2);receipt=b''
  while not receipt.endswith(b'\n'):
   part=conn.recv(1);assert part;receipt+=part
  r['shutdown']=json.loads(receipt);assert r['shutdown']['pid']==p.pid and r['shutdown']['phase']=='drained_device_live';conn.sendall(b'!');conn.close()
  r['exit_code']=p.wait(timeout=10);assert r['exit_code']==0
  r['samples']=samples();r['warm_incomplete']=[x for x in log.read_text().splitlines()[r['warm_start_line']:] if 'gpu_measurement_incomplete ' in x];assert not r['warm_incomplete']
 except BaseException as e:r['error']=repr(e);raise
 finally:
  if p.poll() is None:
   p.terminate()
   try:p.wait(timeout=5)
   except subprocess.TimeoutExpired:p.kill();p.wait(timeout=5)
  listener.close();(out/'observer.sock').unlink(missing_ok=True);save()
