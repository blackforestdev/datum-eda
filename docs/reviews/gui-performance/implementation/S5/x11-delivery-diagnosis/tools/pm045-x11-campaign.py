import json,hashlib,subprocess,tempfile
from pathlib import Path
out=Path(tempfile.mkdtemp(prefix='pm045-x11-campaign-'));print(out,flush=True)
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest()
r={'purpose':'X11 event delivery diagnosis; max6runs9cycles,stopfirstfailure,onepost-failurepointerwake. No acceptance or performance inference.','binary_sha256':sha('target/release/datum-gui'),'source_sha256':{p:sha(p) for p in ['/tmp/pm045_x11_audit.c','/tmp/pm045_x11_audit.so','/tmp/pm045_xlib_offsets.c','/tmp/pm045_xlib_offsets','/tmp/pm045-x11-close.py']},'runs':[]}
(out/'declaration.json').write_text(json.dumps(r,indent=2)+'\n')
for i in range(1,7):
 with (out/f'{i}.log').open('w') as f:p=subprocess.run(['python3','/tmp/pm045-x11-close.py','off'],stdout=f,stderr=subprocess.STDOUT)
 path=(out/f'{i}.log').read_text().splitlines()[0];r['runs'].append({'index':i,'path':path,'returncode':p.returncode});(out/'result.json').write_text(json.dumps(r,indent=2)+'\n');print(r['runs'][-1],flush=True)
 if p.returncode:break
