import hashlib,json,pathlib,statistics,subprocess
root=pathlib.Path('/tmp');names=['baseline-traced','baseline-plain-idle','candidate-plain-idle','candidate-reuse-plain','candidate-reuse-traced','reuse-only-pilot','final-plain','final-traced'];results={}
for name in names:
 d=root/('datum-resize-'+name);p=d/'report.json'
 if not p.exists():continue
 r=json.loads(p.read_text());rows=[]
 for s in r['samples']:
  counters=s['raw_counters'];hz=s.get('clock_ticks_per_second',100)
  peaks=[(b['cpu_ticks']-a['cpu_ticks'])/hz/(b['seconds']-a['seconds'])*100 for a,b in zip(counters,counters[1:])]
  rows.append({k:s.get(k) for k in ['phase','cpu_percent_one_core','input_events_sent','resize_applications','surface_configurations','main_submissions','retained_miss_frames','engine_cpu_seconds','received_xi_events']}|{'peak_cpu_approximately_100ms':max(peaks,default=0),'gpu_render_engine_active_percent':sum(v for k,v in s['gpu_engine_active_percent'].items() if k.endswith('drm-engine-render'))})
 board=next((d/'datum-eda').glob('gui-imports/*/board/board.json'));model=json.loads(board.read_text());model.pop('uuid')
 results[name]={'binary_sha256':r['binary_sha256'],'binary_unchanged':r['binary_sha256']==r['binary_sha256_at_end'],'fixture_sha256':r['fixture_sha256'],'normalized_board_sha256':hashlib.sha256(json.dumps(model,sort_keys=True).encode()).hexdigest(),'rows':rows,'medians':{phase:statistics.median(s['cpu_percent_one_core'] for s in rows if s['phase']==phase) for phase in ['height','width']}}
comparison={}
for image in ['height-after.png','width-after.png','settled-final.png']:
 a=root/'datum-resize-baseline-plain-idle'/image;b=root/'datum-resize-final-plain'/image
 if not a.exists() or not b.exists():continue
 dims=[subprocess.check_output(['identify','-format','%wx%h',str(p)],text=True) for p in [a,b]]
 raw=[subprocess.check_output(['convert',str(p),'-depth','8','rgba:-']) for p in [a,b]]
 w,h=map(int,dims[0].split('x'));slots=[(w-106,12,w-54,25),(w-280,h-22,w-225,h-9)]
 outside=sum(x!=y and not any(x0<=i//4%w<x1 and y0<=i//4//w<y1 for x0,y0,x1,y1 in slots) for i,(x,y) in enumerate(zip(*raw)))
 comparison[image]={'dynamic_revision_slots':slots,'different_channels_outside_revision_slots':outside,'dimensions':dims,'exact_pixels':raw[0]==raw[1],'different_channels':sum(x!=y for x,y in zip(*raw)),'sha256':[hashlib.sha256(p.read_bytes()).hexdigest() for p in [a,b]]}
output={'runs':results,'pixel_comparison':comparison,'limitations':['One-core-equivalent GUI process CPU sums threads; not an individual physical core reading.','100ms peaks have 10ms tick quantization and are not instantaneous peaks.','DRM engine activity is not frequency-normalized utilization or complete GPU cost.','Controlled X11/Xwayland resize requests are not physical Wayland decoration dragging.']}
(root/'datum-resize-summary.json').write_text(json.dumps(output,indent=2)+'\n')
for k,v in results.items():print(k,v['medians'])
print(comparison)
