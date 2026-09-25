from pathlib import Path
import subprocess,shutil,json,hashlib
root=Path('.');out=Path('/tmp/pm045-msaa-golden-comparison');out.mkdir(exist_ok=True)
paths=[Path('crates/gui-render/src/render/gpu_frame.rs'),Path('crates/gui-render/src/render/terminal_graphics.rs')];saved={p:p.read_bytes() for p in paths}
def collect(label):
 d=out/label;d.mkdir(exist_ok=True);records={}
 for p in Path('crates/gui-render/testdata/golden').rglob('*'):
  if p.is_file() and p.name.endswith(('.actual.png','.diff.png','.report.txt')):
   rel=p.relative_to('crates/gui-render/testdata/golden');target=d/rel;target.parent.mkdir(parents=True,exist_ok=True);shutil.copy2(p,target);records[str(rel)]=hashlib.sha256(p.read_bytes()).hexdigest();p.unlink()
 return records
candidate=collect('candidate')
try:
 for p in paths:p.write_bytes(subprocess.check_output(['git','show','HEAD:'+str(p)]))
 with open('/tmp/pm045-msaa-goldens-baseline.log','w') as log:rc=subprocess.run(['python3','scripts/run_cargo_guarded.py','--workload','proof','--','cargo','test','-p','datum-gui-render','--features','visual','--test','visual_goldens','--offline','--','--include-ignored','--test-threads=1'],stdout=log,stderr=subprocess.STDOUT).returncode
 baseline=collect('baseline')
 (out/'result.json').write_text(json.dumps({'baseline_returncode':rc,'candidate_artifacts':candidate,'baseline_artifacts':baseline,'identical_artifact_hashes':candidate==baseline},indent=2)+'\n')
finally:
 for p,data in saved.items():p.write_bytes(data)
