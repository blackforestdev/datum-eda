import os,subprocess,time,re,json,hashlib,sys
from pathlib import Path
root=Path('/home/bfadmin/Documents/datum-eda');out=Path(sys.argv[1]);out.mkdir();results=[]
for host,flag in [('GLOBAL','--open-global-preferences'),('PROJECT','--open-project-preferences')]:
 d=out/host;d.mkdir();(d/'tmp').mkdir();log=d/'native.log';log.write_text('');actions=[]
 env=os.environ.copy();env.pop('WAYLAND_DISPLAY',None);env.update(WINIT_UNIX_BACKEND='x11',TMPDIR=str(d/'tmp'),XDG_CONFIG_HOME=str(d/'config'),XDG_CACHE_HOME=str(d/'cache'),DATUM_GUI_LOG=str(log),DATUM_GUI_VERBOSE_LOG='1',DATUM_GPU_MEASUREMENTS='0',EDA_CLI_BIN=str(root/'target/release/datum-eda'))
 cmd=[str(root/'target/release/datum-gui'),'--board','/tmp/pm045-readiness-x11/MAIN/board.kicad_pcb','--window-size','1280x800','--initial-layout','single',flag,'--visual-scale-factor','1']
 r={'host':host,'command':cmd,'actions':actions,'binary_sha256':hashlib.sha256((root/'target/release/datum-gui').read_bytes()).hexdigest()}
 with (d/'stderr.log').open('w') as err:
  p=subprocess.Popen(cmd,cwd=root,env=env,stdout=err,stderr=err)
  def wait(fn):
   end=time.monotonic()+35
   while time.monotonic()<end:
    assert p.poll() is None
    value=fn()
    if value:return value
    time.sleep(.04)
   raise AssertionError('predicate timeout')
  def xdo(*args):actions.append({'args':args,'time':time.time()});subprocess.run(['xdotool',*args],env=env,check=True,timeout=8)
  def click(x,y):
   g=dict(line.split('=',1) for line in subprocess.check_output(['xdotool','getwindowgeometry','--shell',wid],env=env,text=True).splitlines())
   xdo('mousemove',str(int(g['X'])+x),str(int(g['Y'])+y));time.sleep(.12);xdo('click','1')
  def capture(name):subprocess.run(['import','-window',wid,str(d/(name+'.png'))],env=env,check=True,timeout=8)
  try:
   ids=wait(lambda:v if len(v:=re.findall(r'surface identity window=WindowId\((\d+)\)',log.read_text()))==2 else None);wid=ids[-1];key='WindowId('+wid+')'
   def frames():return sum(json.loads(line.split('native_surface_lifecycle ',1)[1]).get('window')==key and json.loads(line.split('native_surface_lifecycle ',1)[1]).get('reason')=='present' for line in log.read_text().splitlines() if 'native_surface_lifecycle ' in line and line.endswith('}'))
   wait(lambda:frames()>0)
   project=Path(re.search(r'request resolve end project_root=(\S+)',log.read_text())[1]);assert project.is_relative_to(d/'tmp')
   project_file=project/'project.json';before=json.loads(project_file.read_text());(d/'project-before.json').write_text(json.dumps(before,indent=2)+'\n');assert before['project_display_units']['system']=='metric'
   preference_root=d/'config/datum/preferences';head=preference_root/'head.json';head_before=head.read_bytes() if head.exists() else None
   xdo('windowactivate','--sync',wid);xdo('windowsize','--sync',wid,'1000','750');time.sleep(.5)
   if host=='GLOBAL':n=frames();click(50,107);wait(lambda:frames()>n)
   time.sleep(.25);capture('before-choice');n=frames();click(900,130);wait(lambda:frames()>n);time.sleep(.3);capture('choices')
   n=frames();click(900,200);wait(lambda:frames()>n)
   if host=='GLOBAL':
    wait(lambda:head.exists() and head.read_bytes()!=head_before)
    h=json.loads(head.read_text());g=h['generation']['generation'];manifest=preference_root/'generations'/f'g{g:020}'/'manifest.json';m=json.loads(manifest.read_text());(d/'global-after.json').write_text(json.dumps(m,indent=2)+'\n')
    assert m['state']['user']['datum.units.system']['value']=='imperial'
    assert json.loads(project_file.read_text())==before, 'Global changed the existing Project'
    r['scope_proof']='Global User contribution imperial; existing Project document unchanged'
   else:
    wait(lambda:json.loads(project_file.read_text())['project_display_units']['system']=='imperial')
    after=json.loads(project_file.read_text());assert after['uuid']==before['uuid'];assert after['project_units_seed_receipt']==before['project_units_seed_receipt']
    assert (head.read_bytes() if head.exists() else None)==head_before, 'Project changed Global preferences'
    r['scope_proof']='Project Working Units imperial; Project identity/seed receipt and Global head unchanged'
   (d/'project-after.json').write_text(json.dumps(json.loads(project_file.read_text()),indent=2)+'\n');time.sleep(.3);capture('after-choice');r['passed']=True
  except Exception as e:r.update(passed=False,error=repr(e));print(host,repr(e),flush=True)
  finally:
   if p.poll() is None:p.terminate();p.wait(timeout=10)
   results.append(r);(out/'result.json').write_text(json.dumps(results,indent=2)+'\n')
assert all(r['passed'] for r in results)
