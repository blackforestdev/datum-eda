import os,subprocess,time,json,re,hashlib
from pathlib import Path
root=Path('/home/bfadmin/Documents/datum-eda');binary=root/'target/release/datum-gui';out=Path('/tmp/pm045-submit-host-proof');out.mkdir(exist_ok=True)
report={'binary_sha256':hashlib.sha256(binary.read_bytes()).hexdigest(),'runs':[]}
def receipts(log):
 return [json.loads(s.split('native_surface_lifecycle ',1)[1]) for s in log.read_text().splitlines() if 'native_surface_lifecycle ' in s]
for host,flag in [('GLOBAL','--open-global-preferences'),('PROJECT','--open-project-preferences'),('NEW','--open-new-project')]:
 case=out/host;case.mkdir(exist_ok=True);fixture=Path('/tmp/pm045-readiness-x11')/host;log=case/'native.log';log.write_text('');env=os.environ.copy()
 for k in list(env):
  if k.startswith('DATUM_DIAGNOSTIC_') or k.startswith('DATUM_GPU_DIAGNOSTIC_'):env.pop(k)
 env.pop('WAYLAND_DISPLAY',None);env.update(EDA_CLI_BIN=str(root/'target/release/datum-eda'),CARGO_NET_OFFLINE='true',DATUM_GPU_MEASUREMENTS='0',DATUM_GUI_VERBOSE_LOG='1',DATUM_GUI_LOG=str(log),WINIT_UNIX_BACKEND='x11',XDG_CONFIG_HOME=str(fixture/'config'),XDG_CACHE_HOME=str(fixture/'cache'),TMPDIR=str(fixture))
 cmd=[str(binary),'--board',str(fixture/'board.kicad_pcb'),'--window-size','1280x800',flag]
 with (case/'stderr.log').open('w') as stream:
  p=subprocess.Popen(cmd,cwd=root,env=env,stdout=stream,stderr=stream)
  try:
   deadline=time.monotonic()+20;ids=[]
   while time.monotonic()<deadline and p.poll() is None:
    ids=re.findall(r'surface identity window=WindowId\((\d+)\)',log.read_text())
    if len(ids)==2 and any(r['window']=='WindowId('+ids[1]+')' and r['reason']=='present' for r in receipts(log)):break
    time.sleep(.05)
   assert len(ids)==2 and p.poll() is None
   wid=ids[1];key='WindowId('+wid+')';initial=next(r for r in receipts(log) if r['window']==key and r['reason']=='present');original=initial['configured_extent'];observed=[]
   for width,height in [(1100,800),(1200,850),tuple(original)]:
    previous=max(r['presented'] for r in receipts(log) if r['window']==key)
    subprocess.run(['xdotool','windowsize',wid,str(width),str(height)],check=True)
    deadline=time.monotonic()+8;match=None
    while time.monotonic()<deadline and p.poll() is None:
     match=next((r for r in reversed(receipts(log)) if r['window']==key and r['reason']=='present' and r['configured_extent']==[width,height] and r['presented']>previous),None)
     if match:break
     time.sleep(.05)
    assert match,host+' actual auxiliary resize not presented'
    observed.append(match)
   report['runs'].append({'host':host,'window':key,'command':cmd,'initial':initial,'observed':observed,'passed':True});(out/'result.json').write_text(json.dumps(report,indent=2)+'\n');print(host,'actual auxiliary extent changes and restore pass',flush=True)
  finally:
   p.terminate()
   try:p.wait(timeout=5)
   except subprocess.TimeoutExpired:p.kill();p.wait()
assert hashlib.sha256(binary.read_bytes()).hexdigest()==report['binary_sha256']
