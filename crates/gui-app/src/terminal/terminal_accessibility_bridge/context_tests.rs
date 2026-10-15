//! Current live context follows committed selection for an existing owned PTY.
use super::*;

#[test]
#[ignore = "requires X11/Vulkan and pinned CLI; run serially"]
#[allow(deprecated)] // Hidden integration host; production uses ActiveEventLoop.
fn existing_terminal_public_get_tracks_selection_scope_clear_and_activation() {
    use clap::Parser;
    use datum_gui_protocol::{
        AuthoredSelectionClass as Class, AuthoredSelectionIdentity, SelectionSubject,
        SelectionTarget,
    };
    use winit::platform::x11::EventLoopBuilderExtX11;

    let fixture = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../engine/testdata/golden/text/native/text-transform-repro");
    let root = std::env::temp_dir().join(format!(
        "selection-live-context-{}-{}",
        std::process::id(),
        eda_engine::substrate::ObjectId::new_v4()
    ));
    let paths = [
        "project.json",
        "board/board.json",
        "schematic/schematic.json",
        "rules/rules.json",
    ];
    for rel in paths {
        let path = root.join(rel);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::copy(fixture.join(rel), path).unwrap();
    }
    let board: serde_json::Value =
        serde_json::from_slice(&std::fs::read(root.join("board/board.json")).unwrap()).unwrap();
    let mut expected = std::collections::BTreeSet::from([AuthoredSelectionIdentity {
        class: Class::BoardOutline,
        id: serde_json::from_value(board["uuid"].clone()).unwrap(),
        instance_path: vec![],
    }]);
    for text in board["texts"].as_array().unwrap() {
        expected.insert(AuthoredSelectionIdentity {
            class: Class::BoardText,
            id: serde_json::from_value(text["uuid"].clone()).unwrap(),
            instance_path: vec![],
        });
    }
    assert_eq!(expected.len(), 9);
    let event_loop = winit::event_loop::EventLoop::<()>::with_user_event()
        .with_x11()
        .with_any_thread(true)
        .build()
        .unwrap();
    let wake = event_loop.create_proxy();
    let window = Arc::new(
        event_loop
            .create_window(
                winit::window::Window::default_attributes()
                    .with_visible(false)
                    .with_inner_size(winit::dpi::PhysicalSize::new(1280, 800)),
            )
            .unwrap(),
    );
    let args = crate::GuiArgs::try_parse_from([
        "datum-gui",
        "--project-root",
        root.to_str().unwrap(),
        "--focus-pane",
        "board",
    ])
    .unwrap();
    let launch = args.load_launch_state(Some(wake.clone())).unwrap();
    let mut runtime =
        pollster::block_on(crate::Runtime::new(window, launch, Some(1.0), wake)).unwrap();
    runtime.terminal_accessibility = LinuxTerminalAccessibilityBridge::without_platform();
    let canonical: Vec<_> = paths
        .iter()
        .map(|rel| std::fs::read(root.join(rel)).unwrap())
        .collect();
    let journal_path = eda_engine::substrate::transaction_journal_path(&root);
    let journal = std::fs::read(&journal_path).ok();
    let head_path = root.join(".datum/revision/v1/head.json");
    let head = std::fs::read(&head_path).ok();
    let first = runtime.terminal_sessions.active().session_id.clone();
    let context_id = runtime.terminal_sessions.active().context_id.clone();
    let live_path = runtime.terminal_sessions.active().context_path.clone();
    let pinned_path = live_path.with_file_name(format!("{context_id}.pinned.json"));
    let pinned = std::fs::read(&pinned_path).unwrap();
    let initial = public_get(&root, &first);
    assert_eq!(initial["selection_context"]["kind"], "none");
    assert!(initial["selection_context"].get("details").is_none());
    assert!(runtime.spawn_terminal_session_tab());
    let second = finish_pending_tab(&mut runtime);
    assert!(runtime.activate_terminal_session(&first));
    assert!(runtime.native_select_all());
    let SelectionTarget::Native(view) = &runtime.workspace().selection else {
        panic!("native selection required")
    };
    assert_eq!(view.members, expected);
    let SelectionSubject::Compound(compound) = &view.subject else {
        panic!("complete selection required")
    };
    assert_eq!(compound.members, expected);
    assert_eq!(compound.focus, None);
    assert_current(&runtime, &root, &first, &context_id);
    assert!(runtime.set_selection_scope(Some(Class::BoardText)));
    let scoped = assert_current(&runtime, &root, &first, &context_id);
    assert_eq!(
        scoped["selection_context"]["details"]["scope"],
        "board_text"
    );
    assert_eq!(scoped["selection_context"]["details"]["total"], 9);
    // The second existing session was not silently rebound by publication to the first.
    assert_eq!(
        public_get(&root, &second)["selection_context"]["kind"],
        "none"
    );
    assert!(runtime.activate_terminal_session(&second));
    let second_context = runtime.terminal_sessions.active().context_id.clone();
    assert_current(&runtime, &root, &second, &second_context);
    assert!(runtime.activate_terminal_session(&first));
    let before = std::fs::metadata(&live_path).unwrap().modified().unwrap();
    let bytes = std::fs::read(&live_path).unwrap();
    let pane = runtime.workspace().ui.layout.focused;
    runtime.set_application_focus(ApplicationFocus::Terminal);
    runtime.set_application_focus(ApplicationFocus::Editor(pane));
    runtime.session.workspace_mut().ui.selection_region =
        Some(datum_gui_protocol::SelectionRegionFeedback {
            evaluating: Some(3),
            pane,
            path: vec![],
            subtractive: false,
            phase: 0,
        });
    runtime.refresh_selection_accessibility();
    runtime.session.workspace_mut().ui.selection_region = None;
    runtime.refresh_prepared_selection_accessibility();
    assert_eq!(
        std::fs::metadata(&live_path).unwrap().modified().unwrap(),
        before
    );
    assert!(
        std::fs::read(&live_path).unwrap() == bytes,
        "unchanged emitted context must not rewrite owned live metadata"
    );
    // A failed publisher must not advance its successful baseline. Entry retries.
    let saved = live_path.with_extension("saved-live");
    std::fs::rename(&live_path, &saved).unwrap();
    std::fs::create_dir(&live_path).unwrap();
    let old = runtime
        .terminal_sessions
        .active_origin_context()
        .unwrap()
        .selection_context
        .clone();
    assert!(runtime.set_selection_scope(None));
    assert_eq!(
        runtime
            .terminal_sessions
            .active_origin_context()
            .unwrap()
            .selection_context,
        old
    );
    std::fs::remove_dir(&live_path).unwrap();
    std::fs::rename(&saved, &live_path).unwrap();
    runtime.set_application_focus(ApplicationFocus::Terminal);
    let retried = assert_current(&runtime, &root, &first, &context_id);
    assert!(retried["selection_context"]["details"]["scope"].is_null());
    // Clearing the target also publishes its current nonnative transport shape.
    assert!(runtime.dispatch_session_command(datum_gui_protocol::SessionCommand::ClearSelection));
    let cleared = assert_current(&runtime, &root, &first, &context_id);
    assert_eq!(cleared["selection_context"]["kind"], "none");
    assert!(cleared["selection_context"].get("details").is_none());
    assert!(
        std::fs::read(&pinned_path).unwrap() == pinned,
        "selection publication must preserve pinned metadata"
    );
    // A controlled foreign-root child retains its origin on current selection changes.
    let foreign = root.join("foreign-origin");
    std::fs::create_dir_all(&foreign).unwrap();
    let mut foreign_launch = runtime.terminal_launch_context.clone().unwrap();
    foreign_launch.project_root = foreign.clone();
    foreign_launch.launch_working_directory = foreign;
    runtime
        .terminal_sessions
        .begin_spawn_and_activate(
            &foreign_launch,
            &mut runtime.session.workspace_mut().ui.terminal,
        )
        .unwrap();
    let foreign_id = finish_pending_tab(&mut runtime);
    let foreign_path = runtime.terminal_sessions.active().context_path.clone();
    let foreign_before = std::fs::read(&foreign_path).unwrap();
    assert!(runtime.native_select_all());
    assert!(runtime.activate_terminal_session(&foreign_id));
    runtime.set_application_focus(ApplicationFocus::Terminal);
    assert!(
        std::fs::read(&foreign_path).unwrap() == foreign_before,
        "current selection must preserve foreign-origin metadata"
    );
    assert_eq!(
        runtime
            .terminal_sessions
            .active_origin_context()
            .unwrap()
            .project_root,
        foreign_launch.project_root
    );
    for (rel, bytes) in paths.iter().zip(&canonical) {
        assert_eq!(&std::fs::read(root.join(rel)).unwrap(), bytes);
    }
    assert_eq!(std::fs::read(&journal_path).ok(), journal);
    assert_eq!(std::fs::read(&head_path).ok(), head);
    drop(runtime);
    std::fs::remove_dir_all(root).unwrap();
}

fn public_get(root: &std::path::Path, session: &str) -> serde_json::Value {
    let cli = std::env::var_os("EDA_CLI_BIN").expect("pinned CLI required; no nested Cargo");
    let mut command = std::process::Command::new(cli);
    for (name, _) in std::env::vars_os() {
        if name.to_string_lossy().starts_with("DATUM_") {
            command.env_remove(name);
        }
    }
    let output = command
        .args([
            "--format",
            "json",
            "context",
            "get",
            "--session",
            session,
            "--project-root",
        ])
        .arg(root)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "public Get failed; raw output discarded"
    );
    let mut raw: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("public Get must return valid JSON");
    let object = raw
        .as_object_mut()
        .expect("public Get must return an object");
    let mut safe = serde_json::Map::new();
    for key in [
        "project_root",
        "project_id",
        "model_revision",
        "session_id",
        "terminal_session_id",
        "context_id",
        "selection_context",
    ] {
        safe.insert(
            key.into(),
            object.remove(key).expect("required public field missing"),
        );
    }
    serde_json::Value::Object(safe)
}

fn assert_current(
    runtime: &Runtime,
    root: &std::path::Path,
    session: &str,
    context: &str,
) -> serde_json::Value {
    let value = public_get(root, session);
    assert_eq!(value["project_root"], root.to_str().unwrap());
    assert_eq!(value["project_id"], runtime.workspace().scene.project_uuid);
    assert_eq!(
        value["model_revision"],
        runtime.workspace().scene.source_revision
    );
    assert_eq!(value["session_id"], session);
    assert_eq!(value["terminal_session_id"], session);
    assert_eq!(value["context_id"], context);
    assert_eq!(
        value["selection_context"],
        serde_json::to_value(datum_gui_protocol::DatumSelectionContext::from_selection(
            &runtime.workspace().selection
        ))
        .unwrap()
    );
    value
}

fn finish_pending_tab(runtime: &mut Runtime) -> String {
    for _ in 0..100 {
        runtime
            .terminal_sessions
            .complete_pending_spawns(&mut runtime.session.workspace_mut().ui.terminal);
        runtime
            .terminal_sessions
            .drain_all(&mut runtime.session.workspace_mut().ui.terminal);
        if runtime.terminal_sessions.active_attached() {
            return runtime.terminal_sessions.active().session_id.clone();
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    panic!("owned test tab did not attach");
}
