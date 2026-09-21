from pathlib import Path
import subprocess
p=Path('crates/gui-render/src/render/text_buffer_cache.rs');original=p.read_bytes();s=original.decode();old='shaped.get_or_insert(index);';assert s.count(old)==1
try:
 p.write_text(s.replace(old,'if old_extent == extent { shaped.get_or_insert(index); }'))
 with open('/tmp/pm045-s2-relayout-negative.log','w') as log:
  r=subprocess.run(['python3','scripts/run_cargo_guarded.py','--workload','proof','--','cargo','test','--offline','-p','datum-gui-render','--lib','--features','visual','extent_changes_relayout_cached_shaping_with_fresh_buffer_parity','--','--test-threads=1','--nocapture'],stdout=log,stderr=subprocess.STDOUT)
 assert r.returncode and 'test result: FAILED' in Path('/tmp/pm045-s2-relayout-negative.log').read_text()
 print('Repeated fresh shaping rejected by extent reuse regression')
finally:
 p.write_bytes(original)
 assert p.read_bytes()==original
