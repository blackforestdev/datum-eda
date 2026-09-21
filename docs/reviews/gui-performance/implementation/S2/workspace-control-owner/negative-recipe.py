from pathlib import Path
import subprocess
p=Path('crates/gui-render/src/render/frame_preparation.rs');original=p.read_bytes();s=original.decode()
old='&mut self.control_meshes,'
assert s.count(old)==1
try:
 p.write_text(s.replace(old,'&mut crate::global_preferences_primitives::ControlMeshCache::default(),'))
 with open('/tmp/pm045-s2-workspace-controls-negative.log','w') as log:
  r=subprocess.run(['python3','scripts/run_cargo_guarded.py','--workload','proof','--','cargo','test','--offline','-p','datum-gui-render','--lib','--features','visual','new_project_controls_retain','--','--ignored','--nocapture'],stdout=log,stderr=subprocess.STDOUT)
 assert r.returncode and 'test result: FAILED' in Path('/tmp/pm045-s2-workspace-controls-negative.log').read_text()
 print('Transient-owner negative rejected')
finally:
 p.write_bytes(original)
 assert p.read_bytes()==original
