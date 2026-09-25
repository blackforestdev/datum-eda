import json,hashlib,tarfile,io,shutil
from pathlib import Path
from PIL import Image,ImageChops
root=Path.cwd();base=root/'docs/reviews/gui-performance/implementation/S5/pane-native-sequence';base.mkdir(exist_ok=False)
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
review=Path('/tmp/pm045-route-review-progress.json');cache=json.loads(review.read_text());assert all(sha(Path(p))==h for p,h in cache['reviewed_sources_consumers'].items())
run=Path('/tmp/pm045-panes-native-45e37ela');r=json.loads((run/'result.json').read_text());assert len(r['actions'])==60 and r['normal_exit_code']==0 and r['display_restored'] and r['model_unchanged'] and r['binary_unchanged']
assert [a['kind'] for a in r['actions']]==['split','switch','close','switch']*15
assert all(a['scheduled_ns']==r['workload_start_ns']+a['index']*500000000 for a in r['actions'])
rows=(run/'native.log').read_text().splitlines();native=[s for s in rows if 'window event WindowId('+str(r['window'])+') keyboard physical=' in s and 'state=Pressed' in s];enters=[s for s in native if 'physical=Code(Enter)' in s];downs=[s for s in native if 'physical=Code(ArrowDown)' in s];assert len(enters)==60 and len(downs)==630
initial=Image.open(run/'initial.png').convert('RGB');final=Image.open(run/'final.png').convert('RGB');board=(227,65,981,385);diff=ImageChops.difference(initial.crop(board),final.crop(board));assert not diff.getbbox()
analysis={'actions':60,'cycles':15,'native_enter_presses':len(enters),'native_down_presses':len(downs),'workload_seconds':(r['workload_end_ns']-r['workload_start_ns'])/1e9,'max_controller_start_lateness_ms':max((a['started_ns']-a['scheduled_ns'])/1e6 for a in r['actions']),'max_controller_ack_elapsed_ms':max((a['acknowledged_ns']-a['started_ns'])/1e6 for a in r['actions']),'latency_limit':'controller/menu-navigation elapsed, not semantic action CPU or calibrated input-to-photon latency','endpoint_board_interior_rgb_ae':0,'endpoint_board_crop':board,'endpoint_scope':'Board interior only; focus/header/footer/console appropriately differ; no full-image equality claim','normal_exit':0,'device_live_receipt':r['shutdown_receipt'],'qualification_pass':False}
(run/'analysis.json').write_text(json.dumps(analysis,indent=2)+'\n')
archives=[]
for name,path in [('rejected-receipt-oracle',Path('/tmp/pm045-panes-native-774564bv')),('candidate-sequence',run)]:
 files=[p for p in sorted(path.iterdir()) if p.is_file()];manifest={p.name:{'bytes':p.stat().st_size,'sha256':sha(p)} for p in files};archive=base/(name+'.tar.xz')
 with tarfile.open(archive,'w:xz') as t:
  for p in files:t.add(p,arcname=p.name,recursive=False)
  data=(json.dumps(manifest,indent=2)+'\n').encode();info=tarfile.TarInfo('MANIFEST.json');info.size=len(data);t.addfile(info,io.BytesIO(data))
 with tarfile.open(archive) as t:
  for n,v in manifest.items():
   b=t.extractfile(n).read();assert len(b)==v['bytes'] and hashlib.sha256(b).hexdigest()==v['sha256']
 archives.append({'file':archive.name,'sha256':sha(archive),'members':len(files),'original_path':str(path),'raw_bytes':sum(v['bytes'] for v in manifest.values())})
(base/'archive-index.json').write_text(json.dumps(archives,indent=2)+'\n')
(base/'tools').mkdir()
for path in ('/tmp/pm045-panes-native-v1.py','/tmp/pm045-panes-native-v2.py','/tmp/pm045_wm_close.py',__file__):shutil.copy2(path,base/'tools'/Path(path).name)
for name in ('initial.png','final.png'):shutil.copy2(run/name,base/name)
report={'status':'partial native structural evidence; no S5 acceptance','frontier_step':'GPI-S5','candidate_commit':r['candidate'],'binary_sha256':r['binary_sha256'],'normalized_model_sha256':r['normalized_model_sha256'],'scope':'Pinned F-DOA Board plus existing unadmitted Schematic placeholder, X11/Xwayland Intel P630 Vulkan, physical display1x60Hz; client1280x800; no resolved schematic claim. Default split creates complementary content; native production semantics unchanged.','recipe':r['schedule'],'result':analysis,'native_state_oracles':r['oracle'],'failure_retained':'v1 completed the first split but waited for an unavailable registry dispatch record. It also incorrectly described all leaves as Board. Native input was delivered; default split adds complementary Schematic content. Forced cleanup and display restoration retained. v2 uses native Enter delivery, menu/echo/state receipts and explicit placeholder scope, not a performance rerun.','visual_review':'Both endpoint compositor captures inspected. Board content remains exact within the declared interior crop. Focus moves to Schematic placeholder after15cycles, headers/footer and echo reflect the last action.','remaining':['Three-trial native configuration coverage and distinct reviewer replay','Wayland and supported physical scale rows','Prescribed mutation negatives, hidden-store retirement and document replacement','Complete resource/no-leak and numerical CPU/GPU/latency proof','Resolved schematic fixture admission, T2 and all pinned resize/temporal exclusions remain unqualified','Endurance and owner UX acceptance'],'preserved':'No product/dependency changes; S0-S4 and GEFN preserved. No acceptance rules or row statuses changed.'}
(base/'review.json').write_text(json.dumps(report,indent=2)+'\n')
map_path=Path('docs/reviews/gui-performance/implementation-map.json');m=json.loads(map_path.read_text());row=next(r for r in m['requirements'] if r['id']=='SH-11');artifact=str((base/'review.json').relative_to(root));row['observed_artifacts'].append(artifact);row['s5_native_pane_sequence']={'status':'partial; qualification remains open','evidence':artifact,'boundaries':report['remaining']};m['s5_native_pane_sequence']={'status':report['status'],'qualification_pass':False,'evidence':artifact,'limits':report['remaining']};map_path.write_text(json.dumps(m,indent=2)+'\n');cache['reviewed_sources_consumers'][str(map_path)]=sha(map_path);review.write_text(json.dumps(cache,indent=2)+'\n')
print(json.dumps({'archives':archives,'analysis':analysis},indent=2))
