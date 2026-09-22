from pathlib import Path
import subprocess,os,json,hashlib
root=Path('/home/bfadmin/Documents/datum-eda');copy=Path('/tmp/pm045-scroll-negatives');out=Path('/tmp/pm045-scroll-proof');out.mkdir()
p=copy/'crates/gui-viewport/src/scroll.rs';old=p.read_bytes();fixed=(root/'crates/gui-viewport/src/scroll.rs').read_bytes();env=os.environ.copy();env['CARGO_TARGET_DIR']=str(root/'target');results=[]
for name,source,needle,replacement,test,expected in [('original-grab',old,'self.drag_grab = Some(y - thumb.y);','self.drag_grab = Some(0.0);','thumb_endpoints_drag_page_and_resize_share_content_bounds',0),('corrected-grab',fixed,'self.drag_grab = Some(y - thumb.y);','self.drag_grab = Some(0.0);','thumb_endpoints_drag_page_and_resize_share_content_bounds',101),('stale-capture',fixed,'            self.release();','            // Negative: keep obsolete grab.','reflow_cancels_old_grab_but_unchanged_layout_preserves_drag',101)]:
 cmd=['python3',str(copy/'scripts/run_cargo_guarded.py'),'--workload','proof','--','cargo','test','--offline','--manifest-path',str(copy/'Cargo.toml'),'-p','datum-gui-viewport','--lib',test,'--','--test-threads=1','--nocapture']
 assert source.decode().count(needle)==1
 try:
  p.write_text(source.decode().replace(needle,replacement));(out/(name+'.patch')).write_bytes(subprocess.check_output(['git','-C',str(copy),'diff','--unified=0','--',str(p)]))
  with (out/(name+'-negative.log')).open('w') as log:n=subprocess.run(cmd,env=env,stdout=log,stderr=log).returncode
 finally:p.write_bytes(source)
 with (out/(name+'-restored.log')).open('w') as log:r=subprocess.run(cmd,env=env,stdout=log,stderr=log).returncode
 results.append({'case':name,'command':cmd,'negative_exit':n,'restored_exit':r,'source_sha256':hashlib.sha256(source).hexdigest()})
 (out/'result.json').write_text(json.dumps({'results':results},indent=2)+'\n')
 assert n==expected and r==0
 assert ('1 passed; 0 failed' if expected==0 else '0 passed; 1 failed') in (out/(name+'-negative.log')).read_text()
 assert '1 passed; 0 failed' in (out/(name+'-restored.log')).read_text()
 print(name,'expected outcome and restored positive checked',flush=True)
