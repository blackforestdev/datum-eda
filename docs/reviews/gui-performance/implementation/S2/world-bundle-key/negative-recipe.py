from pathlib import Path
import subprocess
p=Path('crates/gui-render/src/render/gpu_surface_pass/world_bundles.rs');original=p.read_bytes();s=original.decode()
variants=[('order','            && draw_batches(commands).eq(self.batches.iter().cloned())',''),('reuse','.is_some_and(|cached| cached.matches(vertex, stroke, bind_group, commands))','.is_some_and(|_| false)')]
try:
 for name,old,new in variants:
  assert s.count(old)==1;p.write_text(s.replace(old,new))
  path=Path('/tmp/pm045-s2-bundle-key-negative-'+name+'.log')
  with path.open('w') as log:
   r=subprocess.run(['python3','scripts/run_cargo_guarded.py','--workload','proof','--','cargo','test','--offline','-p','datum-gui-render','--lib','--features','visual','world_bundle_reuse_and_invalidation','--','--ignored','--nocapture'],stdout=log,stderr=subprocess.STDOUT)
  assert r.returncode and 'test result: FAILED' in path.read_text(),name
  print(name,'rejected',flush=True);p.write_bytes(original)
finally:p.write_bytes(original)
