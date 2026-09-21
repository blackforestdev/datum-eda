import hashlib,json,os,pathlib,shutil,subprocess,tempfile,time
root=pathlib.Path('/home/bfadmin/Documents/datum-eda')
out=pathlib.Path('/tmp/pm045-readiness-x11');out.mkdir(exist_ok=True)
binary=root/'target/release/datum-gui'
fixture=pathlib.Path('/tmp/datum-gui-candidate-boundary-v3/DOA2526.kicad_pcb')
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
report={'scope':'S1 capture cancellation candidate: existing six native extent/final-state checks for four hosts. Does not exercise DPI moves or establish input/visual/performance acceptance.','revision':subprocess.check_output(['git','rev-parse','HEAD'],cwd=root,text=True).strip(),'binary_sha256':sha(binary),'fixture_sha256':sha(fixture),'runs':[]}
for host,flag in [('MAIN',None),('GLOBAL','--open-global-preferences'),('PROJECT','--open-project-preferences'),('NEW','--open-new-project')]:
 case=out/host;case.mkdir(exist_ok=True)
 board=case/'board.kicad_pcb';shutil.copyfile(fixture,board)
 env=os.environ.copy()
 for key in list(env):
  if key.startswith('DATUM_DIAGNOSTIC_') or key.startswith('DATUM_GPU_DIAGNOSTIC_'):env.pop(key)
 env.update(DATUM_GPU_MEASUREMENTS='0',DATUM_GUI_VERBOSE_LOG='1',DATUM_TRACE_TIMING='1',DATUM_GUI_LOG=str(case/'native.log'),WINIT_UNIX_BACKEND='x11',XDG_CONFIG_HOME=str(case/'config'),XDG_CACHE_HOME=str(case/'cache'),TMPDIR=str(case))
 env.pop('WAYLAND_DISPLAY',None)
 cmd=[str(binary),'--board',str(board),'--window-size','1280x800','--resize-torture-smoke']+([flag] if flag else [])
 (case/'native.log').write_text('')
 with (case/'stderr.log').open('w') as stream:
  p=subprocess.Popen(cmd,cwd=root,env=env,stdout=stream,stderr=stream)
  try:
   deadline=time.monotonic()+30
   while time.monotonic()<deadline:
    log=(case/'native.log').read_text() if (case/'native.log').exists() else ''
    if 'native resize smoke end' in log or p.poll() is not None:break
    time.sleep(.1)
   rows=log.splitlines();observed=[r for r in rows if 'native resize smoke observed' in r];identity=[r for r in rows if 'surface identity' in r]
   ok=p.poll() is None and len(observed)==6 and '1280x800 configured 1280x800 presented=true' in observed[-1] and len(identity)==(1 if host=='MAIN' else 2) and all('window_backend=x11' in r for r in identity)
   report['runs'].append(dict(host=host,command=cmd,passed=ok,observed=observed,identity=identity,unexpected_exit=p.poll(),log_sha256=sha(case/'native.log')))
   (out/'result.json').write_text(json.dumps(report,indent=2)+'\n')
   print(host,ok,flush=True)
   if not ok:raise SystemExit('baseline failed; investigate without relabeling')
  finally:
   p.terminate()
   try:p.wait(timeout=5)
   except subprocess.TimeoutExpired:p.kill();p.wait()
assert sha(binary)==report['binary_sha256']
