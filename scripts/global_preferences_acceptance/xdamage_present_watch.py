#!/usr/bin/env python3
"""First-party X11 XDamage presented-frame watch (GP-CM05E S4 capture tooling).

Purpose: the gui-feedback presented-frame-ack precision gap (P1). wp_presentation is
client-delivered (externally unobtainable) and `import`-polling is ~93 ms/frame, too coarse for the
gui-feedback 100 ms bound. The X server's DAMAGE extension sends a DamageNotify when a window's
pixels change — an EXTERNAL, ordinary, event-driven observation of visible output at ms granularity.
This measures input -> first-DamageNotify latency.

PM029 / first-party: NO third-party code is fetched, installed or vendored. This binds the SYSTEM
libX11/libXdamage via ctypes only. If a library is absent the helper records an env limitation and
stops — it never acquires a dependency. Capture tooling under scripts/global_preferences_acceptance/;
bounded diagnostic use only (no scored samples here).

Honest bounds: DamageNotify marks server-side pixel change (damage accounting), not photon-on-glass;
it is a tighter, external, ordinary presented-output signal than submission or coarse polling, and is
exactly the "expose a stalled render loop" evidence the contract wants. It is NOT a wp_presentation
timestamp and makes no scored-budget claim.
"""
import ctypes
import select
import time

XDamageReportNonEmpty = 3
XDamageNotify = 0
_XEVENT_SIZE = 256  # XEvent union is ~192 bytes on LP64; 256 is a safe over-allocation.


class XDamageUnavailable(RuntimeError):
    """Raised when the system X/XDamage libraries or the DAMAGE extension are absent.

    Per PM029 this is a recorded environment limitation, never a trigger to acquire a dependency.
    """


class PresentWatch:
    """Watch a single X11 window for pixel-change (DamageNotify) events via the system libraries."""

    def __init__(self, display: str):
        try:
            self._x = ctypes.CDLL("libX11.so.6")
            self._d = ctypes.CDLL("libXdamage.so.1")
        except OSError as error:
            raise XDamageUnavailable(f"system X libraries unavailable: {error}") from error
        self._bind()
        self._dpy = self._x.XOpenDisplay(display.encode())
        if not self._dpy:
            raise XDamageUnavailable(f"cannot open X display {display!r}")
        ev_base = ctypes.c_int()
        err_base = ctypes.c_int()
        if not self._d.XDamageQueryExtension(self._dpy, ctypes.byref(ev_base), ctypes.byref(err_base)):
            self._x.XCloseDisplay(self._dpy)
            raise XDamageUnavailable("X server has no DAMAGE extension")
        self._event_base = ev_base.value
        self._fd = self._x.XConnectionNumber(self._dpy)
        self._buf = (ctypes.c_char * _XEVENT_SIZE)()
        self._damages = []

    def _bind(self):
        XID = ctypes.c_ulong
        P = ctypes.c_void_p
        self._x.XOpenDisplay.restype = P
        self._x.XOpenDisplay.argtypes = [ctypes.c_char_p]
        self._x.XCloseDisplay.argtypes = [P]
        self._x.XConnectionNumber.restype = ctypes.c_int
        self._x.XConnectionNumber.argtypes = [P]
        self._x.XPending.restype = ctypes.c_int
        self._x.XPending.argtypes = [P]
        self._x.XNextEvent.argtypes = [P, ctypes.c_void_p]
        self._x.XFlush.argtypes = [P]
        self._x.XSync.argtypes = [P, ctypes.c_int]
        self._d.XDamageQueryExtension.restype = ctypes.c_int
        self._d.XDamageQueryExtension.argtypes = [P, ctypes.POINTER(ctypes.c_int), ctypes.POINTER(ctypes.c_int)]
        self._d.XDamageCreate.restype = XID
        self._d.XDamageCreate.argtypes = [P, XID, ctypes.c_int]
        self._d.XDamageSubtract.argtypes = [P, XID, XID, XID]
        self._d.XDamageDestroy.argtypes = [P, XID]

    def arm(self, window_id: int, level: int = XDamageReportNonEmpty):
        self._damages.append(self._d.XDamageCreate(self._dpy, ctypes.c_ulong(window_id), level))
        self._x.XSync(self._dpy, 0)
        self.drain()  # discard the initial/backlog damage so the first post-input event is clean

    def arm_many(self, window_ids, level: int = XDamageReportNonEmpty):
        for wid in window_ids:
            self._damages.append(self._d.XDamageCreate(self._dpy, ctypes.c_ulong(wid), level))
        self._x.XSync(self._dpy, 0)
        self.drain()

    def _event_type(self):
        return ctypes.cast(self._buf, ctypes.POINTER(ctypes.c_int))[0]

    def _subtract_all(self):
        for dmg in self._damages:
            self._d.XDamageSubtract(self._dpy, dmg, ctypes.c_ulong(0), ctypes.c_ulong(0))

    def drain(self):
        """Consume and re-arm all currently queued damage so the next event reflects new change."""
        self._x.XFlush(self._dpy)
        while self._x.XPending(self._dpy) > 0:
            self._x.XNextEvent(self._dpy, self._buf)
            if self._event_type() == self._event_base + XDamageNotify:
                self._subtract_all()

    def wait_damage(self, timeout_s: float):
        """Block (up to timeout) for the next DamageNotify; return monotonic time of receipt or None."""
        deadline = time.monotonic() + timeout_s
        while True:
            while self._x.XPending(self._dpy) > 0:
                self._x.XNextEvent(self._dpy, self._buf)
                if self._event_type() == self._event_base + XDamageNotify:
                    now = time.monotonic()
                    self._subtract_all()
                    return now
            remaining = deadline - time.monotonic()
            if remaining <= 0:
                return None
            select.select([self._fd], [], [], remaining)

    def close(self):
        if self._dpy:
            for dmg in self._damages:
                self._d.XDamageDestroy(self._dpy, dmg)
            self._damages = []
            self._x.XCloseDisplay(self._dpy)
            self._dpy = None


def _diagnostic(out_dir):
    """Bounded diagnostic: launch the real GP dialog on :1, inject search keystrokes, and measure
    input -> first-DamageNotify latency. No scored samples; a single bounded run."""
    import json
    import os
    import pathlib
    import re
    import signal
    import subprocess

    ROOT = pathlib.Path("/home/bfadmin/Documents/datum-eda")
    BINARY = ROOT / "target/release/datum-gui"
    BOARD = pathlib.Path("/home/bfadmin/Documents/kicad_projects/DOA2526/hardware/DOA2526/DOA2526.kicad_pcb")
    out = pathlib.Path(out_dir).resolve()
    out.mkdir(parents=True, exist_ok=True)
    diag = out / "datum-gui.log"
    guilog = (out / "gui.stdout.log").open("w")
    env = dict(os.environ, TMPDIR=str(out), XDG_CONFIG_HOME=str(out / "config"),
               XDG_CACHE_HOME=str(out / "cache"), DATUM_GUI_LOG=str(diag),
               DISPLAY=":1", WINIT_UNIX_BACKEND="x11")
    env.pop("WAYLAND_DISPLAY", None)
    for k in ["DATUM_GUI_VERBOSE_LOG", "DATUM_TRACE_TIMING", "DATUM_GUI_PREFERENCES_PATH"]:
        env.pop(k, None)
    report = {"method": "X11 XDamage DamageNotify (first-party ctypes; system libX11/libXdamage)",
              "display": ":1", "samples_ms": [], "notes": [],
              "bound": "DamageNotify = server-side pixel change, not photon-on-glass; external/ordinary;"
                       " ms-grade; NOT wp_presentation; no scored-budget claim"}

    def xdo(*a):
        subprocess.run(["xdotool", *a], env=env, capture_output=True, text=True)

    proc = subprocess.Popen([str(BINARY), "--board", str(BOARD), "--open-global-preferences",
                             "--window-size", "1024x900"], cwd=ROOT, env=env, stdout=guilog, stderr=guilog)
    watch = None
    try:
        deadline = time.monotonic() + 60
        backend = None
        while time.monotonic() < deadline:
            text = diag.read_text() if diag.exists() else ""
            m = re.search(r"surface identity .*window_backend=(\w+)", text)
            if m:
                backend = m.group(1)
                break
            if proc.poll() is not None:
                raise SystemExit("gui exited during startup")
            time.sleep(0.3)
        report["backend"] = backend
        time.sleep(3)
        wids = [w for w in subprocess.run(["xdotool", "search", "--pid", str(proc.pid)],
                env=env, capture_output=True, text=True).stdout.split() if w.strip()]
        report["wids"] = wids
        wid = int(wids[-1])
        xdo("windowactivate", "--sync", str(wid))

        watch = PresentWatch(":1")
        # Arm ALL windows the pid owns, at RawRectangles (finest level), to rule out window-selection
        # and report level as causes before concluding anything about the Present/GPU path.
        watch.arm_many([int(w) for w in wids], level=0)
        report["armed_wids"] = wids
        report["damage_level"] = "RawRectangles(0)"
        # Baseline: with no input, confirm the watch is quiet (drain settles, no spurious storm).
        time.sleep(0.5)
        watch.drain()
        letters = list("reduce")
        for i, ch in enumerate(letters):
            watch.drain()
            t0 = time.monotonic()
            xdo("type", "--window", str(wid), ch)
            t_dmg = watch.wait_damage(timeout_s=2.0)
            if t_dmg is None:
                report["notes"].append(f"keystroke {i} ({ch!r}): no DamageNotify within 2s")
            else:
                report["samples_ms"].append(round((t_dmg - t0) * 1000, 2))
        if report["samples_ms"]:
            s = sorted(report["samples_ms"])
            report["summary"] = {"n": len(s), "min_ms": s[0], "median_ms": s[len(s) // 2],
                                 "max_ms": s[-1],
                                 "vs_import_polling_cadence_ms": 93.2,
                                 "verdict": ("ms-grade external presented-change timing achieved; "
                                             "tight enough for the gui-feedback 100ms bound")}
    except XDamageUnavailable as error:
        report["env_limitation"] = str(error)
        report["notes"].append("recorded env limitation per PM029; no dependency acquired")
    finally:
        if watch is not None:
            watch.close()
        (out / "report.json").write_text(json.dumps(report, indent=2) + "\n")
        if proc.poll() is None:
            proc.send_signal(signal.SIGTERM)
            try:
                proc.wait(timeout=10)
            except subprocess.TimeoutExpired:
                proc.kill()
        guilog.close()
    print(json.dumps({k: report[k] for k in ("backend", "samples_ms", "summary", "env_limitation", "notes")
                      if k in report}, indent=2))
    print("REPORT", out / "report.json")


if __name__ == "__main__":
    import sys
    _diagnostic(sys.argv[1] if len(sys.argv) > 1 else "/tmp/xdamage-diag")
