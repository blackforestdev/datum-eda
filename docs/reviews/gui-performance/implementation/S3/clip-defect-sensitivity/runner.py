from pathlib import Path
import subprocess,os,json,hashlib
root=Path('/home/bfadmin/Documents/datum-eda');copy=Path('/tmp/pm045-clip-negatives');out=Path('/tmp/pm045-clip-proof');out.mkdir()
p=copy/'crates/gui-render/src/render/hit_clipping.rs';old=p.read_bytes();fixed=(root/'crates/gui-render/src/render/hit_clipping.rs').read_bytes();env=os.environ.copy();env['CARGO_TARGET_DIR']=str(root/'target');results=[]
def clamp(s):
 start=s.index('        for indices in [[0, 1, 2], [0, 2, 3]] {');end=s.index('    }\n    let start = text_start',start)
 return s[:start]+"        quads.push(Quad { points: quad.points.map(|(x,y)| (x.clamp(viewport.x, viewport.x + viewport.width), y.clamp(viewport.y, viewport.y + viewport.height))), color: quad.color });\n"+s[end:]
for name,source,mutate,test,expected in [('original-clamp',old,clamp,'four_edge_clip_preserves_triangle_area_and_pinned_content',0),('corrected-clamp',fixed,clamp,'four_edge_clip_preserves_triangle_area_and_pinned_content',101),('hidden-hits',fixed,lambda s:s.replace('    clip_new_hit_regions(hits, hit_start, viewport);','    let _ = (hits, hit_start);'),'descendant_text_and_hits_intersect_both_axes_without_touching_chrome',101)]:
 cmd=['python3',str(copy/'scripts/run_cargo_guarded.py'),'--workload','proof','--','cargo','test','--offline','--manifest-path',str(copy/'Cargo.toml'),'-p','datum-gui-render','--lib',test,'--','--test-threads=1','--nocapture']
 try:
  p.write_text(mutate(source.decode()));(out/(name+'.patch')).write_bytes(subprocess.check_output(['git','-C',str(copy),'diff','--unified=0','--',str(p)]))
  with (out/(name+'-negative.log')).open('w') as log:n=subprocess.run(cmd,env=env,stdout=log,stderr=log).returncode
 finally:p.write_bytes(source)
 with (out/(name+'-restored.log')).open('w') as log:r=subprocess.run(cmd,env=env,stdout=log,stderr=log).returncode
 results.append({'case':name,'command':cmd,'negative_exit':n,'restored_exit':r,'source_sha256':hashlib.sha256(source).hexdigest()})
 (out/'result.json').write_text(json.dumps({'results':results},indent=2)+'\n')
 assert n==expected and r==0
 assert ('1 passed; 0 failed' if expected==0 else '0 passed; 1 failed') in (out/(name+'-negative.log')).read_text()
 assert '1 passed; 0 failed' in (out/(name+'-restored.log')).read_text()
 print(name,'expected outcome and restored positive checked',flush=True)
