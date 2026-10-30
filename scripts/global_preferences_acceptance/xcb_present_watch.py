#!/usr/bin/env python3
"""First-party X Present PresentCompleteNotify watch (GP-CM05E S4 capture tooling).

The ideal external presented-frame acknowledgement: the X server's Present extension emits a
PresentCompleteNotify when a present (GPU buffer flip) completes, carrying a UST (microsecond)
present-completion timestamp. datum-gui presents via DRI3/Present (which is exactly why core
XDamage sees nothing), so PresentCompleteNotify is the correct, external, ordinary signal — a true
present timestamp, not a poll or a submission log.

PM029 / first-party: binds the SYSTEM libxcb + libxcb-present via ctypes only (both present on this
host; libXpresent is absent, so we go through xcb). Nothing fetched/installed; xcb/present.h is not
installed, so the Present-protocol ABI is reconstructed here from the protocol spec. A missing
library/extension is a recorded env limitation. Bounded diagnostic only.

Honest bounds: UST clock domain is driver-dependent (commonly CLOCK_MONOTONIC microseconds under
DRM); this records BOTH the UST and the local monotonic receipt time, and notes the caveat. This is
a present-COMPLETION signal (server/compositor attests the flip), the strongest external ordinary
presented-ack available without touching the product.
"""
import ctypes
import select
import time

XCB_GE_GENERIC = 35
XCB_PRESENT_COMPLETE_NOTIFY = 1
XCB_PRESENT_EVENT_MASK_COMPLETE_NOTIFY = 2


class XcbPresentUnavailable(RuntimeError):
    pass


class PresentWatch:
    """Receive PresentCompleteNotify events for a window via libxcb + libxcb-present."""

    def __init__(self, display: str):
        try:
            self._xcb = ctypes.CDLL("libxcb.so.1")
            self._xp = ctypes.CDLL("libxcb-present.so.0")
            self._libc = ctypes.CDLL("libc.so.6")
        except OSError as error:
            raise XcbPresentUnavailable(f"system xcb libraries unavailable: {error}") from error
        self._bind()
        screen = ctypes.c_int()
        self._conn = self._xcb.xcb_connect(display.encode(), ctypes.byref(screen))
        if not self._conn or self._xcb.xcb_connection_has_error(self._conn):
            raise XcbPresentUnavailable(f"xcb_connect failed for {display!r}")
        # present major opcode via xcb_get_extension_data(conn, &xcb_present_id)
        ext_addr = ctypes.addressof(ctypes.c_char.in_dll(self._xp, "xcb_present_id"))
        reply = self._xcb.xcb_get_extension_data(self._conn, ctypes.c_void_p(ext_addr))
        if not reply:
            raise XcbPresentUnavailable("Present extension not available on this server")
        # xcb_query_extension_reply_t: response_type u8, pad0 u8, sequence u16, length u32,
        # present u8 @8, major_opcode u8 @9, first_event u8 @10, first_error u8 @11
        buf = (ctypes.c_uint8 * 12).from_address(reply)
        if not buf[8]:  # 'present' flag
            raise XcbPresentUnavailable("server reports Present extension absent")
        self._major = buf[9]
        self._fd = self._xcb.xcb_get_file_descriptor(self._conn)

    def _bind(self):
        P = ctypes.c_void_p
        self._xcb.xcb_connect.restype = P
        self._xcb.xcb_connect.argtypes = [ctypes.c_char_p, ctypes.POINTER(ctypes.c_int)]
        self._xcb.xcb_connection_has_error.restype = ctypes.c_int
        self._xcb.xcb_connection_has_error.argtypes = [P]
        self._xcb.xcb_generate_id.restype = ctypes.c_uint32
        self._xcb.xcb_generate_id.argtypes = [P]
        self._xcb.xcb_flush.restype = ctypes.c_int
        self._xcb.xcb_flush.argtypes = [P]
        self._xcb.xcb_get_file_descriptor.restype = ctypes.c_int
        self._xcb.xcb_get_file_descriptor.argtypes = [P]
        self._xcb.xcb_get_extension_data.restype = P
        self._xcb.xcb_get_extension_data.argtypes = [P, P]
        self._xcb.xcb_poll_for_event.restype = P
        self._xcb.xcb_poll_for_event.argtypes = [P]
        self._xcb.xcb_disconnect.argtypes = [P]
        self._xp.xcb_present_select_input.restype = ctypes.c_uint  # void cookie {uint sequence}
        self._xp.xcb_present_select_input.argtypes = [P, ctypes.c_uint32, ctypes.c_uint32, ctypes.c_uint32]
        self._libc.free.argtypes = [P]

    def select_window(self, window_id: int):
        eid = self._xcb.xcb_generate_id(self._conn)
        self._xp.xcb_present_select_input(self._conn, eid, ctypes.c_uint32(window_id),
                                          ctypes.c_uint32(XCB_PRESENT_EVENT_MASK_COMPLETE_NOTIFY))
        self._xcb.xcb_flush(self._conn)
        self._eid = eid

    def _is_complete_notify(self, ptr):
        b = (ctypes.c_uint8 * 10).from_address(ptr)
        if (b[0] & 0x7F) != XCB_GE_GENERIC or b[1] != self._major:
            return False
        event_type = ctypes.c_uint16.from_address(ptr + 8).value
        return event_type == XCB_PRESENT_COMPLETE_NOTIFY

    def _ust_msc(self, ptr):
        # layout after the 10-byte GE+kind/mode-less header: event@12, window@16, serial@20,
        # ust(u64)@24, msc(u64)@32
        ust = ctypes.c_uint64.from_address(ptr + 24).value
        msc = ctypes.c_uint64.from_address(ptr + 32).value
        return ust, msc

    def drain(self):
        while True:
            ev = self._xcb.xcb_poll_for_event(self._conn)
            if not ev:
                return
            self._libc.free(ev)

    def wait_present(self, timeout_s: float):
        """Return (recv_monotonic_ns, ust_us, msc) of the next PresentCompleteNotify, or None."""
        deadline = time.monotonic() + timeout_s
        while True:
            ev = self._xcb.xcb_poll_for_event(self._conn)
            if ev:
                try:
                    if self._is_complete_notify(ev):
                        now = time.monotonic_ns()
                        ust, msc = self._ust_msc(ev)
                        return now, ust, msc
                finally:
                    self._libc.free(ev)
                continue
            remaining = deadline - time.monotonic()
            if remaining <= 0:
                return None
            select.select([self._fd], [], [], remaining)

    def close(self):
        if self._conn:
            self._xcb.xcb_disconnect(self._conn)
            self._conn = None


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
    report = {"method": "X Present PresentCompleteNotify via libxcb + libxcb-present (first-party ctypes)",
              "present_major_opcode": None, "idle_presents_in_1s": None,
              "input_to_present_ms": [], "notes": [],
              "bound": "present-COMPLETION signal (server attests flip); UST clock domain driver-"
                       "dependent (recorded alongside local monotonic receipt); true external presented-ack"}

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

        watch = PresentWatch(":1")
        report["present_major_opcode"] = watch._major
        watch.select_window(wid)
        # 1) idle present rate: how many PresentCompleteNotify in 1s with no input?
        time.sleep(0.3)
        watch.drain()
        t_end = time.monotonic() + 1.0
        idle = 0
        while time.monotonic() < t_end:
            r = watch.wait_present(timeout_s=max(0, t_end - time.monotonic()))
            if r is not None:
                idle += 1
        report["idle_presents_in_1s"] = idle
        # 2) input -> present: Tab (forces a redraw+present), measure to next completion
        for _ in range(6):
            watch.drain()
            t0 = time.monotonic_ns()
            xdo("key", "--window", str(wid), "Tab")
            r = watch.wait_present(timeout_s=2.0)
            if r is None:
                report["input_to_present_ms"].append(None)
            else:
                recv_ns, ust_us, msc = r
                report["input_to_present_ms"].append({
                    "recv_latency_ms": round((recv_ns - t0) / 1e6, 2),
                    "ust_minus_t0_ms": round((ust_us * 1000 - t0) / 1e6, 2),  # valid only if UST==CLOCK_MONOTONIC
                    "msc": msc})
        got = [r for r in report["input_to_present_ms"] if isinstance(r, dict)]
        if got:
            recvs = sorted(r["recv_latency_ms"] for r in got)
            usts = [r["ust_minus_t0_ms"] for r in got]
            report["summary"] = {
                "n": len(got), "recv_latency_median_ms": recvs[len(recvs) // 2],
                "recv_latency_min_ms": recvs[0],
                "ust_minus_t0_sample_ms": usts[:6],
                "ust_clock_domain_matches_monotonic": all(-50 <= u <= 500 for u in usts),
                "verdict": ("PresentCompleteNotify delivers a true per-present completion signal "
                            "externally; ms-grade; retires the gui-feedback presented-ack gap if "
                            "UST aligns with CLOCK_MONOTONIC (checked above), else use recv time.")}
    except XcbPresentUnavailable as error:
        report["env_limitation"] = str(error)
    except Exception as error:  # preserve + diagnose; do not retry-to-green
        import traceback
        report["error"] = repr(error)
        report["traceback"] = traceback.format_exc()
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
    print(json.dumps({k: report[k] for k in ("backend", "present_major_opcode", "idle_presents_in_1s",
                      "summary", "env_limitation", "error") if k in report}, indent=2))
    print("REPORT", out / "report.json")


if __name__ == "__main__":
    import sys
    _diagnostic(sys.argv[1] if len(sys.argv) > 1 else "/tmp/xcb-present-diag")
