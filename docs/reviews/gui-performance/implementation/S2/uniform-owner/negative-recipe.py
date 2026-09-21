from pathlib import Path
import subprocess
uniform=Path('crates/gui-render/src/render/uniform_buffer.rs');surface=Path('crates/gui-render/src/render/gpu_surface_pass.rs')
original={p:p.read_bytes() for p in [uniform,surface]}
variants=[('always-upload',uniform,'.is_some_and(|old| bytemuck::bytes_of(old) == bytes)','.is_some_and(|_| false)'),('stale-value',uniform,'.is_some_and(|old| bytemuck::bytes_of(old) == bytes)','.is_some()'),('retirement',surface,'        self.surface_scene_uniforms\n            .truncate(prepared.surface_passes().len());','')]
try:
 for name,p,old,new in variants:
  s=original[p].decode();assert s.count(old)==1;p.write_text(s.replace(old,new))
  path=Path('/tmp/pm045-s2-uniform-negative-'+name+'.log')
  with path.open('w') as log:
   r=subprocess.run(['python3','scripts/run_cargo_guarded.py','--workload','proof','--','cargo','test','--offline','-p','datum-gui-render','--lib','--features','visual','uniform_uploads_stay_warm','--','--ignored','--nocapture'],stdout=log,stderr=subprocess.STDOUT)
  assert r.returncode and 'test result: FAILED' in path.read_text(),name
  print(name,'rejected',flush=True);p.write_bytes(original[p])
finally:
 for p,data in original.items():p.write_bytes(data)
