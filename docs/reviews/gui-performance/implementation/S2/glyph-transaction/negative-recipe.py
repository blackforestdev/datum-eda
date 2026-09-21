from pathlib import Path
import subprocess,hashlib
p=Path('crates/gui-render/src/render/gpu_text.rs');original=p.read_bytes();s=original.decode()
mutations={
 'retry-workspace':('self.prepare_text_pair(device, queue, prepared, &workspace, &overlay, !overlay_only)','self.prepare_text_pair(device, queue, prepared, &workspace, &overlay, false)'),
 'cache-revision':('*old_revision == revision && old == signature','{ let _ = old_revision; old == signature }'),
}
try:
 for name,(old,new) in mutations.items():
  assert s.count(old)==1,(name,s.count(old))
  p.write_text(s.replace(old,new))
  with open('/tmp/pm045-s2-glyph-negative-'+name+'.log','w') as log:
   result=subprocess.run(['python3','scripts/run_cargo_guarded.py','--workload','proof','--','cargo','test','--offline','-p','datum-gui-render','--lib','--features','visual','shared_atlas_retry_and_cache_reindex_refresh_workspace_glyphs','--','--ignored','--test-threads=1','--nocapture'],stdout=log,stderr=subprocess.STDOUT)
  output=Path('/tmp/pm045-s2-glyph-negative-'+name+'.log').read_text()
  assert result.returncode!=0 and 'test result: FAILED' in output,(name,result.returncode)
  print(name,'rejected by GPU regression',flush=True)
  p.write_bytes(original)
finally:
 p.write_bytes(original)
 assert hashlib.sha256(p.read_bytes()).digest()==hashlib.sha256(original).digest()
