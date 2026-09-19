import subprocess,pathlib,json
root=pathlib.Path('/home/bfadmin/Documents/datum-eda')
out=pathlib.Path('/tmp/datum-resize-isolation-proof');out.mkdir(exist_ok=False)
runs=[('baseline-global','global',root/'target/datum-resize-isolation-baseline'),('candidate-global','global',root/'target/release/datum-gui'),('candidate-project','project',root/'target/release/datum-gui'),('candidate-new','new',root/'target/release/datum-gui')]
for label,dialog,binary in runs:
 print('Starting',label,flush=True)
 with (out/(label+'.log')).open('w') as log:
  subprocess.run(['python3','/tmp/datum_wayland_resize_isolation.py','--output',str(out/label),'--dialog',dialog,'--binary',str(binary)],cwd=root,stdout=log,stderr=log,check=True)
 r=json.loads((out/label/'report.json').read_text())
 for s in r['samples']:
  extra=s['all_renderer_calls']-s['present_calls']
  print(label,s['phase'],'main',s['present_calls'],'extra renderers',extra,'CPU',round(s['cpu_percent_one_core'],2),flush=True)
  if label.startswith('candidate'):assert extra==0, 'Unrelated renderer work remains'
  elif s['phase']!='idle':assert extra>0, 'Negative control did not expose defect'
 print('Passed',label,flush=True)
