"""Single-dialog C09 scenario behind the existing native Wayland QA runner."""
import json
import ctypes
import os
import subprocess
import shutil
import tempfile
from pathlib import Path
import re
from gi.repository import Atspi


def value(qa, name):
    node = qa.find(name)
    return qa.attributes(node).get("value", "") if node else ""


def available(qa, name):
    node = qa.find(name)
    return node and qa.attributes(node).get("available") == "true"


def folders(qa):
    return next(n for n in qa.nodes() if n.get_role_name() == "list")


def focus_folders(qa, main):
    qa.focus_control(main, folders(qa).get_name())


def navigate(qa, main, field, path):
    qa.fill_control(main, field, str(path))
    # Location edits take effect directly; no Enter is needed for discovery or action.
    qa.wait("current typed Location and completed discovery", lambda:
            value(qa, field) == str(path)
            and "Reading" not in value(qa, "Folder navigation status"))


def reveal_browse(qa, main, label, pointer=False, is_new=True):
    before = qa.wait_presented()["window"]
    node = qa.find("Browse for Location")
    assert not node.get_state_set().contains(Atspi.StateType.EXPANDED)
    assert not any(n.get_role_name() in {"list", "list item"} for n in qa.nodes())
    qa.capture(label + "-compact", main)
    if pointer:
        # Default native client is 760x720 at the owned kiosk output origin.
        qa.command("xdotool", "mousemove", "--window", main, "710",
                   "159" if is_new else "113", "click", "1")
    else:
        qa.focus_control(main, "Browse for Location")
        qa.key(main, "space")
    qa.wait("Browse expands inline", lambda: qa.find("Browse for Location").get_state_set().contains(Atspi.StateType.EXPANDED))
    qa.wait("Browse listing completes", lambda: "Reading" not in value(qa, "Folder navigation status"))
    assert qa.wait_presented()["window"] == before
    assert folders(qa).get_state_set().contains(Atspi.StateType.FOCUSED)
    qa.capture(label + "-expanded", main)
    qa.focus_control(main, "Browse for Location")
    qa.key(main, "Return")
    assert qa.find("Browse for Location").get_state_set().contains(Atspi.StateType.EXPANDED)
    assert folders(qa).get_state_set().contains(Atspi.StateType.FOCUSED)
    qa.observations[label + "_same_native_host"] = before


def checked(qa):
    node = qa.find("Use project name")
    return node.get_state_set().contains(Atspi.StateType.CHECKED)


def preserved(qa, main, shell, child, marker):
    qa.terminal_command(main, f'printf "{marker}:%s:%s:%s:%s\\n" "$$" "$C09_CHILD" "$C09_KEEP" "${{DATUM_PROJECT_ROOT-unset}}"')
    qa.wait("original shell/child/environment retained", lambda:
            f"{marker}:{shell}:{child}:preserved:unset" in qa.terminal_text())
    assert Path(f"/proc/{child}").exists()


def clipboard_owner(qa):
    assert qa.file_actions and qa.env.get("DISPLAY") != os.environ.get("DISPLAY")
    lib = ctypes.CDLL("libX11.so.6")
    lib.XOpenDisplay.argtypes = [ctypes.c_char_p]; lib.XOpenDisplay.restype = ctypes.c_void_p
    lib.XInternAtom.argtypes = [ctypes.c_void_p, ctypes.c_char_p, ctypes.c_int]; lib.XInternAtom.restype = ctypes.c_ulong
    lib.XGetSelectionOwner.argtypes = [ctypes.c_void_p, ctypes.c_ulong]; lib.XGetSelectionOwner.restype = ctypes.c_ulong
    lib.XCloseDisplay.argtypes = [ctypes.c_void_p]
    display = lib.XOpenDisplay(qa.env["DISPLAY"].encode())
    assert display, "owned compositor clipboard display must be available"
    try:
        return lib.XGetSelectionOwner(display, lib.XInternAtom(display, b"CLIPBOARD", 0))
    finally:
        lib.XCloseDisplay(display)

def clipboard_seed(qa, value):
    previous = clipboard_owner(qa)
    previous_writer = qa.clipboard_writer
    argv = ["xclip", "-selection", "clipboard", "-target", "UTF8_STRING", "-in", "-quiet"]
    qa.clipboard_writer = subprocess.Popen(argv, env=qa.env, stdin=subprocess.PIPE,
        stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    qa.clipboard_writer.stdin.write(value.encode()); qa.clipboard_writer.stdin.close()
    owner = qa.wait("owned clipboard writer selection ownership", lambda:
        (current if (current := clipboard_owner(qa)) and current != previous else None))
    if previous_writer:
        if previous_writer.poll() is None:
            previous_writer.terminate()
        previous_writer.wait(timeout=5)
    qa.actions.append({"clipboard_writer_pid": qa.clipboard_writer.pid, "argv": argv,
        "owned_display": qa.env["DISPLAY"], "selection_owner": owner, "value": value})

def clipboard_read(qa):
    argv = ["xclip", "-selection", "clipboard", "-target", "UTF8_STRING", "-out"]
    result = subprocess.run(argv, env=qa.env, text=True, capture_output=True, timeout=5)
    qa.actions.append({"argv": argv, "owned_display": qa.env["DISPLAY"],
        "exit": result.returncode, "stdout": result.stdout, "stderr": result.stderr})
    assert result.returncode == 0
    return result.stdout


def select_entry(qa, main, name):
    focus_folders(qa, main)
    node = qa.find(name)
    position = int(re.search(r", (\d+) of ", qa.attributes(node)["value"]).group(1)) - 1
    qa.key(main, "Home")
    if position:
        qa.key(main, "Down", repeat=position)
    qa.wait("selected exact row", lambda: qa.find(name).get_state_set().contains(Atspi.StateType.FOCUSED))
    return position


def execute_host_actions(qa, main, is_new, protected=None):
    label = "file-actions-new" if is_new else "file-actions-open"
    parent = Path(__file__).resolve().parents[1] / "target/qa" / ("host-actions-" + qa.root.name) / label
    parent.mkdir(parents=True)
    (parent / "folder to rename").mkdir()
    (parent / "notes.txt").write_bytes(b"exact retained note bytes\n")
    (parent / "collision.txt").write_bytes(b"collision is never replaced\n")
    navigate(qa, main, "Location", parent)
    if is_new:
        qa.fill_control(main, "Project name", "abcdef")
        # Actual field left/right pointer placement and drag selection, using
        # the approved default native geometry (not a private App action).
        qa.command("xdotool", "mousemove", "--window", main, "181", "76", "click", "1")
        qa.key(main, "Delete")
        assert value(qa, "Project name") == "bcdef"
        qa.command("xdotool", "mousemove", "--window", main, "181", "76", "mousedown", "1",
                   "mousemove", "--window", main, "735", "76", "mouseup", "1")
        qa.key(main, "ctrl+c")
        assert clipboard_read(qa) == "bcdef"
        qa.key(main, "ctrl+x")
        assert value(qa, "Project name") == "" and clipboard_read(qa) == "bcdef"
        qa.key(main, "ctrl+v")
        assert value(qa, "Project name") == "bcdef"
        qa.key(main, "Home"); qa.key(main, "Right"); qa.key(main, "shift+Right"); qa.key(main, "Delete")
        assert value(qa, "Project name") == "bdef"
        qa.fill_control(main, "Project name", "")
    focus_folders(qa, main); qa.key(main, "ctrl+shift+n")
    qa.wait("shared New Folder naming bar", lambda: qa.find("New folder name"))
    qa.fill_control(main, "New folder name", "new child")
    qa.capture(label + "-create-folder-draft", main)
    qa.key(main, "Return")
    qa.wait("actual child creation and naming completion", lambda: (parent / "new child").is_dir() and not qa.find("New folder name"))
    assert value(qa, "Location") == str(parent) and not (parent / "project.json").exists()
    focus_folders(qa, main); qa.key(main, "ctrl+shift+n")
    qa.wait("second naming draft", lambda: qa.find("New folder name"))
    qa.fill_control(main, "New folder name", "collision.txt"); qa.key(main, "Return")
    qa.wait("collision preserves naming draft", lambda: available(qa,"Create Folder") and qa.find("Folder action refusal"))
    assert value(qa,"New folder name") == "collision.txt" and (parent / "collision.txt").read_bytes() == b"collision is never replaced\n"
    qa.capture(label + "-collision-retained", main)
    qa.key(main,"Escape"); assert qa.find("New folder name") is None
    select_entry(qa,main,"notes.txt")
    if not is_new:
        assert value(qa,"Project directory").removesuffix(" (this folder)") == str(parent)
        qa.key(main,"Return"); assert value(qa,"Location") == str(parent)
    qa.key(main,"F2"); qa.wait("file Rename starts all-selected",lambda:qa.find("New name"))
    assert "Selected: notes.txt" in qa.attributes(qa.find("New name"))["description"]
    clipboard_seed(qa,"invalid\nname"); qa.key(main,"ctrl+v")
    assert value(qa,"New name") == "notes.txt" and "Selected: notes.txt" in qa.attributes(qa.find("New name"))["description"]
    qa.capture(label + "-multiline-paste-refused",main)
    clipboard_seed(qa," notes λ.txt "); qa.key(main,"ctrl+v")
    assert value(qa,"New name") == " notes λ.txt "
    qa.key(main,"Return"); qa.wait("file rename completed",lambda:(parent / " notes λ.txt ").is_file() and not qa.find("New name"))
    assert (parent / " notes λ.txt ").read_bytes() == b"exact retained note bytes\n"
    selected = select_entry(qa,main,"folder to rename")
    # Right-click hits the exact row. It must not use the previously highlighted file.
    select_entry(qa,main," notes λ.txt ")
    qa.command("xdotool","mousemove","--window",main,"320",str((220 if is_new else 174)+selected*24+12),"click","3")
    qa.wait("row popup qualified",lambda:available(qa,"Rename…"))
    qa.capture(label + "-right-click-folder-popup",main)
    qa.key(main,"Down");qa.key(main,"Return")
    qa.wait("right-click exact folder naming",lambda:qa.find("New name"))
    assert value(qa,"New name") == "folder to rename"
    qa.fill_control(main,"New name","renamed folder");qa.key(main,"Return")
    qa.wait("folder rename completed",lambda:(parent / "renamed folder").is_dir() and not qa.find("New name"))
    qa.capture(label + "-completed-actions",main)
    qa.command("xdotool","mousemove","--window",main,"320","500" if is_new else "524","click","3")
    qa.wait("blank-space New Folder available",lambda:available(qa,"New Folder"))
    assert not available(qa,"Rename…")
    qa.capture(label + "-blank-popup-rename-disabled",main)
    qa.key(main,"Return");qa.wait("blank-space New Folder draft",lambda:qa.find("New folder name"))
    qa.key(main,"Escape");assert qa.find("New folder name") is None
    # Keyboard popup uses the selection, and Escape restores browser focus.
    select_entry(qa,main,"collision.txt");qa.key(main,"shift+F10")
    qa.wait("keyboard popup",lambda:qa.find("Rename…"));qa.key(main,"Escape")
    assert folders(qa).get_state_set().contains(Atspi.StateType.FOCUSED)
    for outside,point in [("location",(200,159 if is_new else 113)),("list",(700,500 if is_new else 524))]:
        qa.key(main,"shift+F10");qa.wait("popup before outside dismissal",lambda:qa.find("Rename…"))
        qa.command("xdotool","mousemove","--window",main,str(point[0]),str(point[1]),"click","1")
        qa.wait("outside click dismissed popup",lambda:qa.find("Rename…") is None)
        qa.command("xdotool","mousemove","--window",main,"200","159" if is_new else "113","click","1")
        qa.wait("fresh Location click after owned dismissal release",lambda:qa.find("Location").get_state_set().contains(Atspi.StateType.FOCUSED))
        assert value(qa,"Location")==str(parent)
        qa.capture(label+"-outside-"+outside+"-fresh-click",main)
    if protected:
        before=qa.source_files(protected)
        with tempfile.TemporaryDirectory(prefix="datum-file-actions-project-",dir="/tmp") as temporary:
            # Read-only native classification needs an eligible host ancestor, not
            # an early refusal caused by the QA log directory's reserved .datum.
            copy=Path(temporary) / "protected Project"
            for relative in before:
                if ".datum" not in Path(relative).parts:
                    destination=copy / relative;destination.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(protected / relative,destination)
            copied_before=qa.source_files(copy)
            qa.observations[label+"_readonly_project_fixture"]={"copied_from_gui_created":str(protected),"copy":str(copy),"source_hashes":copied_before,"limit":"canonical owned byte-copy for host classification, not another Project creation workflow"}
            navigate(qa,main,"Location",copy.parent)
            select_entry(qa,main,copy.name);qa.key(main,"shift+F10")
            qa.wait("read-only Project classification disables Rename",lambda:qa.find("Rename…") and "Checking" not in qa.attributes(qa.find("Rename…"))["description"])
            assert not available(qa,"Rename…") and available(qa,"New Folder")
            qa.capture(label + "-project-rename-protected",main);qa.key(main,"Escape")
            navigate(qa,main,"Location",copy / "board");qa.key(main,"ctrl+shift+n")
            qa.wait("managed namespace disables New Folder",lambda:qa.find("New Folder") and "Checking" not in qa.attributes(qa.find("New Folder"))["description"])
            assert not available(qa,"New Folder")
            qa.capture(label + "-managed-storage-protected",main);qa.key(main,"Escape")
            assert qa.source_files(protected)==before and qa.source_files(copy)==copied_before
            navigate(qa,main,"Location",parent)
        qa.observations[label+"_readonly_project_fixture"]["temporary_cleaned"]=not copy.exists()
    # Outer Cancel preserves every completed host action.
    qa.key(main,"Escape")
    qa.wait("whole dialog cancelled",lambda:qa.find("New Project" if is_new else "Open Project") is None)
    assert (parent / "new child").is_dir() and (parent / "renamed folder").is_dir()
    assert (parent / " notes λ.txt ").read_bytes() == b"exact retained note bytes\n"
    qa.observations[label + "_filesystem"] = {"parent":str(parent),"retained_after_outer_cancel":True,
        "entries":sorted(p.name for p in parent.iterdir()),"note_bytes_sha256":__import__("hashlib").sha256((parent / " notes λ.txt ").read_bytes()).hexdigest()}


def clipboard_failure(qa):
    qa.stop()
    display=qa.env.pop("DISPLAY")
    try:
        qa.terminal_dock_open=True
        main=qa.launch("clipboard-unavailable-native-wayland")
        qa.menu_item(main,"File","New Project")
        qa.fill_control(main,"Project name","retained")
        qa.key(main,"ctrl+a");qa.key(main,"ctrl+x")
        assert value(qa,"Project name")=="retained"
        qa.wait("actual unavailable clipboard write explained",lambda:"Clipboard write unavailable" in value(qa,"Folder navigation status"))
        qa.capture("file-actions-unavailable-cut-retains-value-selection",main)
        qa.key(main,"Q");assert value(qa,"Project name")=="Q"
        qa.key(main,"ctrl+a");qa.key(main,"ctrl+v")
        assert value(qa,"Project name")=="Q"
        qa.wait("actual unavailable clipboard read explained",lambda:"Clipboard read unavailable" in value(qa,"Folder navigation status"))
        qa.key(main,"R");assert value(qa,"Project name")=="R"
        qa.observations["clipboard_provider_failure"]={"forced_display_absent":True,"backend":"native Wayland/Vulkan","cut_paste_preserve_value_and_selection":True}
        qa.key(main,"Escape");qa.terminal_command(main,"exit")
        qa.wait("unavailable-clipboard probe terminal closed",qa.terminal_ptys_closed)
    finally:
        qa.env["DISPLAY"]=display


def execute_editor_reveal(qa):
    parent=Path(__file__).resolve().parents[1] / "target/qa" / ("editor-reveal-"+qa.root.name)
    for index in range(3):
        parent=parent / (f"parent-{index}-"+"exact-path-segment-"*6)
        parent.mkdir(parents=True)
    parent=parent / "visible-end-marker"
    parent.mkdir()
    qa.start_envelope(desktop=True)
    main=qa.launch("shared-editor-reveal-main")
    qa.wait_presented()
    qa.key(main,"ctrl+shift+f")
    for is_new in [True,False]:
        title="New Project" if is_new else "Open Project"
        qa.menu_item(main,"File",title)
        navigate(qa,main,"Location",parent)
        reveal_browse(qa,main,"editor-reveal-"+("new" if is_new else "open"),is_new=is_new)
        for minimum in [False,True]:
            label="editor-reveal-"+("new" if is_new else "open")+("-minimum" if minimum else "-default")
            if minimum:
                minimum_dialog(qa,main,label+"-surface")
            for key,suffix in [("Home","home"),("End","end"),("ctrl+a","selection")]:
                qa.focus_control(main,"Location")
                qa.key(main,key)
                assert value(qa,"Location")==str(parent)
                qa.capture(label+"-"+suffix,main)
            qa.key(main,"Right")
            qa.observations[label+"_full_value"]={"path":str(parent),"unchanged_by_reveal_selection":True}
        qa.key(main,"Escape")
        qa.wait("reveal dialog cancelled",lambda:qa.find(title) is None)
    qa.terminal_command(main,"exit")
    qa.wait("reveal probe terminal closed",qa.terminal_ptys_closed)


def execute_browse(qa, new_only=False):
    if os.environ.get("DATUM_QA_TERMINAL_HANDOFF") == "1":
        from native_project_terminal_handoff import execute
        return execute(qa)
    if os.environ.get("DATUM_QA_EDITOR_REVEAL_ONLY")=="1":
        return execute_editor_reveal(qa)
    target = qa.root / "00-created-project"
    (qa.root / ".hidden").mkdir()
    (qa.root / "occupied").mkdir()
    for i in range(140):
        (qa.root / f"folder-{i:03}").mkdir()
    (qa.root / ("zzz-" + "long-name-" * 19)).mkdir()
    deep = qa.root
    for i in range(5):
        deep = deep / (f"deep-{i}-" + "full-path-segment-" * 10)
        deep.mkdir()
    qa.start_envelope()
    main = qa.launch("inline-main")
    qa.terminal_command(main, 'C09_KEEP=preserved; sleep 900 & C09_CHILD=$!; printf "C09_KEEP:%s:%s\\n" "$$" "$C09_CHILD"')
    shell, child = map(int, qa.wait("actual initial process IDs", lambda:
        re.search(r"C09_KEEP:(\d+):(\d+)", qa.terminal_text())).groups())

    # One draft proves correction, override/recheck and cancellation without a
    # second page or any Project publication.
    qa.menu_item(main, "File", "New Project")
    qa.wait("ordinary New dialog", lambda: qa.find("New Project"))
    qa.assert_new_has_no_units()
    assert checked(qa)
    reveal_browse(qa, main, "browse-new-pointer", pointer=True)
    if qa.file_actions:
        execute_host_actions(qa,main,True)
        qa.menu_item(main,"File","New Project")
        reveal_browse(qa,main,"file-actions-new-reopened")
    qa.fill_control(main, "Project name", "occupied")
    navigate(qa, main, "Location", qa.root)
    qa.wait("known collision blocks Create", lambda: not available(qa, "Create") and "occupied already exists" in value(qa,"Folder navigation status"))
    qa.capture("inline-occupied-default", main)
    qa.fill_control(main, "Project name", "invalid/name")
    assert not available(qa, "Create")
    qa.capture("inline-invalid-derived-name", main)
    qa.focus_control(main, "Use project name")
    qa.key(main, "space")
    qa.wait("checkbox cleared", lambda: not checked(qa))
    qa.fill_control(main, "Folder name", "explicit-override")
    qa.fill_control(main, "Project name", "renamed-display")
    assert value(qa, "Folder name") == "explicit-override"
    assert value(qa, "New Project folder") == str(qa.root / "explicit-override")
    qa.capture("inline-override-independent", main)
    qa.focus_control(main, "Use project name")
    qa.key(main, "space")
    qa.wait("recheck restores derived name", lambda: checked(qa)
            and value(qa, "Folder name") == "renamed-display")
    qa.capture("inline-recheck-derived", main)
    navigate(qa, main, "Location", qa.root / "missing")
    qa.wait("missing folder refused", lambda: "Cannot read" in value(qa, "Folder navigation status"))
    assert available(qa, "Create"), "Listing failure is discovery only; engine validates Create"
    qa.capture("inline-missing-parent", main)
    navigate(qa, main, "Location", qa.root)
    qa.wait("correction retains Global eligibility", lambda: available(qa, "Create"))
    # Ordinary tilde spelling must resolve before entering the selected row.
    tilde_parent = "~/" + str(qa.root.relative_to(Path.home()))
    navigate(qa, main, "Location", tilde_parent)
    focus_folders(qa, main)
    qa.key(main, "Home")
    qa.key(main, "Return")
    qa.wait("tilde row enters actual child and reads it", lambda:
            value(qa, "Location") == str(qa.root / ".hidden")
            and value(qa, "Folder navigation status") == "No folders here")
    assert value(qa, "New Project folder") == str(qa.root / ".hidden" / "renamed-display")
    qa.capture("browse-new-tilde-row-enter", main)
    qa.key(main, "alt+Up")
    qa.wait("tilde child Up returns resolved parent", lambda: value(qa, "Location") == str(qa.root)
            and "Reading" not in value(qa, "Folder navigation status"))
    qa.observations["tilde_row_navigation"] = {"typed": tilde_parent, "entered": str(qa.root / ".hidden")}
    focus_folders(qa, main)
    qa.key(main, "End")
    qa.capture("inline-long-list-end", main)
    rows = lambda: {n.get_name() for n in qa.nodes() if n.get_role_name() == "list item"}
    before_rows = rows()
    qa.command("xdotool", "mousemove", "--window", main, "300", "350", "click", "--repeat", "3", "--delay", "120", "4")
    qa.wait("wheel publishes visible accessibility rows", lambda: rows() - before_rows)
    qa.capture("inline-wheel-accessibility", main)
    qa.observations["inline_wheel_rows"] = {"before": sorted(before_rows), "after": sorted(rows())}
    navigate(qa, main, "Location", deep)
    qa.focus_control(main, "New Project folder")
    qa.key(main, "Home")
    full_path = str(deep / "renamed-display")
    assert value(qa, "New Project folder") == full_path
    qa.capture("inline-long-preview-home", main)
    for page in range(1, 6):
        qa.key(main, "Next")
        qa.capture(f"inline-long-preview-page-{page}", main)
    qa.key(main, "End")
    qa.capture("inline-long-preview-end", main)
    qa.key(main, "Home")
    qa.capture("inline-long-preview-return-home", main)
    assert value(qa, "New Project folder") == full_path and not Path(full_path).exists()
    qa.assert_new_has_no_units()
    qa.key(main, "Escape")
    qa.wait("Cancel closes complete New form", lambda: qa.find("New Project") is None)
    assert not target.exists() and not (qa.root / "explicit-override").exists()

    # Fresh New: the name is typed once, its visible default is used, and the
    # sole Create action performs genesis followed by existing adoption.
    qa.menu_item(main, "File", "New Project")
    qa.fill_control(main, "Project name", target.name)
    navigate(qa, main, "Location", qa.root)
    assert checked(qa) and value(qa, "Folder name") == target.name
    assert "Read-only" in qa.attributes(qa.find("Folder name"))["description"]
    qa.command("xdotool", "mousemove", "--window", main, "300", "110", "click", "1")
    qa.key(main, "x")
    assert value(qa,"Project name")==target.name and value(qa,"Folder name")==target.name
    assert not qa.find("Browse for Location").get_state_set().contains(Atspi.StateType.EXPANDED)
    assert not any(n.get_role_name() == "list" for n in qa.nodes())
    assert value(qa, "Location") == str(qa.root)
    assert value(qa, "New Project folder") == str(target)
    qa.observations["direct_typed_create_without_enter_or_browse"] = str(target)
    qa.capture("inline-name-once-before-create", main)
    qa.focus_control(main, "Create")
    qa.key(main, "Return")
    qa.wait("ordinary single Create adopted", lambda: target.joinpath("project.json").exists()
            and not qa.find("New Project") and not qa.find("Open Project"))
    project = json.loads(target.joinpath("project.json").read_text())
    receipt = project["project_units_seed_receipt"]
    assert len(receipt["items"]) == 8 and receipt["source"]["kind"] == "global"
    qa.terminal_dock_open = False
    preserved(qa, main, shell, child, "C09_CREATED")
    source = qa.source_files(target)
    if new_only:
        qa.menu_item(main, "Edit", "Preferences", child="Project Preferences…")
        qa.wait("existing Project Preferences Units", lambda: qa.find("Project Preferences") and qa.find("Units"))
        controls = [n for n in qa.nodes() if n.get_role_name() == "combo box"]
        assert len(controls) == 8 and all(qa.attributes(n).get("available") == "true" for n in controls)
        qa.capture("automatic-global-created-project-preferences-units", main)
        qa.key(main, "Escape")
        qa.wait("Project Preferences cancelled", lambda: qa.find("Project Preferences") is None)
        assert qa.source_files(target) == source
        preserved(qa, main, shell, child, "C09_AUTOMATIC_GLOBAL")
        qa.capture("automatic-global-created-original-terminal", main)
        qa.observations["automatic_global_delivery"] = {"target":str(target), "receipt":receipt,
            "source_hashes":source, "shell":shell, "child":child, "full_preview":full_path,
            "name_typed_once_for_creation":target.name, "project_preferences_writable_units":len(controls)}
        qa.terminal_command(main, "exit", tab=1)
        qa.terminal_command(main, 'kill "$C09_CHILD"; wait "$C09_CHILD"; exit')
        qa.wait("owned terminal PTYs closed", qa.terminal_ptys_closed)
        return

    if qa.file_actions:
        qa.menu_item(main,"File","Open Project")
        reveal_browse(qa,main,"file-actions-open",is_new=False)
        execute_host_actions(qa,main,False,protected=target)
    # A relative Location uses the same real row-navigation route as New.
    qa.menu_item(main, "File", "Open Project")
    gui_cwd = Path(f"/proc/{qa.process.pid}/cwd").resolve()
    relative_parent = str(qa.root.relative_to(gui_cwd))
    navigate(qa, main, "Location", relative_parent)
    reveal_browse(qa, main, "browse-open-relative-keyboard", is_new=False)
    focus_folders(qa, main)
    qa.key(main, "Home")
    qa.key(main, "Down")
    qa.key(main, "Return")
    qa.wait("relative row enters actual Project and reads children", lambda:
            Path(value(qa, "Location")) == target
            and "Reading" not in value(qa, "Folder navigation status")
            and any(n.get_role_name() == "list item" and n.get_name() == "board" for n in qa.nodes()))
    qa.capture("browse-open-relative-row-enter", main)
    qa.observations["relative_row_navigation"] = {"typed": relative_parent,
        "actual_gui_cwd": str(gui_cwd), "entered": value(qa, "Location"), "expected_child": str(target)}
    qa.key(main, "Escape")
    qa.wait("relative navigation cancelled whole Open form", lambda: qa.find("Open Project") is None)
    assert qa.source_files(target) == source

    # Both highlighted and current-directory candidates go directly to Open.
    for current in [False, True]:
        qa.menu_item(main, "File", "Open Project")
        navigate(qa, main, "Location", target if current else qa.root)
        if not current:
            reveal_browse(qa, main, "browse-open-keyboard", is_new=False)
            focus_folders(qa, main)
            qa.key(main, "Home")
            qa.key(main, "Down")
        expected = str(target)
        qa.wait("exact Open candidate visible", lambda: value(qa, "Project directory") == expected)
        if current:
            assert not qa.find("Browse for Location").get_state_set().contains(Atspi.StateType.EXPANDED)
            qa.observations["direct_typed_open_without_enter_or_browse"] = str(target)
        qa.capture("inline-open-current" if current else "inline-open-highlighted", main)
        assert qa.source_files(target) == source
        qa.focus_control(main, "Open")
        qa.key(main, "Return")
        qa.wait("ordinary direct Open adopted", lambda: qa.find("Open Project") is None)
        qa.terminal_dock_open = False
        preserved(qa, main, shell, child, "C09_CURRENT" if current else "C09_SELECTED")
        assert qa.source_files(target) == source

    # Read-directory permission is discovery only. Engine Open still reads the
    # existing Project through its canonical path; restore owned permissions always.
    target.chmod(0o300)
    try:
        qa.menu_item(main, "File", "Open Project")
        navigate(qa, main, "Location", target)
        reveal_browse(qa, main, "browse-open-listing-permission", is_new=False)
        qa.wait("actual listing permission refusal", lambda: "Permission denied" in value(qa, "Folder navigation status"))
        assert available(qa, "Open") and value(qa, "Project directory") == str(target)
        qa.capture("browse-open-discovery-failure-action-enabled", main)
        qa.focus_control(main, "Open")
        qa.key(main, "Return")
        qa.wait("canonical Open independent of listing", lambda: qa.find("Open Project") is None)
    finally:
        target.chmod(0o700)
    qa.terminal_dock_open = False
    preserved(qa, main, shell, child, "C09_LISTING_INDEPENDENT")
    assert qa.source_files(target) == source
    qa.observations["listing_permission_does_not_gate_open"] = str(target)

    qa.menu_item(main, "File", "Open Project")
    navigate(qa, main, "Location", qa.root / ".hidden")
    qa.focus_control(main, "Open")
    qa.key(main, "Return")
    qa.wait("engine refuses directory without a Project", lambda: qa.find("Project not opened"))
    qa.capture("inline-open-refusal", main)
    qa.key(main, "Escape")
    qa.wait("Open Cancel closes complete form", lambda: qa.find("Open Project") is None)
    assert not (qa.root / ".hidden/project.json").exists() and qa.source_files(target) == source
    preserved(qa, main, shell, child, "C09_FINAL")
    qa.capture("inline-created-opened-original-terminal", main)
    qa.observations["inline_delivery"] = {"target": str(target), "receipt": project["project_units_seed_receipt"],
        "source_hashes": source, "shell": shell, "child": child, "full_preview": full_path,
        "name_typed_once_for_creation": target.name, "automatic_global": True}
    for tab in [4, 3, 2, 1]:
        qa.terminal_command(main, "exit", tab=tab)
    qa.terminal_command(main, 'kill "$C09_CHILD"; wait "$C09_CHILD"; exit')
    qa.wait("owned terminal PTYs closed", qa.terminal_ptys_closed)
    if qa.file_actions:
        clipboard_failure(qa)


def minimum_dialog(qa, main, label):
    # Stock Weston desktop controls, with native focus and actual client-size
    # receipts. Resize by the measured delta instead of relying on WM clamping.
    def surface():
        text = Path(qa.env["DATUM_GUI_LOG"]).read_text()
        host = re.findall(r"surface identity window=(WindowId\([0-9]+\))", text)[-1]
        records = [json.loads(line.split("native_surface_lifecycle ", 1)[1])
                   for line in text.splitlines() if "native_surface_lifecycle " in line]
        return next((r for r in reversed(records) if r["window"] == host), None)
    qa.wait_presented()
    qa.key(main, "ctrl+shift+Left")
    qa.wait_presented()
    width, height = surface()["configured_extent"]
    qa.command("xdotool", "mousemove", "--window", main, "600", "700",
               "keydown", "ctrl", "mousedown", "3", "sleep", "0.2",
               "mousemove", "--window", main, str(600 + 620 - width), str(700 + 620 - height),
               "sleep", "0.2", "mouseup", "3", "keyup", "ctrl")
    def minimum():
        latest = surface()
        return latest if latest and latest["configured_extent"] == [620, 620] else False
    qa.observations[label + "_actual_minimum_surface"] = qa.wait("actual owned dialog 620x620", minimum)
    qa.capture(label, main)


def minimum_host_actions(qa, main, is_new):
    label="minimum-file-actions-new" if is_new else "minimum-file-actions-open"
    parent=Path(__file__).resolve().parents[1] / "target/qa" / ("host-actions-"+qa.root.name) / label
    parent.mkdir(parents=True);(parent/"occupied.txt").write_bytes(b"minimum retained bytes")
    navigate(qa,main,"Location",parent)
    reveal_browse(qa,main,label,is_new=is_new)
    focus_folders(qa,main);qa.key(main,"shift+F10")
    qa.wait("minimum popup reason qualified",lambda:available(qa,"New Folder"))
    assert not available(qa,"Rename…")
    qa.capture(label+"-popup-disabled-reason",main);qa.key(main,"Escape")
    focus_folders(qa,main);qa.key(main,"ctrl+shift+n")
    qa.wait("minimum naming editor",lambda:qa.find("New folder name"))
    qa.fill_control(main,"New folder name","occupied.txt");qa.key(main,"Return")
    qa.wait("minimum retained collision refusal",lambda:qa.find("Folder action refusal") and available(qa,"Create Folder"))
    qa.capture(label+"-wrapped-refusal-editor-footer",main)
    qa.focus_control(main,"Cancel naming");qa.capture(label+"-inner-cancel-footer",main)
    qa.key(main,"Return");assert qa.find("New folder name") is None
    select_entry(qa,main,"occupied.txt");qa.key(main,"F2")
    qa.wait("minimum file rename selected",lambda:qa.find("New name"))
    qa.capture(label+"-rename-selection",main)
    qa.key(main,"Escape");qa.key(main,"Escape")
    qa.wait("minimum outer dialog cancelled",lambda:qa.find("New Project" if is_new else "Open Project") is None)
    assert (parent/"occupied.txt").read_bytes()==b"minimum retained bytes" and len(list(parent.iterdir()))==1


def execute_minimum_forms(qa, new_only=False):
    deep = qa.root
    for i in range(5):
        deep = deep / (f"deep-{i}-" + "full-path-segment-" * 10)
        deep.mkdir()
    qa.start_envelope(desktop=True)
    main = qa.launch("minimum-inline-main")
    qa.wait_presented()
    qa.key(main, "ctrl+shift+f")
    qa.menu_item(main, "File", "New Project")
    qa.wait("ordinary New dialog", lambda: qa.find("New Project"))
    qa.assert_new_has_no_units()
    minimum_dialog(qa, main, "minimum-new-compact")
    if qa.file_actions:
        minimum_host_actions(qa,main,True)
        qa.menu_item(main,"File","New Project")
        minimum_dialog(qa,main,"minimum-new-reopened-after-file-actions")
    name = "overlong-" + "n" * 280
    qa.fill_control(main, "Project name", name)
    navigate(qa, main, "Location", deep)
    reveal_browse(qa, main, "minimum-new-browse-keyboard")
    qa.capture("minimum-new-before-create", main)
    qa.focus_control(main, "Project name")
    qa.key(main, "Next")
    qa.capture("minimum-new-body-scrolled", main)
    qa.focus_control(main, "Create")
    qa.key(main, "Return")
    qa.wait("actual engine filesystem refusal", lambda: qa.find("Project not created"))
    engine_refusal = value(qa, "Project not created")
    qa.key(main, "Next")
    assert name not in {entry.name for entry in deep.iterdir()}
    qa.assert_new_has_no_units()
    assert qa.find("Create").get_state_set().contains(Atspi.StateType.FOCUSED)
    qa.capture("minimum-new-fixed-engine-refusal-create-focus", main)
    # Remove only empty directories made by this run, then ordinary Alt+Up
    # encounters their real disappearance while retaining Create focus.
    deep.rmdir()
    deep.parent.rmdir()
    qa.key(main, "alt+Up")
    qa.wait("actual long folder refusal", lambda: "Cannot read" in value(qa, "Folder navigation status"))
    refusal = value(qa, "Folder navigation status")
    destination = str(deep.parent / name)
    assert qa.find("Create").get_state_set().contains(Atspi.StateType.FOCUSED)
    qa.assert_new_has_no_units()
    first_frame = qa.capture("minimum-new-navigation-completion-first-frame", main)
    settled_frame = qa.capture("minimum-new-fixed-long-refusal-create-focus", main)
    qa.observations["minimum_navigation_presentations"] = {
        "first": first_frame, "settled": settled_frame,
        "scope": "Both current owned-host/focus/presented/nonpending receipts; equal receipts are valid when the refusal frame is already current. Independent pixel review remains required."
    }
    assert value(qa, "Folder navigation status") == refusal
    assert qa.find("Create").get_state_set().contains(Atspi.StateType.FOCUSED)
    qa.focus_control(main, "New Project folder")
    assert value(qa, "New Project folder") == destination
    assert refusal in qa.attributes(qa.find("New Project folder"))["description"]
    qa.key(main, "Home")
    qa.capture("minimum-new-long-refusal-preview-home", main)
    qa.key(main, "Next")
    qa.capture("minimum-new-long-refusal-preview-page", main)
    qa.key(main, "End")
    qa.capture("minimum-new-long-refusal-preview-end", main)
    qa.key(main, "Home")
    qa.capture("minimum-new-long-refusal-preview-return-home", main)
    qa.key(main, "Escape")
    qa.wait("New cancelled", lambda: qa.find("New Project") is None)

    # Fault fixture is confined to this run's isolated XDG repository path.
    # It exercises actual reader refusal and the ordinary explicit retry action;
    # it is not a preference mutation or publication workflow claim.
    repository = qa.root / "config/datum/preferences"
    repository.parent.mkdir(parents=True, exist_ok=True)
    assert not repository.exists()
    repository.write_bytes(b"C09 bounded non-directory repository fault fixture\n")
    qa.menu_item(main, "File", "New Project")
    qa.wait("Global reader refused fixture", lambda: available(qa, "Re-read Global defaults"))
    assert not available(qa, "Create")
    qa.assert_new_has_no_units()
    qa.fill_control(main, "Project name", "repaired-global")
    navigate(qa, main, "Location", qa.root)
    minimum_dialog(qa, main, "minimum-new-global-reader-refusal")
    qa.focus_control(main, "Project name")
    qa.key(main, "Next")
    qa.focus_control(main, "Re-read Global defaults")
    qa.capture("minimum-new-fixed-reread-action", main)
    repository.unlink()
    qa.key(main, "Return")
    qa.wait("explicit reread restores Create eligibility", lambda: available(qa, "Create"))
    assert checked(qa) and not (qa.root / "repaired-global").exists()
    qa.assert_new_has_no_units()
    qa.capture("minimum-new-reread-restored-eligibility", main)
    qa.key(main, "Escape")
    qa.wait("New cancelled after retry", lambda: qa.find("New Project") is None)
    qa.observations["minimum_delivery"] = {"automatic_global": True, "full_refusal": refusal,
        "engine_refusal": engine_refusal, "exact_destination": destination, "repository_fault": str(repository),
        "limit": "Owned non-directory repository fixture, repaired before explicit native retry; no preference mutation claim"}

    if new_only:
        return
    qa.menu_item(main, "File", "Open Project")
    navigate(qa, main, "Location", deep.parent.parent)
    minimum_dialog(qa, main, "minimum-open-compact")
    if qa.file_actions:
        minimum_host_actions(qa,main,False)
        qa.menu_item(main,"File","Open Project")
        navigate(qa,main,"Location",deep.parent.parent)
        minimum_dialog(qa,main,"minimum-open-reopened-after-file-actions")
    reveal_browse(qa, main, "minimum-open-browse-keyboard", is_new=False)
    qa.capture("minimum-open-current-directory", main)
    qa.focus_control(main, "Project directory")
    qa.key(main, "End")
    qa.capture("minimum-open-preview-end-fixed-footer", main)
    qa.key(main, "Home")
    qa.capture("minimum-open-preview-home", main)
    qa.key(main, "Escape")
    qa.wait("Open cancelled", lambda: qa.find("Open Project") is None)
