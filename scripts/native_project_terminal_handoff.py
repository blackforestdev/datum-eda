"""Targeted ordinary New/Open terminal handoff in the existing Wayland runner."""
import json
import os
from pathlib import Path
import re
import shlex
from gi.repository import Atspi
from native_project_browse_workflow import navigate, available


def terminal(qa):
    return next(n for n in qa.nodes() if n.get_role_name() == "terminal")


def state(qa, label):
    node = terminal(qa)
    bounds = Atspi.Component.get_extents(node.get_component_iface(), Atspi.CoordType.WINDOW)
    data = {"name": node.get_name(), "bounds": [bounds.x, bounds.y, bounds.width, bounds.height],
            "focused": node.get_state_set().contains(Atspi.StateType.FOCUSED)}
    qa.observations[label] = data
    return data


def selected(qa, expected):
    return qa.wait("selected session context", lambda: expected in terminal(qa).get_name())


def owned_shell_state(qa, pid, label):
    # PID comes from this run's ordinary shell output. Read only the relevant
    # context keys, never retain the rest of the process environment.
    environment = dict(item.split(b"=", 1) for item in Path(f"/proc/{pid}/environ").read_bytes().split(b"\0") if b"=" in item)
    keys = ["DATUM_PROJECT_ROOT", "DATUM_SESSION_ID", "DATUM_DISCOVERY"]
    data = {"pid": pid, "cwd": str(Path(f"/proc/{pid}/cwd").resolve()),
            "context_environment": {key: environment.get(key.encode(), b"").decode() for key in keys}}
    qa.observations[label] = data
    return data


def output_extent(qa, main, width, height):
    # Resize only this runner's owned Weston output. Main stays fullscreen, so
    # its actual native client extent follows without decoration offsets.
    qa.command("xdotool", "windowsize", main, str(width), str(height))
    def configured():
        lines = Path(qa.env["DATUM_GUI_LOG"]).read_text().splitlines()
        rows = [json.loads(line.split("native_surface_lifecycle ", 1)[1])
                for line in lines if "native_surface_lifecycle " in line]
        return rows[-1] if rows and rows[-1]["configured_extent"] == [width, height] else None
    qa.observations[f"owned_output_{width}x{height}"] = qa.wait("actual Main native client extent", configured)
    qa.wait_presented()


def execute(qa):
    if os.environ.get("DATUM_QA_PATH_ASSISTANCE") == "1":
        from native_project_path_assistance import execute as execute_path
        return execute_path(qa)
    if os.environ.get("DATUM_QA_MANIFEST_OPEN") == "1":
        from native_project_manifest_open import execute as execute_manifest
        return execute_manifest(qa)
    if os.environ.get("DATUM_QA_HANDOFF_REFIT_ONLY") == "1":
        return execute_refit(qa)
    qa.start_envelope(desktop=True)
    main = qa.launch("terminal-handoff-main")
    # Existing input helper uses owned-output coordinates; remove stock Weston
    # decoration/placement offsets before its first tab click.
    qa.key(main, "ctrl+shift+f")
    qa.wait_presented()
    assert Path(f"/proc/{qa.process.pid}/cwd").resolve() == Path.home()
    qa.terminal_command(main,
        'C09_KEEP=preserved; sleep 900 & C09_CHILD=$!; '
        'printf "TH_IDS:%s:%s:%s\\n" "$$" "$C09_CHILD" "$PWD"; '
        'printf "TH_HISTORY:original-scroll-marker\\n"; '
        'printf "\\033]7;file://%s\\007" "$PWD"')
    match = qa.wait("actual original shell and child IDs", lambda:
                    re.search(r"TH_IDS:(\d+):(\d+):([^\r\n]+)", qa.terminal_text()))
    shell, child = map(int, match.groups()[:2])
    original_cwd = owned_shell_state(qa, shell, "actual_original_shell")["cwd"]
    selected(qa, "Independent shell")
    qa.capture("handoff-before-home", main)
    before = state(qa, "initial_terminal")
    # Drag the real published six-pixel dock divider to a nondefault height.
    x, y = 520, before["bounds"][1] - 49  # 44px strip +8px cell inset -3px handle center.
    qa.command("xdotool", "mousemove", "--window", main, str(x), str(y),
               "mousedown", "1", "sleep", "0.1", "mousemove", "--window", main,
               str(x), str(y - 65), "sleep", "0.1", "mouseup", "1")
    qa.wait_presented()
    resized = qa.wait("actual nondefault dock height", lambda:
                      state(qa, "resized_terminal") if state(qa, "resized_probe")["bounds"][3] != before["bounds"][3] else False)
    qa.capture("handoff-resized-before-new", main)

    name = "northwind-environmental-sensor-node-rev-c-low-power"
    target = qa.root / name
    qa.menu_item(main, "File", "New Project")
    qa.fill_control(main, "Project name", name)
    navigate(qa, main, "Location", qa.root)
    qa.wait("automatic Global defaults admit ordinary Create", lambda: available(qa, "Create"))
    qa.capture("handoff-new-before-create", main)
    qa.focus_control(main, "Create")
    qa.key(main, "Return")
    qa.wait("ordinary Create and adoption", lambda: (target / "project.json").exists()
            and not qa.find("New Project") and not qa.find("Open Project"))
    selected(qa, "Project: " + name)
    # No panel, tab, restore or terminal click between publication and capture.
    qa.capture("handoff-new-immediate-publication", main)
    after_new = state(qa, "immediate_new_terminal")
    assert after_new["bounds"] == resized["bounds"] and not after_new["focused"]
    qa.terminal_command(main, 'printf "TH_PROJECT_PID:%s\\n" "$$"; '
        'test -r "$DATUM_DISCOVERY" && printf "TH_DISCOVERY:readable\\n"', tab=1)
    project_shell = int(qa.wait("actual prepared Project-root shell and readable context", lambda:
        re.search(r"TH_PROJECT_PID:(\d+)", qa.terminal_text())
        if "TH_DISCOVERY:readable" in qa.terminal_text() else None).group(1))
    project_process = owned_shell_state(qa, project_shell, "actual_created_project_shell")
    assert project_process["cwd"] == str(target.resolve())
    assert project_process["context_environment"]["DATUM_PROJECT_ROOT"] == str(target)
    assert Path(project_process["context_environment"]["DATUM_DISCOVERY"]).is_file()
    source = qa.source_files(target)
    qa.menu_item(main, "File", "Open Project")
    navigate(qa, main, "Location", target)
    qa.focus_control(main, "Open")
    qa.key(main, "Return")
    qa.wait("ordinary Open publication", lambda: not qa.find("Open Project"))
    selected(qa, "Project: " + name)
    qa.capture("handoff-open-immediate-publication", main)
    after_open = state(qa, "immediate_open_terminal")
    assert after_open["bounds"] == resized["bounds"] and not after_open["focused"]
    assert qa.source_files(target) == source
    qa.terminal_command(main, 'printf "TH_OPEN_PID:%s\\n" "$$"', tab=2)
    reopened_shell = int(qa.wait("actual newly opened Project-root shell", lambda:
        re.search(r"TH_OPEN_PID:(\d+)", qa.terminal_text())).group(1))
    reopened_process = owned_shell_state(qa, reopened_shell, "actual_opened_project_shell")
    assert reopened_process["cwd"] == str(target.resolve())
    assert reopened_process["context_environment"]["DATUM_PROJECT_ROOT"] == str(target)
    assert reopened_shell != project_shell and Path(f"/proc/{project_shell}").exists()
    maximize_before = state(qa, "before_ordinary_maximize_attempt")
    qa.menu_item(main, "View", "Maximize Pane")
    qa.wait_presented()
    maximize_after = state(qa, "after_ordinary_maximize_attempt")
    qa.capture("handoff-existing-maximize-menu-attempt", main)
    qa.observations["ordinary_maximize_route"] = {
        "entry": "View > Maximize Pane after explicit terminal focus",
        "before": maximize_before, "after": maximize_after,
        "terminal_bounds_changed": maximize_before["bounds"] != maximize_after["bounds"],
        "scope": "Actual existing menu attempt only. Menu restores Editor focus before dispatch; unchanged terminal geometry is not a terminal-maximize PASS. Focused Runtime proof covers retained maximized state."
    }
    qa.terminal_command(main, 'printf "TH_KEPT:%s:%s:%s:%s:%s\\n" "$$" "$C09_CHILD" "$C09_KEEP" "$PWD" "${DATUM_PROJECT_ROOT-unset}"')
    expected = f"TH_KEPT:{shell}:{child}:preserved:{original_cwd}:unset"
    qa.wait("original PID child cwd environment and history preserved", lambda:
            expected in qa.terminal_text() and "TH_HISTORY:original-scroll-marker" in qa.terminal_text())
    assert Path(f"/proc/{child}").exists()
    selected(qa, "Independent shell")
    qa.capture("handoff-original-live-session-preserved", main)
    qa.terminal_command(main, f"cd {shlex.quote(str(target))}; printf '\\033]7;file://%s\\007' \"$PWD\"; "
                        "printf '\\033]0;codex\\007'")
    selected(qa, "Cwd: " + str(target))
    assert "Independent shell" in terminal(qa).get_name()
    qa.capture("handoff-child-title-independent-in-project-folder", main)
    qa.terminal_command(main, "exit", tab=1)
    qa.wait("older Project shell exits only on explicit exit", lambda: not Path(f"/proc/{project_shell}").exists())
    qa.select_terminal_tab(main, 1)
    output_extent(qa, main, 620, 620)
    state(qa, "minimum_terminal")
    qa.capture("handoff-main-620", main)
    assert terminal(qa).get_state_set().contains(Atspi.StateType.FOCUSED)
    qa.key(main, "ctrl+shift+w")
    full_hint = "close terminal? Enter confirms; Escape cancels"
    selected(qa, full_hint)
    confirmation = state(qa, "minimum_full_close_hint")
    assert "Project: " + name in confirmation["name"]
    assert str(target) in confirmation["name"] and Path(f"/proc/{reopened_shell}").exists()
    qa.wait_presented()
    qa.capture("handoff-620-context-and-close-confirmation", main)
    qa.key(main, "Escape")
    qa.wait("close confirmation cancelled without ending the selected shell", lambda:
            full_hint not in terminal(qa).get_name() and Path(f"/proc/{reopened_shell}").exists())
    output_extent(qa, main, 1344, 806)
    for tab, pid in [(1, reopened_shell)]:
        qa.terminal_command(main, "exit", tab=tab)
        qa.wait("owned Project shell exited", lambda: not Path(f"/proc/{pid}").exists())
    qa.terminal_command(main, f"kill {child}; wait {child} 2>/dev/null; exit")
    qa.wait("all owned PTYs closed", qa.terminal_ptys_closed)
    qa.observations["terminal_handoff_summary"] = {
        "original_shell": shell, "original_child": child, "original_cwd": original_cwd,
        "project_root": str(target), "project_shells": [project_shell, reopened_shell],
        "immediate_publication_dock_bounds": [resized["bounds"], after_new["bounds"], after_open["bounds"]],
        "limits": "Ordinary expanded-dock New/Open and620 context; closed/maximized/cancel/stale publication covered by focused actual Runtime test. Final GUI SIGTERM is not ordinary Close. No full terminal/platform qualification."
    }


def execute_refit(qa):
    """Only the changed close-label fit on an existing owned GUI-created Project."""
    target = Path(os.environ["DATUM_QA_HANDOFF_PROJECT"]).resolve()
    assert target.is_relative_to(qa.root.parent.resolve())
    assert target.parent.name.startswith("c09-terminal-handoff-wayland-attempt-")
    assert (target / "project.json").is_file()
    source = qa.source_files(target)
    qa.start_envelope()
    main = qa.launch("terminal-handoff-lifecycle-refit")
    qa.menu_item(main, "File", "Open Project")
    navigate(qa, main, "Location", target)
    qa.focus_control(main, "Open")
    qa.key(main, "Return")
    qa.wait("ordinary existing Project adopted", lambda: not qa.find("Open Project"))
    selected(qa, "Project:")
    assert str(target) in terminal(qa).get_name()
    qa.capture("refit-open-selected-project", main)
    qa.terminal_command(main, 'printf "REFIT_PID:%s\\n" "$$"', tab=1)
    pid = int(qa.wait("actual refit Project shell", lambda: re.search(r"REFIT_PID:(\d+)", qa.terminal_text())).group(1))
    process = owned_shell_state(qa, pid, "refit_actual_shell")
    assert process["cwd"] == str(target) and process["context_environment"]["DATUM_PROJECT_ROOT"] == str(target)
    output_extent(qa, main, 620, 620)
    qa.capture("refit-620-idle-context", main)
    qa.key(main, "ctrl+shift+w")
    full_hint = "close terminal? Enter confirms; Escape cancels"
    selected(qa, full_hint)
    state(qa, "refit_full_close_hint")
    qa.capture("refit-620-full-compact-prompt-and-controls", main)
    qa.key(main, "Escape")
    qa.wait("refit Cancel keeps actual Project shell", lambda:
            full_hint not in terminal(qa).get_name() and Path(f"/proc/{pid}").exists())
    assert qa.source_files(target) == source
    output_extent(qa, main, 1344, 806)
    qa.terminal_command(main, "exit", tab=1)
    qa.wait("refit Project shell exited", lambda: not Path(f"/proc/{pid}").exists())
    qa.terminal_command(main, "exit")
    qa.wait("refit owned PTYs closed", qa.terminal_ptys_closed)
    qa.observations["refit_qualification"] = "Only selected close-label measurement changed; full New/Open/preservation facts reused from final ff3b attempt4. Existing owned GUI-created Project, canonical files unchanged; actual620 output constraint, no platform sizing/full terminal qualification."
