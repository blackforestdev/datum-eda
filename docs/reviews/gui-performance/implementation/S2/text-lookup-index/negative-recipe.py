from pathlib import Path
import subprocess
p=Path('crates/gui-render/src/render/text_buffer_cache.rs');original=p.read_bytes();s=original.decode()
variants=[('collision',s.replace('if matches_run(&entry.key, run, old_extent) {','if true {')),('linear',s.replace('self.lookup[first..]','self.lookup[..]').replace('.take_while(|(hash, _)| *hash == fingerprint)','.take_while(|_| true)'))]
try:
 for name,source in variants:
  assert source != s;p.write_text(source)
  path=Path('/tmp/pm045-s2-text-index-negative-'+name+'.log')
  with path.open('w') as log:
   r=subprocess.run(['python3','scripts/run_cargo_guarded.py','--workload','proof','--','cargo','test','--offline','-p','datum-gui-render','--lib','--features','visual','indexed_lookup_checks_collisions','--','--nocapture'],stdout=log,stderr=subprocess.STDOUT)
  assert r.returncode and 'test result: FAILED' in path.read_text(),name
  print(name,'rejected',flush=True);p.write_bytes(original)
finally:p.write_bytes(original)
