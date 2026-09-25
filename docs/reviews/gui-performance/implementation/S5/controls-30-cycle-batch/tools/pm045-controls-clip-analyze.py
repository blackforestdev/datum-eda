import json,hashlib
from pathlib import Path
from PIL import Image,ImageChops
p=Path('/tmp/pm045-controls-clip-native-ez9zcg5x');r=json.loads((p/'result.json').read_text());c=r['controls'][0];assert r['normal_exit_code']==0 and not r.get('error') and c['completed_cycles']==30 and c['automatic_focus_restored'];assert len(c['clip_probes'])==30
ref=Image.open(p/'PROJECT-0-search-refocus.png').convert('RGB');log=(p/'native.log').read_bytes();proof=[]
for probe in c['clip_probes']:
 cycle=probe['cycle'];assert probe['x']==900 and probe['y']==70
 im=Image.open(p/f'PROJECT-{cycle}-clip-probe.png').convert('RGB');assert ImageChops.difference(im,ref).getbbox() is None
 s=log[probe['begin_log_offset']:probe['end_log_offset']].decode()
 for state in ['Pressed','Released']:assert f"window event WindowId({c['window']}) mouse input Left {state}" in s
 proof.append({'cycle':cycle,'RGB_AE':0,'press_release_receipts':True})
geometry_sources=['crates/gui-render/src/render/preferences_rows.rs','crates/gui-render/src/global_preferences_dialog.rs','crates/gui-render/src/global_preferences_primitives.rs','crates/gui-viewport/src/scroll.rs']
out={'host':'PROJECT','probes':30,'normal_exit':0,'focus_restored':True,'checks':proof,'geometry':c['clip_geometry'],'geometry_source_sha256':{s:hashlib.sha256(Path(s).read_bytes()).hexdigest() for s in geometry_sources},'limits':'Code-derived overlap coordinates plus native button receipts and exact compositor state; no native numerical-state telemetry, physical latency or full prescribed negative/backend-scale/resource acceptance.'}
(p/'clip-analysis.json').write_text(json.dumps(out,indent=2)+'\n');print('30 probes verified; normal exit and Main focus restored')
print('displayrestored',json.load(open('/tmp/pm045-controls-followup-campaign-sq39z7ji/result.json'))['display_restored'])
