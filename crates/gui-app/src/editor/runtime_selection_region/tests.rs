//! Authored labels admit region ownership before source-geometry refusal.
//! Hidden Runtime dispatch is supporting regression proof, not physical input.
use crate::{GuiArgs, Runtime, WorkspaceTool};
use clap::Parser;
use datum_gui_protocol::{SelectionSubject, SelectionTarget};
use datum_gui_render::HitTarget;
use std::{path::PathBuf, sync::Arc, time::Duration};
use winit::{keyboard::ModifiersState, platform::x11::EventLoopBuilderExtX11};

const FOOTPRINT: &str = "b27c9ea9-4f20-5e6f-8dea-fb6856aa0f1b";
const ZONE_REFUSAL: &str =
    "Certified Zone region geometry unavailable for 0aa7c34d-bae9-55b9-b951-538ad0bb7107";

fn copied_input() -> (PathBuf, Vec<(PathBuf, Vec<u8>)>) {
    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../test-harness/testdata/selection/s5a_v1/replay-inputs");
    let index: serde_json::Value =
        serde_json::from_slice(&std::fs::read(base.join("index.json")).unwrap()).unwrap();
    let root = std::env::temp_dir().join(format!(
        "datum-authored-region-{}-{}",
        std::process::id(),
        uuid::Uuid::new_v4()
    ));
    let prefix = "user-edit-selection/journal-read-project/";
    let mut canonical = Vec::new();
    for row in index["files"].as_array().unwrap() {
        let source = row["path"].as_str().unwrap();
        let Some(relative) = source.strip_prefix(prefix) else {
            continue;
        };
        let bytes = std::fs::read(base.join("native").join(source)).unwrap();
        let path = root.join(relative);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, &bytes).unwrap();
        canonical.push((path, bytes));
    }
    assert_eq!(
        canonical.len(),
        149,
        "complete existing source/history input"
    );
    (root, canonical)
}

fn present(runtime: &mut Runtime) {
    let deadline = std::time::Instant::now() + Duration::from_secs(10);
    while !runtime.render().unwrap() {
        assert!(!runtime.device_health.failed());
        assert!(std::time::Instant::now() < deadline, "Runtime presentation");
        runtime.device.poll(wgpu::PollType::Poll).unwrap();
        std::thread::sleep(Duration::from_millis(10));
    }
}

fn authored_label(runtime: &Runtime, point: (f32, f32)) {
    assert!(matches!(
        runtime.presented_hits.hit_test(point.0, point.1),
        Some(HitTarget::AuthoredObject(key)) if key == &format!("component:{FOOTPRINT}")
    ));
}

fn preserved_selection(before: &SelectionTarget, after: &SelectionTarget) {
    let mut expected = before.clone();
    if let (SelectionTarget::Native(view), SelectionTarget::Native(actual)) = (&mut expected, after)
    {
        view.notice = actual.notice.clone();
    }
    assert_eq!(after, &expected, "only the refusal notice may change");
}

fn frozen_basis(runtime: &mut Runtime) {
    assert_eq!((runtime.config.width, runtime.config.height), (1344, 806));
    let bounds = &runtime.workspace().scene.bounds;
    assert_eq!(
        (bounds.min_x, bounds.min_y, bounds.max_x, bounds.max_y),
        (133050000, 77145786, 172565000, 113100000)
    );
    assert_eq!(
        (
            runtime.camera.center_x_nm,
            runtime.camera.center_y_nm,
            runtime.camera.zoom
        ),
        (152807504.0, 95122896.0, 1.0)
    );
    let prepared = runtime.prepared_scene().unwrap();
    let viewport = prepared.scene_viewport;
    assert_eq!(
        (viewport.x, viewport.y, viewport.width, viewport.height),
        (240.0, 75.0, 379.5, 657.0)
    );
    for (screen, expected) in [
        ((351.0, 557.0), (144151575, 111995088)),
        ((356.0, 562.0), (144701158, 112544671)),
        ((347.0, 559.0), (143711909, 112214922)),
        ((352.0, 564.0), (144261491, 112764504)),
    ] {
        let world = prepared
            .world_point_for_pane_unclipped(datum_gui_protocol::PaneId(0), screen.0, screen.1)
            .unwrap();
        assert_eq!((world.x, world.y), expected, "frozen pane conversion");
    }
}

fn drag(
    runtime: &mut Runtime,
    start: (f32, f32),
    end: (f32, f32),
    modifiers: ModifiersState,
) -> String {
    present(runtime);
    frozen_basis(runtime);
    authored_label(runtime, start);
    runtime.modifiers = modifiers;
    runtime.last_cursor_pos = Some(start);
    runtime.selection_release_suppressed = false;
    assert!(
        runtime.begin_native_selection_press(),
        "authored label admits press"
    );
    runtime.last_cursor_pos = Some(end);
    assert!(runtime.advance_native_selection(end), "motion exceeds 4px");
    assert_eq!(
        runtime.selection_gesture.as_ref().unwrap().kind,
        Some(datum_gui_viewport::selection_gesture::RegionKind::Rectangle)
    );
    assert_eq!(runtime.finish_native_selection_press(), Some(true));
    let deadline = std::time::Instant::now() + Duration::from_secs(10);
    while runtime.selection_evaluation.is_some() {
        runtime.poll_native_selection_tick();
        assert!(std::time::Instant::now() < deadline, "region evaluation");
    }
    let subject = match &runtime.workspace().selection {
        SelectionTarget::Native(view) => view.subject.kind(),
        SelectionTarget::None => "none",
        _ => "legacy",
    };
    eprintln!(
        "region {start:?}->{end:?}, modifiers={modifiers:?}, subject={subject}, latest_notice={:?}",
        runtime
            .workspace()
            .ui
            .console
            .records()
            .next_back()
            .map(|record| &record.message)
    );
    assert!(runtime.selection_gesture.is_none());
    runtime.modifiers = ModifiersState::empty();
    runtime
        .workspace()
        .ui
        .console
        .records()
        .next_back()
        .expect("region refusal remains visible")
        .message
        .clone()
}

#[test]
#[ignore = "requires X11/Vulkan and pinned CLI; run serially"]
#[allow(deprecated)] // Existing hidden Runtime host, not ordinary Wayland proof.
fn authored_label_drag_arms_region_and_preserves_atomic_refusal() {
    assert!(std::env::var_os("EDA_CLI_BIN").is_some(), "pinned CLI");
    let (root, canonical) = copied_input();
    let args =
        GuiArgs::try_parse_from(["datum-gui", "--project-root", root.to_str().unwrap()]).unwrap();
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
                    .with_inner_size(winit::dpi::PhysicalSize::new(1344, 806)),
            )
            .unwrap(),
    );
    let launch = args.load_launch_state(Some(wake.clone())).unwrap();
    let mut runtime = pollster::block_on(Runtime::new(window, launch, Some(1.0), wake)).unwrap();
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        assert_eq!(
            runtime.workspace().scene.source_revision,
            "031555f5310981cbf23f651b8fff3a8443ef9ca22528dd668e305f7f9da073bb"
        );
        assert!(matches!(
            runtime.workspace().selection,
            SelectionTarget::None
        ));
        present(&mut runtime);
        frozen_basis(&mut runtime);
        let filters = runtime.workspace().ui.filters.clone();
        let layout = runtime.workspace().ui.layout.clone();
        eprintln!(
            "current Zone fill projections: {:?}",
            runtime
                .workspace()
                .scene
                .zones
                .iter()
                .map(|zone| {
                    let value = serde_json::to_value(&zone.fill_projection).unwrap();
                    (
                        zone.zone_uuid.clone(),
                        value["state"].clone(),
                        value["value"].as_str().map(str::to_owned),
                    )
                })
                .collect::<Vec<_>>()
        );
        assert_eq!(runtime.workspace().scene.zones.len(), 18);
        assert!(runtime.workspace().scene.zones.iter().all(|zone| {
            serde_json::to_value(&zone.fill_projection).unwrap()["state"] == "unavailable"
        }));
        authored_label(&runtime, (351.0, 557.0));

        // Actual in-canvas menu chrome still wins over region acquisition.
        runtime.last_cursor_pos = Some((351.0, 557.0));
        assert!(runtime.open_marking_menu_at_cursor());
        present(&mut runtime);
        let card = runtime.presented_hits.regions().iter().find(|hit| {
            matches!(&hit.target, HitTarget::MarkingMenuItem { slot, .. } if slot == "SELECT")
        }).unwrap().rect;
        runtime.last_cursor_pos = Some((card.x + card.width * 0.5, card.y + card.height * 0.5));
        runtime.modifiers = ModifiersState::SHIFT;
        assert!(!runtime.begin_native_selection_press());
        assert!(runtime.selection_gesture.is_none());
        assert!(runtime.dismiss_marking_menu());

        // Tool, authoring and conflicting-modifier gates remain in force.
        runtime.last_cursor_pos = Some((351.0, 557.0));
        assert!(runtime.set_workspace_tool(WorkspaceTool::Move));
        assert!(!runtime.begin_native_selection_press());
        assert!(runtime.set_workspace_tool(WorkspaceTool::Select));
        runtime.session.workspace_mut().schematic_authoring.picker = true;
        assert!(!runtime.begin_native_selection_press());
        runtime.session.workspace_mut().schematic_authoring.picker = false;
        runtime.modifiers = ModifiersState::SHIFT | ModifiersState::CONTROL;
        assert!(!runtime.begin_native_selection_press());
        assert_eq!(runtime.workspace().ui.filters, filters);
        assert_eq!(runtime.workspace().ui.layout, layout);
        frozen_basis(&mut runtime);

        // Frozen native01 coordinates now prove routing and atomic refusal only.
        let notice = drag(
            &mut runtime,
            (351.0, 557.0),
            (356.0, 562.0),
            ModifiersState::SHIFT,
        );
        assert_eq!(notice, ZONE_REFUSAL);
        assert!(matches!(
            runtime.workspace().selection,
            SelectionTarget::None
        ));
        let notice = drag(
            &mut runtime,
            (347.0, 559.0),
            (352.0, 564.0),
            ModifiersState::SHIFT,
        );
        assert_eq!(notice, ZONE_REFUSAL);
        assert!(matches!(
            runtime.workspace().selection,
            SelectionTarget::None
        ));

        // Sub-threshold motion still falls through to the ordinary point click.
        present(&mut runtime);
        runtime.last_cursor_pos = Some((351.0, 557.0));
        runtime.selection_release_suppressed = false;
        runtime.modifiers = ModifiersState::empty();
        assert!(runtime.begin_native_selection_press());
        runtime.last_cursor_pos = Some((353.0, 558.0));
        assert!(!runtime.advance_native_selection((353.0, 558.0)));
        assert_eq!(runtime.finish_native_selection_press(), None);
        assert!(runtime.handle_primary_click());
        let SelectionTarget::Native(view) = &runtime.workspace().selection else {
            panic!("point click must retain its Object behavior");
        };
        assert!(matches!(view.subject, SelectionSubject::Object(_)));
        assert_eq!(view.focus().unwrap().id.to_string(), FOOTPRINT);

        // Control refusal preserves the existing Object and its member focus.
        let previous = runtime.workspace().selection.clone();
        let notice = drag(
            &mut runtime,
            (347.0, 559.0),
            (352.0, 564.0),
            ModifiersState::CONTROL,
        );
        assert_eq!(notice, ZONE_REFUSAL);
        preserved_selection(&previous, &runtime.workspace().selection);
        // Plain drag remains reserved/inert rather than becoming a point click.
        drag(
            &mut runtime,
            (351.0, 557.0),
            (356.0, 562.0),
            ModifiersState::empty(),
        );
        preserved_selection(&previous, &runtime.workspace().selection);
    }));

    runtime.begin_application_terminal_shutdown();
    let deadline = std::time::Instant::now() + Duration::from_secs(6);
    while !runtime.application_terminal_shutdown_complete() {
        runtime.poll_terminal_output();
        assert!(
            std::time::Instant::now() < deadline,
            "owned terminal shutdown"
        );
        std::thread::sleep(Duration::from_millis(10));
    }
    drop(runtime);
    for (path, bytes) in canonical {
        assert_eq!(
            std::fs::read(path).unwrap(),
            bytes,
            "canonical input/history unchanged"
        );
    }
    if let Err(error) = result {
        eprintln!("failed regression input retained at {}", root.display());
        std::panic::resume_unwind(error);
    }
    std::fs::remove_dir_all(root).unwrap();
}
