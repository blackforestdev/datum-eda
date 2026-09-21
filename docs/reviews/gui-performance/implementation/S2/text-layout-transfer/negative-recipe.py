from pathlib import Path
import subprocess
p=Path('crates/gui-render/src/render/text_buffer_cache.rs');original=p.read_bytes()
try:
 s=original.decode();assert s.count('if entry.last_used_frame != self.frame {')==1
 p.write_text(s.replace('if entry.last_used_frame != self.frame {','if true {'))
 with open('/tmp/pm045-text-move-active-negative.log','w') as log:
  result=subprocess.run(['python3','scripts/run_cargo_guarded.py','--workload','proof','--','cargo','test','--offline','-p','datum-gui-render','--lib','--features','visual','relayout_moves_unused_entries'],stdout=log,stderr=subprocess.STDOUT)
 assert result.returncode!=0,'current-frame guard negative unexpectedly passed'
 assert 'assertion `left != right` failed' in Path('/tmp/pm045-text-move-active-negative.log').read_text()
finally:
 p.write_bytes(original)
