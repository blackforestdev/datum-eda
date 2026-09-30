"""Run sealed rebuilt A1 diagnostic with unchanged controls and limits."""
import hashlib,json,os,signal,stat,subprocess,tarfile,time
from pathlib import Path
root=Path(__file__).resolve().parents[6]
os.chdir(root)
base=root/'docs/reviews/gui-performance/gpu-redraw-proposal/attribution-a1'
old=root/'target/gpu-attribution-a1'
out=root/'target/gpu-attribution-a1-completion/run-1'
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest()
def pin(p,h):
 assert sha(p)==h,f'hash mismatch: {p}'
 return h
assert not out.exists(),'relaunch already reserved; no retry'
d=json.loads((root/'target/gpu-attribution-a1-completion/declaration.json').read_text());state=json.loads((old/'execution-state.json').read_text());inputs=json.loads((base/'inputs.json').read_text())
assert all(state[x]==0 for x in ['build_exit','offline_controls_exit','default_check_exit'])
pins={str(d['binary']):pin(d['binary'],d['binary_sha256'])}
assert os.access(d['binary'],os.X_OK)
for name,h in [('packet.md','808f9220527ce2f53a0695f70d09fe0d89be0e1763f3ffbcf1112381c8b083c4'),('inputs.json','792bfec10c8e8b6729b8f4eb75d05f890018e398eb28fd9ce50bcd23d39233aa'),('readback-correction.md','02944763da7fe574588a2dee74c11a8a5ac25d996b199ed5b2c3a81fcd77eb83')]:pins[name]=pin(base/name,h)
receipt=json.loads((base/'readback-result/receipt.json').read_text())
for name,h in receipt['artifacts_sha256'].items():pins['failed/'+name]=pin(base/'readback-result'/name,h)
with tarfile.open(base/'readback-result/raw.tar.gz') as t:
 for name,h in d['source_hashes'].items():assert hashlib.sha256(t.extractfile('source/'+name).read()).hexdigest()==h,name
pins['cli']=pin(inputs['cli']['path'],inputs['cli']['sha256'])
pins['production_r4']=pin(root/'target/release/datum-gui',receipt['restoration']['production_binary_sha256'])
pins['reference']=pin(inputs['reference_png']['path'],inputs['reference_png']['sha256'])
board=Path(inputs['project']['path'])/'board/board.json';pins['board']=pin(board,inputs['project']['board_file_sha256']);model=json.loads(board.read_text());model.pop('uuid',None);assert hashlib.sha256(json.dumps(model,sort_keys=True,separators=(',',':')).encode()).hexdigest()==inputs['project']['normalized_board_sha256']
pins['analysis']=pin(old/'analyze.py',d['analysis_sha256'])
assert not subprocess.check_output(['git','diff','--name-only','HEAD','--','crates'],text=True).strip(),'production source dirty'
assert os.environ.get('DISPLAY')==d['display']==':1'
assert stat.S_ISSOCK(Path('/tmp/.X11-unix/X1').stat().st_mode)
mesa=subprocess.check_output(['dpkg-query','-W','-f=${Version}','mesa-vulkan-drivers'],text=True);assert mesa==d['package_mesa']
busy=[]
for p in Path('/proc').iterdir():
 if not p.name.isdigit():continue
 try:
  comm=(p/'comm').read_text().strip()
  if comm in ('cargo','rustc','datum-gui') or comm.startswith('datum_gui-'):busy.append([p.name,comm])
 except (FileNotFoundError,PermissionError,ProcessLookupError):pass
assert not busy,('competing process',busy)
auth=Path(os.environ.get('XAUTHORITY',str(Path.home()/'.Xauthority')));assert auth.is_file() and os.access(auth,os.R_OK),'Xauthority unreadable'
assert d['limits']=={'gpu_processes':1,'seconds':300,'observations':646,'submissions':2200,'diagnostic_cpu_mib':64,'numeric_mib':16,'tracked_gpu_mib':512}
out.mkdir()
env={k:v for k,v in os.environ.items() if not k.startswith('DATUM_')}
for key in ('WAYLAND_DISPLAY','LD_AUDIT','LD_PRELOAD'):env.pop(key,None)
for kind in ('CONFIG','CACHE','DATA','STATE'):
 p=out/kind.lower();p.mkdir();env['XDG_'+kind+'_HOME']=str(p)
env.update(WINIT_UNIX_BACKEND='x11',WINIT_X11_SCALE_FACTOR='1',DATUM_GPU_DIAGNOSTIC_BACKEND='vulkan',EDA_CLI_BIN=inputs['cli']['path'],DATUM_NATIVE_TEST_PROJECT=inputs['project']['path'],DATUM_A1_OUTPUT=str(out/'observations.jsonl'),DATUM_A1_ARTIFACT_DIR=str(out),RUST_TEST_THREADS='1')
command=[d['binary'],'--exact','native_gpu::attribution::attribution_native_study','--ignored','--nocapture','--test-threads=1'];assert command==[d['binary'],'--exact','native_gpu::attribution::attribution_native_study','--ignored','--nocapture','--test-threads=1']
driver_env={k:v for k,v in env.items() if k.startswith(('VK_','WGPU_','MESA_','LIBGL_','GALLIUM_','DRI_'))}
assert not driver_env,('unpinned driver override',driver_env)
pre={'driver_environment_overrides':driver_env,'head':subprocess.check_output(['git','rev-parse','HEAD'],text=True).strip(),'hashes':pins,'mesa':mesa,'limits':d['limits'],'prior_checks':'reused sealed successful build/offline/default checks','launch_environment':{k:v for k,v in env.items() if k.startswith(('DATUM_','WINIT_','XDG_')) or k in ('DISPLAY','XAUTHORITY','EDA_CLI_BIN','RUST_TEST_THREADS')},'xauthority_readable':True,'competing_processes':busy,'command':command,'launcher_sha256':sha(__file__),'reserved_relaunches':1,'additional_builds':1,'sampling_change':False}
(out/'preflight.json').write_text(json.dumps(pre,indent=2)+'\n')
preserved=Path(d['preserved_directory'])
(preserved/'launcher.py').write_bytes(Path(__file__).read_bytes())
(preserved/'preflight.json').write_bytes((out/'preflight.json').read_bytes())
(preserved/'declaration.json').write_bytes((root/'target/gpu-attribution-a1-completion/declaration.json').read_bytes())
(out/'analyze.py').write_bytes((old/'analyze.py').read_bytes())
print('All preflight checks passed; single relaunch reserved.',flush=True)
start=time.monotonic()
with (out/'native.log').open('wb') as log:
 proc=subprocess.Popen(command,env=env,stdout=log,stderr=subprocess.STDOUT,start_new_session=True)
 try:code=proc.wait(timeout=max(0,300-(time.monotonic()-start)));timedout=False
 except subprocess.TimeoutExpired:
  os.killpg(proc.pid,signal.SIGKILL);proc.wait();code=proc.returncode;timedout=True
result={'exit_code':code,'wall_seconds':time.monotonic()-start,'timeout':timedout,'relaunches':1,'binary_sha256_after':pin(d['binary'],d['binary_sha256'])}
(out/'execution-state.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(result))
