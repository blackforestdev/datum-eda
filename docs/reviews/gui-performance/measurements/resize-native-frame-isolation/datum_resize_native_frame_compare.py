import os, subprocess, pathlib, json, hashlib
root=pathlib.Path('/home/bfadmin/Documents/datum-eda')
output=pathlib.Path('/tmp/datum-resize-native-frame-comparison')
output.mkdir(exist_ok=False)
binary=root/'target/release/datum-gui'
expected=hashlib.sha256(binary.read_bytes()).hexdigest()
for index,mode in enumerate(['full','clear','clear','full']):
 env=dict(os.environ,DATUM_MEASUREMENT_BOARD='/tmp/datum-gui-candidate-boundary-v3/DOA2526.kicad_pcb',DATUM_DIAGNOSTIC_NATIVE_FRAME=mode,DATUM_GPU_DIAGNOSTIC_BACKEND='vulkan',DATUM_RESIZE_IDLE_REPEATS='1')
 for name in ['DATUM_TRACE_TIMING','DATUM_RESIZE_CPU_PROBE','DATUM_GUI_VERBOSE_LOG','DATUM_GPU_DIAGNOSTIC_LATENCY']:env.pop(name,None)
 dest=output/f'{index}-{mode}'
 print('Starting',index,mode,flush=True)
 with (output/f'{index}-{mode}-driver.log').open('w') as log:
  subprocess.run(['python3','/tmp/datum_resize_measure_h2.py','--mode','main','--phases','height,width','--seconds','5','--trials','1','--trace','--output',str(dest)],env=env,cwd=root,stdout=log,stderr=log,check=True)
 report=json.loads((dest/'report.json').read_text())
 assert report['binary_sha256']==expected==report['binary_sha256_at_end']
 for sample in report['samples']:
  print(json.dumps({k:sample[k] for k in ['phase','cpu_percent_one_core','input_events_sent','surface_configurations','main_submissions','gpu_engine_active_percent','received_xi_events']}),flush=True)
  assert not any(sample['received_xi_events'].values()), 'Contaminated input; stop comparison'
 print('Finished',index,mode,flush=True)
