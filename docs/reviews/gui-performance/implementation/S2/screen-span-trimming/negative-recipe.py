from pathlib import Path
import subprocess
p=Path('crates/gui-render/src/render/screen_buffer.rs');original=p.read_bytes();s=original.decode()
a=s.index('    let word = wgpu::COPY_BUFFER_ALIGNMENT as usize;',s.index('fn write_trimmed_span'))
b=s.index('    if begin < end {',a)
try:
 p.write_text(s[:a]+s[b:])
 with open('/tmp/pm045-s2-dirty-words-negative.log','w') as log:
  r=subprocess.run(['python3','scripts/run_cargo_guarded.py','--workload','proof','--','cargo','test','--offline','-p','datum-gui-render','--lib','--features','visual','production_dialog_upload_reuses','--','--ignored','--nocapture'],stdout=log,stderr=subprocess.STDOUT)
 assert r.returncode and 'test result: FAILED' in Path('/tmp/pm045-s2-dirty-words-negative.log').read_text()
 print('untrimmed-span negative rejected')
finally:p.write_bytes(original)
