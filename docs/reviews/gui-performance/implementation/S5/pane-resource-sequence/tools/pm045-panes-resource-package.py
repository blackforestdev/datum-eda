import json,hashlib,tarfile,io,shutil,subprocess
from pathlib import Path
from PIL import Image,ImageChops
root=Path.cwd();base=root/'docs/reviews/gui-performance/implementation/S5/pane-resource-sequence';base.mkdir(exist_ok=False)
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
cachepath=Path('/tmp/pm045-route-review-progress.json');cache=json.loads(cachepath.read_text());assert all(sha(Path(p))==h for p,h in cache['reviewed_sources_consumers'].items())
run=Path('/tmp/pm045-panes-resource-izo05o5k');r=json.loads((run/'result.json').read_text());a=json.loads((run/'allocation-analysis.json').read_text());assert len(r['actions'])==60 and r['normal_exit_code']==0 and r['display_restored'] and r['binary_unchanged'] and r['model_unchanged']
rows=(run/'native.log').read_text().splitlines();keys=[l for l in rows if 'window event WindowId('+str(r['window'])+') keyboard physical=' in l and 'state=Pressed' in l];assert sum('physical=Code(Enter)' in l for l in keys)==60 and sum('physical=Code(ArrowDown)' in l for l in keys)==630
initial=Image.open(run/'initial.png').convert('RGB');final=Image.open(run/'final.png').convert('RGB');assert not ImageChops.difference(initial.crop((227,65,981,384)),final.crop((227,65,981,384))).getbbox()
a['native_actions']=60;a['native_enter_presses']=60;a['native_down_presses']=630;a['workload_seconds']=(r['workload_end_ns']-r['workload_start_ns'])/1e9;a['endpoint_board_interior_rgb_ae']=0;a['endpoint_board_crop']=[227,65,981,384]
(run/'qualification-analysis.json').write_text(json.dumps(a,indent=2)+'\n')
files=[p for p in sorted(run.iterdir()) if p.is_file()];manifest={p.name:{'bytes':p.stat().st_size,'sha256':sha(p)} for p in files};archive=base/'native.tar.xz'
with tarfile.open(archive,'w:xz') as t:
 for p in files:t.add(p,arcname=p.name,recursive=False)
 data=(json.dumps(manifest,indent=2)+'\n').encode();info=tarfile.TarInfo('MANIFEST.json');info.size=len(data);t.addfile(info,io.BytesIO(data))
with tarfile.open(archive) as t:
 for n,v in manifest.items():
  data=t.extractfile(n).read();assert len(data)==v['bytes'] and hashlib.sha256(data).hexdigest()==v['sha256']
(base/'archive-index.json').write_text(json.dumps([{'file':archive.name,'sha256':sha(archive),'original_path':str(run),'members':len(files),'raw_bytes':sum(v['bytes'] for v in manifest.values())}],indent=2)+'\n')
(base/'tools').mkdir()
for path in ('/tmp/pm045-panes-resource.py','/tmp/pm045-panes-resource-analyze.py','/tmp/pm045_wm_close.py',__file__):shutil.copy2(path,base/'tools'/Path(path).name)
for name in ('initial.png','final.png'):shutil.copy2(run/name,base/name)
report={'status':'partial native allocation-lifetime evidence; S5 qualification remains open','frontier_step':'GPI-S5','candidate_commit':r['candidate'],'binary_sha256':r['binary_sha256'],'normalized_model_sha256':r['normalized_model_sha256'],'scope':'Pinned F-DOA Board and existing complementary Schematic placeholders, X11/Xwayland Intel P630 Vulkan, physical1x60Hz client1280x800. Placeholder content never qualifies a resolved schematic.','purpose':'Additional resource observation of established native pane sequence, not an unchanged structural rerun or performance resampling. Existing production diagnostics only; no product changes.','recipe':r['schedule'],'observers':'20ms minimum opportunistic resource snapshots on existing event turns; no new timer/redraw. Full tracked GPU allocation event delivery. Verbose/action-state traces remain enabled.','result':a,'interpretation':'All406 allocation batches reconcile against25,040 ordered events;606 registrations have606 releases. Seven additional64-byte Board-tagged uniforms live aboutone second each and release before processclose; aggregateUniform highwater272bytes. This is consistent with transient secondBoard leaves from complementary splitting; the ledger lacks a directPaneId join, so it does not independently identify each uniform with a specificPaneId. Single document CPUretention3,089,535 and GPUreservation2,883,216 remain stable in sampled views, finaldocument lists empty and host reservations released.','visual_review':'Endpoint Board interior comparison exact within the declared crop; native state oracles retain cameras and expected focus. Initial/final fullimages preserved; no fullimage equality or physical latency claim.','boundaries':['Tracked GPU peak38,556,720bytes is registration accounting, not exact API-live peak, driver residency or complete PM047 memory accounting.','GUI RSS snapshots include observers and omit cooperating engine total; overlapping counters must not be summed.','No private-text per-call peak, observer overhead admission, full no-leak or60minute endurance acceptance.','No camera hidden-store/document-replacement negative qualification; visiblecamera receipts and final allocator release are narrower.','Three-trial/relative statistics, numerical caps, otherbackend-scales, distinctreviewer and ownerUX remain open.','S0-S4/GEFN and dependency authority unchanged; all pinned resize/temporal/T2/unadmittedschematic exclusions preserved.'],'qualification_pass':False}
(base/'review.json').write_text(json.dumps(report,indent=2)+'\n')
p=Path('docs/reviews/gui-performance/implementation-map.json');m=json.loads(p.read_text());row=next(r for r in m['requirements'] if r['id']=='SH-11');artifact=str((base/'review.json').relative_to(root));row['observed_artifacts'].append(artifact);row['s5_pane_resource_observation']={'status':report['status'],'evidence':artifact,'boundaries':report['boundaries']};m['s5_pane_resource_observation']={'status':report['status'],'qualification_pass':False,'evidence':artifact};p.write_text(json.dumps(m,indent=2)+'\n');cache['reviewed_sources_consumers'][str(p)]=sha(p);cachepath.write_text(json.dumps(cache,indent=2)+'\n')
old=json.loads(subprocess.check_output(['git','show','HEAD:'+str(p)],text=True));assert len(m['requirements'])==len(old['requirements'])==225
for before,after in zip(old['requirements'],m['requirements']):
 for key in before:
  if key=='observed_artifacts' and before['id']=='SH-11':assert before[key]==after[key][:-1]
  else:assert before[key]==after[key]
for key in ('by_clause','by_consumer','by_slice','historical_cases','scope_exclusions'):assert old[key]==m[key]
print('Verified archive members:',len(files),'; retained225original predicates/statuses/authority indexes')
