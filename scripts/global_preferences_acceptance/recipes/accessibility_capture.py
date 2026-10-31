#!/usr/bin/env python3
"""Shared GP-CM05E accessibility-capture backbone (built ONCE, configured per scenario).

The §6-N06 / accessibility-interaction capture capability lives here as ONE shared library the
accessibility recipes import and CONFIGURE (the Lean / shared-by-construction law: the tools change,
the scene doesn't). A recipe drives the GUI scenario; this library captures the three channels while it
does, with deterministic teardown. It never drives the GUI itself and asserts nothing -- it returns
honest captured data for the recipe/validator to adjudicate.

Three channels (F4a-proven: .datum/scratch/frontend-dev-01/f4-screen-reader-capture/F4A-PROBE.md):
  ch1 AT-SPI tree   -- role / name / get_description() (the PROPERTY) / FOCUSED state, read inline
  ch2 announcements -- Atspi.EventListener on object:announcement (detail1: 1=polite, 2=assertive)
  ch3 screen reader -- the pinned Orca -> private speech-dispatcher -> text-logging sink: the STRINGS
                       Orca speaks (NOT audio -- PM ruling (a); labelled as such in provenance)

The three F4a constraints are encoded HERE, not copied per recipe:
  (1) the speech-dispatcher unix socket path must be < 108 bytes -> a SHORT /tmp runtime dir;
  (2) sd_generic initialises audio at startup -> PULSE_SERVER points at the host pipewire-pulse so the
      module starts (headless, no synthesis); isolating only -S/-P keeps that socket reachable;
  (3) full isolation from the host speechd (private -S socket + -P pidfile + private config dir) and
      from the host Orca (--replace with an isolated --user-prefs).

Readiness is EVENT-BASED (PM requirement from the F4a >12s latency datum): wait until Orca's own
"Screen reader on." appears in the sink, generously bounded -- never a short fixed sleep, which would be
a false-OWED machine for N06's per-step screen-reader-spoken-output under ruling (b).

This module has NO heavy import side effects at import time; gi/Atspi is imported lazily inside the
functions that need a live display, so the recipe package imports cleanly in a headless unit check.
"""
import json
import os
import re
import shutil
import subprocess
import tempfile
import time
from contextlib import contextmanager
from pathlib import Path

# Orca's own session boundary markers in the spoken stream (used only to gate readiness/teardown).
SCREEN_READER_ON = "Screen reader on."
SCREEN_READER_OFF = "Screen reader off."
# A record delimiter the sink appends after each spoken message. speech-dispatcher's generic module
# strips backslashes from the synth command before printf sees them (so "\n"/"\x1e" become literal
# "n"/"x1e"), therefore the delimiter is a LITERAL multi-char ASCII marker needing no escape -- one
# that cannot occur in ordinary UI speech -- so records parse unambiguously.
RECORD_DELIM = "@@@GPA11Y_REC@@@"
HOST_PULSE = "unix:/run/user/1000/pulse/native"
MODULE_DIR = "/usr/lib/speech-dispatcher-modules"


def _tool_version(argv):
    try:
        out = subprocess.run(argv, capture_output=True, text=True, timeout=10)
        return (out.stdout or out.stderr).strip().splitlines()[0] if (out.stdout or out.stderr) else None
    except Exception:
        return None


def capture_provenance(gui_binary):
    """Reproducibility + honesty provenance for every row this capture backs. The text-sink labelling
    is binding (PM ruling (a)): channel 3 is the STRINGS the screen reader sends, never audio."""
    import gi
    gi.require_version("Atspi", "2.0")
    from gi.repository import Atspi
    sha = None
    try:
        import hashlib
        sha = hashlib.sha256(Path(gui_binary).read_bytes()).hexdigest()
    except Exception:
        pass
    return {
        "screen_reader": _tool_version(["orca", "--version"]),
        "speech_dispatcher": _tool_version(["speech-dispatcher", "-v"]),
        "atspi": getattr(Atspi, "get_version", lambda: None)() and ".".join(
            map(str, Atspi.get_version())) if hasattr(Atspi, "get_version") else None,
        "xdotool": _tool_version(["xdotool", "--version"]),
        "gui_binary_sha256": sha,
        "spoken_output_kind": "text-strings-sent-to-speech-dispatcher",
        "spoken_output_caveat": ("Channel 3 captures the strings the screen reader (Orca) sends to "
                                 "speech-dispatcher -- its spoken output in harness-bindable form -- "
                                 "NOT a synthesised audio waveform (PM ruling a; disclosed at GP-CM05A)."),
        "external_tools_note": ("Orca + speech-dispatcher are installed system packages DRIVEN via "
                                "subprocess (weston/xdotool precedent); not vendored/linked (PM029)."),
    }


class SpeechSink:
    """A private speech-dispatcher whose only output module logs each spoken message to a file (no
    audio synthesis). Encodes constraints (1)-(3). Context manager; deterministic stop()."""

    def __init__(self):
        # Constraint (1): a SHORT /tmp runtime so the unix socket path stays < 108 bytes.
        self.runtime = Path(tempfile.mkdtemp(prefix="gpa11y-", dir="/tmp"))
        self.sock = self.runtime / "sd.sock"
        self.pidfile = self.runtime / "sd.pid"
        self.spoken = self.runtime / "spoken.rec"
        self.proc = None

    def __enter__(self):
        cfg = self.runtime / "cfg"
        (cfg / "modules").mkdir(parents=True)
        self.spoken.write_text("")
        # Each spoken message -> $DATA followed by our literal ASCII marker (no backslash -- the module
        # strips those), appended to the capture file. Parsing splits on the marker, robust to whatever
        # the message text contains and to printf's escape handling.
        (cfg / "modules" / "logsink.conf").write_text(
            f"GenericExecuteSynth \"printf '%s{RECORD_DELIM}' \\'$DATA\\' >> {self.spoken}\"\n"
            'AddVoice "en" "MALE1" "none"\nDefaultVoice "MALE1"\n')
        (cfg / "speechd.conf").write_text(
            'LogLevel 4\nAddModule "logsink" "sd_generic" "logsink.conf"\n'
            'DefaultModule "logsink"\nDefaultLanguage "en"\nDefaultVoiceType "MALE1"\n')
        env = dict(os.environ, PULSE_SERVER=HOST_PULSE)  # constraint (2)
        self.proc = subprocess.Popen(
            ["speech-dispatcher", "-s", "-C", str(cfg), "-m", MODULE_DIR, "-l", "4",
             "-L", str(self.runtime / "log"), "-S", str(self.sock), "-P", str(self.pidfile)],
            env=env, stdout=open(self.runtime / "speechd.out", "w"), stderr=subprocess.STDOUT)
        for _ in range(50):
            if self.sock.exists():
                break
            time.sleep(0.2)
        return self

    @property
    def address(self):
        return f"unix_socket:{self.sock}"

    def records(self):
        """The ordered list of spoken strings captured so far (split on the RS delimiter)."""
        try:
            raw = self.spoken.read_text()
        except OSError:
            return []
        return [r for r in raw.split(RECORD_DELIM) if r != ""]

    def selftest(self, timeout=15):
        """Route one string through the sink directly (no Orca) to prove it captures before Orca runs."""
        marker = "GPA11Y_SINK_SELFTEST"
        env = dict(os.environ, SPEECHD_ADDRESS=self.address)
        try:
            subprocess.run(["spd-say", "-w", "-o", "logsink", marker], env=env, timeout=timeout,
                           stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        except Exception:
            return False
        deadline = time.time() + 3
        while time.time() < deadline:
            if any(marker in r for r in self.records()):
                return True
            time.sleep(0.2)
        return False

    def __exit__(self, *exc):
        self.stop()

    def stop(self):
        if self.proc:
            try:
                self.proc.terminate(); self.proc.wait(timeout=5)
            except Exception:
                try:
                    self.proc.kill()
                except Exception:
                    pass
            self.proc = None
        shutil.rmtree(self.runtime, ignore_errors=True)


class ScreenReader:
    """The pinned Orca, pointed at a SpeechSink, with an isolated prefs dir and host Orca --replace.
    Readiness is EVENT-BASED: wait for Orca's "Screen reader on." in the sink (generously bounded)."""

    def __init__(self, sink, *, user_prefs=None):
        self.sink = sink
        self.user_prefs = Path(user_prefs) if user_prefs else (sink.runtime / "orca-prefs")
        self.proc = None
        self.ready_latency_s = None

    def __enter__(self):
        self.user_prefs.mkdir(parents=True, exist_ok=True)
        env = dict(os.environ, SPEECHD_ADDRESS=self.sink.address)
        started = time.time()
        self.proc = subprocess.Popen(
            ["orca", "--replace", "--user-prefs", str(self.user_prefs),
             "--debug-file", str(self.sink.runtime / "orca-debug.out")],
            env=env, stdout=open(self.sink.runtime / "orca.out", "w"), stderr=subprocess.STDOUT)
        self._started = started
        return self

    def wait_ready(self, pump, *, timeout=60):
        """Block until Orca's "Screen reader on." appears in the sink, pumping the GLib loop via the
        supplied `pump(seconds)` callable. Generous bound (the F4a datum was >12s); returns True on
        ready. NEVER a short fixed sleep -- a premature gate would falsely OWE spoken steps (ruling b)."""
        deadline = time.time() + timeout
        while time.time() < deadline:
            pump(0.3)
            if any(SCREEN_READER_ON in r for r in self.sink.records()):
                self.ready_latency_s = time.time() - self._started
                return True
        return False

    def __exit__(self, *exc):
        self.stop()

    def stop(self):
        if self.proc:
            try:
                self.proc.terminate(); self.proc.wait(timeout=6)
            except Exception:
                try:
                    self.proc.kill()
                except Exception:
                    pass
            self.proc = None


class AnnouncementRecorder:
    """Channel 2: record object:announcement events (text + priority) over a live AT-SPI connection."""

    def __init__(self):
        self.events = []
        self._listener = None

    def __enter__(self):
        import gi
        gi.require_version("Atspi", "2.0")
        from gi.repository import Atspi
        self._listener = Atspi.EventListener.new(self._on)
        self._listener.register("object:announcement")
        return self

    def _on(self, event):
        try:
            text = getattr(event, "any_data", None)
            self.events.append({"text": str(text) if text is not None else None,
                                "detail1": event.detail1, "detail2": event.detail2,
                                "source_role": event.source.get_role_name() if event.source else None,
                                "t": time.time()})
        except Exception as e:
            self.events.append({"capture_error": str(e), "t": time.time()})

    def __exit__(self, *exc):
        if self._listener:
            try:
                self._listener.deregister("object:announcement")
            except Exception:
                pass


def pump_factory():
    """A GLib main-context pump bound to the default context, for draining AT-SPI events in-line."""
    from gi.repository import GLib
    ctx = GLib.MainContext.default()

    def pump(seconds):
        end = time.time() + seconds
        while time.time() < end:
            while ctx.pending():
                ctx.iteration(False)
            time.sleep(0.05)
    return pump


# --- channel 1 tree helpers (read INLINE; never hold a node ref -- immediate-mode rebuilds stale it) ---
VALUE_ROLES = ("combo box", "toggle button", "spin button", "slider", "check box", "radio button")


def focused_name(qa):
    from gi.repository import Atspi
    for n in qa.nodes():
        try:
            if n.get_state_set().contains(Atspi.StateType.FOCUSED):
                return n.get_name()
        except Exception:
            continue
    return None


def control_states(qa):
    """Every value-control's (name, role, description, enabled, read-only) read inline from the tree."""
    from gi.repository import Atspi
    rows = []
    for n in qa.nodes():
        try:
            if n.get_role_name() in VALUE_ROLES:
                states = n.get_state_set()
                rows.append({
                    "name": n.get_name(),
                    "role": n.get_role_name(),
                    "description": n.get_description(),
                    "enabled": states.contains(Atspi.StateType.ENABLED),
                    "read_only": states.contains(Atspi.StateType.READ_ONLY),
                    "focused": states.contains(Atspi.StateType.FOCUSED),
                })
        except Exception:
            continue
    return rows


def store_fingerprint(work_root):
    """An independently-confirmable NO-WRITE artifact for a read-only row: the Global Preferences
    store's on-disk inventory under <work_root>/config/datum/preferences -- its generation list and
    every file's relative path + sha256 -- or an explicit absent-store marker when the store was never
    created. A read-only row ships this in its §5 payload so no-write-by-absence is verifiable from the
    shipped bytes (parity with a write row's g0/g1 generation chain), not merely asserted by absence."""
    import hashlib
    repo = Path(work_root) / "config" / "datum" / "preferences"
    if not repo.exists():
        return {"store_present": False, "generations": [], "files": [], "no_write": True}
    files = []
    for path in sorted(repo.rglob("*")):
        if path.is_file() and not path.is_symlink():
            files.append({"path": path.relative_to(repo).as_posix(),
                          "sha256": hashlib.sha256(path.read_bytes()).hexdigest()})
    gens = (sorted(d.name for d in (repo / "generations").iterdir())
            if (repo / "generations").is_dir() else [])
    return {"store_present": True, "generations": gens, "files": files, "no_write": len(gens) == 0}


def status_node(qa):
    """The persistent Status node (role=status bar, name 'Global Preferences status') whose Description
    carries the typed preference-status announcement (ANNOUNCEMENT-EVENT-DETERMINATION: the second live
    AT-SPI channel beside the events). Returns {name, role, description} or None."""
    for n in qa.nodes():
        try:
            if n.get_role_name() == "status bar" and "Global Preferences status" in (n.get_name() or ""):
                return {"name": n.get_name(), "role": n.get_role_name(),
                        "description": n.get_description()}
        except Exception:
            continue
    return None


@contextmanager
def accessibility_capture(gui_binary, *, orca_ready_timeout=60, run_selftest=True):
    """Compose the three channels with deterministic teardown. Usage (the recipe configures the scene):

        with accessibility_capture(gui_binary) as cap:
            cap.start_screen_reader()                # after the GUI/dialog is up on the AT-SPI bus
            ...recipe drives the scenario, calling cap.pump(t) to drain events...
            data = cap.snapshot(qa)                  # ch1 tree + ch2 events + ch3 spoken, + provenance

    Everything (sink, Orca, listener) is torn down on exit, Orca and speechd included, host untouched.
    """
    sink = SpeechSink().__enter__()
    recorder = AnnouncementRecorder().__enter__()
    pump = pump_factory()
    reader = ScreenReader(sink)
    cap = _Capture(gui_binary, sink, recorder, reader, pump)
    if run_selftest:
        cap.sink_selftest_ok = sink.selftest()
    try:
        yield cap
    finally:
        try:
            reader.stop()
        except Exception:
            pass
        try:
            recorder.__exit__(None, None, None)
        except Exception:
            pass
        try:
            sink.stop()
        except Exception:
            pass


class _Capture:
    def __init__(self, gui_binary, sink, recorder, reader, pump):
        self.gui_binary = gui_binary
        self.sink = sink
        self.recorder = recorder
        self.reader = reader
        self.pump = pump
        self.sink_selftest_ok = None
        self._orca_ready = None
        self._baseline_spoken = None

    def start_screen_reader(self, *, timeout=60):
        """Start Orca and EVENT-GATE on its "Screen reader on." in the sink. Records the latency and
        the spoken baseline so later snapshots attribute only NEW speech to the driven scenario."""
        self.reader.__enter__()
        self._orca_ready = self.reader.wait_ready(self.pump, timeout=timeout)
        self._baseline_spoken = self.sink.records()
        return self._orca_ready

    def spoken_since_baseline(self):
        base = len(self._baseline_spoken or [])
        return self.sink.records()[base:]

    def snapshot(self, qa):
        """Return the composed honest capture (no assertions) -- the F4-DESIGN §5 PRIMARY typed payload:
        the ordered ch2 event list (text + detail1 priority + source), the Status-node description, the
        ch1 control name/role/description/state table + focus-walk, and the ch3 Orca spoken strings. The
        recipe writes this into the row's after-state (content-addressed, bound); the validator adjudicates."""
        spoken = self.sink.records()
        return {
            "channels": {
                "ch1_controls": control_states(qa),
                "ch1_focused": focused_name(qa),
                "ch1_status_node": status_node(qa),
                "ch2_announcements": list(self.recorder.events),
                "ch3_spoken_all": spoken,
                "ch3_spoken_since_drive": self.spoken_since_baseline(),
            },
            "screen_reader": {
                "started": self.reader.proc is not None,
                "ready": bool(self._orca_ready),
                "ready_latency_s": self.reader.ready_latency_s,
                "sink_selftest_ok": self.sink_selftest_ok,
            },
            "provenance": capture_provenance(self.gui_binary),
        }
