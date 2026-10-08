//! Real owner-board-derived Project through production input dispatch and worker.
//! Programmatic callbacks are not physical Wayland input qualification.
use super::*;
use clap::Parser;
use std::{
    path::PathBuf,
    time::{Duration, Instant},
};
use winit::platform::wayland::EventLoopBuilderExtWayland;

fn screen(runtime: &mut Runtime, world: PointNm) -> (f32, f32) {
    let prepared = runtime.prepared_scene().unwrap();
    let v = prepared.scene_viewport;
    let origin = (v.x + 20.0, v.y + 20.0);
    let a = prepared
        .world_point_at_screen(origin.0, origin.1)
        .unwrap()
        .0;
    let b = prepared
        .world_point_at_screen(origin.0 + 100.0, origin.1 + 100.0)
        .unwrap()
        .0;
    (
        origin.0 + (world.x - a.x) as f32 * 100.0 / (b.x - a.x) as f32,
        origin.1 + (world.y - a.y) as f32 * 100.0 / (b.y - a.y) as f32,
    )
}
fn finish(runtime: &mut Runtime) {
    let started = Instant::now();
    while runtime.native_edit.pending() {
        runtime.poll_native_edit();
        assert!(
            started.elapsed() < Duration::from_secs(30),
            "edit worker completion"
        );
        std::thread::sleep(Duration::from_millis(5));
    }
    assert!(!runtime.native_edit.blocked, "qualified scene must publish");
    runtime.prepared_scene().expect("refreshed scene prepares");
}

#[test]
#[ignore = "requires owner-derived DATUM_FIRST_EDIT_PROJECT and native Wayland; mutates that owned Project"]
#[allow(deprecated)] // Bounded test host; ordinary app uses ActiveEventLoop.
fn wayland_first_edit_dispatch_preview_cancel_commit_history_save_reopen() {
    let root = PathBuf::from(std::env::var_os("DATUM_FIRST_EDIT_PROJECT").unwrap());
    let input = PathBuf::from(std::env::var_os("DATUM_FIRST_EDIT_ORIGINAL").unwrap());
    let original = std::fs::read(&input).unwrap();
    let board_path = root.join("board/board.json");
    let before = std::fs::read(&board_path).unwrap();
    let args =
        GuiArgs::try_parse_from(["datum-gui", "--project-root", root.to_str().unwrap()]).unwrap();
    let event_loop = winit::event_loop::EventLoop::<()>::with_user_event()
        .with_wayland()
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
    assert!(runtime.native_edit_available());
    let component = runtime
        .workspace()
        .scene
        .components
        .iter()
        .find(|c| {
            !c.locked
                && runtime
                    .workspace()
                    .scene
                    .pads
                    .iter()
                    .any(|p| p.component_uuid == c.component_uuid)
        })
        .unwrap()
        .clone();
    let request = runtime
        .workspace()
        .backing
        .as_ref()
        .unwrap()
        .request
        .clone();
    let count = runtime.workspace().scene.pads.len();
    let first_revision = runtime.workspace().scene.source_revision.clone();
    assert!(!runtime.workspace().scene.zones.is_empty());
    assert!(runtime.set_workspace_tool(WorkspaceTool::Move));
    runtime.last_cursor_pos = Some(screen(&mut runtime, component.position));
    assert!(
        !runtime.begin_native_selection_press(),
        "Move press must not begin region selection"
    );
    assert!(runtime.finish_native_selection_press().is_none());
    assert!(
        runtime.handle_primary_click(),
        "production click starts Move"
    );
    let preview = runtime
        .native_edit
        .preview
        .as_ref()
        .expect("eligible footprint hit through copper");
    assert_eq!(preview.component.component_uuid, component.component_uuid);
    let destination = PointNm {
        x: component.position.x + 1_000_000,
        y: component.position.y + 1_000_000,
    };
    let pointer = screen(&mut runtime, destination);
    assert!(runtime.handle_authoring_pointer_move(pointer));
    assert_eq!(
        std::fs::read(&board_path).unwrap(),
        before,
        "preview is byte-preserving"
    );
    assert_eq!(runtime.workspace().scene.pads.len(), count);
    assert!(runtime.dispatch_session_command(SessionCommand::CancelAuthoringGesture));
    assert_eq!(
        runtime
            .workspace()
            .scene
            .components
            .iter()
            .find(|c| c.component_uuid == component.component_uuid)
            .unwrap()
            .position,
        component.position
    );
    assert_eq!(std::fs::read(&board_path).unwrap(), before);

    runtime.last_cursor_pos = Some(screen(&mut runtime, component.position));
    runtime.handle_primary_click();
    let world = runtime
        .prepared_scene()
        .unwrap()
        .world_point_at_screen(pointer.0, pointer.1)
        .unwrap()
        .0;
    let preview = runtime.native_edit.preview.as_ref().unwrap();
    let expected = preview
        .destination(world, runtime.workspace().authoring.snap)
        .unwrap();
    let expected_scene = preview.scene_at(expected).unwrap();
    runtime.handle_authoring_pointer_move(pointer);
    runtime.last_cursor_pos = Some(pointer);
    assert!(!runtime.begin_native_selection_press());
    assert!(runtime.finish_native_selection_press().is_none());
    let started = Instant::now();
    assert!(runtime.handle_primary_click());
    assert!(
        runtime.native_edit.pending(),
        "dispatch queues instead of waiting for writer"
    );
    eprintln!(
        "first-edit commit dispatch {:?}; worker pending",
        started.elapsed()
    );
    finish(&mut runtime);
    let position = |r: &Runtime| {
        r.workspace()
            .scene
            .components
            .iter()
            .find(|c| c.component_uuid == component.component_uuid)
            .unwrap()
            .position
    };
    assert_eq!(position(&runtime), expected);
    assert_ne!(runtime.workspace().scene.source_revision, first_revision);
    assert_eq!(runtime.workspace().scene.pads.len(), count);
    for pad in expected_scene
        .pads
        .iter()
        .filter(|p| p.component_uuid == component.component_uuid)
    {
        let actual = runtime
            .workspace()
            .scene
            .pads
            .iter()
            .find(|p| p.object_id == pad.object_id)
            .unwrap();
        assert_eq!(actual.center, pad.center, "complete translated Pad centre");
        assert_eq!(
            actual.bounds, pad.bounds,
            "complete translated Pad aperture"
        );
    }
    let moved = std::fs::read(&board_path).unwrap();
    assert_ne!(moved, before);
    assert!(runtime.queue_native_history(false));
    finish(&mut runtime);
    assert_eq!(position(&runtime), component.position);
    assert_eq!(
        std::fs::read(&board_path).unwrap(),
        before,
        "undo exact source preimage"
    );
    assert!(runtime.queue_native_history(true));
    finish(&mut runtime);
    assert_eq!(position(&runtime), expected);
    assert_eq!(
        std::fs::read(&board_path).unwrap(),
        moved,
        "redo exact committed bytes"
    );
    assert!(runtime.queue_native_edit(NativeEditAction::Save));
    finish(&mut runtime);
    assert_eq!(std::fs::read(&board_path).unwrap(), moved);
    let reopened = load_board_editor_workspace_state(&request).unwrap();
    assert_eq!(
        reopened
            .scene
            .components
            .iter()
            .find(|c| c.component_uuid == component.component_uuid)
            .unwrap()
            .position,
        expected
    );
    assert_eq!(reopened.scene.pads.len(), count);
    assert_eq!(std::fs::read(input).unwrap(), original);
    eprintln!(
        "first-edit {}: cancel bytes exact; move {:?} -> {:?}; undo/redo exact; Save/reopen qualified; {} Pads; zones {}",
        component.reference,
        component.position,
        expected,
        count,
        reopened.scene.zones.len()
    );
    #[cfg(feature = "visual")]
    runtime
        .write_visual_screenshot(&root.join("first-edit-delivery.png"))
        .unwrap();
    runtime.begin_application_terminal_shutdown();
    let deadline = Instant::now() + Duration::from_secs(6);
    while !runtime.application_terminal_shutdown_complete() {
        assert!(Instant::now() < deadline, "owned terminal shutdown");
        runtime.poll_terminal_output();
        std::thread::sleep(Duration::from_millis(10));
    }
}
