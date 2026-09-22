from pathlib import Path
import subprocess, os, json, hashlib
root=Path('/home/bfadmin/Documents/datum-eda'); copy=Path('/tmp/pm045-extent-negative'); out=Path('/tmp/pm045-visible-extent-corrected-proof'); out.mkdir()
p=copy/'crates/gui-render/src/render/preferences_rows.rs'; original=p.read_bytes()
old='scroll.layout(viewport, total);'; new='let total = starts.iter().filter(|(top, group, height, _, _)| *top + group + height > scroll.offset() && *top < scroll.offset() + viewport.height).map(|(_, group, height, _, _)| group + height).sum(); scroll.layout(viewport, total);'
assert original.decode().count(old)==1
(copy/'crates/gui-render/src/render/preferences_scene.rs').write_bytes((root/'crates/gui-render/src/render/preferences_scene.rs').read_bytes())
cmd=['python3',str(copy/'scripts/run_cargo_guarded.py'),'--workload','proof','--','cargo','test','--offline','--manifest-path',str(copy/'Cargo.toml'),'-p','datum-gui-render','--lib','continuous_scroll_clips_rows_and_hits_with_stable_content_bounds','--','--test-threads=1','--nocapture']
env=os.environ.copy(); env['CARGO_TARGET_DIR']=str(root/'target')
try:
 p.write_text(original.decode().replace(old,new))
 (out/'negative.patch').write_bytes(subprocess.check_output(['git','-C',str(copy),'diff','--unified=0','--',str(p)]))
 with (out/'negative.log').open('w') as log: negative=subprocess.run(cmd,env=env,stdout=log,stderr=log).returncode
finally: p.write_bytes(original)
with (out/'restored.log').open('w') as log: restored=subprocess.run(cmd,env=env,stdout=log,stderr=log).returncode
(out/'result.json').write_text(json.dumps({'command':cmd,'source_sha256':hashlib.sha256(original).hexdigest(),'negative_exit':negative,'restored_exit':restored},indent=2)+'\n')
assert negative==101 and restored==0
assert '0 passed; 1 failed' in (out/'negative.log').read_text()
assert '1 passed; 0 failed' in (out/'restored.log').read_text()
print('Expected extent negative detected; byte-restored positive passes',flush=True)
