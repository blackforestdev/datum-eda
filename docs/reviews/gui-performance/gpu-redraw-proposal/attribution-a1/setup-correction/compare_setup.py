"""Offline comparison to archived native input, not diagnostic self-oracles."""
import hashlib,json,struct,tarfile
from pathlib import Path
from PIL import Image
root=Path(__file__).resolve().parents[6]
base=root/'docs/reviews/gui-performance/gpu-redraw-proposal/attribution-a1'
run=root/'target/gpu-attribution-a1-setup/run-1'
inputs=json.loads((base/'inputs.json').read_text())
with tarfile.open(root/'docs/reviews/gui-performance/gpu-redraw-proposal/r4-native-result/raw.tar.gz') as t:
 raw=t.extractfile(inputs['archived_input'].split(':',1)[1]).read()
assert hashlib.sha256(raw).hexdigest()==inputs['input_sha256']
archive=json.loads(raw)
rows=[json.loads(l) for l in (run/'observations.jsonl').read_text().splitlines()]
assert [r['kind'] for r in rows]==['identity','setup_case','setup_case','setup_complete']
assert rows[-1]['timing_samples']==rows[-1]['conformance_observations']==0
f32=lambda n:struct.unpack('<f',struct.pack('<f',n))[0]
refpath=Path(inputs['reference_png']['path']);assert hashlib.sha256(refpath.read_bytes()).hexdigest()==inputs['reference_png']['sha256']
ref=Image.open(refpath).convert('RGBA')
# Independent historical endpoint crosshair, not a case reference generated here.
line=ref.getpixel((300,80));assert line==ref.getpixel((245,150))
def outline(im):
 points=[(x,y) for y in range(75,726) for x in range(240,968) if (lambda c:c[0]>140 and c[1]>110 and c[2]<90)(im.getpixel((x,y)))]
 return [min(x for x,y in points),min(y for x,y in points),max(x for x,y in points),max(y for x,y in points)]
reference_outline=outline(ref)
results=[]
for case,sequence in enumerate([409,1048]):
 expected=next(r for r in archive['records'] if r['sequence']==sequence);assert expected==inputs['cases'][case]
 actual=rows[case+1]['actual']
 assert actual['layout']=='single' and actual['pane_count']==1
 assert actual['focus']==archive['final_focus']==inputs['focus']
 assert actual['selection']==archive['final_selection']==inputs['selection']=='None'
 assert actual['focused_pane']==archive['final_focused_pane']==0
 assert actual['board_frame']==[224,33,760,709]
 checks=[]
 for observed in actual['observations']:
  phase=observed['phase'];pin=expected[phase]
  assert observed['pointer']==pin['cursor']==pin['native_cursor']
  camera=[f32(x) for x in observed['camera_center_nm']]
  assert camera==pin['camera_center_nm'] and observed['camera_zoom']==pin['camera_zoom']
  assert observed['hit_tested_hover']==actual['assigned_hover']==pin['hover_utf8']
  assert observed['hover_surface']==pin['hover_surface']=='Board'
  assert observed['board_pane_id']==0 and observed['crosshair_style']=='FullViewport'
  assert observed['scene_viewport']==[240,75,728,651]
  image_path=run/f'case-{case}-{phase}.png';im=Image.open(image_path).convert('RGBA');assert list(im.size)==pin['extent']==[1280,800];assert pin['scale']==1
  x,y=map(int,observed['pointer']);arms=[(x,80),(x,720),(245,y),(960,y)]
  assert all(im.getpixel(p)==line for p in arms)
  assert im.getpixel((x+4,80))!=line and im.getpixel((245,y+4))!=line
  bounds=outline(im);assert bounds==reference_outline
  checks.append({'phase':phase,'pointer':observed['pointer'],'camera_float32_exact_nm':camera,'hover':observed['hit_tested_hover'],'image_sha256':hashlib.sha256(image_path.read_bytes()).hexdigest(),'crosshair_arm_points':arms,'crosshair_rgba':line,'board_outline_bbox':bounds})
 results.append({'case':case,'archived_sequence':sequence,'checks':checks})
result={'status':'pass_requested_setup_fields_only','archived_input_sha256':inputs['input_sha256'],'independent_reference_sha256':inputs['reference_png']['sha256'],'reference_board_outline_bbox':reference_outline,'cases':results,'new_timing_samples':0,'scope':'State comparisons use original native receipt. Archived endpoint PNG independently anchors layout/static board geometry and crosshair pixel color; it is not an exactH/V image oracle. Diagnostic outputs differ intentionally in pointer/hover and also in terminal label/status chrome; full raster equivalence not established.','camera_serialization':'Rust f32 Display emits shortest roundtrip decimals151634200,95122900. Parsing at the source f32 type recovers exactly151634208,95122896; no epsilon or modified camera.'}
(base/'setup-correction/comparison.json').write_text(json.dumps(result,indent=2)+'\n');print(result['status'])
