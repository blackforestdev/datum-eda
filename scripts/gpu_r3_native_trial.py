"""One predeclared r3 W-POINTER run. Never retries a trial or launches a campaign."""
import hashlib, json, math, os, re, signal, socket, subprocess, sys, tempfile, time, uuid
from pathlib import Path
ROOT = Path('/home/bfadmin/Documents/datum-eda')
sys.path.insert(0,str(ROOT/'docs/reviews/gui-performance/gpu-redraw-proposal/stopped-experiment-r2'))
from gpu_r3_pointer_input import pointer_input
sys.path.insert(0, str(ROOT / "scripts"))
from gpu_r3_receipts import validate as validate_gpu, validate_demands
from gpu_r2_wm_close import close_window

declaration = json.loads(Path(sys.argv[1]).read_text())
index = int(sys.argv[2]); spec = declaration['runs'][index]
assert Path(sys.argv[1]).resolve() == ROOT/'target/gpu-r3-proof/declaration.json'
campaign=json.loads((ROOT/'target/gpu-r3-proof/campaign-state.json').read_text())
assert campaign['status']=='running' and campaign['reserved_index']==index
assert campaign['declaration_sha256']==hashlib.sha256(Path(sys.argv[1]).read_bytes()).hexdigest()

assert spec['workload'] == 'W-POINTER'
role,mode = spec['role'],spec['mode']; assert role in ('baseline','candidate') and mode in ('quiet','gpu')
sha = lambda p: hashlib.sha256(Path(p).read_bytes()).hexdigest()
for path,digest in declaration['method_sha256'].items(): assert sha(path)==digest,path
binary=Path(declaration['binaries'][role]['path']); assert sha(binary)==declaration['binaries'][role]['sha256']
assert sha(declaration['cli']['path'])==declaration['cli']['sha256']
assert sha(declaration['reference_png']['path'])==declaration['reference_png']['sha256']
project=Path(declaration['project']['path'])
assert sha(project/'board/board.json')==declaration['project']['board_file_sha256']
model=json.loads((project/'board/board.json').read_text());model.pop('uuid',None)
assert hashlib.sha256(json.dumps(model,sort_keys=True,separators=(',',':')).encode()).hexdigest()==declaration['project']['normalized_board_sha256']
# Never measure alongside compilation. This is a read-only preflight, not tuning.
processes=subprocess.check_output(['ps','-eo','comm'],text=True).splitlines()
assert not any(p.strip() in ('cargo','rustc') for p in processes),'compiler active'
out=Path(tempfile.mkdtemp(prefix=f'gpu-r3-{index}-{role}-{mode}-'));print(out,flush=True)
report={'run_index':index,'spec':spec,'declaration_sha256':sha(sys.argv[1]),'binary_sha256':sha(binary),
        'status':'started','gpu_duty':'unavailable: complete DRM lifetime observer absent',
        'limits':['Descriptive bounded experiment only; not S4 qualification or formal relative inference.',
                  'Minimal buffered input observer enabled in both modes; its overhead is not subtracted.',
                  'No CPU/action, uninstrumented CPU/resource or physical presentation latency acceptance.']}
# Absolute schedule is sealed before process launch. Startup has a bounded 40s
# readiness allowance; the producer cannot move these boundaries after launch.
warmup = time.monotonic_ns() + 40_000_000_000
workload = dict(epoch=uuid.uuid4().int & ((1 << 63) - 1), warmup_ns=warmup,
                active_ns=warmup+5_000_000_000, still_ns=warmup+35_000_000_000,
                drain_ns=warmup+40_000_000_000)
(out/'workload.json').write_text(json.dumps(workload,indent=2)+'\n')
report['workload'] = workload

def wait_until(deadline):
    while (remaining := (deadline-time.monotonic_ns())/1e9) > 0:
        time.sleep(min(remaining, .05))
def save(): (out/'result.json').write_text(json.dumps(report,indent=2)+'\n')
def xd(*args):
    r=subprocess.run(['xdotool',*map(str,args)],capture_output=True,text=True,timeout=10)
    if r.returncode and args[0]!='search': raise RuntimeError(r.stderr)
    return r.stdout.strip()
log=out/'native.log';log.touch()
env={k:v for k,v in os.environ.items() if not k.startswith('DATUM_')}
for key in ('WAYLAND_DISPLAY','LD_AUDIT','LD_PRELOAD'):env.pop(key,None)
env.update(WINIT_UNIX_BACKEND='x11',WINIT_X11_SCALE_FACTOR='1',
    XDG_CONFIG_HOME=str(out/'config'),XDG_CACHE_HOME=str(out/'cache'),
    XDG_DATA_HOME=str(out/'data'),XDG_STATE_HOME=str(out/'state'),
    DATUM_GUI_LOG=str(log),DATUM_GPU_MEASUREMENTS='1' if mode=='gpu' else '0',
    DATUM_INPUT_RECEIPT=str(out/'input-receipt.json'),
    DATUM_WORKLOAD_MANIFEST=str(out/'workload.json'),EDA_CLI_BIN=declaration['cli']['path'])
listener=socket.socket(socket.AF_UNIX,socket.SOCK_STREAM)
listener.bind(str(out/'observer.sock'));listener.listen(1);listener.settimeout(15)
env['DATUM_MEASUREMENT_SHUTDOWN_SOCKET']=str(out/'observer.sock')
group=Path('/sys/fs/cgroup/user.slice')/f'user-{os.getuid()}.slice'/f'user@{os.getuid()}.service'/f'datum-gpu-r3-{uuid.uuid4().hex}'
group.mkdir()
def cpu():
    values={k:int(v) for k,v in (row.split() for row in (group/'cpu.stat').read_text().splitlines())}
    value={'monotonic_ns':time.monotonic_ns(),'cpu_stat':values,'members':(group/'cgroup.procs').read_text().split()}
    report.setdefault('cpu_samples',[]).append(value);return value
def enter_group(): (group/'cgroup.procs').write_text(str(os.getpid()))
command=[str(binary),'--project-root',str(project),'--initial-layout','single',
         '--window-size','1280x800','--visual-scale-factor','1']
report['command']=command; report['observer_storage_limit_records']=4096;report['cpu_before_spawn']=cpu()
report['owned_cgroup']=str(group);save()
p=None
producer=None
try:
    with (out/'stderr.log').open('w') as stream:
        p=subprocess.Popen(command,cwd=ROOT,env=env,stdout=stream,stderr=stream,
                           start_new_session=True,preexec_fn=enter_group)
    report['pid']=p.pid;save()
    deadline=time.monotonic()+20
    main=None
    while time.monotonic()<deadline:
        assert p.poll() is None,('startup exit',p.returncode)
        found=xd('search','--all','--onlyvisible','--pid',p.pid,'--name','Datum')
        if found: main=int(found.splitlines()[0]);break
        time.sleep(.025)
    assert main,'native window unavailable'
    report['window']=main
    xd('windowactivate','--sync',main)
    geometry=dict(line.split('=',1) for line in xd('getwindowgeometry','--shell',main).splitlines() if '=' in line)
    assert (int(geometry['WIDTH']),int(geometry['HEIGHT']))==(1280,800)
    report['geometry']=geometry
    xd('mousemove','--window',main,300,150)
    # Readiness is inside this run, not a separate producer rehearsal.
    deadline=time.monotonic()+15
    ready=False
    while time.monotonic()<deadline:
        assert p.poll() is None
        subprocess.run(['import','-window',str(main),str(out/'warm.png')],check=True,timeout=10)
        comparison=subprocess.run(['compare','-metric','AE',declaration['reference_png']['path'],str(out/'warm.png'),'null:'],capture_output=True,text=True,timeout=10)
        assert comparison.returncode in (0,1)
        if comparison.returncode==0:ready=True;break
        time.sleep(.1)
    report['readiness_pixel_difference']=comparison.stderr
    assert ready,'exact source-reviewed readiness image not observed'
    identity=log.read_text()
    assert 'Intel(R) HD Graphics P630 (KBL GT2)' in identity and 'gpu_backend=Vulkan' in identity
    assert '1280x800 format=Bgra8UnormSrgb present=Fifo msaa=8' in identity
    report['identity_lines']=[s for s in identity.splitlines() if 'surface identity ' in s or 'initial surface configuration ' in s]
    assert time.monotonic_ns() < workload['warmup_ns'], 'readiness missed declared warmup'
    wait_until(workload['warmup_ns'])
    report['warmup_start']=cpu()
    wait_until(workload['active_ns']-500_000_000)
    assert int(xd('getwindowfocus'))==main
    producer=subprocess.Popen([sys.executable,declaration['input_driver'],geometry['X'],geometry['Y'],
                               str(out/'schedule.json'),str(workload['active_ns'])])
    wait_until(workload['active_ns'])
    report['active_start']=cpu()
    producer.wait(timeout=35)
    assert producer.returncode==0, 'input producer failed'
    report['active_end']=cpu()
    wait_until(workload['drain_ns'])
    report['tail_end']=cpu()
    assert int(xd('getwindowfocus'))==main
    subprocess.run(['import','-window',str(main),str(out/'final.png')],check=True,timeout=10)
    comparison=subprocess.run(['compare','-metric','AE',declaration['reference_png']['path'],str(out/'final.png'),'null:'],capture_output=True,text=True,timeout=10)
    report['final_pixel_difference']=comparison.stderr
    assert comparison.returncode==0,'changed final pixels'
    report['drain_start']=cpu();close_window(main)
    connection,_=listener.accept();connection.settimeout(2)
    with connection:
        data=b''
        while not data.endswith(b'\n'):
            part=connection.recv(1);assert part;data+=part;assert len(data)<4096
        report['shutdown_receipt']=json.loads(data)
        assert report['shutdown_receipt']['pid']==p.pid
        assert report['shutdown_receipt']['phase']=='drained_device_live'
        report['drain_end']=cpu()
        connection.sendall(b'!')
    p.wait(timeout=10);report['exit_code']=p.returncode;assert p.returncode==0
    # The observer exports AFTER endpoint acknowledgement; now require the file.
    receipt=json.loads((out/'input-receipt.json').read_text())
    schedule=json.loads((out/'schedule.json').read_text())
    assert schedule['started_ns']==workload['active_ns'], 'producer moved declared active boundary'
    report['input']=pointer_input(schedule,receipt,report['tail_end']['monotonic_ns'],declaration['expected_context'],declaration['max_producer_lateness_ns'])
    assert receipt['pid']==p.pid,'foreign input receipt PID'
    expected_epoch=report['shutdown_receipt']['device_epoch']
    assert receipt['final_state']['device_epoch']==expected_epoch,'input/shutdown epoch mismatch'
    assert all(r[phase]['device_epoch']==expected_epoch for r in receipt['records'] for phase in ('before','after'))
    validate_demands(receipt,workload)
    report['input_storage_bytes']=receipt['record_storage_bytes']
    raw=log.read_bytes();lines=raw.decode().splitlines()
    assert not any('gpu_measurement_incomplete ' in line or 'gpu_measurement_log_failed ' in line for line in lines)
    samples=[json.loads(line.split('gpu_measurement ',1)[1]) for line in lines if line.startswith('gpu_measurement ')]
    if mode=='gpu':
        assert samples,'missing GPU samples'
        epochs={s['device_epoch'] for s in samples};hosts={s['host'] for s in samples}
        assert epochs=={expected_epoch} and len(hosts)==1
        bindings=re.findall(r'gpu_measurement_host host=(\d+) epoch=(\d+) window=WindowId\((\d+)\)',raw.decode())
        assert len(bindings)==1 and tuple(map(int,bindings[0]))==(next(iter(hosts)),expected_epoch,main),'foreign GPU host/window binding'
        classified=validate_gpu(samples,receipt,workload)
        (out/'classified-gpu.json').write_text(json.dumps(classified,indent=2)+'\n')
        active_ids={tuple(row['identity']) for row in classified if row['phase'] == 'active'}
        timed=[s for s in samples if (s['host'],s['device_epoch'],s['frame']) in active_ids]
        assert timed, 'no causally active GPU frames'
        spans=[s['frame_span_ns']/1e6 for s in timed]
        ordered=sorted(spans)
        stats={'n':len(spans),'p95_ms':ordered[math.ceil(.95*len(ordered))-1],
               'p99_ms':ordered[math.ceil(.99*len(ordered))-1],'max_ms':ordered[-1]}
        report['gpu']=stats;report['gpu_pass_names']={str(names):sum([p[0] for p in s['passes_ns']]==names for s in timed)
            for names in [list(names) for names in sorted({tuple(p[0] for p in s['passes_ns']) for s in timed})]}
        report['budget_stop']=stats['p95_ms']>4 or stats['p99_ms']>8
    else:
        assert not samples and receipt['gpu_drained'] is None
        report['gpu']='disabled';report['budget_stop']=False
    report['status']='valid_budget_failure' if report['budget_stop'] else 'valid_descriptive_run'
    report['post_export_cpu']=cpu()
    assert sha(project/'board/board.json')==declaration['project']['board_file_sha256'],'authored fixture changed'
except BaseException as error:
    report['status']='invalid';report['error']=repr(error)
finally:
    if producer is not None and producer.poll() is None:
        producer.terminate()
        try: producer.wait(timeout=2)
        except subprocess.TimeoutExpired: producer.kill();producer.wait(timeout=2)
        report['status']='invalid';report['error']='input producer required forced cleanup'
    if p is not None and p.poll() is None:
        os.killpg(p.pid,signal.SIGTERM)
        try:p.wait(timeout=5)
        except subprocess.TimeoutExpired:os.killpg(p.pid,signal.SIGKILL);p.wait(timeout=5)
        report['forced_cleanup']=True
    remaining=(group/'cgroup.procs').read_text().strip();report['remaining_before_cleanup']=remaining
    if remaining:
        report['status']='invalid';report['error']='workload descendants survived controlled root exit'
        (group/'cgroup.kill').write_text('1')
        deadline=time.monotonic()+5
        while (group/'cgroup.procs').read_text().strip() and time.monotonic()<deadline:time.sleep(.01)
    report['cpu_after_cleanup']=cpu()
    if not (group/'cgroup.procs').read_text().strip():group.rmdir()
    listener.close();(out/'observer.sock').unlink(missing_ok=True)
    report['binary_unchanged']=sha(binary)==report['binary_sha256']
    if not report['binary_unchanged']: report['status']='invalid';report['error']='binary changed'
    save()
print(report['status'],report.get('gpu'),report.get('error'),flush=True)
raise SystemExit(1 if report['status']=='invalid' else 2 if report['budget_stop'] else 0)
