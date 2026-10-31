#!/usr/bin/env python3
"""Committed GP-CM05E GUI accessibility-interaction NAV case recipe (dispatching persona).

The four read-only accessibility-interaction scenarios -- keyboard-all-controls, search-and-clear,
explanation-and-return, native-close-reopen -- are thin personas over the SHARED capture backbone
``recipes/accessibility_capture.py`` (built once, configured per scenario -- the Lean law). This recipe
dispatches on the coverage variant to the scenario drive; the backbone, the five-assertion shape, and
the F4-DESIGN section-5 typed-payload emission (written to <work_root>/accessibility-capture/ and folded
by gui_producer._gui_a11y_capture_after into row["after"], the only content-addressed row-bound slot the
closed CASE-row schemas allow) are shared with the set-reset-refusal anchor
(``preferences_gui_a11y_case.py``). These four are READ-ONLY on the Global store: the after-state's
preferences/ subtree stays EMPTY (no generation = no-write proven by absence); the payload is evidence.

    python3 .../recipes/preferences_gui_a11y_nav_case.py --fixture-json <frozen fixture JSON>
    DATUM_EDA_GUI / DATUM_EDA_WORK_ROOT  (as for the anchor recipe)

The announcement recorder registers BEFORE Global Preferences opens, so the open/reopen announcement
("Global Preferences opened. Appearance and Units, eleven settings. ...") is captured on ch2. The five
assertions (native-at-spi-evidence / correct-focus / correct-name-role-value-state / no-trap /
truthful-announcement) are adjudicated from the three channels; truthful-announcement reads the typed
announced string + priority directly (the 4fhu live-AT-SPI ruling), the screen reader + pixels
corroborate. Honest observations only; always exits 0; never retries to green.
"""
import argparse
import json
import os
import sys
import time
from pathlib import Path

SCHEMA = "datum.preferences.case-observations.v2"
EXPECTED_CASE = "accessibility-interaction"
NAV_VARIANTS = ("keyboard-all-controls", "search-and-clear", "explanation-and-return",
                "native-close-reopen")
POLITE, ASSERTIVE = 1, 2
VALUE_ROLES = ("combo box", "toggle button", "spin button", "slider", "check box", "radio button")
OPEN_ANNOUNCEMENT = "Global Preferences opened."
ASSERTION_IDS = ("native-at-spi-evidence", "correct-focus", "correct-name-role-value-state",
                 "no-trap", "truthful-announcement")


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


def _coverage(fixture):
    coverage = (fixture.get("parameters") or {}).get("coverage")
    if not isinstance(coverage, list) or len(coverage) != 5:
        raise SystemExit("fixture coverage must be a 5-tuple")
    case, variant, subcase, surface = coverage[:4]
    if case != EXPECTED_CASE or variant not in NAV_VARIANTS or surface != "gui":
        raise SystemExit(f"recipe handles {EXPECTED_CASE}/<{NAV_VARIANTS}>/gui, got {coverage[:4]}")
    c = coverage[4]
    if (not isinstance(c, list) or len(c) != 3 or not isinstance(c[0], str)
            or not isinstance(c[1], (int, float)) or not isinstance(c[2], list) or len(c[2]) != 2
            or not all(isinstance(n, int) for n in c[2])):
        raise SystemExit("gui coordinate must be [backend, scale, [width, height]]")
    return variant, (c[0], float(c[1]), (c[2][0], c[2][1]))


# ---- shared tree/focus helpers (Atspi imported lazily inside observe) ----
def _mk_helpers(qa, main, cap, Atspi):
    def focused():
        for n in qa.nodes():
            try:
                if n.get_state_set().contains(Atspi.StateType.FOCUSED):
                    return (n.get_role_name(), n.get_name())
            except Exception:
                continue
        return (None, None)

    def tab_to(predicate, max_tabs=30):
        for _ in range(max_tabs):
            if predicate(focused()):
                return True
            qa.key(main, "Tab")
            cap.pump(0.3)
        return predicate(focused())

    def gp_open():
        return any(n.get_role_name() == "dialog" and n.get_name() == "Global Preferences"
                   for n in qa.nodes())
    return focused, tab_to, gp_open


def _name_role_state_ok(controls):
    """correct-name-role-value-state base: every value-control has a value role, a name, a provenance
    description, and is enabled."""
    return bool(controls) and all(
        c["role"] in VALUE_ROLES and c["name"] and c["description"]
        and ("Global" in c["description"] or "default" in c["description"].lower()
             or "Set by you" in c["description"]) and c["enabled"]
        for c in controls)


# ---- per-scenario drives: each returns (observations {5 bools}, payload_extra dict) ----
def _drive_keyboard_all_controls(ctx):
    qa, main, cap, Atspi, ac, focused, tab_to, gp_open, open_anns = ctx
    # Enumerate ALL 11 controls (Appearance 3 + Units 8) FIRST, by cycling the section nav with Right.
    # Right cycles the section only while the section nav holds focus -- which it does right after GP
    # open -- so collect BEFORE any Tab moves focus away (the proven clean-install enumeration pattern).
    seen = {}
    for _ in range(4):  # Appearance -> Units -> wrap; covers both sections
        for c in ac.control_states(qa):
            seen[(c["name"], c["role"])] = c
        qa.key(main, "Right")
        cap.pump(0.5)
    controls = list(seen.values())
    # Then Tab-walk for the focus/no-trap evidence (reaching value controls by keyboard).
    focus_walk = []
    reached_control = False
    for _ in range(18):
        qa.key(main, "Tab")
        cap.pump(0.2)
        f = focused()
        focus_walk.append({"role": f[0], "name": f[1]})
        if f[0] in VALUE_ROLES:
            reached_control = True
    open_ann_ok = any(OPEN_ANNOUNCEMENT in str(a.get("text", "")) and "eleven settings" in str(a.get("text", ""))
                      and a.get("detail1") == POLITE for a in open_anns)
    distinct = len({(f["role"], f["name"]) for f in focus_walk if f["role"]})
    obs = {
        "native-at-spi-evidence": bool(controls) and bool(cap.recorder.events),
        "correct-focus": reached_control,
        "correct-name-role-value-state": _name_role_state_ok(controls) and len(controls) >= 11,
        "no-trap": distinct >= 5,
        # truthful-announcement: the open announcement states "eleven settings" AND >= 11 controls are
        # genuinely reachable -- the announced count is truthful.
        "truthful-announcement": open_ann_ok and len(controls) >= 11,
    }
    return obs, {"controls": controls, "focus_walk": focus_walk, "control_count": len(controls),
                 "open_announcement_eleven": open_ann_ok}


def _drive_search_and_clear(ctx):
    qa, main, cap, Atspi, ac, focused, tab_to, gp_open, open_anns = ctx
    reached = tab_to(lambda f: f[1] and "search" in (f[1] or "").lower() and f[0] in ("entry", "text"))
    search_focus = focused()
    i = len(cap.recorder.events)
    qa.text(main, "motion")
    cap.pump(1.0)
    type_anns = cap.recorder.events[i:]
    visible_after_type = ac.control_states(qa)
    j = len(cap.recorder.events)
    for _ in range(8):
        qa.key(main, "BackSpace")
        cap.pump(0.3)
    clear_anns = cap.recorder.events[j:]
    visible_after_clear = ac.control_states(qa)

    def count_in(anns):
        for a in reversed(anns):
            t = str(a.get("text", ""))
            if "preference search results" in t:
                try:
                    return int(t.split()[0])
                except ValueError:
                    return None
        return None
    typed_count = count_in(type_anns)
    cleared_count = count_in(clear_anns)
    # truthful: the announced count matches the ACTUAL visible rows at each point -- filtered ("motion"
    # -> 1 = Reduced motion) and cleared (empty query -> the current section's rows). The announcement
    # never overstates/understates. (An empty query shows the section, not all 11; "11" appears only
    # while a broad query is active -- so the bar is count==visible, not an absolute number.)
    truthful = bool(typed_count is not None and typed_count == len(visible_after_type)
                    and cleared_count is not None and cleared_count == len(visible_after_clear)
                    and typed_count != cleared_count  # the search genuinely filtered then restored
                    and all(a.get("detail1") == POLITE for a in type_anns + clear_anns if a.get("text")))
    obs = {
        "native-at-spi-evidence": bool(visible_after_clear) and bool(cap.recorder.events),
        "correct-focus": bool(reached) and search_focus[0] in ("entry", "text"),
        "correct-name-role-value-state": _name_role_state_ok(visible_after_clear),
        "no-trap": True,  # finalized by the shared no-trap pass below
        "truthful-announcement": truthful,
    }
    return obs, {"search_focus": search_focus, "typed_count": typed_count,
                 "cleared_count": cleared_count, "type_announcements": _anns(type_anns),
                 "clear_announcements": _anns(clear_anns),
                 "visible_after_type": [c["name"] for c in visible_after_type]}


def _drive_explanation_and_return(ctx):
    qa, main, cap, Atspi, ac, focused, tab_to, gp_open, open_anns = ctx
    # Tab to the setting-name BUTTON (role=button named after the setting) -- activating it opens the
    # explanation (the Control toggle target would commit instead).
    reached = tab_to(lambda f: f == ("button", "Reduced motion"))
    invoker = focused()
    i = len(cap.recorder.events)
    qa.key(main, "Return")
    cap.pump(1.0)
    exp_anns = cap.recorder.events[i:]
    # Tab to the explanation Close button and activate -> focus must RETURN to the invoker.
    tab_to(lambda f: f[0] == "button" and "close" in (f[1] or "").lower() and "explanation" in (f[1] or "").lower())
    qa.key(main, "Return")
    cap.pump(0.8)
    return_focus = focused()
    explanation_opened = any("Preference explanation opened." in str(a.get("text", ""))
                             and a.get("detail1") == POLITE for a in exp_anns)
    focus_returned = return_focus == ("button", "Reduced motion")
    controls = ac.control_states(qa)
    obs = {
        "native-at-spi-evidence": bool(controls) and bool(cap.recorder.events),
        # correct-focus: the invoker held focus when activated AND focus returned to it after close.
        "correct-focus": bool(reached) and invoker == ("button", "Reduced motion") and focus_returned,
        "correct-name-role-value-state": _name_role_state_ok(controls),
        "no-trap": True,  # finalized below
        "truthful-announcement": explanation_opened,
    }
    return obs, {"invoker_focus": invoker, "return_focus": return_focus,
                 "focus_returned": focus_returned, "explanation_announcements": _anns(exp_anns)}


def _drive_native_close_reopen(ctx):
    qa, main, cap, Atspi, ac, focused, tab_to, gp_open, open_anns = ctx
    before_controls = {(c["name"], c["role"]) for c in ac.control_states(qa)}
    qa.key(main, "Escape")
    cap.pump(0.6)
    closed = not gp_open()
    i = len(cap.recorder.events)
    qa.menu_item(main, "Edit", "Preferences", child="Global Preferences…")
    qa.wait("GP reopened", gp_open)
    cap.pump(1.0)
    reopen_anns = cap.recorder.events[i:]
    after_controls = {(c["name"], c["role"]) for c in ac.control_states(qa)}
    reopened = gp_open()
    reopen_ann_ok = any(OPEN_ANNOUNCEMENT in str(a.get("text", "")) and "eleven settings" in str(a.get("text", ""))
                        and a.get("detail1") == POLITE for a in reopen_anns)
    persisted = bool(before_controls) and before_controls == after_controls
    controls = ac.control_states(qa)
    reached_control = tab_to(lambda f: f[0] in VALUE_ROLES)
    obs = {
        "native-at-spi-evidence": bool(controls) and bool(cap.recorder.events),
        "correct-focus": bool(reached_control),
        "correct-name-role-value-state": _name_role_state_ok(controls) and persisted,
        "no-trap": True,  # finalized below
        "truthful-announcement": closed and reopened and reopen_ann_ok,
    }
    return obs, {"closed_on_escape": closed, "reopened": reopened, "persisted": persisted,
                 "reopen_announcements": _anns(reopen_anns)}


DRIVES = {
    "keyboard-all-controls": _drive_keyboard_all_controls,
    "search-and-clear": _drive_search_and_clear,
    "explanation-and-return": _drive_explanation_and_return,
    "native-close-reopen": _drive_native_close_reopen,
}


def _anns(events):
    return [{"text": a.get("text"), "detail1": a.get("detail1"), "detail2": a.get("detail2"),
             "source_role": a.get("source_role"), "t": a.get("t"),
             "priority": ("assertive" if a.get("detail1") == ASSERTIVE
                          else "polite" if a.get("detail1") == POLITE else "other")}
            for a in events]


def observe(fixture):
    variant, (backend, scale, window) = _coverage(fixture)
    if backend not in ("wayland", "x11") or float(scale) != 1.0:
        raise SystemExit(f"this envelope supports wayland/1.0 and x11/1.0 (scale 2.0 pending its probe); "
                         f"got {backend}/{scale}")
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

    qa = StartupQA(work)
    for k in ("DATUM_PROJECT_ROOT", "DATUM_DISCOVERY", "DATUM_AGENT_CREDENTIAL_FILE",
              "EDA_ENGINE_SOCKET", "DATUM_ENGINE_SOCKET", "DATUM_MCP_ENDPOINT",
              "DATUM_AGENT_PROVENANCE"):
        qa.env.pop(k, None)
    observations = {k: False for k in ASSERTION_IDS}
    payload_extra = {}
    try:
        qa.start_envelope(output_size=(int(window[0]), int(window[1])), backend=backend)
        main = qa.launch(f"accessibility-interaction-{variant}-gui")
        with ac.accessibility_capture(gui_binary) as cap:
            focused, tab_to, gp_open = _mk_helpers(qa, main, cap, Atspi)
            # recorder is registered now -> open GP so the open announcement is captured on ch2.
            i0 = len(cap.recorder.events)
            qa.menu_item(main, "Edit", "Preferences", child="Global Preferences…")
            qa.wait("GP dialog open", gp_open)
            cap.pump(0.8)
            open_anns = cap.recorder.events[i0:]
            cap.start_screen_reader(timeout=60)

            ctx = (qa, main, cap, Atspi, ac, focused, tab_to, gp_open, open_anns)
            observations, payload_extra = DRIVES[variant](ctx)

            snap = cap.snapshot(qa)
            # decisive corroborating frame while the surface is in its scenario state (dialog open for
            # all but native-close-reopen, which is already reopened here).
            try:
                qa.capture(f"accessibility-{variant}", main)
            except Exception:
                pass

            # shared no-trap finaliser: Tab cycles >= distinct stops AND Escape leaves the dialog.
            if gp_open():
                walk = set()
                for _ in range(10):
                    qa.key(main, "Tab")
                    cap.pump(0.25)
                    walk.add(focused())
                distinct = len([s for s in walk if s != (None, None)])
                before_escape = gp_open()
                qa.key(main, "Escape")
                cap.pump(0.5)
                observations["no-trap"] = bool(distinct >= 3 and before_escape and not gp_open())
            else:
                observations["no-trap"] = bool(observations.get("no-trap"))

            payload = {
                "schema": "datum.preferences.accessibility-capture.v1",
                "coverage": f"{EXPECTED_CASE}/{variant}/default/gui",
                "scenario": variant,
                "ch1_tree": {"controls": snap["channels"]["ch1_controls"],
                             "status_node": snap["channels"]["ch1_status_node"]},
                "ch2_announcements": _anns(cap.recorder.events),
                "ch3_screen_reader": {"ready_latency_s": snap["screen_reader"]["ready_latency_s"],
                                      "spoken_all": snap["channels"]["ch3_spoken_all"],
                                      "spoken_since_drive": snap["channels"]["ch3_spoken_since_drive"]},
                "scenario_evidence": payload_extra,
                "observations": observations,
                "provenance": snap["provenance"],
            }
            evdir = work / "accessibility-capture"
            evdir.mkdir(parents=True, exist_ok=True)
            (evdir / f"{variant}.json").write_text(json.dumps(payload, indent=1, default=str))
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
    parser = argparse.ArgumentParser(description="GP-CM05E GUI a11y NAV case recipe (dispatching)")
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
