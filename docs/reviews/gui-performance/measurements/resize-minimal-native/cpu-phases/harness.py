"""Bounded extension of the existing PID-targeted KWin resize pilot."""
import argparse, hashlib, json, os, pathlib, re, subprocess, time
P=argparse.ArgumentParser()
P.add_argument('--output',required=True)
P.add_argument('--mode',choices=['full','clear'],default='full')
P.add_argument('--check',action='store_true')
a=P.parse_args()
root=pathlib.Path('/home/bfadmin/Documents/datum-eda')
binary=root/'target/release/datum-gui'
board=pathlib.Path('/tmp/datum-gui-candidate-boundary-v3/DOA2526.kicad_pcb')
hz=os.sysconf('SC_CLK_TCK')
def cpu(pid):
 f=pathlib.Path(f'/proc/{pid}/stat').read_text().rsplit(')',1)[1].split()
 return int(f[11])+int(f[12])
def gpu(pid):
 result={}
 for p in pathlib.Path(f'/proc/{pid}/fdinfo').iterdir():
  try: fields=dict(l.split(':',1) for l in p.read_text().splitlines() if ':' in l)
  except OSError: continue
  if 'drm-client-id' not in fields:continue
  for k,v in fields.items():
   if k.startswith('drm-engine-') and v.strip().endswith(' ns'):
    key=fields.get('drm-pdev','').strip()+'/'+fields['drm-client-id'].strip()+'/'+k
    result[key]=max(result.get(key,0),int(v.split()[0]))
 return result
def dbus(*args):
 return subprocess.check_output(['qdbus6','org.kde.KWin',*args],text=True,timeout=10).strip()
def script(pid,axis):
 return '''import QtQuick
import org.kde.kwin
Item {
 property var owned: null
 property int count: 0
 property rect original
 Component.onCompleted: {
  let matches = Workspace.stackingOrder.filter(w => w.pid === PID);
  if (matches.length !== 1) { console.error("DATUM refuses ambiguous resize target"); return; }
  owned = matches[0]; original = owned.frameGeometry; timer.start();
 }
 Timer { id: timer; interval: 20; repeat: true
  onTriggered: {
   if (!owned || owned.pid !== PID) { stop(); return; }
   if (count === 240) {
    owned.frameGeometry = Qt.rect(original.x, original.y, original.width+17, original.height+17);
    count++; return;
   }
   if (count > 240 && count < 245) { count++; return; }
   if (count >= 245) { owned.frameGeometry = original; stop(); return; }
   let delta = Math.abs((count % 120)-60)*3;
   owned.frameGeometry = Qt.rect(original.x, original.y,
      original.width + (AXIS === "width" ? delta : 0),
      original.height + (AXIS === "height" ? delta : 0));
   count++;
  }
 }
}
'''.replace('PID',str(pid)).replace('AXIS',json.dumps(axis))
if a.check:
 assert '=== 123' in script(123,'height')
 assert '"height" === "height"' in script(123,'height')
 assert cpu(os.getpid())>=0
 print('Pure helper checks passed; no GUI or compositor action.')
 raise SystemExit
assert os.environ.get('DATUM_DIAGNOSTIC_MINIMAL_WINDOW')=='1', 'Minimal loop must be selected'
assert os.environ.get('DATUM_RESIZE_CPU_PROBE')=='1', 'CPU probes must be selected'
out=pathlib.Path(a.output);out.mkdir(exist_ok=False)
assert os.environ.get('WAYLAND_DISPLAY'), 'Native Wayland display required'
env=dict(os.environ,TMPDIR=str(out),XDG_CONFIG_HOME=str(out/'config'),XDG_CACHE_HOME=str(out/'cache'),EDA_CLI_BIN=str(root/'target/debug/datum-eda'),DATUM_GUI_VERBOSE_LOG='1',DATUM_TRACE_TIMING='1',DATUM_DIAGNOSTIC_NATIVE_FRAME=a.mode)
for k in ['DISPLAY','WAYLAND_SOCKET','DATUM_ENGINE_SOCKET','EDA_ENGINE_SOCKET','DATUM_GPU_DIAGNOSTIC_LATENCY','DATUM_GUI_LOG','DATUM_GPU_DIAGNOSTIC_BACKEND','WINIT_UNIX_BACKEND','DATUM_GUI_PREFERENCES_PATH']:env.pop(k,None)
expected=hashlib.sha256(binary.read_bytes()).hexdigest()
rows=[];plugin=None
with (out/'gui.log').open('w') as log:
 p=subprocess.Popen([str(binary),'--board',str(board),'--window-size','1280x800'],cwd=root,env=env,stdout=log,stderr=log)
 try:
  diag=out/'datum-gui-last.log';deadline=time.monotonic()+60
  while time.monotonic()<deadline:
   text=diag.read_text() if diag.exists() else ''
   if 'window_backend=wayland' in text and 'runtime render ' in (out/'gui.log').read_text():break
   if p.poll() is not None:raise RuntimeError('GUI exited during startup')
   time.sleep(.1)
  else:raise TimeoutError('No identified native Wayland frame')
  time.sleep(3)
  for axis in ['idle','height','width','idle']:
   if axis!='idle':
    plugin=f'datum-resize-accounting-{p.pid}-{axis}'
    qml=out/(axis+'.qml');qml.write_text(script(p.pid,axis))
    sid=dbus('/Scripting','org.kde.kwin.Scripting.loadDeclarativeScript',str(qml),plugin)
    assert int(sid)>=0
   prior=diag.read_text();sizes=re.findall(r'resize apply .*? -> (\d+)x(\d+)',prior) or re.findall(r'initial surface configure begin (\d+)x(\d+)',prior)
   original=tuple(map(int,sizes[-1]))
   d0=diag.stat().st_size;l0=(out/'gui.log').stat().st_size
   t0=time.monotonic();c0=cpu(p.pid);g0=gpu(p.pid)
   if axis!='idle':dbus('/Scripting/Script'+sid,'org.kde.kwin.Script.run')
   timeline=[]
   completed=axis=='idle'
   while time.monotonic()-t0<10:
    changes=[tuple(map(int,v)) for v in re.findall(r'resize apply .*? -> (\d+)x(\d+)',diag.read_bytes()[d0:].decode(errors='replace'))]
    if axis!='idle' and len(changes)>=2:
     completed=changes[-1]==original and all(changes[-2][i]>original[i] for i in range(2))
    if completed and time.monotonic()-t0>=5.5:break
    if p.poll() is not None:raise RuntimeError('GUI exited during measurement')
    timeline.append({'elapsed_s':time.monotonic()-t0,'cpu_ticks':cpu(p.pid),'gpu':gpu(p.pid)})
    time.sleep(.1)
   elapsed=time.monotonic()-t0;c1=cpu(p.pid);g1=gpu(p.pid)
   d=diag.read_bytes()[d0:].decode(errors='replace');l=(out/'gui.log').read_bytes()[l0:].decode(errors='replace')
   phases={}
   for name,wall,process,thread in re.findall(r'cpu_probe phase=(\w+) wall_ns=(\d+) process_ns=(\d+) thread_ns=(\d+)',d):
    row=phases.setdefault(name,dict(count=0,wall_ns=0,process_ns=0,thread_ns=0));row['count']+=1
    for key,value in [('wall_ns',wall),('process_ns',process),('thread_ns',thread)]:row[key]+=int(value)
   if axis!='idle':
    required=['minimal_configure','minimal_acquire','minimal_encode','minimal_submit','minimal_present']
    assert all(phases.get(name,{}).get('count',0)>0 for name in required), 'Missing minimal phase evidence'
    delivered=l.count('[datum-timing] runtime render diagnostic_minimal=true')
    assert delivered>0 and all(phases[name]['count']==delivered for name in required[1:]), 'Partial-frame phase boundary'
   events={k:d.count('window event '+k) for k in ['keyboard','mouse input','mouse wheel','cursor moved','ime','modifiers changed']}
   rows.append({'phase':axis,'cpu_phase_totals':phases,'completion_observed':completed,'original_extent':original,'elapsed_s':elapsed,'cpu_percent_one_core':(c1-c0)/hz/elapsed*100,'gpu_percent':{k:(g1[k]-v)/(elapsed*1e9)*100 for k,v in g0.items() if k in g1},'lost_gpu_clients':sorted(set(g0)-set(g1)),'configurations':d.count('surface configure begin'),'present_calls':l.count('[datum-timing] runtime render '),'resize_events':d.count('window event resized'),'timeouts':d.count('surface acquire timeout'),'recoveries':d.count('surface acquire recovered'),'input_event_counts':events,'raw_counters':timeline})
   (out/'samples.json').write_text(json.dumps(rows,indent=2)+'\n')
   print(json.dumps({k:v for k,v in rows[-1].items() if k!='raw_counters'}),flush=True)
   if plugin:
    assert dbus('/Scripting','org.kde.kwin.Scripting.unloadScript',plugin)=='true'
    plugin=None
   assert completed, 'Native sentinel and restoration not observed; phase incomplete'
   if axis!='idle':assert rows[-1]['resize_events']>80, 'Native resize injection failed'
   if any(events.values()):raise RuntimeError('Input observed; preserve samples and stop for review')
   time.sleep(2)
  assert hashlib.sha256(binary.read_bytes()).hexdigest()==expected
  (out/'report.json').write_text(json.dumps({'binary_sha256':expected,'diagnostic':'minimal native CPU phases','fixture_loaded':False,'required_environment':{'DATUM_DIAGNOSTIC_MINIMAL_WINDOW':'1','DATUM_RESIZE_CPU_PROBE':'1'},'mode':a.mode,'fixture_sha256':hashlib.sha256(board.read_bytes()).hexdigest(),'identity':[l for l in diag.read_text().splitlines() if 'surface identity' in l],'samples':rows,'limitations':['CPU phase probes enabled: non-overlapping call windows, not causal attribution of worker-thread work. No visual or resource acceptance.','Verbose and timing diagnostics enabled; no uninstrumented resource acceptance.','Compositor-script changes frame geometry; not physical border dragging.','Sampling includes script dispatch, two-axis completion sentinel and restoration/settling tail; not exactly 5 seconds of resizing.','Application input-event counts cannot observe desktop input delivered elsewhere.','Presentation calls are not displayed frames; no temporal visual oracle.','GPU counters cover surviving clients; check lost_gpu_clients.']},indent=2)+'\n')
 finally:
  if plugin:subprocess.run(['qdbus6','org.kde.KWin','/Scripting','org.kde.kwin.Scripting.unloadScript',plugin],capture_output=True,text=True,timeout=10)
  if p.poll() is None:p.terminate();p.wait(timeout=10)
