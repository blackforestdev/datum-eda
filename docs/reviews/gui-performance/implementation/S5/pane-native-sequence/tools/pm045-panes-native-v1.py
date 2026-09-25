import os,sys,json,time,hashlib,subprocess,tempfile,socket
from pathlib import Path
from pm045_wm_close import close_window
root=Path('/home/bfadmin/Documents/datum-eda');out=Path(tempfile.mkdtemp(prefix='pm045-panes-native-'));print(out,flush=True)
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest()
binary=root/'target/release/datum-gui';expected='b8c1fa1bf390d0dd768554b7b86548a6dc9b9777d8f6fdb5a1c7107f8e7e2b5e';assert sha(binary)==expected
project=Path('/tmp/pm045-admission-hqugp169/project')
def modelsha():
 m=json.loads((project/'board/board.json').read_text());m.pop('uuid',None);return hashlib.sha256(json.dumps(m,sort_keys=True,separators=(',',':')).encode()).hexdigest()
assert modelsha()=='33e62de1c1da2020f0608444a4802eac23fb97a9f56cc8cf87c844b6077499ed'
source=json.loads(Path('/tmp/pm045-focus-campaign-zaw4cz7n/declaration.json').read_text())['source_sha256']
for path,digest in source.items():assert sha(root/path)==digest,path
report={'purpose':'W-PANES native structural sequence only','candidate':subprocess.check_output(['git','rev-parse','HEAD'],cwd=root,text=True).strip(),'binary_sha256':expected,'normalized_model_sha256':modelsha(),'source_sha256':source,'driver_sha256':sha(__file__),'schedule':'five second warmup, 60 actions at 500ms start intervals: split vertical, focus next, close focused, focus next; initial/final two Board leaves; five second tail','oracle':'Every accepted menu action has native dispatch receipt. Split adds one inherited camera and retains existing ids/cameras/focus; focus follows ordered leaves; close removes only focused id and retains survivors; after every four actions original two ids survive. No project edits.','limits':['Verbose and action-state diagnostics enabled; no numerical CPU/GPU/latency/resource cap or formal relative inference.','One X11/1x run; other configurations, three-trial qualification and independent replay remain open.','Endpoint compositor images are visual evidence, not physical presentation calibration.'],'actions':[]}
def save():(out/'result.json').write_text(json.dumps(report,indent=2)+'\n')
save();(out/'declaration.json').write_text(json.dumps(report,indent=2)+'\n')
log=out/'native.log';log.touch();err=out/'stderr.log';err.touch()
env=os.environ.copy()
for key in list(env):
 if key.startswith(('DATUM_DIAGNOSTIC_','DATUM_GPU_DIAGNOSTIC_','DATUM_RESOURCE_TRACE','DATUM_GPU_ALLOCATION_TRACE','DATUM_PRIVATE_TEXT_TRACE')):env.pop(key)
for key in ('WAYLAND_DISPLAY','LD_AUDIT','PM045_X11_AUDIT_PATH'):env.pop(key,None)
env.update(WINIT_UNIX_BACKEND='x11',WINIT_X11_SCALE_FACTOR='1',XDG_CONFIG_HOME=str(out/'config'),XDG_CACHE_HOME=str(out/'cache'),DATUM_GUI_LOG=str(log),DATUM_GUI_VERBOSE_LOG='1',DATUM_GPU_MEASUREMENTS='0',DATUM_ACTION_EVIDENCE='1',EDA_CLI_BIN=str(root/'target/release/datum-eda'),DATUM_MEASUREMENT_SHUTDOWN_SOCKET=str(out/'observer.sock'))
cmd=[str(binary),'--project-root',str(project),'--initial-layout','horizontal-split','--window-size','1280x800','--visual-scale-factor','1'];report['command']=cmd
listener=socket.socket(socket.AF_UNIX,socket.SOCK_STREAM);listener.bind(str(out/'observer.sock'));listener.listen(1);listener.settimeout(15)
p=None;stream=None

def xd(*args):
 r=subprocess.run(['xdotool',*map(str,args)],capture_output=True,text=True,timeout=5)
 if r.returncode and args[0]!='search':raise RuntimeError(r.stderr)
 return r.stdout.strip()
def records():
 text=err.read_text();return [json.loads(line.split('DATUM_ACTION_EVIDENCE ',1)[1]) for line in text[:text.rfind('\n')+1].splitlines() if 'DATUM_ACTION_EVIDENCE ' in line]
def state():return [r['value'] for r in records() if r['event']=='state'][-1]
def until(fn,seconds=10):
 end=time.monotonic()+seconds
 while time.monotonic()<end:
  assert p.poll() is None,('process exit',p.returncode)
  result=fn()
  if result:return result
  time.sleep(.005)
 raise AssertionError('native condition timeout')
def cameras(s):return {r['pane']:r['camera'] for r in s['pane_cameras']}
def capture(name):
 path=out/(name+'.png');subprocess.run(['spectacle','-b','-n','-a','-e','-S','-o',str(path)],capture_output=True,check=True,timeout=15)
 from PIL import Image
 assert Image.open(path).size==(1280,800)
def action(index,kind,due):
 assert int(xd('getwindowfocus'))==wid
 before=state();previous=max(r['sequence'] for r in records());started=time.monotonic_ns()
 # Open the actual View menu with native pointer input, then select its known row.
 xd('mousemove','--window',wid,189,16,'click',1)
 until(lambda:state()['menu']=='View',.35)
 row={'split':8,'switch':12,'close':10}[kind]
 xd('key','--delay',5,*(['Down']*row+['Return']))
 key={'split':'view.split_vertical','switch':'view.focus_next','close':'view.close_pane'}[kind]
 receipt=until(lambda:next((r for r in records() if r['sequence']>previous and r['event']=='dispatch' and r['value']['dispatch_key']==key and r['value']['invoked']),None),.4)
 until(lambda:state()['menu'] is None,.3)
 after=state();b=cameras(before);a=cameras(after);focused=before['focused_pane'];ids=list(b)
 if kind=='split':
  assert len(a)==len(b)+1 and set(b)<set(a)
  assert all(a[k]==v for k,v in b.items())
  assert a[next(iter(set(a)-set(b)))]==b[focused]
  assert after['focused_pane']==focused
 elif kind=='switch':
  assert a==b
  assert after['focused_pane']==ids[(ids.index(focused)+1)%len(ids)]
 else:
  assert set(a)==set(b)-{focused}
  assert all(a[k]==v for k,v in b.items() if k!=focused)
  assert after['focused_pane'] in a
 assert all(r['content']=='Board' for r in after['pane_cameras'])
 report['actions'].append({'index':index,'kind':kind,'scheduled_ns':due,'started_ns':started,'acknowledged_ns':time.monotonic_ns(),'dispatch':receipt,'before':{'focused':focused,'cameras':b},'after':{'focused':after['focused_pane'],'cameras':a}});save()
 if index%4==3:assert set(a)==initial_ids
 assert time.monotonic_ns()<due+500_000_000,'action exceeded next deadline'

(out/'display-before.txt').write_text(subprocess.check_output(['kscreen-doctor','-o'],text=True));assert '1.05' in (out/'display-before.txt').read_text()
try:
 subprocess.run(['kscreen-doctor','output.eDP-1.scale.1'],check=True,capture_output=True);time.sleep(2)
 (out/'display-during.txt').write_text(subprocess.check_output(['kscreen-doctor','-o'],text=True))
 stream=err.open('a');p=subprocess.Popen(cmd,cwd=root,env=env,stdout=stream,stderr=stream,start_new_session=True);report['pid']=p.pid;save()
 wid=int(until(lambda:xd('search','--all','--onlyvisible','--pid',p.pid,'--name','^Datum EDA')).splitlines()[0]);report['window']=wid
 xd('windowactivate','--sync',wid);until(lambda:int(xd('getwindowfocus'))==wid)
 until(lambda:'frame present end' in log.read_text())
 until(lambda:len(records())>0)
 initial=state();assert len(cameras(initial))==2;assert initial['window_size']==[1280,800] and initial['scale']==1
 initial_ids=set(cameras(initial));report['initial_state']=initial;capture('initial');save();time.sleep(5)
 start=time.monotonic_ns();report['workload_start_ns']=start
 for index in range(60):
  due=start+index*500_000_000;delay=(due-time.monotonic_ns())/1e9
  if delay>0:time.sleep(delay)
  action(index,('split','switch','close','switch')[index%4],due)
 delay=(start+30_000_000_000-time.monotonic_ns())/1e9
 if delay>0:time.sleep(delay)
 report['workload_end_ns']=time.monotonic_ns();time.sleep(5);report['final_state']=state();capture('final')
 assert cameras(state())==cameras(initial) and state()['focused_pane']==initial['focused_pane']
 close_window(wid)
 connection,_=listener.accept();connection.settimeout(2)
 with connection:
  receipt=b''
  while not receipt.endswith(b'\n'):
   part=connection.recv(1);assert part and len(receipt)<4096;receipt+=part
  report['shutdown_receipt']=json.loads(receipt);assert report['shutdown_receipt']['pid']==p.pid and report['shutdown_receipt']['phase']=='drained_device_live';connection.sendall(b'!')
 p.wait(timeout=15);report['normal_exit_code']=p.returncode;assert p.returncode==0
except BaseException as e:
 report['error']=repr(e);raise
finally:
 if p and p.poll() is None:
  p.terminate()
  try:p.wait(timeout=5)
  except subprocess.TimeoutExpired:p.kill();p.wait(timeout=5)
  report['forced_cleanup']=True
 if stream:stream.close()
 listener.close();(out/'observer.sock').unlink(missing_ok=True)
 subprocess.run(['kscreen-doctor','output.eDP-1.scale.1.05'],check=True,capture_output=True)
 (out/'display-after.txt').write_text(subprocess.check_output(['kscreen-doctor','-o'],text=True));report['display_restored']=True
 report['binary_unchanged']=sha(binary)==expected;report['model_unchanged']=modelsha()==report['normalized_model_sha256'];save()
