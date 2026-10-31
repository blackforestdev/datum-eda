#!/usr/bin/env python3
"""Committed GP-CM05E GUI accessibility-interaction case recipe (set-reset-refusal/gui anchor).

The accessibility-interaction family's capture capability is the SHARED backbone
``recipes/accessibility_capture.py`` (built once, configured per scenario -- Lean law); this recipe is
the thin set-reset-refusal persona over it. It drives the real Global Preferences dialog through genuine
OS input and observes the five governed assertions from the three captured channels, then always exits 0
with honest observations (the validator, not the recipe, adjudicates; never retries to green).

    python3 scripts/global_preferences_acceptance/recipes/preferences_gui_a11y_case.py \
        --fixture-json <canonical frozen fixture JSON>

    DATUM_EDA_GUI         candidate datum-gui (gui surface)
    DATUM_EDA_WORK_ROOT   producer work root; the GUI's SET/RESET commit under <work_root>/config and
                          the producer captures the real post-run store (write family: after != before).

Coverage row:  ("accessibility-interaction", "set-reset-refusal", "default", "gui", coordinate)

Scenario drive (discovered + confirmed in F4c-discovery, evidence in
.datum/scratch/frontend-dev-01/f4-screen-reader-capture/):
  SET      Tab to the "Reduced motion" TOGGLE BUTTON control (role-matched, so it is the control hit
           target, not the setting-name/explanation target) -> Return -> commits generation 0, polite
           announcement "Reduced motion changed for this device." (detail1=1).
  RESET    Return again on the control -> a second committed change (polite).
  REFUSAL  hold the store's writer.lock (flock) and Return again -> the write is refused with an
           ASSERTIVE announcement "writer lease is unavailable" (detail1=2) and NO new generation.

Five assertions (inventory.py: ['native-at-spi-evidence','correct-focus','correct-name-role-value-state',
'no-trap','truthful-announcement']). truthful-announcement reads the typed announced string + priority
DIRECTLY from the AT-SPI event channel (the 4fhu ruling: the refusal is on live AT-SPI, not pixel-only);
the screen-reader spoken rendering (ch3) and pixels corroborate. Each spoken transition is recorded with
its PASS/OWED status per PM ruling (b) -- captured here as corroboration; it is the N06 native-observation
row (not this case row's five) whose screen-reader-spoken-output is credited per-step.
"""
import argparse
import fcntl
import json
import os
import sys
import time
from pathlib import Path

SCHEMA = "datum.preferences.case-observations.v2"
EXPECTED_HEAD = ("accessibility-interaction", "set-reset-refusal", "default", "gui")
REDUCED_MOTION = "Reduced motion"
TOGGLE_ROLE = "toggle button"
POLITE, ASSERTIVE = 1, 2


def _require_binary(variable):
    value = os.environ.get(variable)
    if not value or not Path(value).is_file():
        raise SystemExit(f"{variable} must point at the candidate binary for this surface")
    return value


def _work_root():
    provided = os.environ.get("DATUM_EDA_WORK_ROOT")
    if not provided:
        raise SystemExit("DATUM_EDA_WORK_ROOT must be set by the producer for a GUI write family")
    return Path(provided)


def _coordinate(fixture):
    coverage = (fixture.get("parameters") or {}).get("coverage")
    if not isinstance(coverage, list) or len(coverage) != 5:
        raise SystemExit("fixture coverage must be a 5-tuple [case, variant, subcase, surface, coordinate]")
    head = tuple(coverage[:4])
    if head != EXPECTED_HEAD:
        raise SystemExit(f"recipe bound to {EXPECTED_HEAD}, fixture selects {head}")
    c = coverage[4]
    if (not isinstance(c, list) or len(c) != 3 or not isinstance(c[0], str)
            or not isinstance(c[1], (int, float)) or not isinstance(c[2], list) or len(c[2]) != 2
            or not all(isinstance(n, int) for n in c[2])):
        raise SystemExit("gui coordinate must be [backend, scale, [width, height]]")
    return c[0], float(c[1]), (c[2][0], c[2][1])


def observe(fixture):
    backend, scale, window = _coordinate(fixture)
    if backend not in ("wayland", "x11") or float(scale) != 1.0:
        raise SystemExit(f"this capture envelope supports wayland/1.0 and x11/1.0 (scale 2.0 pending its "
                         f"own feasibility probe); got {backend}/{scale}")
    gui_binary = _require_binary("DATUM_EDA_GUI")
    work = _work_root()

    repo = Path(__file__).resolve().parents[3]
    scripts_dir = str(repo / "scripts")
    if scripts_dir not in sys.path:
        sys.path.insert(0, scripts_dir)
    os.environ["DATUM_QA_BINARY_DIR"] = str(Path(gui_binary).resolve().parent)
    import gi
    gi.require_version("Atspi", "2.0")
    from gi.repository import Atspi
    from prove_native_project_startup_gui import StartupQA
    sys.path.insert(0, str(Path(__file__).resolve().parent))
    import accessibility_capture as ac

    def focused(qa):
        for n in qa.nodes():
            try:
                if n.get_state_set().contains(Atspi.StateType.FOCUSED):
                    return n.get_role_name(), n.get_name()
            except Exception:
                continue
        return None, None

    def tab_to(qa, win, cap, role, name, max_tabs=24):
        """Tab until the FOCUSED node is exactly (role, name) -- lands on the control hit target, not
        the same-named setting-name/explanation target. Returns True when focused."""
        for _ in range(max_tabs):
            if focused(qa) == (role, name):
                return True
            qa.key(win, "Tab")
            cap.pump(0.4)
        return focused(qa) == (role, name)

    observations = {k: False for k in
                    ("native-at-spi-evidence", "correct-focus", "correct-name-role-value-state",
                     "no-trap", "truthful-announcement")}

    qa = StartupQA(work)
    for k in ("DATUM_PROJECT_ROOT", "DATUM_DISCOVERY", "DATUM_AGENT_CREDENTIAL_FILE",
              "EDA_ENGINE_SOCKET", "DATUM_ENGINE_SOCKET", "DATUM_MCP_ENDPOINT",
              "DATUM_AGENT_PROVENANCE"):
        qa.env.pop(k, None)
    try:
        qa.start_envelope(output_size=(int(window[0]), int(window[1])), backend=backend)
        main = qa.launch("accessibility-interaction-set-reset-refusal-gui")
        qa.menu_item(main, "Edit", "Preferences", child="Global Preferences…")
        qa.wait("GP dialog open", lambda: any(
            n.get_role_name() == "dialog" and n.get_name() == "Global Preferences" for n in qa.nodes()))

        with ac.accessibility_capture(gui_binary) as cap:
            cap.start_screen_reader(timeout=60)
            store = work / "config" / "datum" / "preferences"

            def generations():
                g = store / "generations"
                return sorted(p.name for p in g.iterdir()) if g.is_dir() else []

            # ORDER MATTERS: SET -> RESET -> REFUSAL all run while the GP dialog is OPEN; the no-trap
            # Escape (which closes the dialog) is LAST, so it never robs the refusal/snapshot of a dialog.

            # --- SET: focus the Reduced motion control, Return -> commit (polite) ---
            focused_control = tab_to(qa, main, cap, TOGGLE_ROLE, REDUCED_MOTION)
            set_before = generations()
            ann0 = len(cap.recorder.events)
            qa.key(main, "Return")
            cap.pump(1.5)
            set_committed = len(generations()) > len(set_before)
            set_anns = cap.recorder.events[ann0:]
            # Capture the control state NOW -- right after the SET, the user contribution is live, so the
            # Reduced motion control's value/state/provenance reflect it ("Set by you ...").
            post_set_controls = ac.control_states(qa)

            # --- RESET: Return again -> second committed change (polite) ---
            tab_to(qa, main, cap, TOGGLE_ROLE, REDUCED_MOTION)
            reset_before = generations()
            qa.key(main, "Return")
            cap.pump(1.5)
            reset_committed = len(generations()) > len(reset_before)

            # --- REFUSAL (dialog still open): hold writer.lock, Return on the control -> assertive, no-write ---
            refusal_assertive = refusal_no_write = False
            refusal_anns = []
            if (store / "writer.lock").is_file():
                tab_to(qa, main, cap, TOGGLE_ROLE, REDUCED_MOTION)
                ref_before = generations()
                annr = len(cap.recorder.events)
                holder = open(store / "writer.lock", "r+")
                try:
                    fcntl.flock(holder.fileno(), fcntl.LOCK_EX | fcntl.LOCK_NB)
                    qa.key(main, "Return")
                    cap.pump(1.8)
                    refusal_anns = cap.recorder.events[annr:]
                    refusal_no_write = generations() == ref_before
                    refusal_assertive = any(a.get("detail1") == ASSERTIVE and a.get("text")
                                            for a in refusal_anns)
                finally:
                    fcntl.flock(holder.fileno(), fcntl.LOCK_UN)
                    holder.close()

            # --- snapshot (dialog still open) ---
            snap = cap.snapshot(qa)
            # Decisive corroborating frame while the dialog is OPEN in the post-refusal state (the
            # Status node shows the typed refusal). Pixels corroborate; the §5 typed payload is primary.
            try:
                qa.capture("accessibility-set-reset-refusal", main)
            except Exception:
                pass

            # --- no-trap LAST: Tab cycles across distinct controls AND Escape leaves the dialog ---
            focus_walk = []
            seen = set()
            for _ in range(10):
                qa.key(main, "Tab")
                cap.pump(0.3)
                f = focused(qa)
                focus_walk.append({"role": f[0], "name": f[1]})
                seen.add(f)
            distinct = len([s for s in seen if s != (None, None)])
            dialog_before_escape = any(n.get_role_name() == "dialog"
                                       and n.get_name() == "Global Preferences" for n in qa.nodes())
            qa.key(main, "Escape")
            cap.pump(0.5)
            dialog_after_escape = any(n.get_role_name() == "dialog"
                                      and n.get_name() == "Global Preferences" for n in qa.nodes())
            no_trap = distinct >= 3 and dialog_before_escape and not dialog_after_escape

            # ---- the five assertions ----
            # native-at-spi-evidence: real tree (controls present) AND real events fired.
            observations["native-at-spi-evidence"] = bool(post_set_controls) and bool(cap.recorder.events)
            # correct-focus: the driven control was genuinely focusable/focused.
            observations["correct-focus"] = bool(focused_control)
            # correct-name-role-value-state: Reduced motion is a toggle button, enabled, whose description
            # carries the resolver provenance and, after the SET, reflects the user contribution.
            rm = next((c for c in post_set_controls if c["name"] == REDUCED_MOTION), None)
            observations["correct-name-role-value-state"] = bool(
                rm and rm["role"] == TOGGLE_ROLE and rm["enabled"] and rm["description"]
                and "Set by you" in rm["description"])
            observations["no-trap"] = bool(no_trap)
            # truthful-announcement: a committed SET fired a polite "changed" announcement AND the
            # refusal fired the assertive typed message -- exact text + correct priority (ch2 PRIMARY).
            polite_set = any(a.get("detail1") == POLITE
                             and "changed for this device" in str(a.get("text", "")).lower()
                             for a in set_anns)
            assertive_refusal = any(a.get("detail1") == ASSERTIVE
                                    and "writer lease is unavailable" in str(a.get("text", "")).lower()
                                    for a in refusal_anns)
            observations["truthful-announcement"] = bool(polite_set and assertive_refusal)

            # ---- PRIMARY TYPED EVIDENCE (F4-DESIGN §5) -> the row's after-state ----
            # The CASE-row schemas are all closed (assertion evidence = {schema,context,id,observed};
            # the row's fields are fixed; stdout = {schema,executed_tests,observations}==the 5 bools), so
            # the rich payload cannot live in them. Its content-addressed, row-bound home is the
            # after-state: the recipe WRITES it under <work_root>/accessibility-capture/ and the producer's
            # a11y capture_after folds it (prefix "accessibility-capture/") into row["after"] (state.v2,
            # each file sha-bound). per-step PASS/OWED for the spoken channel is recorded (PM ruling b).
            spoken_since = snap["channels"]["ch3_spoken_since_drive"]

            def _spoke(text):
                return any(text.lower() in str(s).lower() for s in spoken_since)

            payload = {
                "schema": "datum.preferences.accessibility-capture.v1",
                "coverage": "/".join(EXPECTED_HEAD),
                "ch1_tree": {
                    "controls_post_set": post_set_controls,
                    "status_node": snap["channels"]["ch1_status_node"],
                    "focus_walk": focus_walk,
                    "no_trap": {"distinct_focus_stops": distinct,
                                "dialog_open_before_escape": dialog_before_escape,
                                "dialog_closed_after_escape": not dialog_after_escape},
                },
                "ch2_announcements": [
                    {"text": a.get("text"), "detail1": a.get("detail1"), "detail2": a.get("detail2"),
                     "source_role": a.get("source_role"), "t": a.get("t"),
                     "priority": ("assertive" if a.get("detail1") == ASSERTIVE
                                  else "polite" if a.get("detail1") == POLITE else "other")}
                    for a in cap.recorder.events],
                "ch3_screen_reader": {
                    "ready_latency_s": snap["screen_reader"]["ready_latency_s"],
                    "spoken_all": snap["channels"]["ch3_spoken_all"],
                    "spoken_since_drive": spoken_since,
                },
                "transitions": {
                    "set": {"committed": set_committed,
                            "announcement": next((a.get("text") for a in set_anns
                                                  if a.get("detail1") == POLITE), None),
                            "spoken_pass": _spoke("changed for this device")},
                    "reset": {"committed": reset_committed},
                    "refusal": {"no_write": refusal_no_write, "assertive": refusal_assertive,
                                "announcement": next((a.get("text") for a in refusal_anns
                                                      if a.get("detail1") == ASSERTIVE), None),
                                "spoken_pass": _spoke("writer lease is unavailable")},
                },
                "provenance": snap["provenance"],
            }
            evdir = work / "accessibility-capture"
            evdir.mkdir(parents=True, exist_ok=True)
            (evdir / "set-reset-refusal.json").write_text(json.dumps(payload, indent=1, default=str))
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
    parser = argparse.ArgumentParser(description="GP-CM05E GUI a11y case recipe (set-reset-refusal/gui)")
    parser.add_argument("--fixture-json", required=True)
    args = parser.parse_args(argv)
    fixture = json.loads(args.fixture_json)
    start = time.monotonic()
    observations = observe(fixture)
    elapsed = time.monotonic() - start
    # executed_tests = the five adjudicated assertions (the _diagnostics key is not an assertion).
    scored = {k: v for k, v in observations.items() if not k.startswith("_")}
    print(f"Ran {len(scored)} tests in {elapsed:.3f}s", file=sys.stderr)
    print(json.dumps({"schema": SCHEMA, "executed_tests": len(scored),
                      "observations": observations}, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main_cli())
