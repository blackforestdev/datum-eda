"""Ordinary shared New/Open path queries and completion on an owned Wayland GUI."""
import os
import re
from pathlib import Path
from gi.repository import Atspi
from native_project_browse_workflow import value, available, minimum_dialog
from native_project_terminal_handoff import state, owned_shell_state


def execute(qa):
    if os.environ.get("DATUM_QA_PATH_SUCCESSIVE") == "1":
        return execute_successive(qa)
    choices = qa.root / "choices"
    for name in ["Documents", "Documents-old", "Downloads", "space λ "]:
        (choices / name).mkdir(parents=True)
    target = choices / "Documents" / "assisted-node"
    (choices / "Documents" / "assisted-node-old").mkdir()
    qa.start_envelope(desktop=True)
    main = qa.launch("path-assistance-main")
    qa.key(main, "ctrl+shift+f")
    qa.wait_presented()
    qa.terminal_command(main, 'P_KEEP=preserved; printf "P_ORIGINAL:%s\\n" "$$"')
    original = int(qa.wait("original shell PID", lambda: re.search(r"P_ORIGINAL:(\d+)", qa.terminal_text())).group(1))
    initial = state(qa, "initial_terminal")

    def query(path, count):
        qa.fill_control(main, "Location", str(path))
        qa.wait("current matching suggestion result", lambda:
                any(n.get_role_name() == "list" and n.get_name().startswith("Folder suggestions in ") for n in qa.nodes())
                and "Reading" not in value(qa, "Folder navigation status")
                and len([n for n in qa.nodes() if n.get_role_name() == "list item"]) == count)
        assert value(qa, "Location") == str(path)
        assert qa.find("Location").get_state_set().contains(Atspi.StateType.FOCUSED)

    def exact(path):
        qa.wait("exact completed directory listing", lambda:
                value(qa, "Location") == str(path) + "/"
                and any(n.get_role_name() == "list" and n.get_name().startswith("Files and folders in ") for n in qa.nodes())
                and "Reading" not in value(qa, "Folder navigation status"))

    qa.menu_item(main, "File", "New Project")
    qa.fill_control(main, "Project name", "assisted-node")
    query(choices / "Doc", 2)
    assert value(qa, "New Project folder") == str(choices / "Doc" / "assisted-node")
    qa.capture("path-new-ambiguous", main)
    qa.key(main, "Tab")
    qa.wait("common-prefix exact name stays query", lambda: value(qa,"Location")==str(choices/"Documents") and any(n.get_role_name()=="list" and n.get_name().startswith("Folder suggestions in ") for n in qa.nodes()))
    assert qa.find("Location").get_state_set().contains(Atspi.StateType.FOCUSED)
    qa.key(main, "Tab")
    assert qa.find("Browse for Location").get_state_set().contains(Atspi.StateType.FOCUSED)
    assert value(qa, "Location") == str(choices / "Documents")
    qa.key(main, "shift+Tab")
    assert qa.find("Location").get_state_set().contains(Atspi.StateType.FOCUSED)
    qa.key(main, "Down")
    assert qa.find("Location").get_state_set().contains(Atspi.StateType.FOCUSED)
    assert "selected" in qa.attributes(qa.find("Documents"))["value"]
    assert value(qa, "New Project folder") == str(choices / "Documents" / "assisted-node")
    qa.key(main, "Escape")
    assert qa.find("New Project") and "selected" not in qa.attributes(qa.find("Documents"))["value"]
    qa.capture("path-new-selection-cleared", main)
    # A unique literal substring completes with exact case, Unicode and space.
    query(choices / "pace", 1)
    qa.key(main, "Tab")
    exact(choices / "space λ ")
    qa.capture("path-new-unique-unicode-completion", main)
    query(choices / "missing", 0)
    assert not qa.find("Project not created")
    qa.capture("path-new-no-match-neutral", main)
    minimum_dialog(qa, main, "path-new-minimum-long-parent")
    qa.focus_control(main, "Create")
    assert available(qa, "Create")
    qa.key(main, "Return")
    qa.wait("explicit invalid New refusal", lambda: qa.find("Project not created"))
    assert not (choices / "missing").exists()
    assert value(qa, "Project name") == "assisted-node"
    qa.capture("path-new-explicit-invalid-refusal", main)
    query(choices / "Doc", 2)
    qa.key(main, "Down")
    qa.key(main, "Tab")
    exact(choices / "Documents")
    qa.capture("path-new-selected-tab-completion", main)
    qa.focus_control(main, "Create")
    qa.key(main, "Return")
    qa.wait("ordinary assisted Create/adoption", lambda: (target / "project.json").exists() and not qa.find("New Project") and not qa.find("Open Project"))
    qa.capture("path-new-created", main)
    assert state(qa, "after_new")["bounds"] == initial["bounds"]
    before = qa.source_files(target)
    qa.terminal_command(main, 'printf "P_CREATED:%s\\n" "$$"', tab=1)
    created = int(qa.wait("created Project shell PID", lambda: re.search(r"P_CREATED:(\d+)", qa.terminal_text())).group(1))
    assert owned_shell_state(qa, created, "created_shell")["cwd"] == str(target)
    qa.menu_item(main, "File", "Open Project")
    query(choices / "Documents" / "missing", 0)
    assert not qa.find("Project not opened")
    qa.focus_control(main, "Open")
    qa.key(main, "Return")
    qa.wait("explicit invalid Open refusal", lambda: qa.find("Project not opened"))
    qa.capture("path-open-explicit-invalid-refusal", main)
    assert qa.source_files(target) == before
    query(choices / "Documents" / "assisted", 2)
    qa.key(main, "Down")
    assert qa.find("Location").get_state_set().contains(Atspi.StateType.FOCUSED)
    assert value(qa, "Project directory") == str(choices / "Documents" / "assisted")
    qa.capture("path-open-selected-candidate-still-typed", main)
    qa.key(main, "Return")
    exact(target)
    assert qa.find("Open Project"), "suggestion Enter only navigates"
    qa.capture("path-open-enter-completion", main)
    qa.focus_control(main, "Open")
    qa.key(main, "Return")
    qa.wait("ordinary assisted Open publication", lambda: not qa.find("Open Project"))
    qa.capture("path-open-published", main)
    assert qa.source_files(target) == before
    qa.terminal_command(main, 'printf "P_OPENED:%s\\n" "$$"', tab=2)
    opened = int(qa.wait("opened Project shell PID", lambda: re.search(r"P_OPENED:(\d+)", qa.terminal_text())).group(1))
    assert opened != created and owned_shell_state(qa, opened, "opened_shell")["cwd"] == str(target)
    qa.terminal_command(main, 'printf "P_RETAIN:%s:%s\\n" "$$" "$P_KEEP"')
    qa.wait("original shell variable preserved", lambda: f"P_RETAIN:{original}:preserved" in qa.terminal_text())
    qa.capture("path-original-session-preserved", main)
    for tab in [2, 1, 0]:
        qa.terminal_command(main, "exit", tab=tab)
    qa.wait("all owned PTYs closed", qa.terminal_ptys_closed)
    qa.observations["path_assistance_scope"] = {"created_project": str(target), "source_files_hash_set": before,
        "ordinary_new_and_open": True, "case_unicode_space_exact": True, "all_owned_ptys_closed": True,
        "limits": "Local transient query; no recursive search/index. Owned nested Wayland/Vulkan; final GUI SIGTERM after shell exits."}


def execute_successive(qa):
    initial_prefix=os.environ.get("DATUM_QA_INITIAL_PREFIX")=="1"
    proof_prefix="initial-prefix" if initial_prefix else "successive"
    choices=qa.root/"choices"
    documents=choices/"Documents"
    datum=documents/"datum-eda"
    design=datum/"design-targets"
    for name in ["datum-eda", "datum-eda-backups", "datum-eda-workspaces"]:
        (documents/name).mkdir(parents=True)
    if initial_prefix:
        for name in ["datum_projects","plugdata"]:(documents/name).mkdir()
    design.mkdir()
    target=design/"successive-project"
    qa.start_envelope()
    main=qa.launch("successive-main")
    qa.terminal_command(main,'S_KEEP=preserved; printf "S_ORIGINAL:%s\\n" "$$"')
    original=int(qa.wait("original shell PID",lambda:re.search(r"S_ORIGINAL:(\d+)",qa.terminal_text())).group(1))
    raw="~/"+str(choices.relative_to(Path.home()))

    def observed_query(text,count):
        qa.wait("current successive query",lambda:value(qa,"Location")==text and "Reading" not in value(qa,"Folder navigation status")
                and any(n.get_role_name()=="list" and n.get_name().startswith("Folder suggestions in ") for n in qa.nodes())
                and len([n for n in qa.nodes() if n.get_role_name()=="list item"])==count)

    def suggestions(text,count):
        qa.fill_control(main,"Location",text)
        observed_query(text,count)

    def exact(text):
        qa.wait("explicit successive directory",lambda:value(qa,"Location")==text and "Reading" not in value(qa,"Folder navigation status")
                and any(n.get_role_name()=="list" and n.get_name().startswith("Files and folders in ") for n in qa.nodes()))

    def initial_contents(label):
        home=str(Path.home())
        qa.wait("initial strict Home contents without Browse/edit",lambda:value(qa,"Location")==home
                and "Reading" not in value(qa,"Folder navigation status")
                and any(n.get_role_name()=="list" and n.get_name()=="Files and folders in "+home for n in qa.nodes())
                and any(n.get_role_name()=="list item" for n in qa.nodes()))
        field="Project name" if label.endswith("-new") else "Location"
        assert qa.find(field).get_state_set().contains(Atspi.StateType.FOCUSED)
        assert qa.find("Browse for Location").get_state_set().contains(Atspi.StateType.EXPANDED)
        qa.observations[label+"_initial_contents"]={"location":home,"focus":field,"before_any_Location_edit_or_Browse":True}
        qa.capture(label+"-initial-home-contents",main)

    def chain(label):
        suggestions(raw+"/Doc",1)
        qa.key(main,"Tab");exact(raw+"/Documents/")
        fragment="dat" if initial_prefix else "datum-ed"
        shared="datum" if initial_prefix else "datum-eda"
        qa.text(main,fragment)
        observed_query(raw+"/Documents/"+fragment,5 if initial_prefix else 3)
        if initial_prefix:assert qa.find("plugdata") is not None
        qa.capture(label+"-prefix-matches",main)
        qa.key(main,"Tab")
        qa.wait("common-prefix refresh retains current suggestions",lambda:value(qa,"Location")==raw+"/Documents/"+shared and "Reading" not in value(qa,"Folder navigation status")
                and any(n.get_role_name()=="list" and n.get_name().startswith("Folder suggestions in ") for n in qa.nodes())
                and len([n for n in qa.nodes() if n.get_role_name()=="list item"])==(4 if initial_prefix else 3))
        if initial_prefix:assert qa.find("plugdata") is None
        assert qa.find("Location").get_state_set().contains(Atspi.StateType.FOCUSED)
        assert not qa.find("design-targets"),"prefix extension cannot enter child listing"
        qa.capture(label+("-datum-four-prefix-matches" if initial_prefix else "-exact-name-still-query"),main)
        qa.key(main,"Down")
        assert "selected" in qa.attributes(qa.find("datum-eda"))["value"]
        candidate=value(qa,"New Project folder" if label.endswith("-new") else "Project directory")
        expected=documents/shared
        if label.endswith("-new"):expected=expected/"successive-project"
        assert Path(os.path.expanduser(candidate))==expected
        qa.observations[label+"_exact_candidate_before_acceptance"]={"raw_location":value(qa,"Location"),"observed_preview":candidate,"expected_absolute":str(expected)}
        qa.key(main,"Tab" if label.endswith("-new") else "Return");exact(raw+"/Documents/datum-eda/")
        qa.text(main,"design")
        observed_query(raw+"/Documents/datum-eda/design",1)
        qa.key(main,"Tab");exact(raw+"/Documents/datum-eda/design-targets/")
        qa.capture(label+"-further-child-completed",main)

    qa.menu_item(main,"File","New Project")
    if initial_prefix:initial_contents(proof_prefix+"-new")
    qa.fill_control(main,"Project name","successive-project")
    chain(proof_prefix+"-new")
    qa.focus_control(main,"Create");qa.key(main,"Return")
    qa.wait("successive ordinary Create",lambda:(target/"project.json").exists() and not qa.find("New Project") and not qa.find("Open Project"))
    qa.capture("successive-new-created",main)
    before=qa.source_files(target)
    qa.menu_item(main,"File","Open Project")
    if initial_prefix:initial_contents(proof_prefix+"-open")
    chain(proof_prefix+"-open")
    # An explicitly loaded unsuffixed directory accepts only its separator.
    qa.fill_control(main,"Location",raw+"/Documents/datum-eda")
    qa.key(main,"Return");exact(str(datum))
    qa.key(main,"Down")
    qa.key(main,"Tab");exact(str(datum)+"/")
    assert value(qa,"Location")!=str(design)+"/"
    qa.capture("successive-open-explicit-loaded-tab-same-directory",main)
    suggestions(raw+"/Documents/datum-eda/design-targets/success",1)
    qa.key(main,"Tab");exact(raw+"/Documents/datum-eda/design-targets/successive-project/")
    qa.focus_control(main,"Open");qa.key(main,"Return")
    qa.wait("successive ordinary Open",lambda:not qa.find("Open Project"))
    qa.capture("successive-open-published",main)
    assert qa.source_files(target)==before
    qa.terminal_command(main,'printf "S_OPENED:%s\\n" "$$"',tab=2)
    opened=int(qa.wait("opened shell PID",lambda:re.search(r"S_OPENED:(\d+)",qa.terminal_text())).group(1))
    assert owned_shell_state(qa,opened,"successive_opened_shell")["cwd"]==str(target)
    qa.terminal_command(main,'printf "S_RETAIN:%s:%s\\n" "$$" "$S_KEEP"')
    qa.wait("original shell variable preserved",lambda:f"S_RETAIN:{original}:preserved" in qa.terminal_text())
    qa.capture("successive-original-session-preserved",main)
    for tab in [2,1,0]:qa.terminal_command(main,"exit",tab=tab)
    qa.wait("all owned PTYs closed",qa.terminal_ptys_closed)
    scope={"project":str(target),"raw_home_query":raw,"source_files_hash_set":before,
        "new_and_open_successive_components":True,"explicit_loaded_Tab_accepts_same_directory":True,
        "limit":"Owned Wayland/Vulkan; no owner data/process; shell exits then GUI SIGTERM, no full qualification"}
    if initial_prefix:
        scope.update(initial_contents_verified=True,dat_to_datum=True,prefix_priority_retains_four_siblings=True,
                     incidental_substring_visible_for_dat=True,incidental_substring_removed_after_datum_refresh=True)
    else:
        scope["LCP_exact_name_retains_three_siblings"]=True
    qa.observations["initial_prefix_scope" if initial_prefix else "successive_scope"]=scope
