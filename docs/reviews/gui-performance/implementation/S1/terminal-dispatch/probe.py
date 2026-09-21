import os,sys,time,subprocess,json,re,hashlib,shutil
from pathlib import Path
root=Path('/home/bfadmin/Documents/datum-eda');binary=Path(sys.argv[1]).resolve();tag=sys.argv[2];out=Path('/tmp/pm045-budget-proof')/tag;out.mkdir(parents=True,exist_ok=True)
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
report={'binary_sha256':sha(binary),'revision':subprocess.check_output(['git','rev-parse','HEAD'],text=True).strip(),'runs':[]}
for host,flag in [('GLOBAL','--open-global-preferences')]:
 case=out/host;case.mkdir(exist_ok=True);log=case/'native.log';log.write_text('')
 for name in ['go','sent.json']:(case/name).unlink(missing_ok=True)
 fixture_case=Path('/tmp/pm045-readiness-x11')/host;board=fixture_case/'board.kicad_pcb'
 env=os.environ.copy()
 for key in list(env):
  if key.startswith('DATUM_DIAGNOSTIC_') or key.startswith('DATUM_GPU_DIAGNOSTIC_'):env.pop(key)
 env.pop('WAYLAND_DISPLAY',None);env.update(EDA_CLI_BIN=str(root/'target/release/datum-eda'),CARGO_NET_OFFLINE='true',DATUM_GPU_MEASUREMENTS='0',DATUM_GUI_VERBOSE_LOG='1',DATUM_GUI_LOG=str(log),WINIT_UNIX_BACKEND='x11',XDG_CONFIG_HOME=str(fixture_case/'config'),XDG_CACHE_HOME=str(fixture_case/'cache'),TMPDIR=str(fixture_case))
 cmd=[str(binary),'--board',str(board),flag,'--window-size','1280x800','--terminal-session-profile','custom','--terminal-program','/usr/bin/python3','--terminal-arg=-u','--terminal-arg=/tmp/pm045_budget_producer.py','--terminal-arg='+str(case)]
 with (case/'stderr.log').open('w') as stream:
  p=subprocess.Popen(cmd,cwd=root,env=env,stdout=stream,stderr=stream)
  try:
   deadline=time.monotonic()+20
   while time.monotonic()<deadline and p.poll() is None:
    text=log.read_text();ids=re.findall(r'surface identity window=WindowId\((\d+)\)',text)
    if len(ids)==2 and text.count('native frame round')>=2:break
    time.sleep(.05)
   assert p.poll() is None and len(ids)==2,'native readiness failed'
   time.sleep(1);offset=len(log.read_text());(case/'go').touch();deadline=time.monotonic()+10
   while not (case/'sent.json').exists() and p.poll() is None and time.monotonic()<deadline:
    subprocess.run(['xdotool','key','--window',ids[1],'Tab'],check=True);time.sleep(.04)
   assert (case/'sent.json').exists() and p.poll() is None,'PTY producer did not finish'
   time.sleep(.5);segment=log.read_text()[offset:];(case/'output-window.log').write_text(segment)
   rounds=re.findall(r'native frame round hosts=\[([^\n]+)\]',segment)
   counts={i:sum('WindowId('+i+')' in r for r in rounds) for i in ids}
   main,aux=[counts[i] for i in ids]
   expected_aux=aux>0
   # Positive control occurs after the measured output-only interval.
   before=len(log.read_text())
   subprocess.run(['xdotool','key','--window',ids[1],'Tab'],check=True)
   deadline=time.monotonic()+3;focus_rounds=0
   while time.monotonic()<deadline:
    after=log.read_text()[before:]
    focus_rounds=sum('WindowId('+ids[1]+')' in r for r in re.findall(r'native frame round hosts=\[([^\n]+)\]',after))
    if focus_rounds:break
    time.sleep(.05)
   (case/'focus-window.log').write_text(after)
   ok=main>0 and expected_aux and focus_rounds>0
   report['runs'].append({'host':host,'command':cmd,'passed':ok,'main_rounds':main,'auxiliary_rounds':aux,'native_Tab_focus_rounds':focus_rounds,'producer':json.loads((case/'sent.json').read_text()),'native_ids':ids,'log_sha256':sha(log),'output_window_sha256':sha(case/'output-window.log')})
   print(tag,host,main,aux,ok,flush=True);(out/'result.json').write_text(json.dumps(report,indent=2)+'\n')
  finally:
   p.terminate()
   try:p.wait(timeout=5)
   except subprocess.TimeoutExpired:p.kill();p.wait()
assert sha(binary)==report['binary_sha256']
assert all(r['passed'] for r in report['runs'])
