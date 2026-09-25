from pathlib import Path
import json,os,hashlib,subprocess,tempfile,time
root=Path('/home/bfadmin/Documents/datum-eda');out=Path(tempfile.mkdtemp(prefix='pm045-msaa-campaign-'));print(out,flush=True)
binaries={'baseline':root/'target/pm045-msaa-baseline/datum-gui','candidate':root/'target/release/datum-gui'}
order=[('baseline',1),('candidate',1),('candidate',2),('baseline',2),('baseline',3),('candidate',3)]
r={'status':'declared before execution','order':order,'cycles_per_run':10,'warmup':'5s initial and5s after first menu open/close','binary_sha256':{k:hashlib.sha256(p.read_bytes()).hexdigest() for k,p in binaries.items()},'source_sha256':{p:hashlib.sha256((root/p).read_bytes()).hexdigest() for p in ['crates/gui-render/src/render/gpu_frame.rs','crates/gui-render/src/render/terminal_graphics.rs','crates/gui-render/src/render/gpu_pass_tests.rs']},'driver_sha256':hashlib.sha256(Path('/tmp/pm045-msaa-native.py').read_bytes()).hexdigest(),'method':'Same native menu actions and timestamp observation both builds, no ptrace. Three descriptive matched pairs only; no formal statistical or whole GPU duty acceptance. One menu and one closed pixel readback per run; image capture cost not subtracted.','failure':'Stop on structural/native/pixel failure; retain raw failures and numerical excesses. No selective resampling.','runs':[]}
model_path=Path('/tmp/pm045-admission-hqugp169/project/board/board.json')
def fixture_hash():
 model=json.loads(model_path.read_text());model.pop('uuid',None)
 return hashlib.sha256(json.dumps(model,sort_keys=True,separators=(',',':')).encode()).hexdigest()
assert fixture_hash()=='33e62de1c1da2020f0608444a4802eac23fb97a9f56cc8cf87c844b6077499ed'
r['normalized_fixture_sha256']=fixture_hash()
(out/'declaration.json').write_text(json.dumps(r,indent=2)+'\n')
for name,trial in order:
 path=out/f'{name}-{trial}'
 with (out/f'{name}-{trial}.log').open('w') as log:rc=subprocess.run(['python3','/tmp/pm045-msaa-native.py',str(binaries[name]),str(path)],cwd=root,stdout=log,stderr=subprocess.STDOUT).returncode
 r['runs'].append({'name':name,'trial':trial,'path':str(path),'returncode':rc});(out/'result.json').write_text(json.dumps(r,indent=2)+'\n');print(name,trial,rc,flush=True)
 if rc:raise SystemExit(rc)
 if name=='candidate' or trial>1:
  for image in ['menu','closed']:
   compare=subprocess.run(['compare','-metric','AE',str(out/'baseline-1'/f'{image}.png'),str(path/f'{image}.png'),'null:'],capture_output=True,text=True)
   r['runs'][-1].setdefault('pixel_comparisons',{})[image]={'returncode':compare.returncode,'difference':compare.stderr}
   (out/'result.json').write_text(json.dumps(r,indent=2)+'\n')
   if compare.returncode:raise SystemExit('pixel comparison failed')
r['status']='All six prescribed runs and pixel comparisons passed; descriptive analysis pending';(out/'result.json').write_text(json.dumps(r,indent=2)+'\n')

assert fixture_hash()==r['normalized_fixture_sha256']
