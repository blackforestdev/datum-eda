import os, sys, time, json, subprocess, hashlib, tempfile, re
from pathlib import Path
root=Path('/home/bfadmin/Documents/datum-eda')
out=Path(tempfile.mkdtemp(prefix='datum-s5-windows-')); print(out,flush=True)
board=Path('/tmp/pm045-readiness-x11/MAIN/board.kicad_pcb')
assert hashlib.sha256(board.read_bytes()).hexdigest()=='7df67d426cc0cd808096824b0b7a05dcde351a1305fd41244d8c05a15801c08d'
binary=root/'target/release/datum-gui'; log=out/'native.log';log.touch()
env=os.environ.copy()
for k in list(env):
 if k.startswith(('DATUM_DIAGNOSTIC_','DATUM_GPU_DIAGNOSTIC_')):env.pop(k)
env.pop('WAYLAND_DISPLAY',None)
env.update(WINIT_UNIX_BACKEND='x11',WINIT_X11_SCALE_FACTOR='1',XDG_CONFIG_HOME=str(out/'config'),XDG_CACHE_HOME=str(out/'cache'),DATUM_GUI_LOG=str(log),DATUM_GUI_VERBOSE_LOG='1',DATUM_GPU_MEASUREMENTS='0',EDA_CLI_BIN=str(root/'target/release/datum-eda'))
cmd=[str(binary),'--board',str(board),'--initial-layout','single','--window-size','1280x800','--visual-scale-factor','1']
report={'candidate':subprocess.check_output(['git','rev-parse','HEAD'],cwd=root,text=True).strip(),'binary_sha256':hashlib.sha256(binary.read_bytes()).hexdigest(),'command':cmd,'backend':'x11 on Xwayland','cycles':[],'limits':['Partial W-WINDOWS: CPU and attachment ownership; GPU duty/timing and complete memory accounting not sampled.','No endurance or independent replay acceptance.']}
def save(): (out/'result.json').write_text(json.dumps(report,indent=2)+'\n')
def xd(*args):return subprocess.check_output(['xdotool',*map(str,args)],text=True).strip()
def lines():
 data=log.read_text()
 # Never parse the incomplete final record while the application is writing.
 return data[:data.rfind('\n')+1].splitlines()
def parsed(rows,tag):return [json.loads(s.split(tag,1)[1]) for s in rows if tag in s]
def until(fn):
 end=time.monotonic()+15
 while time.monotonic()<end:
  assert p.poll() is None,('process exited',p.returncode)
  v=fn()
  if v:return v
  time.sleep(.025)
 raise AssertionError('native readiness/closure timed out')
def cpu():
 s=Path(f'/proc/{p.pid}/stat').read_text().rsplit(')',1)[1].split()
 return (int(s[11])+int(s[12]))/os.sysconf('SC_CLK_TCK')
def rss():return {s.split(':')[0]:s.split(':')[1].strip() for s in Path(f'/proc/{p.pid}/status').read_text().splitlines() if s.startswith(('VmRSS:','VmHWM:'))}
with (out/'stderr.log').open('w') as stream:
 p=subprocess.Popen(cmd,cwd=root,env=env,stdout=stream,stderr=stream)
 try:
  initial=until(lambda:next((r for r in parsed(lines(),'native_surface_lifecycle ') if r['reason']=='present'),None))
  main=int(re.search(r'\d+',initial['window'])[0]);xd('windowactivate','--sync',main);time.sleep(5)
  report['initial']=initial;report['identity']=[s for s in lines() if 'surface identity window=' in s];report['rss_initial']=rss()
  for host in ('GLOBAL','PROJECT','NEW'):
   for i in range(30):
    start=len(lines());c0=cpu();t0=time.monotonic()
    # Native pointer opens menu; native keys select its actual model entries.
    xd('mousemove','--window',main,148 if host!='NEW' else 110,16,'click',1)
    if host=='NEW':xd('key','--delay',20,'Return')
    else:xd('key','--delay',20,'Up','Right',*(['Down'] if host=='PROJECT' else []),'Return')
    frame=until(lambda:next((r for r in parsed(lines()[start:],'native_surface_lifecycle ') if r['reason']=='present' and r['window']!=initial['window']),None))
    wid=int(re.search(r'\d+',frame['window'])[0]);until(lambda:'Map State: IsViewable' in subprocess.check_output(['xwininfo','-id',str(wid)],text=True))
    until(lambda:int(xd('getwindowfocus'))==wid)
    title=xd('getwindowname',wid)
    assert {'GLOBAL':'Global Preferences','PROJECT':'Project Preferences','NEW':'New Project'}[host] in title, title
    opened={'cpu_ms':(cpu()-c0)*1000,'wall_ms':(time.monotonic()-t0)*1000}
    if i==0:subprocess.run(['import','-window',str(wid),str(out/(host+'.png'))],check=True)
    close_start=len(lines());c1=cpu();t1=time.monotonic();xd('key','Escape')
    dropped=until(lambda:next((r for r in parsed(lines()[close_start:],'native_surface_lifecycle ') if r['reason']=='owner_drop' and r['window']==frame['window']),None))
    until(lambda:int(xd('getwindowfocus'))==main)
    ledger=until(lambda:next((r for r in reversed(parsed(lines()[close_start:],'native attachment ledger ')) if all(a['host']!=frame['host'] for a in r['attachments']['allocations'])),None))
    report['cycles'].append({'host':host,'title':title,'cycle':i+1,'open':opened,'close':{'cpu_ms':(cpu()-c1)*1000,'wall_ms':(time.monotonic()-t1)*1000},'present':frame,'drop':dropped,'attachment_after_close':ledger,'focus_restored':True,'rss':rss()});save()
   print(host,'30 native cycles complete',flush=True)
  report['passed_native_cycles']=True
 except Exception as e:
  report['error']=repr(e);print(repr(e),flush=True)
 finally:
  report['rss_final']=rss() if p.poll() is None else None
  if p.poll() is None:
   p.terminate()
   try:p.wait(timeout=5)
   except subprocess.TimeoutExpired:p.kill();p.wait()
  report['cleanup']='Owned process terminated; final application teardown not qualified.';save()
  assert hashlib.sha256(binary.read_bytes()).hexdigest()==report['binary_sha256']
