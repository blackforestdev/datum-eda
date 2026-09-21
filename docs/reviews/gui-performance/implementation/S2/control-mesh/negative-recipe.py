from pathlib import Path
import subprocess
p=Path('crates/gui-render/src/render/control_mesh.rs');original=p.read_bytes();s=original.decode()
mutations=[('dpi','dpi: self.dpi.to_bits(),','dpi: 1.0_f32.to_bits(),','placement_and_color_reuse_control_mesh_but_geometry_and_dpi_miss',False),('no-reuse','.position(|entry| entry.key == key)','.position(|entry| entry.key == key && false)','renderer_owned_preferences_meshes_stay_warm_and_match_fresh_pixels',True)]
try:
 for name,old,new,test,gpu in mutations:
  assert s.count(old)==1,(name,s.count(old));p.write_text(s.replace(old,new))
  with open('/tmp/pm045-s2-controls-negative-'+name+'.log','w') as log:
   result=subprocess.run(['python3','scripts/run_cargo_guarded.py','--workload','proof','--','cargo','test','--offline','-p','datum-gui-render','--lib','--features','visual',test,'--','--test-threads=1','--nocapture']+(['--ignored'] if gpu else []),stdout=log,stderr=subprocess.STDOUT)
  assert result.returncode and 'test result: FAILED' in Path('/tmp/pm045-s2-controls-negative-'+name+'.log').read_text(),name
  print(name,'rejected',flush=True);p.write_bytes(original)
finally:
 p.write_bytes(original);assert p.read_bytes()==original
