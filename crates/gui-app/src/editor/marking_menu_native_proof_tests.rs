//! MM-C04 native proof: the inert marking-menu shell renders through the running
//! wgpu Runtime, is navigated by real pointer input, and commits NO design change.
//! Everything here is programmatically captured (windowed Runtime + encode_capture
//! -> PNG, validated by scripts/native_capture_pixels.py). The only owner residual,
//! scoped out explicitly and never simulated, is the literal human visual verdict
//! of the captured frames (PM060 class).
//!
//! #[ignore]d: requires X11/Vulkan, a pinned CLI, and the `visual` feature. Run:
//!   EDA_CLI_BIN=$PWD/target/debug/datum-eda \
//!   cargo test -p datum-gui-app --features visual -- --ignored marking_menu_native_proof
use crate::{GuiArgs, Runtime};
use clap::Parser;
use std::{path::PathBuf, sync::Arc, time::Duration};
use winit::platform::x11::EventLoopBuilderExtX11;

/// Copy the canonical selection replay project into a temp root (same fixture the
/// region proof uses), returning its root.
fn copied_project_root() -> PathBuf {
    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../test-harness/testdata/selection/s5a_v1/replay-inputs");
    let index: serde_json::Value =
        serde_json::from_slice(&std::fs::read(base.join("index.json")).unwrap()).unwrap();
    let root = std::env::temp_dir().join(format!(
        "datum-mm-c04-{}-{}",
        std::process::id(),
        uuid::Uuid::new_v4()
    ));
    let prefix = "user-edit-selection/journal-read-project/";
    for row in index["files"].as_array().unwrap() {
        let source = row["path"].as_str().unwrap();
        let Some(relative) = source.strip_prefix(prefix) else {
            continue;
        };
        let bytes = std::fs::read(base.join("native").join(source)).unwrap();
        let path = root.join(relative);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, &bytes).unwrap();
    }
    root
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

fn new_runtime(root: &std::path::Path) -> Runtime {
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
    pollster::block_on(Runtime::new(window, launch, Some(1.0), wake)).unwrap()
}

fn marking_hit(runtime: &Runtime, slot: &str) -> Option<(f32, f32)> {
    runtime.presented_hits.regions().iter().find_map(|region| {
        matches!(
            &region.target,
            datum_gui_render::HitTarget::MarkingMenuItem { slot: found, .. } if found == slot
        )
        .then(|| {
            (
                region.rect.x + region.rect.width * 0.5,
                region.rect.y + region.rect.height * 0.5,
            )
        })
    })
}

fn marking_surface_center(runtime: &Runtime) -> Option<(f32, f32)> {
    runtime.presented_hits.regions().iter().find_map(|region| {
        matches!(
            &region.target,
            datum_gui_render::HitTarget::MarkingMenuSurface { .. }
        )
        .then(|| {
            (
                region.rect.x + region.rect.width * 0.5,
                region.rect.y + region.rect.height * 0.5,
            )
        })
    })
}

fn capture(runtime: &mut Runtime, dir: &std::path::Path, name: &str) -> PathBuf {
    let path = dir.join(format!("{name}.png"));
    runtime
        .write_visual_screenshot(&path)
        .unwrap_or_else(|error| panic!("capture {name}: {error}"));
    let bytes = std::fs::metadata(&path).expect("capture file").len();
    assert!(
        bytes > 1000,
        "capture {name} must be a real frame ({bytes} bytes)"
    );
    path
}

// MM-C04 full matrix: drive the running inert shell through its four states with
// real pointer input, capture each natively, and prove the whole traversal commits
// no design change. The literal human visual verdict of these PNGs is the only
// owner residual (scoped out here, never simulated).
#[test]
#[ignore = "requires X11/Vulkan, pinned CLI, and --features visual; run serially"]
#[allow(deprecated)]
fn marking_menu_native_proof_four_states_navigate_inert_no_mutation() {
    assert!(
        std::env::var_os("EDA_CLI_BIN").is_some(),
        "pinned CLI required"
    );
    let root = copied_project_root();
    let mut runtime = new_runtime(&root);
    present(&mut runtime);

    let before_rev = runtime.workspace().scene.source_revision.clone();
    let before_sel = runtime.workspace().selection.clone();

    let dir = std::env::temp_dir().join(format!("mm-c04-states-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();

    // Real invocation: a right-click opens the contextual marking menu on the
    // running surface (proves the real entry path, not just set state).
    runtime.last_cursor_pos = Some((351.0, 557.0));
    runtime.open_marking_menu_at_cursor();
    present(&mut runtime);
    assert!(
        runtime.workspace().ui.marking_menu.is_some(),
        "right-click opens the marking-menu shell on the running surface"
    );
    let _real_open = capture(&mut runtime, &dir, "0-real-right-click-open");
    runtime.dismiss_marking_menu();

    // STATE 1 — base multi-select wheel (the pcb.multiselect content the shell draws).
    runtime.session.workspace_mut().ui.marking_menu = Some(datum_gui_protocol::MarkingMenuState {
        menu_key: "pcb.multiselect".into(),
        target_object_id: None,
        anchor_x_px: 672,
        anchor_y_px: 403,
        preview_slot: None,
        gesture_dx_px: 0,
        gesture_dy_px: 0,
        selection: None,
        submenu_path: Vec::new(),
    });
    present(&mut runtime);
    let base = capture(&mut runtime, &dir, "1-base-wheel");

    // Real input: click the W (Align ▸) wedge → open its sub-wheel (navigation only).
    let w = marking_hit(&runtime, "W").expect("W Align ▸ hit on the base wheel");
    runtime.last_cursor_pos = Some(w);
    assert!(runtime.handle_primary_click(), "clicking Align navigates");
    present(&mut runtime);
    assert_eq!(
        runtime
            .workspace()
            .ui
            .marking_menu
            .as_ref()
            .unwrap()
            .submenu_path,
        vec!["Align".to_string()],
        "a submenu parent opens its sub-wheel (navigation, not invocation)"
    );
    // STATE 2 — nested Align sub-wheel.
    let subwheel = capture(&mut runtime, &dir, "2-align-subwheel");

    // Real input: click the center back affordance → pop one level.
    let back = marking_surface_center(&runtime).expect("back affordance in the sub-wheel");
    runtime.last_cursor_pos = Some(back);
    assert!(runtime.handle_primary_click(), "clicking back pops");
    present(&mut runtime);
    assert!(
        runtime
            .workspace()
            .ui
            .marking_menu
            .as_ref()
            .unwrap()
            .submenu_path
            .is_empty(),
        "back returns to the top-level wheel"
    );
    // STATE 3 — back at the base wheel.
    let back_png = capture(&mut runtime, &dir, "3-back-to-base");

    // Real input: hover the N (Rotate) wedge → focus the disabled leaf so its
    // truthful reason caption renders. update_marking_menu_preview sets the focus
    // from the gesture delta and invalidates the frame (a direct field write would
    // leave a stale frame).
    let n = marking_hit(&runtime, "N").expect("N Rotate hit on the base wheel");
    runtime.update_marking_menu_preview(n);
    present(&mut runtime);
    assert_eq!(
        runtime
            .workspace()
            .ui
            .marking_menu
            .as_ref()
            .unwrap()
            .preview_slot
            .as_deref(),
        Some("N"),
        "hovering focuses the disabled leaf"
    );
    // STATE 4 — disabled-leaf focus with the truthful reason caption.
    let reason = capture(&mut runtime, &dir, "4-disabled-leaf-reason");

    for png in [&base, &subwheel, &back_png, &reason] {
        assert!(png.exists(), "state capture {} exists", png.display());
    }
    // Distinct states actually rendered (guards against a stale/cached frame): the
    // sub-wheel differs from the base, and the focused disabled leaf (with its
    // reason caption) differs from the plain base.
    assert_ne!(
        std::fs::read(&subwheel).unwrap(),
        std::fs::read(&base).unwrap(),
        "the Align sub-wheel must render a distinct frame from the base wheel"
    );
    assert_ne!(
        std::fs::read(&reason).unwrap(),
        std::fs::read(&back_png).unwrap(),
        "focusing a disabled leaf (reason caption) must render a distinct frame"
    );

    // NONMUTATION — the entire inert traversal commits no design change.
    assert_eq!(
        runtime.workspace().scene.source_revision,
        before_rev,
        "inert shell traversal must not change the design source revision"
    );
    assert_eq!(
        runtime.workspace().selection,
        before_sel,
        "inert shell traversal must not change the selection"
    );

    // ACCESSIBILITY — the menu accessibility model is readable for the shell state
    // (does not panic; disabled entries carry non-empty descriptions where present).
    let model = datum_gui_protocol::gui_menu_model::default_gui_menu_model().unwrap();
    let nodes = datum_gui_protocol::gui_menu_model::accessibility::menu_accessibility_nodes(
        model,
        runtime.workspace(),
    );
    for node in &nodes {
        if !node.available {
            assert!(
                !node.description.is_empty(),
                "a disabled accessibility node must carry a truthful description"
            );
        }
    }

    eprintln!(
        "[mm-c04] four-state native captures ({} a11y nodes) in {}",
        nodes.len(),
        dir.display()
    );
}
