from pathlib import Path
import subprocess
history=Path('crates/gui-app/src/retained_scene_history.rs');hit=Path('crates/gui-viewport/src/hit.rs');original={p:p.read_bytes() for p in [history,hit]}
variants=[('budget',history,'self.owned_history_bytes().saturating_add(bytes) > self.budget','false','datum-gui-app','retained_scene_history'),('active',history,'self.owned_history_bytes().saturating_add(self.active_bytes) > self.budget','self.owned_history_bytes() > self.budget','datum-gui-app','retained_scene_history'),('capacity',hit,'path.capacity()','path.len()','datum-gui-viewport','retained_payload_counts_spare')]
try:
 for name,p,old,new,package,test in variants:
  s=original[p].decode();assert old in s;p.write_text(s.replace(old,new,1 if name=='capacity' else -1))
  path=Path('/tmp/pm045-s2-history-negative-'+name+'.log')
  with path.open('w') as log:
   r=subprocess.run(['python3','scripts/run_cargo_guarded.py','--workload','proof','--','cargo','test','--offline','-p',package,test,'--','--nocapture'],stdout=log,stderr=subprocess.STDOUT)
  assert r.returncode and 'test result: FAILED' in path.read_text(),name
  print(name,'rejected',flush=True);p.write_bytes(original[p])
finally:
 for p,data in original.items():p.write_bytes(data)
