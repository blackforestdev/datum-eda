"""Current-source ordinary horizontal/vertical resize CPU reproduction (Launch 1).

Adapted from the archived PID-targeted KWin resize pilot
(docs/reviews/gui-performance/measurements/resize-minimal-native/harness.py).
Per the owner-approved, PM-re-pinned packet
(docs/reviews/gui-performance/cpu-reconciliation/resize-investigation.md +
gpi-resize-repin-decision.json, option b), this is an explicitly CURRENT-SOURCE
reproduction, not an R4 replay.

Adaptations vs the archived pilot:
  * Launches the REAL product GUI on the DOA2526 board (no clear/minimal modes,
    no DATUM_DIAGNOSTIC_* / DATUM_TRACE_TIMING / DATUM_GUI_VERBOSE_LOG,
    no GPU fdinfo queries). Readiness and resize are read from ORDINARY logs
    (`surface identity ... window_backend=`, `resize apply ... -> WxH`).
  * Application CPU is accounted from a fresh owned cgroup v2 scope
    (systemd-run --user --scope) via cpu.stat, not /proc/pid/stat.
  * KWin compositor CPU is accounted separately over the same windows.
  * Geometry recipe: ~60 Hz triangular 0..180..0 over 2 s cycles, 600 planned
    updates per axis; actual emitted counts and phase boundaries are recorded,
    never assumed.

This file performs NO optimization and no product edit. It is Launch 1 only:
the conditional Launch 2 (DATUM_RESIZE_CPU_PROBE) is a separate invocation and
is used only if Launch 1 validly reproduces the concern.
"""
import argparse, json, os, pathlib, re, subprocess, time, signal

ROOT = pathlib.Path('/home/bfadmin/Documents/datum-eda')
BINARY = ROOT / 'target/release/datum-gui'
BOARD = pathlib.Path(
    '/home/bfadmin/Documents/kicad_projects/DOA2526/hardware/DOA2526/DOA2526.kicad_pcb')
HZ = os.sysconf('SC_CLK_TCK')

# ---- geometry recipe --------------------------------------------------------
TICK_MS = 16          # ~62.5 Hz target; actual delivery is recorded, not assumed
CYCLE_TICKS = 120     # 2 s triangular cycle at the target rate
PLANNED_UPDATES = 600 # 5 cycles ~= 10 s of active resizing per axis
ENDPOINT_DELTA = 180  # held changed-size extent for the output capture


def kwin_qml(pid: int, axis: str) -> str:
    """Triangular 0..180..0 resize of exactly the PID-owned window, then restore.

    Emits `count` ticks; refuses an ambiguous target. AXIS in {width,height};
    the other client dimension is held constant.
    """
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
 Timer { id: timer; interval: TICK; repeat: true
  onTriggered: {
   if (!owned || owned.pid !== PID) { stop(); return; }
   if (count >= PLANNED) { owned.frameGeometry = original; stop(); return; }
   let delta = Math.abs((count % CYCLE) - (CYCLE/2)) * (180.0/(CYCLE/2));
   owned.frameGeometry = Qt.rect(original.x, original.y,
      original.width  + (AXIS === "width"  ? delta : 0),
      original.height + (AXIS === "height" ? delta : 0));
   count++;
  }
 }
}
'''.replace('PLANNED', str(PLANNED_UPDATES)).replace('CYCLE', str(CYCLE_TICKS)) \
   .replace('TICK', str(TICK_MS)).replace('PID', str(pid)).replace('AXIS', json.dumps(axis))


def kwin_hold_qml(pid: int, axis: str, delta: int) -> str:
    """Hold a fixed +delta extent on one axis (endpoint capture), no restore."""
    return '''import QtQuick
import org.kde.kwin
Item {
 Component.onCompleted: {
  let m = Workspace.stackingOrder.filter(w => w.pid === PID);
  if (m.length !== 1) { console.error("DATUM refuses ambiguous resize target"); return; }
  let o = m[0]; let g = o.frameGeometry;
  o.frameGeometry = Qt.rect(g.x, g.y,
     g.width  + (AXIS === "width"  ? DELTA : 0),
     g.height + (AXIS === "height" ? DELTA : 0));
 }
}
'''.replace('PID', str(pid)).replace('AXIS', json.dumps(axis)).replace('DELTA', str(delta))


def kwin_restore_qml(pid: int, w: int, h: int) -> str:
    return '''import QtQuick
import org.kde.kwin
Item { Component.onCompleted: {
  let m = Workspace.stackingOrder.filter(w => w.pid === PID);
  if (m.length === 1) { let o=m[0]; let g=o.frameGeometry; o.frameGeometry = Qt.rect(g.x,g.y,W,H); }
}}'''.replace('PID', str(pid)).replace('W', str(w)).replace('H', str(h))


# ---- cgroup + cpu accounting ------------------------------------------------
def read_cpu_stat(cgpath: pathlib.Path) -> dict:
    """cgroup v2 cpu.stat: usage/user/system in microseconds."""
    out = {}
    for line in (cgpath / 'cpu.stat').read_text().splitlines():
        k, _, v = line.partition(' ')
        if k in ('usage_usec', 'user_usec', 'system_usec'):
            out[k] = int(v)
    return out


def proc_cpu_jiffies(pid: int) -> int:
    f = pathlib.Path(f'/proc/{pid}/stat').read_text().rsplit(')', 1)[1].split()
    return int(f[11]) + int(f[12])  # utime + stime


def qdbus(*args) -> str:
    return subprocess.check_output(['qdbus6', 'org.kde.KWin', *args],
                                   text=True, timeout=10).strip()


def find_kwin_pid() -> int:
    out = subprocess.check_output(['pgrep', '-x', 'kwin_wayland'], text=True).split()
    assert len(out) == 1, f'expected one kwin_wayland, found {out}'
    return int(out[0])


# ---- pure-helper self-check (no GUI, no compositor) -------------------------
def self_check():
    q = kwin_qml(123, 'height')
    assert 'w.pid === 123' in q and 'owned.pid !== 123' in q  # PID as integer
    assert '"height" === "height"' in q and '"height" === "width"' in q
    assert str(PLANNED_UPDATES) in q and str(CYCLE_TICKS) in q and f'interval: {TICK_MS}' in q
    h = kwin_hold_qml(123, 'width', 180)
    assert '"width" === "width"' in h and '? 180 :' in h
    r = kwin_restore_qml(123, 1280, 800)
    assert 'Qt.rect(g.x,g.y,1280,800)' in r
    assert proc_cpu_jiffies(os.getpid()) >= 0
    print('Pure helper checks passed; no GUI or compositor action.')


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('--output')
    ap.add_argument('--check', action='store_true')
    ap.add_argument('--probe', action='store_true',
                    help='Launch 2: enable DATUM_RESIZE_CPU_PROBE and bucket per-phase CPU clocks')
    a = ap.parse_args()
    if a.check:
        self_check()
        return
    assert a.output, '--output is required for a measurement run'

    # Absolute paths only: the engine rejects a relative XDG_CONFIG_HOME.
    out = pathlib.Path(a.output).resolve()
    out.mkdir(exist_ok=False)
    shots = out / 'shots'; shots.mkdir()
    assert os.environ.get('WAYLAND_DISPLAY'), 'Native Wayland display required'
    assert BINARY.exists(), f'built binary missing: {BINARY}'
    assert BOARD.exists(), f'DOA2526 board missing: {BOARD}'

    diag = out / 'datum-gui.log'
    env = dict(os.environ,
               TMPDIR=str(out),
               XDG_CONFIG_HOME=str(out / 'config'),
               XDG_CACHE_HOME=str(out / 'cache'),
               DATUM_GUI_LOG=str(diag))
    # Strip every diagnostic/trace/forcing knob: Launch 1 is ordinary.
    for k in ['DATUM_GUI_VERBOSE_LOG', 'DATUM_TRACE_TIMING', 'DATUM_DIAGNOSTIC_NATIVE_FRAME',
              'DATUM_DIAGNOSTIC_MINIMAL_WINDOW', 'DATUM_RESIZE_CPU_PROBE',
              'DATUM_GPU_DIAGNOSTIC_LATENCY', 'DATUM_GPU_DIAGNOSTIC_BACKEND',
              'WINIT_UNIX_BACKEND', 'DATUM_GUI_PREFERENCES_PATH']:
        env.pop(k, None)
    if a.probe:
        env['DATUM_RESIZE_CPU_PROBE'] = '1'  # Launch 2: existing phase CPU clocks only

    unit = f'datum-resize-{int(time.time())}.scope'
    kwin_pid = find_kwin_pid()
    report = {'unit': unit, 'binary': str(BINARY), 'board': str(BOARD),
              'kwin_pid': kwin_pid, 'tick_ms': TICK_MS, 'cycle_ticks': CYCLE_TICKS,
              'planned_updates': PLANNED_UPDATES, 'probe': bool(a.probe), 'phases': []}

    gui_log = (out / 'gui.stdout.log').open('w')
    # systemd-run --user --scope runs the command as a child, in a fresh owned
    # cgroup, inheriting this env. -p Delegate=yes so cpu accounting is isolated.
    proc = subprocess.Popen(
        ['systemd-run', '--user', '--scope', '--quiet', '-p', 'Delegate=yes',
         '--unit', unit, '--', str(BINARY), '--board', str(BOARD),
         '--window-size', '1280x800'],
        cwd=ROOT, env=env, stdout=gui_log, stderr=gui_log)

    active_plugin = [None]

    def load_script(name: str, qml_text: str) -> str:
        qf = out / (name + '.qml'); qf.write_text(qml_text)
        sid = qdbus('/Scripting', 'org.kde.kwin.Scripting.loadDeclarativeScript',
                    str(qf), name)
        assert int(sid) >= 0
        active_plugin[0] = name
        return sid

    def run_script(sid: str):
        qdbus('/Scripting/Script' + sid, 'org.kde.kwin.Script.run')

    def unload():
        if active_plugin[0]:
            qdbus('/Scripting', 'org.kde.kwin.Scripting.unloadScript', active_plugin[0])
            active_plugin[0] = None

    def shot(tag: str):
        subprocess.run(['spectacle', '-b', '-n', '-f', '-o', str(shots / (tag + '.png'))],
                       check=False, timeout=20,
                       stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)

    try:
        # --- locate the scope cgroup ---
        cg = None
        for _ in range(600):
            try:
                cgline = subprocess.check_output(
                    ['systemctl', '--user', 'show', unit, '-p', 'ControlGroup'],
                    text=True).strip()
                rel = cgline.split('=', 1)[1]
                if rel:
                    cg = pathlib.Path('/sys/fs/cgroup') / rel.lstrip('/')
                    if (cg / 'cpu.stat').exists():
                        break
            except subprocess.CalledProcessError:
                pass
            if proc.poll() is not None:
                raise RuntimeError('GUI scope exited before cgroup was available')
            time.sleep(0.1)
        assert cg and (cg / 'cpu.stat').exists(), 'scope cgroup/cpu.stat not found'
        report['cgroup'] = str(cg)

        # --- readiness: ordinary surface-identity line + KWin sees the window ---
        gui_pid = None
        deadline = time.monotonic() + 60
        while time.monotonic() < deadline:
            text = diag.read_text() if diag.exists() else ''
            m = re.search(r'surface identity .*window_backend=(\w+)', text)
            procs = [int(p) for p in (cg / 'cgroup.procs').read_text().split()]
            cands = [p for p in procs
                     if pathlib.Path(f'/proc/{p}/comm').exists()
                     and pathlib.Path(f'/proc/{p}/comm').read_text().strip() == 'datum-gui']
            if m and cands:
                gui_pid = cands[0]
                report['window_backend'] = m.group(1)
                break
            if proc.poll() is not None:
                raise RuntimeError('GUI exited during startup')
            time.sleep(0.2)
        assert gui_pid, 'no ordinary surface-identity readiness within 60s'
        assert report['window_backend'] == 'wayland', \
            f"unexpected backend {report['window_backend']} (expected wayland); stopping"

        # exactly one unmaximized KWin window owns this PID
        time.sleep(5)  # no-input preparation
        shot('01-readiness')

        def sample_phase(name: str, duration: float, do_run=None):
            """Bracketed cgroup + KWin accounting with a 100ms external series."""
            d0 = diag.stat().st_size
            cg0 = read_cpu_stat(cg); kw0 = proc_cpu_jiffies(kwin_pid)
            t0 = time.monotonic(); series = []
            if do_run:
                do_run()
            while time.monotonic() - t0 < duration:
                if proc.poll() is not None:
                    raise RuntimeError('GUI exited during measurement')
                series.append((time.monotonic() - t0,
                               read_cpu_stat(cg).get('usage_usec', 0)))
                time.sleep(0.1)
            elapsed = time.monotonic() - t0
            cg1 = read_cpu_stat(cg); kw1 = proc_cpu_jiffies(kwin_pid)
            tail = diag.read_bytes()[d0:].decode(errors='replace')
            applies = re.findall(r'resize apply (\d+)x(\d+) -> (\d+)x(\d+)', tail)
            # per-phase probe aggregation (Launch 2 only). Nested/overlapping
            # windows are NOT summed into an exact partition; raw aggregates only.
            probe_phases = {}
            for pname, w, pr, th in re.findall(
                    r'cpu_probe phase=(\w+) wall_ns=(\d+) process_ns=(\d+) thread_ns=(\d+)', tail):
                d = probe_phases.setdefault(
                    pname, {'count': 0, 'wall_ns': 0, 'process_ns': 0, 'thread_ns': 0})
                d['count'] += 1
                d['wall_ns'] += int(w)
                d['process_ns'] += int(pr)
                d['thread_ns'] += int(th)
            # peak 100ms interval
            peak = None
            for i in range(1, len(series)):
                dt = series[i][0] - series[i-1][0]
                du = series[i][1] - series[i-1][1]
                if dt > 0:
                    pct = du / (dt * 1e6) * 100
                    if peak is None or pct > peak['percent_one_core']:
                        peak = {'percent_one_core': pct, 'window_s': dt,
                                't_rel_s': series[i][0]}
            rec = {
                'phase': name, 'elapsed_s': elapsed,
                'app_cpu_percent_one_core': (cg1['usage_usec'] - cg0['usage_usec']) / (elapsed*1e6) * 100,
                'app_user_percent_one_core': (cg1['user_usec'] - cg0['user_usec']) / (elapsed*1e6) * 100,
                'app_system_percent_one_core': (cg1['system_usec'] - cg0['system_usec']) / (elapsed*1e6) * 100,
                'kwin_cpu_percent_one_core': (kw1 - kw0) / HZ / elapsed * 100,
                'resize_apply_count': len(applies),
                'first_apply': applies[0] if applies else None,
                'last_apply': applies[-1] if applies else None,
                'peak_100ms': peak,
                'probe_phases': probe_phases,
                'series_100ms': series,
            }
            report['phases'].append(rec)
            print(json.dumps({k: v for k, v in rec.items() if k != 'series_100ms'}), flush=True)
            return rec

        # --- quiet control (10 s, no input) ---
        sample_phase('quiet_control', 10.0)

        for axis in ('width', 'height'):
            sid = load_script(f'resize-{axis}', kwin_qml(gui_pid, axis))
            rec = sample_phase(f'resize_{axis}', 11.0, do_run=lambda s=sid: run_script(s))
            unload()
            # verify extents actually changed on this axis and restored
            applies = [tuple(map(int, a)) for a in re.findall(
                r'resize apply (\d+)x(\d+) -> (\d+)x(\d+)',
                (out / 'datum-gui.log').read_text())]
            rec['verified_axis_grew'] = any(
                (a[2] > 1280 if axis == 'width' else a[3] > 800) for a in applies)
            rec['verified_restored'] = bool(applies) and applies[-1][2:] == (1280, 800)
            time.sleep(5)  # settle outside the active window

            if not a.probe:
                # Launch 1 endpoint changed-size capture (outside CPU windows).
                # The triangular QML has already self-restored to its captured
                # original; this is only an output-liveness check.
                hsid = load_script(f'hold-{axis}', kwin_hold_qml(gui_pid, axis, ENDPOINT_DELTA))
                run_script(hsid); time.sleep(1.5); shot(f'03-changed-{axis}')
                unload()
                rsid = load_script(f'restore-{axis}', kwin_restore_qml(gui_pid, 1280, 800))
                run_script(rsid); time.sleep(1.0); unload()

        # --- quiet tail (10 s) ---
        sample_phase('quiet_tail', 10.0)
        shot('04-restored')

        report['fixture_board_sha256'] = subprocess.check_output(
            ['sha256sum', str(BOARD)], text=True).split()[0]
        report['binary_sha256'] = subprocess.check_output(
            ['sha256sum', str(BINARY)], text=True).split()[0]
        report['surface_identity'] = [l for l in diag.read_text().splitlines()
                                       if 'surface identity' in l]
        report['limitations'] = [
            'Current-source reproduction, not an R4 replay; no R4 numeric equivalence.',
            'KWin declarative geometry injection is native-window resize delivery, not physical decoration dragging; non-reproduction does not refute a drag-specific owner observation.',
            'App CPU is the owned cgroup scope cpu.stat (includes exited descendants); KWin CPU is /proc utime+stime; compositor/controller CPU is excluded from app accounting.',
            'Phase windows include KWin timer-delivery jitter; emitted resize-apply counts and actual phase bounds are recorded, delivery rate is not assumed to be 60/s.',
            'Captures are output/liveness checks, not a no-flicker, latency, hit-testing or PTY oracle.',
            'No numeric budget/threshold decided (PM051); DISPOSITION gate pending.',
        ]
        (out / 'report.json').write_text(json.dumps(report, indent=2) + '\n')
        print('REPORT', out / 'report.json', flush=True)
    finally:
        try:
            unload()
        except Exception:
            pass
        if proc.poll() is None:
            proc.send_signal(signal.SIGTERM)
            try:
                proc.wait(timeout=10)
            except subprocess.TimeoutExpired:
                proc.kill()
        gui_log.close()
        subprocess.run(['systemctl', '--user', 'stop', unit],
                       capture_output=True, text=True, timeout=10)


if __name__ == '__main__':
    main()
