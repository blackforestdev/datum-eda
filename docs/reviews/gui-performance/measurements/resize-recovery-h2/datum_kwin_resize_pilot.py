import pathlib,os,subprocess,time,re,json,hashlib
root=pathlib.Path('/home/bfadmin/Documents/datum-eda');out=pathlib.Path(os.environ.get('DATUM_KWIN_PILOT_OUTPUT','/tmp/datum-resize-h2-wayland-pilot-v3'));out.mkdir(exist_ok=False)
env=dict(os.environ,TMPDIR=str(out),XDG_CONFIG_HOME=str(out/'config'),XDG_CACHE_HOME=str(out/'cache'),EDA_CLI_BIN=str(root/'target/debug/datum-eda'),DATUM_RESIZE_CPU_PROBE='1',DATUM_GUI_VERBOSE_LOG='1',WINIT_UNIX_BACKEND='wayland')
for k in ['DATUM_ENGINE_SOCKET','EDA_ENGINE_SOCKET']:env.pop(k,None)
binary=root/'target/release/datum-gui';plugin='datum-resize-owned-pilot';loaded=False
with (out/'gui.log').open('w') as log:
 p=subprocess.Popen([str(binary),'--board','/tmp/datum-gui-candidate-boundary-v3/DOA2526.kicad_pcb','--window-size','1280x800'],cwd=root,env=env,stdout=log,stderr=log)
 try:
  f=out/'datum-gui-last.log';deadline=time.monotonic()+60
  while time.monotonic()<deadline:
   s=f.read_text() if f.exists() else ''
   if 'frame present end' in s:break
   if p.poll() is not None:raise RuntimeError('GUI exited')
   time.sleep(.1)
  else:raise TimeoutError('startup')
  time.sleep(2)
  qml=out/'resize.qml';qml.write_text('''import QtQuick
import org.kde.kwin
Item {
 property var owned: null
 property int count: 0
 property rect original
 Component.onCompleted: {
  let matches = Workspace.stackingOrder.filter(w => w.pid === PID);
  if (matches.length !== 1) { console.error("DATUM pilot refuses ambiguous target"); return; }
  owned = matches[0]; original = owned.frameGeometry; timer.start();
 }
 Timer { id: timer; interval: 20; repeat: true
  onTriggered: {
   if (!owned || owned.pid !== PID) { stop(); return; }
   if (count >= 240) { owned.frameGeometry = original; stop(); return; }
   let delta = Math.abs((count % 120)-60)*3;
   owned.frameGeometry = Qt.rect(original.x, original.y, 1200+delta, 780);
   count++;
  }
 }
}
'''.replace('PID',str(p.pid)))
  def dbus(*args):return subprocess.check_output(['qdbus6','org.kde.KWin',*args],text=True,timeout=10).strip()
  sid=dbus('/Scripting','org.kde.kwin.Scripting.loadDeclarativeScript',str(qml),plugin);loaded=True
  (out/'script-id.txt').write_text(sid+'\n')
  # Start only the newly loaded script; never start other pending scripts.
  dbus('/Scripting/Script'+sid,'org.kde.kwin.Script.run')
  time.sleep(7)
  s=f.read_text();resizes=re.findall(r'resize apply .*',s)
  assert len(resizes)>80, f'Only {len(resizes)} real size changes observed'
  (out/'report.json').write_text(json.dumps({'passed':True,'pid':p.pid,'native_resize_events':len(resizes),'backend_identity':[l for l in s.splitlines() if 'surface identity' in l],'binary_sha256':hashlib.sha256(binary.read_bytes()).hexdigest(),'scope':'KWin native Wayland targeted resize pilot only; no CPU or temporal acceptance'},indent=2)+'\n')
  print('native resizes',len(resizes))
 finally:
  if loaded:print(subprocess.run(['qdbus6','org.kde.KWin','/Scripting','org.kde.kwin.Scripting.unloadScript',plugin],capture_output=True,text=True).stdout.strip())
  if p.poll() is None:p.terminate();p.wait(timeout=10)
