import json,hashlib,subprocess,tempfile
from pathlib import Path
out=Path(tempfile.mkdtemp(prefix='pm045-x11-campaign-v3-'));print(out,flush=True)
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest()
r={'purpose':'V3 quiet startup-only XOpenDisplay address capture; V2 per-event logger may perturb critical timing. No event function wrappers bound; pure queue/thread inspection only afterfailure; max3runs9cycles,stopfirstfailure,onepost-failurepointerwake. No acceptance or performance inference.','binary_sha256':sha('target/release/datum-gui'),'source_sha256':{p:sha(p) for p in ['/tmp/pm045_x11_audit_v3.c','/tmp/pm045_x11_audit_v3.so','/tmp/pm045_xlib_offsets.c','/tmp/pm045_xlib_offsets','/tmp/pm045-x11-close-v3.py']},'runs':[]}
(out/'declaration.json').write_text(json.dumps(r,indent=2)+'\n')
for i in range(1,4):
 with (out/f'{i}.log').open('w') as f:p=subprocess.run(['python3','/tmp/pm045-x11-close-v3.py','off'],stdout=f,stderr=subprocess.STDOUT)
 path=(out/f'{i}.log').read_text().splitlines()[0];r['runs'].append({'index':i,'path':path,'returncode':p.returncode});(out/'result.json').write_text(json.dumps(r,indent=2)+'\n');print(r['runs'][-1],flush=True)
 if p.returncode:break
