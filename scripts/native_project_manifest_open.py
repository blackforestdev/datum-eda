"""Bounded manifest-row gestures through the ordinary Open dialog."""
import os
from pathlib import Path
import re
from gi.repository import Atspi
from native_project_browse_workflow import navigate, value
from native_project_terminal_handoff import owned_shell_state, selected, state


def execute(qa):
    target = Path(os.environ["DATUM_QA_MANIFEST_PROJECT"]).resolve()
    assert target.is_relative_to(qa.root.parent) and (target / "project.json").is_file()
    before = qa.source_files(target)
    invalid = qa.root / "invalid-project"
    invalid.mkdir()
    (invalid / "project.json").write_bytes(b"invalid native manifest")
    qa.start_envelope()
    main = qa.launch("manifest-open-main")
    qa.terminal_command(main, 'M_KEEP=preserved; printf "M_ORIGINAL:%s\\n" "$$"')
    original = int(qa.wait("original shell", lambda: re.search(r"M_ORIGINAL:(\d+)", qa.terminal_text())).group(1))
    initial = state(qa, "initial_terminal")

    def open_listing(path):
        qa.menu_item(main, "File", "Open Project")
        navigate(qa, main, "Location", path)
        qa.focus_control(main, "Browse for Location")
        qa.key(main, "Return")
        qa.wait("exact manifest row", lambda: qa.find("project.json"))
        qa.wait("ready manifest listing", lambda: "Reading" not in value(qa, "Folder navigation status"))
        row = qa.find("project.json")
        assert qa.attributes(row)["value"].startswith("file,")
        assert "Enter opens this folder’s Project" in row.get_description()
        return row

    row = open_listing(target)
    qa.capture("manifest-before-doubleclick", main)
    ordinal = int(re.search(r", (\d+) of", qa.attributes(qa.find("project.json"))["value"]).group(1)) - 1
    # Native Open layout: list begins at y=174; rows are 24px. Semantic
    # row ordering identifies the target; actual pointer remains native input.
    x, y = 230, 186 + ordinal * 24
    qa.command("xdotool", "mousemove", "--window", main, str(x), str(y), "click", "1")
    qa.wait_presented()
    assert qa.find("Open Project") and "this folder" in value(qa, "Project directory")
    assert Path(f"/proc/{original}").exists()
    # A keyboard selection interrupts the earlier click. A following single
    # manifest click remains passive even inside the old 400ms click window.
    qa.command("xdotool", "sleep", "0.45", "click", "1", "sleep", "0.05",
               "key", "--clearmodifiers", "Up", "sleep", "0.05", "click", "1")
    qa.wait_presented()
    assert qa.find("Open Project") and Path(f"/proc/{original}").exists()
    qa.observations["keyboard_interrupt_single_click_passive"] = True
    qa.capture("manifest-interrupted-single-click-passive", main)
    # Separate the passive selection from the actual fresh double-click.
    qa.command("xdotool", "sleep", "0.45", "click", "--repeat", "2", "--delay", "80", "1")
    qa.wait("double-click publishes containing Project", lambda: not qa.find("Open Project"))
    selected(qa, "Project:")
    qa.capture("manifest-doubleclick-publication", main)
    assert state(qa, "after_doubleclick")["bounds"] == initial["bounds"]
    qa.terminal_command(main, 'printf "M_DOUBLE:%s\\n" "$$"', tab=1)
    first = int(qa.wait("double-click Project shell", lambda: re.search(r"M_DOUBLE:(\d+)", qa.terminal_text())).group(1))
    assert owned_shell_state(qa, first, "doubleclick_shell")["cwd"] == str(target)
    assert qa.source_files(target) == before

    open_listing(target)
    # Browse focuses list; Home + Down selects the exact semantic row using
    # ordinary keys, independent of fixed row ordering or pointer coordinates.
    ordinal = int(re.search(r", (\d+) of", qa.attributes(qa.find("project.json"))["value"]).group(1)) - 1
    qa.key(main, "Home")
    for _ in range(ordinal):
        qa.key(main, "Down")
    qa.wait("manifest keyboard-selected", lambda: qa.find("project.json").get_state_set().contains(Atspi.StateType.FOCUSED))
    qa.capture("manifest-before-enter", main)
    qa.key(main, "Return")
    qa.wait("Enter publishes containing Project", lambda: not qa.find("Open Project"))
    selected(qa, "Project:")
    qa.capture("manifest-enter-publication", main)
    qa.terminal_command(main, 'printf "M_ENTER:%s\\n" "$$"', tab=2)
    second = int(qa.wait("Enter Project shell", lambda: re.search(r"M_ENTER:(\d+)", qa.terminal_text())).group(1))
    assert second != first and owned_shell_state(qa, second, "enter_shell")["cwd"] == str(target)
    assert qa.source_files(target) == before
    prior = state(qa, "before_invalid")
    open_listing(invalid)
    qa.key(main, "Home")
    qa.key(main, "Return")
    qa.wait("invalid manifest visibly refused", lambda: qa.find("Open Project") and qa.find("Project not opened"))
    qa.capture("manifest-invalid-refusal", main)
    qa.key(main, "Escape")
    qa.wait("refused dialog cancelled", lambda: not qa.find("Open Project"))
    assert state(qa, "after_invalid")["bounds"] == prior["bounds"]
    assert (invalid / "project.json").read_bytes() == b"invalid native manifest"
    assert qa.source_files(target) == before and Path(f"/proc/{first}").exists() and Path(f"/proc/{second}").exists()
    qa.terminal_command(main, 'printf "M_RETAIN:%s:%s\\n" "$$" "$M_KEEP"')
    qa.wait("original shell and variable preserved", lambda: f"M_RETAIN:{original}:preserved" in qa.terminal_text())
    qa.capture("manifest-original-session-preserved", main)
    qa.terminal_command(main, "exit", tab=2)
    qa.terminal_command(main, "exit", tab=1)
    qa.terminal_command(main, "exit")
    qa.wait("all owned PTYs closed", qa.terminal_ptys_closed)
    qa.observations["scope"] = {"project": str(target), "canonical_files_unchanged": True,
        "source_admission": "Existing path-based Open; no row-time byte/inode pinning",
        "reuse": "ec2 retained-host/native-edit/session guards unchanged"}
