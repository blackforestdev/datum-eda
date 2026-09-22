from pathlib import Path
import subprocess,os,json,hashlib
root=Path('/home/bfadmin/Documents/datum-eda');copy=Path('/tmp/pm045-control-negatives');out=Path('/tmp/pm045-control-proof');out.mkdir()
p=copy/'crates/gui-render/src/render/control_mesh.rs';original=p.read_bytes();env=os.environ.copy();env['CARGO_TARGET_DIR']=str(root/'target');results=[]
for name,old,new,test in [('incomplete-key','dpi: self.dpi.to_bits(),','dpi: 1.0_f32.to_bits(),','placement_and_color_reuse_control_mesh_but_geometry_and_dpi_miss'),('pinned-entries','while self.entries.len() >= MAX_ENTRIES || self.payload_bytes + bytes > MAX_PAYLOAD_BYTES {','while false {','control_mesh_keys_bounds_and_lru_are_complete')]:
 cmd=['python3',str(copy/'scripts/run_cargo_guarded.py'),'--workload','proof','--','cargo','test','--offline','--manifest-path',str(copy/'Cargo.toml'),'-p','datum-gui-render','--lib',test,'--','--test-threads=1','--nocapture']
 assert old in original.decode()
 try:
  p.write_text(original.decode().replace(old,new));(out/(name+'.patch')).write_bytes(subprocess.check_output(['git','-C',str(copy),'diff','--unified=0','--',str(p)]))
  with (out/(name+'-negative.log')).open('w') as log:n=subprocess.run(cmd,env=env,stdout=log,stderr=log).returncode
 finally:p.write_bytes(original)
 with (out/(name+'-restored.log')).open('w') as log:r=subprocess.run(cmd,env=env,stdout=log,stderr=log).returncode
 results.append({'case':name,'command':cmd,'negative_exit':n,'restored_exit':r})
 (out/'result.json').write_text(json.dumps({'source_sha256':hashlib.sha256(original).hexdigest(),'results':results},indent=2)+'\n')
 assert n==101 and r==0
 assert '0 passed; 1 failed' in (out/(name+'-negative.log')).read_text()
 assert '1 passed; 0 failed' in (out/(name+'-restored.log')).read_text()
 print(name,'intended negative and restored positive checked',flush=True)
