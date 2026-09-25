import subprocess,os,json
from pathlib import Path
commands=[('native-reference',['cargo','test','-p','datum-gui-render','--features','visual','--lib','stroke_vertex_color_matches_previous_shader','--offline','--','--ignored','--nocapture','--test-threads=1']),('clippy',['cargo','clippy','-p','datum-gui-render','--all-targets','--features','visual','--offline','--','-D','warnings']),('release',['cargo','build','--release','-p','datum-gui-app','--bin','datum-gui','--offline'])]
results=[]
for name,cmd in commands:
 env=os.environ.copy()
 if name=='native-reference':env.update(PM045_STROKE_NATIVE_PROJECT='/tmp/pm045-admission-hqugp169/project',EDA_CLI_BIN=str(Path('target/release/datum-eda').resolve()))
 path=Path('/tmp/pm045-stroke-final-'+name+'.log')
 with path.open('w') as f:r=subprocess.run(['python3','scripts/run_cargo_guarded.py','--workload','proof','--',*cmd],env=env,stdout=f,stderr=subprocess.STDOUT)
 results.append({'name':name,'command':cmd,'returncode':r.returncode,'log':str(path)});Path('/tmp/pm045-stroke-final-proof-results.json').write_text(json.dumps(results,indent=2)+'\n');print(results[-1],flush=True)
 if r.returncode:raise SystemExit(r.returncode)
