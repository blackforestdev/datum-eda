//! Real Runtime completion and cancellation over immutable canonical region input.
use super::*;
use crate::selection_region_evaluation::{RegionBasis, RegionEvaluation};
use datum_gui_protocol::PointNm;
use datum_gui_viewport::selection_acquisition::AcquisitionMode;

#[test]
#[ignore = "requires X11/Vulkan and pinned CLI; run serially"]
#[allow(deprecated)] // Hidden integration host; production uses ActiveEventLoop.
fn native_region_completion_publishes_once_and_pending_cancel_preserves_selection() {
    use clap::Parser;
    use datum_gui_protocol::{
        AuthoredSelectionClass as Class, AuthoredSelectionIdentity, SelectionSubject,
        SelectionTarget,
    };
    use winit::platform::x11::EventLoopBuilderExtX11;

    let fixture = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../engine/testdata/golden/text/native/text-transform-repro");
    let root = std::env::temp_dir().join(format!(
        "selection-region-publication-{}-{}",
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
    assert_eq!(board["outline"]["closed"], true);
    assert_eq!(
        board["outline"]["vertices"],
        serde_json::json!([
            {"x": 0, "y": 0},
            {"x": 240_000_000, "y": 0},
            {"x": 240_000_000, "y": 160_000_000},
            {"x": 0, "y": 160_000_000},
        ])
    );
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
    assert!(runtime.ensure_selection_authority().unwrap());
    let native = runtime.native_selection.as_ref().unwrap();
    let basis = Arc::new(RegionBasis::capture(
        &native.model,
        &native.authority,
        &runtime.workspace().scene,
    ));
    let source_revision = native.model.model_revision.0.clone();
    let pane = runtime.workspace().ui.layout.focused;
    let bounds = &runtime.workspace().scene.bounds;
    let margin = 100_000_000;
    let (x0, y0, x1, y1) = (
        bounds.min_x - margin,
        bounds.min_y - margin,
        bounds.max_x + margin,
        bounds.max_y + margin,
    );
    let rectangle = vec![
        PointNm { x: x0, y: y0 },
        PointNm { x: x1, y: y0 },
        PointNm { x: x1, y: y1 },
        PointNm { x: x0, y: y1 },
    ];
    let lasso = vec![
        rectangle[0],
        rectangle[1],
        rectangle[2],
        PointNm {
            x: (x0 + x1) / 2,
            y: y1 + margin,
        },
        rectangle[3],
    ];
    let mut previous = None;
    for (path, announced) in [(&rectangle, true), (&lasso, false)] {
        runtime.prepared_scene().expect("current frame prepares");
        let revision = runtime.renderer.render_session().content_revision();
        let console = runtime.workspace().ui.console.records().count();
        let announcements = runtime.terminal_accessibility.announcement_log.len();
        runtime.selection_evaluation = Some(
            RegionEvaluation::begin(
                pane,
                AcquisitionMode::Add,
                source_revision.clone(),
                basis.clone(),
                runtime.workspace().ui.filters.clone(),
                path,
            )
            .unwrap(),
        );
        runtime.session.workspace_mut().ui.selection_region =
            Some(datum_gui_protocol::SelectionRegionFeedback {
                pane,
                path: path.clone(),
                subtractive: false,
                phase: 0,
                evaluating: Some(0),
            });
        runtime.selection_tick = Some(std::time::Instant::now());
        assert!(runtime.poll_native_selection_tick());
        assert!(runtime.selection_evaluation.is_none());
        assert!(runtime.selection_tick.is_none());
        assert!(runtime.workspace().ui.selection_region.is_none());
        let SelectionTarget::Native(view) = &runtime.workspace().selection else {
            panic!("complete region selection required");
        };
        assert_eq!(view.members, expected);
        let SelectionSubject::Compound(compound) = &view.subject else {
            panic!("complete text region must produce a Compound");
        };
        assert_eq!(compound.members, expected);
        assert_eq!(compound.focus, None);
        assert_eq!(view.focus(), None);
        assert!(!view.stale);
        assert_eq!(
            runtime
                .terminal_accessibility
                .selection
                .as_ref()
                .unwrap()
                .view
                .as_deref()
                .unwrap(),
            view.as_ref()
        );
        assert_eq!(
            runtime
                .terminal_accessibility
                .selection
                .as_ref()
                .unwrap()
                .pane,
            Some(pane)
        );
        assert_eq!(
            runtime
                .terminal_accessibility
                .selection
                .as_ref()
                .unwrap()
                .evaluating,
            None
        );
        if let Some(previous) = &previous {
            assert_eq!(view.as_ref(), previous);
        }
        previous = Some(view.as_ref().clone());
        assert_eq!(
            runtime.workspace().ui.console.records().count(),
            console + usize::from(announced)
        );
        assert_eq!(
            runtime.terminal_accessibility.announcement_log.len(),
            announcements + usize::from(announced)
        );
        // Existing composition authority detects the removed duplicate refresh.
        assert_eq!(
            runtime.renderer.render_session().content_revision(),
            revision + 1
        );
        assert!(runtime.renderer.render_session().prepared().is_none());
        assert!(runtime.renderer.render_session().has_pending_frame());
        for (rel, bytes) in paths.iter().zip(&canonical) {
            assert_eq!(&std::fs::read(root.join(rel)).unwrap(), bytes);
        }
        assert_eq!(std::fs::read(&journal_path).ok(), journal);
        assert_eq!(std::fs::read(&head_path).ok(), head);
    }
    // Cancel a real staged evaluation before its first poll: no partial selection.
    runtime
        .prepared_scene()
        .expect("completed region frame prepares");
    let prior = runtime.workspace().selection.clone();
    runtime.selection_evaluation = Some(
        RegionEvaluation::begin(
            pane,
            AcquisitionMode::Remove,
            source_revision,
            basis,
            runtime.workspace().ui.filters.clone(),
            &rectangle,
        )
        .unwrap(),
    );
    runtime.session.workspace_mut().ui.selection_region =
        Some(datum_gui_protocol::SelectionRegionFeedback {
            pane,
            path: rectangle,
            subtractive: true,
            phase: 0,
            evaluating: Some(0),
        });
    runtime.selection_tick = Some(std::time::Instant::now());
    runtime.refresh_selection_accessibility();
    assert_eq!(
        runtime
            .terminal_accessibility
            .selection
            .as_ref()
            .unwrap()
            .evaluating,
        Some((pane, 0))
    );
    let revision = runtime.renderer.render_session().content_revision();
    assert_eq!(runtime.workspace().selection, prior);
    assert!(runtime.cancel_native_selection_gesture());
    assert_eq!(runtime.workspace().selection, prior);
    assert_eq!(
        runtime
            .terminal_accessibility
            .selection
            .as_ref()
            .unwrap()
            .evaluating,
        None
    );
    assert!(runtime.selection_evaluation.is_none());
    assert!(runtime.selection_tick.is_none());
    assert!(runtime.workspace().ui.selection_region.is_none());
    assert!(runtime.selection_release_suppressed);
    assert_eq!(runtime.finish_native_selection_press(), Some(true));
    assert!(!runtime.selection_release_suppressed);
    assert!(!runtime.poll_native_selection_tick());
    assert_eq!(runtime.workspace().selection, prior);
    assert_eq!(
        runtime.renderer.render_session().content_revision(),
        revision + 1
    );
    assert!(runtime.renderer.render_session().prepared().is_none());
    assert!(runtime.renderer.render_session().has_pending_frame());
    for (rel, bytes) in paths.iter().zip(&canonical) {
        assert_eq!(&std::fs::read(root.join(rel)).unwrap(), bytes);
    }
    assert_eq!(std::fs::read(&journal_path).ok(), journal);
    assert_eq!(std::fs::read(&head_path).ok(), head);
    drop(runtime);
    std::fs::remove_dir_all(root).unwrap();
}
