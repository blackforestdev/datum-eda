from pathlib import Path
import subprocess,os,json,hashlib
active=Path('/home/bfadmin/Documents/datum-eda');copy=Path('/tmp/pm045-wheel-negatives');out=Path('/tmp/pm045-wheel-negative-proof');out.mkdir()
command=['python3',str(copy/'scripts/run_cargo_guarded.py'),'--workload','proof','--','cargo','test','--offline','--manifest-path',str(copy/'Cargo.toml'),'-p','datum-gui-app','--bin','datum-gui','native_wheel_preserves_fractional_pixels_and_pane_locality','--','--test-threads=1','--nocapture']
env=os.environ.copy();env['CARGO_TARGET_DIR']=str(active/'target');results=[]
for name,path,old,new in [('rounded','crates/gui-app/src/global_preferences_window.rs','scroll.wheel(wheel.pixels(40.0))','scroll.wheel(wheel.pixels(40.0).round())'),('scaled','crates/gui-app/src/native_scroll_input.rs','position.y / f64::from(scale)','position.y * f64::from(scale)')]:
 p=copy/path;original=p.read_bytes();assert original== (active/path).read_bytes();s=original.decode();assert s.count(old)==1
 try:
  p.write_text(s.replace(old,new))
  (out/(name+'.patch')).write_bytes(subprocess.check_output(['git','-C',str(copy),'diff','--',path]))
  with (out/(name+'-negative.log')).open('w') as log:r=subprocess.run(command,env=env,stdout=log,stderr=log)
  assert r.returncode==101
  assert '0 passed; 1 failed' in (out/(name+'-negative.log')).read_text()
 finally:p.write_bytes(original)
 with (out/(name+'-restored.log')).open('w') as log:r=subprocess.run(command,env=env,stdout=log,stderr=log)
 assert r.returncode==0
 assert '1 passed; 0 failed' in (out/(name+'-restored.log')).read_text()
 results.append({'case':name,'path':path,'source_sha256':hashlib.sha256(original).hexdigest(),'negative_exit':101,'restored_exit':0})
 (out/'result.json').write_text(json.dumps({'command':command,'results':results},indent=2)+'\n');print(name,'expected negative and restored PASS',flush=True)
