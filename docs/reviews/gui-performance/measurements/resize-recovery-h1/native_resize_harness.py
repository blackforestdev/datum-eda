import hashlib,json,os,pathlib,subprocess,time
root=pathlib.Path('/home/bfadmin/Documents/datum-eda');out=pathlib.Path(os.environ.get('DATUM_RESIZE_SMOKE_OUTPUT','/tmp/datum-resize-h1-x11'));out.mkdir(exist_ok=False)
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
   if 'native resize smoke end' in s:break
   if p.poll() is not None:raise RuntimeError('GUI exited before passing smoke: '+(out/'gui.log').read_text()[-2000:])
   time.sleep(.1)
  else:raise TimeoutError('native resize smoke')
  time.sleep(2)
  s=path.read_text()
  counts={k:s.count(k) for k in ['native resize smoke begin','native resize smoke end','native resize smoke request','native resize smoke observed']}
  assert list(counts.values())==[1,1,6,6],counts
  identity=[line for line in s.splitlines() if 'surface identity' in line]
  assert identity, 'missing actual backend identity'
  (out/'report.json').write_text(json.dumps({'passed':True,'command':args,'binary_sha256':hashlib.sha256(binary.read_bytes()).hexdigest(),'counts':counts,'identity':identity,'scope':'Six native requested sizes observed and configured before application presentation attempts; completes once. No compositor visibility, CPU or flicker acceptance.'},indent=2)+'\n')
  print(counts,identity,flush=True)
 finally:
  if p.poll() is None:p.terminate();p.wait(timeout=10)
