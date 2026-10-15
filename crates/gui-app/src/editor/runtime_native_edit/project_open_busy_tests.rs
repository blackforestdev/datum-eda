#[path = "../../../../test-harness/src/development_workspace.rs"]
mod development_workspace;
use super::*;
use clap::Parser;
use std::{
    path::PathBuf,
    time::{Duration, Instant},
};
use winit::platform::x11::EventLoopBuilderExtX11;

fn canonical_sources(root: &std::path::Path) -> Vec<Vec<u8>> {
    [
        "project.json",
        "board/board.json",
        "schematic/schematic.json",
        "rules/rules.json",
    ]
    .iter()
    .map(|rel| std::fs::read(root.join(rel)).unwrap())
    .collect()
}

fn canonical_history(root: &std::path::Path) -> std::collections::BTreeMap<PathBuf, Vec<u8>> {
    fn walk(
        root: &std::path::Path,
        directory: &std::path::Path,
        result: &mut std::collections::BTreeMap<PathBuf, Vec<u8>>,
    ) {
        if !directory.exists() {
            return;
        }
        for entry in std::fs::read_dir(directory).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                walk(root, &path, result);
            } else if path != root.join(".datum/revision/writer.lock") {
                result.insert(
                    path.strip_prefix(root).unwrap().to_owned(),
                    std::fs::read(path).unwrap(),
                );
            }
        }
    }
    let mut result = std::collections::BTreeMap::new();
    walk(root, &root.join(".datum/journal"), &mut result);
    walk(root, &root.join(".datum/revision"), &mut result);
    result
}

fn camera_projection(runtime: &Runtime) -> Vec<String> {
    let panes = runtime
        .current_layout()
        .viewport_panes(&runtime.workspace().ui.layout);
    panes
        .panes
        .iter()
        .map(|pane| {
            let camera = if panes.scene_leaf_id() == Some(pane.id) {
                Some(runtime.camera)
            } else {
                runtime.pane_cameras.camera(pane.id, pane.content)
            };
            format!(
                "{:?}:{:?}",
                pane.id,
                camera.map(|c| (c.center_x_nm, c.center_y_nm, c.zoom))
            )
        })
        .collect()
}

#[test]
#[allow(deprecated)] // Integration host; the ordinary App uses ActiveEventLoop.
fn actual_canonical_worker_refuses_open_preserves_receiver_then_retries_destination() {
    let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../engine/testdata/golden/text/native/text-transform-repro");
    let base = development_workspace::path("open-project-actual-worker-tests")
        .join(std::process::id().to_string());
    let roots = [base.join("origin"), base.join("copied destination")];
    for root in &roots {
        for rel in [
            "project.json",
            "board/board.json",
            "schematic/schematic.json",
            "rules/rules.json",
        ] {
            let path = root.join(rel);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::copy(fixture.join(rel), path).unwrap();
        }
    }
    let roots = roots.map(|root| root.canonicalize().unwrap());
    let destination_before = canonical_sources(&roots[1]);
    let origin_before = canonical_sources(&roots[0]);
    let args = GuiArgs::try_parse_from([
        "datum-gui",
        "--project-root",
        roots[0].to_str().unwrap(),
        "--focus-pane",
        "board",
    ])
    .unwrap();
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
    let launch = args.load_launch_state(Some(wake.clone())).unwrap();
    let mut runtime = pollster::block_on(Runtime::new(window, launch, Some(1.0), wake)).unwrap();
    runtime.native_select_all();
    let datum_gui_protocol::SelectionTarget::Native(selected) = &runtime.workspace().selection
    else {
        panic!("actual Board authored selection required");
    };
    assert_eq!(selected.members.len(), 9);
    let original_selection = format!("{:?}", runtime.workspace().selection);
    let original_layout = runtime.workspace().ui.layout.clone();
    let original_cameras = camera_projection(&runtime);
    let original_focus = runtime.workspace().ui.focus.clone();
    let origin_history_before = canonical_history(&roots[0]);
    let destination_history_before = canonical_history(&roots[1]);
    let origin_generation = runtime.project_open.session_generation;
    let original_terminal = runtime.terminal_sessions.active().session_id().to_owned();
    // Hold the SAME existing native client mutex used by the actual spawned worker.
    // The request/receiver is real; no synthetic pending flag or answer injection.
    let client = runtime.native_edit.client.clone();
    let gate = client.lock().unwrap();
    assert!(
        gate.is_some(),
        "startup read must establish the actual companion client"
    );
    runtime.queue_native_edit(NativeEditAction::Save);
    assert!(runtime.native_edit.pending());
    let receiver = runtime.native_edit.pending.as_ref().unwrap() as *const _;
    runtime.open_project();
    runtime.session.workspace_mut().ui.project_open.directory = roots[1].display().to_string();
    runtime.begin_project_open();
    assert!(
        runtime
            .workspace()
            .ui
            .project_open
            .refusal
            .as_deref()
            .unwrap()
            .contains("Wait for the current Project edit")
    );
    assert!(runtime.native_edit.pending());
    assert_eq!(
        runtime.native_edit.pending.as_ref().unwrap() as *const _,
        receiver
    );
    assert!(!runtime.poll_native_edit());
    assert_eq!(
        format!("{:?}", runtime.workspace().selection),
        original_selection
    );
    assert_eq!(runtime.workspace().ui.layout, original_layout);
    assert_eq!(camera_projection(&runtime), original_cameras);
    assert_eq!(runtime.workspace().ui.focus, ApplicationFocus::Overlay);
    runtime.close_project_open();
    assert_eq!(runtime.workspace().ui.focus, original_focus);
    runtime.open_project();
    runtime.session.workspace_mut().ui.project_open.directory = roots[1].display().to_string();
    assert_eq!(canonical_history(&roots[0]), origin_history_before);
    assert_eq!(canonical_history(&roots[1]), destination_history_before);

    assert_eq!(
        runtime
            .workspace()
            .backing
            .as_ref()
            .unwrap()
            .request
            .project_root,
        roots[0]
    );
    assert_eq!(runtime.project_open.session_generation, origin_generation);
    assert_eq!(
        runtime.terminal_sessions.active().session_id(),
        original_terminal
    );
    assert_eq!(canonical_sources(&roots[1]), destination_before);
    drop(gate);
    let deadline = Instant::now() + Duration::from_secs(30);
    while runtime.native_edit.pending() && Instant::now() < deadline {
        runtime.poll_native_edit();
        std::thread::sleep(Duration::from_millis(5));
    }
    assert!(!runtime.native_edit.pending());
    assert!(!runtime.native_edit.blocked);
    assert_eq!(
        runtime
            .workspace()
            .ui
            .console
            .records()
            .next_back()
            .unwrap()
            .message,
        "Project saved"
    );
    assert_eq!(
        runtime
            .workspace()
            .backing
            .as_ref()
            .unwrap()
            .request
            .project_root,
        roots[0]
    );
    assert_eq!(canonical_sources(&roots[0]), origin_before);
    assert_eq!(canonical_sources(&roots[1]), destination_before);
    assert!(roots[0].join(".datum/journal/cursor.json").exists());
    assert!(!roots[1].join(".datum/journal/cursor.json").exists());
    let origin_history_after_save = canonical_history(&roots[0]);
    for (path, bytes) in &origin_history_before {
        assert_eq!(origin_history_after_save.get(path), Some(bytes));
    }
    assert!(
        origin_history_after_save
            .keys()
            .all(|path| origin_history_before.contains_key(path)
                || path == std::path::Path::new(".datum/journal/cursor.json"))
    );
    runtime.begin_project_open();
    assert!(runtime.workspace().ui.project_open.busy);
    let deadline = Instant::now() + Duration::from_secs(30);
    while runtime.workspace().ui.project_open.busy && Instant::now() < deadline {
        runtime.poll_project_open();
        std::thread::sleep(Duration::from_millis(5));
    }
    assert!(!runtime.workspace().ui.project_open.busy);
    assert_eq!(
        runtime
            .workspace()
            .backing
            .as_ref()
            .unwrap()
            .request
            .project_root
            .canonicalize()
            .unwrap(),
        roots[1].canonicalize().unwrap()
    );
    assert_eq!(
        runtime.project_open.session_generation,
        origin_generation + 1
    );
    assert!(matches!(
        runtime.workspace().selection,
        datum_gui_protocol::SelectionTarget::None
    ));
    assert_ne!(
        runtime.terminal_sessions.active().session_id(),
        original_terminal
    );
    assert_eq!(canonical_history(&roots[1]), destination_history_before);
    assert_eq!(canonical_history(&roots[0]), origin_history_after_save);
    assert_eq!(canonical_sources(&roots[1]), destination_before);
    let deadline =
        Instant::now() + Duration::from_millis(crate::terminal_transport::GLOBAL_SHUTDOWN_MS);
    runtime.terminal_sessions.terminate_all_by(deadline);
    while !runtime.terminal_sessions.all_sessions_closed() && Instant::now() < deadline {
        runtime.poll_terminal_output();
        std::thread::sleep(Duration::from_millis(5));
    }
    assert!(runtime.terminal_sessions.all_sessions_closed());
}
