#!/usr/bin/env python3
"""Native readiness-waiting and screenshot capture for StartupQA (dat-xjjn #1 + k6pr de-flake).

This owns one coherent concern extracted from prove_native_project_startup_gui.py under PM022: the
bounded readiness poll (with the extend-while-progress launch deadline and its hard ceiling), the
presentation barrier, focus, and the double-read screenshot-stability admit. It is a mixin configured by
StartupQA, which supplies the process/compositor/env/accessibility state (self.process, self.compositor,
self.env, self.actions, self.observations, self.capture_counts, self.root, self.command, self.nodes,
self.attributes) that these methods read -- the methods are owned here, the harness state stays there.
"""
import os
import subprocess
import time
from pathlib import Path

from gi.repository import GLib

from native_capture_pixels import require_native_capture_pixels
from native_capture_presentation import PresentationReadiness
from run_native_qa_isolated_input import focus_arguments

# dat-xjjn mechanism #1: the native GUI launch ("window visible") first-paint latency scales with host
# load. Under real concurrent-build memory+IO pressure a fixed 20s launch deadline starved while the GUI
# was still making progress (surface-identity emitted, redraw ledgers flowing) and would have painted.
# The launch wait now EXTENDS WHILE PROGRESS continues, capped by a hard ceiling, and still fails loudly:
# a genuine hang emits no new progress signal, so it trips the stall budget fast-ish -- never an
# unbounded wait or an unconditional long sleep.
# PROVENANCE (PM051, ratified): the discriminator proved this is just-slow, not a hang. Under every
# reproducible load (quiet 0.4s; CPU saturation @loadavg 6.5 -> 2.6s; a cold 8-rustc guarded release
# build @loadavg 4.5 -> 1.0s) first paint stayed under 3s with progress flowing; the >20s starvations
# were a transient real-build spike that controlled steady load could not reproduce. So the STALL budget
# is the real hang discriminator (20s with NO progress vs measured inter-signal gaps <0.6s = ~30x margin,
# so it never false-positives a live launch), and the ceiling is a deliberate BACKSTOP -- NOT the ~2.6s
# measured worst-case, because a tight ceiling would re-truncate the very spikes this fix exists to
# survive; 120s is ~6x the observed 20s failure floor and bounded so a true runaway still fails loudly
# within 2min. Full derivation + both probes:
# .datum/scratch/frontend-dev-01/xjjn-diagnosis/CEILING-PROVENANCE.md.
LAUNCH_PROGRESS_STALL_SECONDS = 20    # no new startup progress for this long => genuine hang (loud fail)
LAUNCH_VISIBLE_CEILING_SECONDS = 120  # absolute hard backstop on launch->visible (bounded loud fail)


class NativeReadinessCapture:
    """Readiness-wait + capture methods mixed into StartupQA (see the module docstring)."""

    def wait(self, label, predicate, timeout=20, *, progress=None, ceiling=None):
        """Poll predicate until it is truthy. With progress+ceiling the deadline EXTENDS WHILE PROGRESS
        continues (dat-xjjn #1): `timeout` becomes the no-progress STALL budget (renewed whenever the
        `progress` token advances), capped by a hard `ceiling` -- so a genuinely progressing-but-slow
        launch is admitted while a true hang (no new progress signal) still fails fast-ish and a runaway
        never waits past the ceiling. Without progress/ceiling the behaviour is unchanged."""
        start = time.monotonic()
        stall_deadline = start + timeout
        hard_deadline = start + (ceiling if ceiling is not None else timeout)
        last = None
        last_progress = progress() if progress is not None else None
        while True:
            if self.compositor and self.compositor.poll() is not None:
                raise RuntimeError(f"owned compositor exited: {label}")
            if self.process and self.process.poll() is not None:
                raise RuntimeError(f"App exited {self.process.returncode}: {label}")
            try:
                value = predicate()
                if value:
                    return value
            except (RuntimeError, GLib.Error) as error:
                last = str(error)
            now = time.monotonic()
            if progress is not None:
                token = progress()
                if token != last_progress:
                    last_progress = token
                    stall_deadline = min(now + timeout, hard_deadline)  # renew, never past the ceiling
            if now >= hard_deadline:
                if progress is not None:
                    raise RuntimeError(f"timed out at hard ceiling {ceiling}s (dat-xjjn launch deadline): "
                                       f"{label}; last={last}")
                raise RuntimeError(f"timed out: {label}; last={last}")
            if now >= stall_deadline:
                if progress is not None:
                    raise RuntimeError(f"timed out: no startup progress for {timeout}s -- genuine hang, "
                                       f"not just-slow (dat-xjjn): {label}; last={last}")
                raise RuntimeError(f"timed out: {label}; last={last}")
            time.sleep(0.1)

    def focus(self, window):
        assert self.compositor.poll() is None and window == self.host_window
        self.command(*focus_arguments(window, os.environ))

    def wait_presented(self, after_damage=None, *, require_stable=True, frame_requirement=None):
        readiness = PresentationReadiness(after_damage, require_stable=require_stable,
                                          frame_requirement=frame_requirement)
        log = Path(self.env["DATUM_GUI_LOG"])
        def current_frame():
            return readiness(log.read_text(), owned_dialog=any(
                n.get_role_name() == "dialog" for n in self.nodes()))
        receipt = self.wait("current dialog/Main frame presented before native capture", current_frame)
        self.actions.append({"capture_presentation_barrier": receipt, "require_stable": require_stable})
        return receipt

    def _screenshoot_read(self, directory):
        """One weston-screenshooter read of the private Wayland compositor's presented output (never its
        potentially stale X11 host backing pixmap); return the single PNG's bytes. Each read uses a fresh
        directory because weston-screenshooter filenames have only second precision."""
        directory.mkdir()
        result = subprocess.run(["weston-screenshooter"], cwd=directory,
                                env=dict(self.env, XDG_PICTURES_DIR=str(directory)),
                                text=True, capture_output=True, timeout=10)
        self.actions.append({"argv": ["weston-screenshooter"], "cwd": str(directory),
                             "private_wayland_display": self.env["WAYLAND_DISPLAY"],
                             "exit": result.returncode, "stdout": result.stdout,
                             "stderr": result.stderr})
        images = list(directory.glob("*.png"))
        assert result.returncode == 0 and not result.stderr and len(images) == 1, (
            f"native capture failed: {result.stderr}; images={images}")
        return images[0].read_bytes()

    def capture(self, label, window, *, require_stable=True, frame_requirement=None):
        self.focus(window)
        receipt = self.wait_presented(require_stable=require_stable, frame_requirement=frame_requirement)
        count = self.capture_counts.get(label, 0) + 1
        self.capture_counts[label] = count
        if count > 1:
            label = f"{label}-{count}"
        # Double-read de-flake (dat-native-empty-startup-black-capture-k6pr). weston-screenshooter can
        # read the compositor mid-recomposite and catch a partial/stale frame (a rare transient with no
        # loggable signal -- the first log-fence attempt was unworkable because gpu completion of the
        # last idle frame is never recorded). A SETTLED frame is byte-stable (triage: three reads were
        # byte-identical), so require TWO CONSECUTIVE byte-identical reads with frame-ish spacing; bound
        # the retries, then fail LOUDLY -- never silently accept a one-off bad read as capture evidence.
        max_reads = 4  # the first read + up to 3 retries; two consecutive byte-identical reads admit
        previous = None
        for attempt in range(max_reads):
            shot = self._screenshoot_read(self.root / f"{label}-native-capture-{attempt}")
            if previous is not None and shot == previous:
                output = self.root / f"{label}.png"
                output.write_bytes(shot)
                require_native_capture_pixels(output)
                self.actions.append({"capture_stable_after_reads": attempt + 1})
                self.observations[label] = [
                    {"name": node.get_name(), "attributes": self.attributes(node)}
                    for node in self.nodes() if node.get_role_name() != "terminal"
                ]
                return receipt
            previous = shot
            if attempt < max_reads - 1:
                time.sleep(0.12)  # frame-ish spacing between reads
        raise AssertionError(
            f"native capture never produced two consecutive byte-identical weston-screenshooter reads "
            f"for {label!r} after {max_reads} reads -- a partial/mid-recomposite readback could not be "
            f"excluded (dat-native-empty-startup-black-capture-k6pr de-flake refusing unstable evidence)")
