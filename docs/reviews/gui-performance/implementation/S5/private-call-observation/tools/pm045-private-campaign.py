from pathlib import Path
import subprocess,json,tempfile,hashlib
root=Path.cwd();out=Path(tempfile.mkdtemp(prefix='pm045-private-campaign-'));print(out,flush=True)
paths=['crates/gui-render/src/private_text_call.rs','crates/gui-render/src/private_text_observation.rs','crates/gui-render/src/private_text_observation_tests.rs','crates/gui-app/src/native_private_text_measurements.rs','crates/gui-app/src/app_shell.rs','crates/gui-app/src/app_native_events.rs']
r={'order':['on','off','overflow','existing'],'native_cycles_each_success':9,'source_sha256':{p:hashlib.sha256((root/p).read_bytes()).hexdigest() for p in paths},'binary_sha256':hashlib.sha256((root/'target/release/datum-gui').read_bytes()).hexdigest(),'status':'Declared before execution; bounded observer conformance, no budget/overhead/endurance qualification','runs':[]}
(out/'declaration.json').write_text(json.dumps(r,indent=2)+'\n')
for mode in r['order']:
 with (out/(mode+'.log')).open('w') as log:rc=subprocess.run(['python3','/tmp/pm045-private-native.py',mode],stdout=log,stderr=subprocess.STDOUT).returncode
 text=(out/(mode+'.log')).read_text();location=next((x for x in text.splitlines() if x.startswith('/tmp/pm045-private-native-')),None);r['runs'].append({'mode':mode,'returncode':rc,'path':location});(out/'result.json').write_text(json.dumps(r,indent=2)+'\n');print(mode,rc,location,flush=True)
 if rc:raise SystemExit(rc)
r['status']='Four declared native modes completed; analysis pending';(out/'result.json').write_text(json.dumps(r,indent=2)+'\n')
