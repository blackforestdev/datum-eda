import hashlib,json,os,pathlib,subprocess,time
root=pathlib.Path('/home/bfadmin/Documents/datum-eda');out=pathlib.Path(os.environ.get('DATUM_RESIZE_SMOKE_OUTPUT','/tmp/datum-resize-smoke-final'));out.mkdir(exist_ok=False)
binary=root/'target/release/datum-gui';fixture=pathlib.Path('/tmp/datum-gui-candidate-boundary-v3/DOA2526.kicad_pcb')
env=dict(os.environ,TMPDIR=str(out),XDG_CONFIG_HOME=str(out/'config'),XDG_CACHE_HOME=str(out/'cache'),EDA_CLI_BIN=str(root/'target/debug/datum-eda'),DATUM_GUI_VERBOSE_LOG='1',DATUM_TRACE_TIMING='1',WINIT_UNIX_BACKEND='x11')
for k in ['DATUM_ENGINE_SOCKET','EDA_ENGINE_SOCKET']:env.pop(k,None)
if not os.environ.get('DATUM_RESIZE_NATIVE_WAYLAND'):env.pop('WAYLAND_DISPLAY',None)
else:env['WINIT_UNIX_BACKEND']='wayland'
args=[str(binary),'--board',str(fixture),'--window-size','1280x800','--resize-torture-smoke']
with (out/'gui.log').open('w') as log:
 p=subprocess.Popen(args,cwd=root,env=env,stdout=log,stderr=log)
 try:
  deadline=time.monotonic()+180
  while time.monotonic()<deadline:
   path=out/'datum-gui-last.log';s=path.read_text() if path.exists() else ''
   if 'resize torture end' in s:break
   if p.poll() is not None:raise RuntimeError('GUI exited before passing smoke: '+(out/'gui.log').read_text()[-2000:])
   time.sleep(.1)
  else:raise TimeoutError('native resize smoke')
  start=s.index('resize torture begin');end=s.index('resize torture end',start);section=s[start:end]
  counts={k:section.count(k) for k in ['resize apply','surface configure begin','frame present end','resize torture step','surface acquire recovered','surface acquire timeout','retained scene build begin']}
  assert counts['surface configure begin']==18,counts
  assert counts['frame present end']==7,counts
  assert counts['retained scene build begin']==1,counts
  (out/'report.json').write_text(json.dumps({'passed':True,'command':args,'binary_sha256':hashlib.sha256(binary.read_bytes()).hexdigest(),'counts':counts,'scope':'Native GPU smoke: six two-axis bursts preserve independent retained geometry; DPI discards it and renders again. 18 immediate configurations, seven presentations and only the DPI retained rebuild expected. No latency or terminal-process qualification.'},indent=2)+'\n')
  print(counts,flush=True)
 finally:
  if p.poll() is None:p.terminate();p.wait(timeout=10)
