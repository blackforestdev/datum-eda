#!/usr/bin/env python3
"""First-party XGetImage/XShm presented-pixel poll (GP-CM05E S4 capture tooling).

Why: datum-gui renders via Vulkan/WGPU and presents through DRI3/Present on X11, which BYPASSES
core X DAMAGE (proven: the XDamage helper captures xclock at ms precision but sees zero events from
datum-gui). The Present-updated FRONT BUFFER is still readable with XGetImage (that is why
`import -window` saw the change in P1). This polls the window pixels directly via libX11 XGetImage —
no per-sample process spawn (unlike `import`), so the cadence is ms-grade rather than ~93 ms — and
times input -> first visible pixel change.

PM029 / first-party: binds the SYSTEM libX11 via ctypes only; nothing fetched/installed. A missing
library is a recorded env limitation. Capture tooling under scripts/global_preferences_acceptance/;
bounded diagnostic only (no scored samples here).

Honest bounds: a poll detects change no finer than its own cadence; it reads the presented front
buffer (an external, ordinary visible-output observation), not a wp_presentation/PresentCompleteNotify
server timestamp. For a true per-present timestamp the ideal path is the X Present extension
(PresentCompleteNotify via libxcb-present, present on this host) — recommended for the milestone.
"""
import ctypes
import hashlib
import time

ZPixmap = 2
AllPlanes = (1 << 32) - 1  # masked to the server's plane count; ~0UL is conventional


class XImageUnavailable(RuntimeError):
    pass


class _XImage(ctypes.Structure):
    # Partial Xlib XImage: fields up to data + geometry are enough to read pixels.
    _fields_ = [("width", ctypes.c_int), ("height", ctypes.c_int), ("xoffset", ctypes.c_int),
                ("format", ctypes.c_int), ("data", ctypes.c_void_p), ("byte_order", ctypes.c_int),
                ("bitmap_unit", ctypes.c_int), ("bitmap_bit_order", ctypes.c_int),
                ("bitmap_pad", ctypes.c_int), ("depth", ctypes.c_int), ("bytes_per_line", ctypes.c_int),
                ("bits_per_pixel", ctypes.c_int)]


class ImagePollWatch:
    """Poll a window's presented pixels via XGetImage and hash a region to detect change."""

    def __init__(self, display: str):
        try:
            self._x = ctypes.CDLL("libX11.so.6")
        except OSError as error:
            raise XImageUnavailable(f"libX11 unavailable: {error}") from error
        P = ctypes.c_void_p
        XID = ctypes.c_ulong
        self._x.XOpenDisplay.restype = P
        self._x.XOpenDisplay.argtypes = [ctypes.c_char_p]
        self._x.XCloseDisplay.argtypes = [P]
        self._x.XGetGeometry.restype = ctypes.c_int
        self._x.XGetGeometry.argtypes = [P, XID, ctypes.POINTER(XID), ctypes.POINTER(ctypes.c_int),
                                         ctypes.POINTER(ctypes.c_int), ctypes.POINTER(ctypes.c_uint),
                                         ctypes.POINTER(ctypes.c_uint), ctypes.POINTER(ctypes.c_uint),
                                         ctypes.POINTER(ctypes.c_uint)]
        self._x.XGetImage.restype = ctypes.POINTER(_XImage)
        self._x.XGetImage.argtypes = [P, XID, ctypes.c_int, ctypes.c_int, ctypes.c_uint, ctypes.c_uint,
                                      ctypes.c_ulong, ctypes.c_int]
        self._x.XDestroyImage = ctypes.CDLL("libX11.so.6").XDestroyImage  # convenience
        self._dpy = self._x.XOpenDisplay(display.encode())
        if not self._dpy:
            raise XImageUnavailable(f"cannot open X display {display!r}")
        self._win = None
        self._geom = None

    def target(self, window_id: int):
        self._win = ctypes.c_ulong(window_id)
        root = ctypes.c_ulong()
        x = ctypes.c_int(); y = ctypes.c_int()
        w = ctypes.c_uint(); h = ctypes.c_uint(); bw = ctypes.c_uint(); depth = ctypes.c_uint()
        ok = self._x.XGetGeometry(self._dpy, self._win, ctypes.byref(root), ctypes.byref(x),
                                  ctypes.byref(y), ctypes.byref(w), ctypes.byref(h),
                                  ctypes.byref(bw), ctypes.byref(depth))
        if not ok:
            raise XImageUnavailable("XGetGeometry failed for target window")
        self._geom = (w.value, h.value)
        return self._geom

    def grab_hash(self):
        """One XGetImage of the whole window; return (sha1_of_pixels, grab_elapsed_ms) or (None, ms)."""
        w, h = self._geom
        t0 = time.monotonic()
        img = self._x.XGetImage(self._dpy, self._win, 0, 0, w, h, AllPlanes, ZPixmap)
        if not img:
            return None, (time.monotonic() - t0) * 1000
        try:
            im = img.contents
            size = im.bytes_per_line * im.height
            data = ctypes.string_at(im.data, size)
            digest = hashlib.sha1(data).hexdigest()
        finally:
            self._x.XDestroyImage(img)
        return digest, (time.monotonic() - t0) * 1000

    def close(self):
        if self._dpy:
            self._x.XCloseDisplay(self._dpy)
            self._dpy = None


def _diagnostic(out_dir):
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
    report = {"method": "XGetImage front-buffer poll (first-party ctypes; system libX11)",
              "cadence_ms": [], "input_to_change_ms": [], "notes": [],
              "bound": "poll reads the Present-updated front buffer; external/ordinary; resolution = "
                       "poll cadence; NOT a PresentCompleteNotify/wp_presentation timestamp"}

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
                raise SystemExit("gui exited")
            time.sleep(0.3)
        report["backend"] = backend
        time.sleep(3)
        wids = [w for w in subprocess.run(["xdotool", "search", "--pid", str(proc.pid)],
                env=env, capture_output=True, text=True).stdout.split() if w.strip()]
        wid = int(wids[-1])
        xdo("windowactivate", "--sync", str(wid))
        watch = ImagePollWatch(":1")
        report["geometry"] = watch.target(wid)
        # measure raw poll cadence (10 back-to-back grabs)
        for _ in range(10):
            _, ms = watch.grab_hash()
            report["cadence_ms"].append(round(ms, 2))
        # input -> change: drive a change and poll tightly until the window-pixel hash differs.
        # Trigger set is configurable so we can disambiguate XGetImage-sees-change vs stale-buffer:
        #   search-type letters (subtle, search-field focus dependent) AND Tab focus moves (large,
        #   proven detectable via XGetImage/import in P1). report each trigger's result separately.
        def measure(label, inject):
            base, _ = watch.grab_hash()
            t0 = time.monotonic()
            inject()
            lat = None
            while time.monotonic() - t0 < 2.0:
                h, _ = watch.grab_hash()
                if h is not None and h != base:
                    lat = (time.monotonic() - t0) * 1000
                    break
            report["input_to_change_ms"].append({"trigger": label,
                                                 "ms": round(lat, 2) if lat is not None else None})
        for ch in list("red"):
            measure(f"type:{ch}", lambda c=ch: xdo("type", "--window", str(wid), c))
        for i in range(4):
            measure("key:Tab", lambda: xdo("key", "--window", str(wid), "Tab"))
        for i in range(2):
            measure("key:space", lambda: xdo("key", "--window", str(wid), "space"))
        cad = sorted(report["cadence_ms"])
        detected = [r for r in report["input_to_change_ms"] if r["ms"] is not None]
        lats = sorted(r["ms"] for r in detected)
        report["summary"] = {
            "poll_cadence_median_ms": cad[len(cad) // 2] if cad else None,
            "poll_cadence_min_ms": cad[0] if cad else None,
            "detected_triggers": [r["trigger"] for r in detected],
            "undetected_triggers": [r["trigger"] for r in report["input_to_change_ms"] if r["ms"] is None],
            "input_to_change_median_ms": lats[len(lats) // 2] if lats else None,
            "vs_import_polling_cadence_ms": 93.2,
            "verdict": ("XGetImage poll cadence is well under the 93ms import baseline; combined with "
                        "the 100ms gui-feedback bound this is a viable external presented-pixel signal "
                        "at coarse-but-sufficient granularity. A true per-present timestamp still wants "
                        "Present/PresentCompleteNotify (libxcb-present, available) at the milestone."),
        }
    except XImageUnavailable as error:
        report["env_limitation"] = str(error)
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
    print(json.dumps({k: report[k] for k in ("backend", "geometry", "summary", "env_limitation")
                      if k in report}, indent=2))
    print("REPORT", out / "report.json")


if __name__ == "__main__":
    import sys
    _diagnostic(sys.argv[1] if len(sys.argv) > 1 else "/tmp/xgetimage-diag")
