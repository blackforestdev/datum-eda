import json,hashlib,tarfile,io,shutil,subprocess
from pathlib import Path
root=Path.cwd();base=root/'docs/reviews/gui-performance/implementation/S5/quad-vertex-color';base.mkdir(exist_ok=False)
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
cachepath=Path('/tmp/pm045-route-review-progress.json');cache=json.loads(cachepath.read_text());assert all(sha(Path(p))==h for p,h in cache['reviewed_sources_consumers'].items())
campaign=Path('/tmp/pm045-quad-campaign-echcy9pn');r=json.loads((campaign/'result.json').read_text());analysis=json.loads((campaign/'analysis.json').read_text());assert len(r['runs'])==6 and r['display_restored'];assert all(x['returncode']==0 and x['normal_exit_code']==0 and x['final_pixel_AE']=='0' and not x['action_gpu_incomplete'] for x in analysis['runs'])
proof=Path('/tmp/pm045-quad-proof-package');proof.mkdir(exist_ok=False)
shutil.copytree('/tmp/pm045-quad-final-candidate-sources',proof/'candidate-sources');shutil.copytree('/tmp/pm045-quad-originals',proof/'originals')
for p in ['/tmp/pm045-quad-candidate-proof.log','/tmp/pm045-quad-production-pixel-proof.log','/tmp/pm045-quad-candidate-release.log','/tmp/pm045-quad-candidate-declaration.json','/tmp/pm045-quad-current-sources.json','/tmp/pm045-quad-trial-cleanup.json','/tmp/pm045-quad-extra-test.rs','/tmp/pm045-quad-extra-test-plan.json']:shutil.copy2(p,proof/Path(p).name)
archives=[]
def pack(name,path):
 files=[p for p in sorted(path.rglob('*')) if p.is_file()];manifest={str(p.relative_to(path)):{'bytes':p.stat().st_size,'sha256':sha(p)} for p in files};archive=base/(name+'.tar.xz')
 with tarfile.open(archive,'w:xz') as t:
  for p in files:t.add(p,arcname=str(p.relative_to(path)),recursive=False)
  data=(json.dumps(manifest,indent=2)+'\n').encode();info=tarfile.TarInfo('MANIFEST.json');info.size=len(data);t.addfile(info,io.BytesIO(data))
 with tarfile.open(archive) as t:
  for n,v in manifest.items():
   data=t.extractfile(n).read();assert len(data)==v['bytes'] and hashlib.sha256(data).hexdigest()==v['sha256']
 archives.append({'file':archive.name,'original_path':str(path),'sha256':sha(archive),'members':len(files),'raw_bytes':sum(v['bytes'] for v in manifest.values())})
pack('campaign',campaign);pack('proof',proof)
for row in r['runs']:pack(row['role']+'-'+str(row['trial']),Path(row['path']))
(base/'archive-index.json').write_text(json.dumps(archives,indent=2)+'\n');(base/'tools').mkdir()
for path in ('/tmp/pm045-quad-native.py','/tmp/pm045-quad-campaign.py','/tmp/pm045-quad-analyze.py','/tmp/pm045-pointer-stream.py','/tmp/pm045_wm_close.py',__file__):shutil.copy2(path,base/'tools'/Path(path).name)
shutil.copy2(campaign/'analysis.json',base/'analysis.json');shutil.copy2('/tmp/pm045-quad-native-qc9loglk/final.png',base/'final-candidate.png')
subprocess.run(['git','diff','--exit-code','--','crates/gui-render'],check=True)
assert sha(root/'target/release/datum-gui')=='b8c1fa1bf390d0dd768554b7b86548a6dc9b9777d8f6fdb5a1c7107f8e7e2b5e'
report={'status':'Candidate not adopted; native improvement not demonstrated','qualification_pass':False,'baseline_commit':'57066c97','binaries':r['binary_sha256'],'experiment':'Move screen and filled-world constant-quad sRGB conversion from fragment to vertex shader, using flat color varying. Distinct from earlier rejected stroke-only experiment. Same vertex layout, 8xMSAA, painter order, passes, allocations and dependencies.','correctness':'Initial test-only pipeline trial and subsequent candidate-production-pipeline comparison each passed9pinnedFDOA scale/zoom cases with exact RGBA equality plus9wrong-world-color negative controls on IntelP630/Vulkan8xMSAA. No numerical performance acceptance from isolated timestamp samples.','native':'Six fixed-order quiet GPU-timestamp-on X11/1x runs completed normaldevice-live drain0, all finalreadbacks AE0. No selective replacement. Candidate frame p95 values9.8866/9.9328/9.9334ms overlap baseline10.1337/9.9148/9.9060ms; scene p95 remains about7.96..7.99ms and CPU dutyabout14.6..15.0percent. Threepairs do not establish formalcomparative improvement orcap acceptance.','method_limits':analysis['limits'],'restoration':'All changed production/test files restored exactly; temporary testmodule/frozenWGSL removed fromlive source. Originalb8release binary restored. Rejected d014candidate retained separately at target/pm045-quad-candidate/datum-gui, currentcandidate sources archived.','proof_limits':'Only candidate-specific ignoredpixeltest and guardedrelease build ran. Broader unit/GPU/Clippy and prepared supplementaldialog/threshold test were deliberately not run after nativebenefit was not demonstrated. Initial test-only sourcehashes retained but that intermediate testsource was overwritten; final production-candidate tested sources are archived exactly. Prepared extra-test is unexecuted, not evidence.','resource_cleanup':'Initial Cargo preflight refused when tmp reserve was19,529,728bytes below6GiB. Removed only1392SHA/length/archive-verified copies from already durableControls30Project/Global packet, totaling150,920,047bytes. Receipt retained; sharedCargo target untouched.','next_boundary':'Do not repeat this unchanged shader experiment. Remaining GPU cost is not explained by moving constant-quad or stroke color conversion alone; no new cause established. Preserve retained-rendering improvements and existingcaps.','preserved':'All225adoption predicates/statuses/scopes/history and authorityindexes remain unchanged; S0-S4/GEFN, PM029 dependencies, resize/temporal/T2/unadmittedschematic exclusions preserved.'}
(base/'review.json').write_text(json.dumps(report,indent=2)+'\n')
p=Path('docs/reviews/gui-performance/implementation-map.json');m=json.loads(p.read_text());artifact=str((base/'review.json').relative_to(root))
for row in m['requirements']:
 if row['id'] in ('AC-02','AC-06'):
  row.setdefault('observed_artifacts', []).append(artifact);row['s5_quad_color_experiment']={'status':report['status'],'qualification_pass':False,'evidence':artifact}
m['s5_quad_color_experiment']={'status':report['status'],'qualification_pass':False,'evidence':artifact};p.write_text(json.dumps(m,indent=2)+'\n');cache['reviewed_sources_consumers'][str(p)]=sha(p);cachepath.write_text(json.dumps(cache,indent=2)+'\n')
old=json.loads(subprocess.check_output(['git','show','HEAD:'+str(p)],text=True));assert len(m['requirements'])==len(old['requirements'])==225
for before,after in zip(old['requirements'],m['requirements']):
 for key in before:
  if key=='observed_artifacts' and before['id'] in ('AC-02','AC-06'):assert before[key]==after[key][:-1]
  else:assert before[key]==after[key],(before['id'],key)
for key in ('by_clause','by_consumer','by_slice','historical_cases','scope_exclusions'):assert old[key]==m[key]
print('archives',len(archives),'verified_members',sum(x['members'] for x in archives),'all225original predicates preserved')
