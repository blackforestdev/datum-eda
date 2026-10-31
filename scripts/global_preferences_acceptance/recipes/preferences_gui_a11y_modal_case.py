#!/usr/bin/env python3
"""Committed GP-CM05E GUI accessibility-interaction recipe: new-project-modes/gui.

A thin persona over the shared accessibility_capture backbone for the ONE scenario whose surface is the
New Project MODAL (not the Global Preferences dialog): the modal's modes. The modal was previously
silent to the accessibility announcement channel (dat-7ynp); the mv1k fix wired it into the SAME
announce_console/events machinery (modal-open + use-project-name mode-change, polite), and this recipe
is the row that lands on that real behavior.

    python3 .../recipes/preferences_gui_a11y_modal_case.py --fixture-json <frozen fixture JSON>
    DATUM_EDA_GUI / DATUM_EDA_WORK_ROOT  (as for the other GUI a11y recipes)

Coverage row:  ("accessibility-interaction", "new-project-modes", "default", "gui", coordinate)

Drive: Ctrl+N opens the modal (recorder registered first, so the open announcement is captured) -> toggle
"Use project name" (the mode change) -> Tab-walk + Escape. The five matrix assertions are adjudicated
from the three channels; truthful-announcement reads the typed announced strings + priority directly
(4fhu live-AT-SPI ruling), the screen reader + pixels corroborate. READ-ONLY on the Global store (mode
exploration never creates a Project or writes the store): the §5 payload ships an explicit no-write
fingerprint (store absent) so no-write-by-absence is verifiable from the shipped bytes. Honest
observations only; always exits 0; never retries to green.
"""
import argparse
import json
import os
import sys
import time
from pathlib import Path

SCHEMA = "datum.preferences.case-observations.v2"
EXPECTED_HEAD = ("accessibility-interaction", "new-project-modes", "default", "gui")
MODAL_TITLE = "New Project — Datum"
MODAL_NAME = "New Project"
USE_PROJECT_NAME = "Use project name"
POLITE, ASSERTIVE = 1, 2
# Key modal controls the accessibility tree must expose with the correct name, role AND state. Create
# is DISABLED while the form is empty (no project name entered) -- that disabled state is TRUTHFUL and
# part of what correct-name-role-value-state verifies (the control truthfully reports it cannot act yet).
EXPECTED_CONTROLS = {"Project name": ("entry", True), "Use project name": ("check box", True),
                     "Location": ("entry", True), "Create": ("button", False)}


def _require_binary(variable):
    value = os.environ.get(variable)
    if not value or not Path(value).is_file():
        raise SystemExit(f"{variable} must point at the candidate binary for this surface")
    return value


def _work_root():
    provided = os.environ.get("DATUM_EDA_WORK_ROOT")
    if not provided:
        raise SystemExit("DATUM_EDA_WORK_ROOT must be set by the producer")
    return Path(provided)


def _coordinate(fixture):
    coverage = (fixture.get("parameters") or {}).get("coverage")
    if not isinstance(coverage, list) or len(coverage) != 5:
        raise SystemExit("fixture coverage must be a 5-tuple")
    if tuple(coverage[:4]) != EXPECTED_HEAD:
        raise SystemExit(f"recipe bound to {EXPECTED_HEAD}, fixture selects {tuple(coverage[:4])}")
    c = coverage[4]
    if (not isinstance(c, list) or len(c) != 3 or not isinstance(c[0], str)
            or not isinstance(c[1], (int, float)) or not isinstance(c[2], list) or len(c[2]) != 2
            or not all(isinstance(n, int) for n in c[2])):
        raise SystemExit("gui coordinate must be [backend, scale, [width, height]]")
    return c[0], float(c[1]), (c[2][0], c[2][1])


def _anns(events, ASSERTIVE=ASSERTIVE, POLITE=POLITE):
    return [{"text": a.get("text"), "detail1": a.get("detail1"), "detail2": a.get("detail2"),
             "source_role": a.get("source_role"), "t": a.get("t"),
             "priority": ("assertive" if a.get("detail1") == ASSERTIVE
                          else "polite" if a.get("detail1") == POLITE else "other")}
            for a in events]


def observe(fixture):
    backend, scale, window = _coordinate(fixture)
    if backend not in ("wayland", "x11") or float(scale) != 1.0:
        raise SystemExit(f"this envelope supports wayland/1.0 and x11/1.0; got {backend}/{scale}")
    gui_binary = _require_binary("DATUM_EDA_GUI")
    work = _work_root()
    repo = Path(__file__).resolve().parents[3]
    if str(repo / "scripts") not in sys.path:
        sys.path.insert(0, str(repo / "scripts"))
    os.environ["DATUM_QA_BINARY_DIR"] = str(Path(gui_binary).resolve().parent)
    import gi
    gi.require_version("Atspi", "2.0")
    from gi.repository import Atspi
    from prove_native_project_startup_gui import StartupQA
    sys.path.insert(0, str(Path(__file__).resolve().parent))
    import accessibility_capture as ac

    observations = {k: False for k in
                    ("native-at-spi-evidence", "correct-focus", "correct-name-role-value-state",
                     "no-trap", "truthful-announcement")}
    payload = {}

    def focused(qa):
        for n in qa.nodes():
            try:
                if n.get_state_set().contains(Atspi.StateType.FOCUSED):
                    return (n.get_role_name(), n.get_name())
            except Exception:
                continue
        return (None, None)

    def modal_controls(qa):
        rows = []
        for n in qa.nodes():
            try:
                name = n.get_name()
                role = n.get_role_name()
                if name and role in ("entry", "check box", "button", "push button", "toggle button"):
                    states = n.get_state_set()
                    rows.append({"name": name, "role": role,
                                 "description": n.get_description(),
                                 "enabled": states.contains(Atspi.StateType.ENABLED),
                                 "focused": states.contains(Atspi.StateType.FOCUSED)})
            except Exception:
                continue
        return rows

    qa = StartupQA(work)
    for k in ("DATUM_PROJECT_ROOT", "DATUM_DISCOVERY", "DATUM_AGENT_CREDENTIAL_FILE",
              "EDA_ENGINE_SOCKET", "DATUM_ENGINE_SOCKET", "DATUM_MCP_ENDPOINT",
              "DATUM_AGENT_PROVENANCE"):
        qa.env.pop(k, None)
    try:
        qa.start_envelope(output_size=(int(window[0]), int(window[1])), backend=backend)
        main = qa.launch("accessibility-interaction-new-project-modes-gui")
        with ac.accessibility_capture(gui_binary) as cap:
            # recorder registered -> open the modal so the open announcement is captured on ch2.
            i0 = len(cap.recorder.events)
            qa.key(main, "ctrl+n")
            qa.wait("New Project modal open", lambda: qa.window(MODAL_TITLE) is not None)
            cap.pump(0.8)
            open_anns = cap.recorder.events[i0:]
            cap.start_screen_reader(timeout=60)

            def tab_to(predicate, max_tabs=28):
                for _ in range(max_tabs):
                    if predicate(focused(qa)):
                        return True
                    qa.key(main, "Tab")
                    cap.pump(0.3)
                return predicate(focused(qa))

            # --- mode change: focus "Use project name", toggle (Space) -> mode announcement ---
            reached_toggle = tab_to(lambda f: f[1] == USE_PROJECT_NAME)
            toggle_focus = focused(qa)
            i = len(cap.recorder.events)
            qa.key(main, "space")
            cap.pump(1.0)
            mode_anns = cap.recorder.events[i:]

            controls = modal_controls(qa)
            snap = cap.snapshot(qa)
            try:
                qa.capture("accessibility-new-project-modes", main)
            except Exception:
                pass

            # --- no-trap: Tab cycles distinct modal stops AND Escape closes the modal ---
            seen = set()
            focus_walk = []
            for _ in range(10):
                qa.key(main, "Tab")
                cap.pump(0.25)
                f = focused(qa)
                focus_walk.append({"role": f[0], "name": f[1]})
                seen.add(f)
            distinct = len([s for s in seen if s != (None, None)])
            modal_before = qa.window(MODAL_TITLE) is not None
            qa.key(main, "Escape")
            cap.pump(0.5)
            modal_after = qa.window(MODAL_TITLE) is not None
            no_trap = distinct >= 3 and modal_before and not modal_after

            # ---- the five assertions ----
            observations["native-at-spi-evidence"] = bool(controls) and bool(cap.recorder.events)
            observations["correct-focus"] = bool(reached_toggle) and toggle_focus[1] == USE_PROJECT_NAME
            by_name = {c["name"]: c for c in controls}
            observations["correct-name-role-value-state"] = all(
                name in by_name and by_name[name]["role"] == role
                and by_name[name]["enabled"] == enabled
                for name, (role, enabled) in EXPECTED_CONTROLS.items())
            observations["no-trap"] = bool(no_trap)
            open_ok = any("New Project opened" in str(a.get("text", "")) and a.get("detail1") == POLITE
                          for a in open_anns)
            mode_ok = any("use project name" in str(a.get("text", "")).lower()
                          and a.get("detail1") == POLITE for a in mode_anns)
            observations["truthful-announcement"] = bool(open_ok and mode_ok)

            full_payload = {
                "schema": "datum.preferences.accessibility-capture.v1",
                "coverage": "/".join(EXPECTED_HEAD),
                "scenario": "new-project-modes",
                "ch1_tree": {"modal_controls": controls, "focus_walk": focus_walk},
                "ch2_announcements": _anns(cap.recorder.events),
                "ch3_screen_reader": {"ready_latency_s": snap["screen_reader"]["ready_latency_s"],
                                      "spoken_all": snap["channels"]["ch3_spoken_all"],
                                      "spoken_since_drive": snap["channels"]["ch3_spoken_since_drive"]},
                "transitions": {"open": {"announcement": next((a.get("text") for a in open_anns
                                                               if a.get("detail1") == POLITE), None)},
                                "mode_change": {"reached_toggle": reached_toggle,
                                                "announcement": next((a.get("text") for a in mode_anns
                                                                      if a.get("detail1") == POLITE), None)}},
                "no_write_artifact": ac.store_fingerprint(work),
                "observations": observations,
                "provenance": snap["provenance"],
            }
            evdir = work / "accessibility-capture"
            evdir.mkdir(parents=True, exist_ok=True)
            (evdir / "new-project-modes.json").write_text(json.dumps(full_payload, indent=1, default=str))
    finally:
        try:
            qa.stop()
        except Exception:
            pass
        try:
            qa.close_envelope()
        except Exception:
            pass
    return observations


def main_cli(argv=None):
    parser = argparse.ArgumentParser(description="GP-CM05E GUI a11y recipe (new-project-modes/gui)")
    parser.add_argument("--fixture-json", required=True)
    args = parser.parse_args(argv)
    fixture = json.loads(args.fixture_json)
    start = time.monotonic()
    observations = observe(fixture)
    elapsed = time.monotonic() - start
    print(f"Ran {len(observations)} tests in {elapsed:.3f}s", file=sys.stderr)
    print(json.dumps({"schema": SCHEMA, "executed_tests": len(observations),
                      "observations": observations}, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main_cli())
