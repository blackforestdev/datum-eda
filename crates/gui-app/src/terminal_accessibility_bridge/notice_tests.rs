//! Existing refusal publication refreshes complete selection once without writes.
use super::*;

#[test]
#[ignore = "requires X11/Vulkan and pinned CLI; run serially"]
#[allow(deprecated)] // Hidden integration host; production uses ActiveEventLoop.
fn native_selection_notice_publishes_once_for_none_and_complete_selection() {
    use clap::Parser;
    use datum_gui_protocol::{
        AuthoredSelectionClass as Class, AuthoredSelectionIdentity, SelectionSubject,
        SelectionTarget,
    };
    use winit::platform::x11::EventLoopBuilderExtX11;

    let fixture = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../engine/testdata/golden/text/native/text-transform-repro");
    let root = std::env::temp_dir().join(format!(
        "selection-notice-publication-{}-{}",
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
    use datum_gui_protocol::selection_view::inspector_state::InspectorPage;
    use datum_gui_protocol::{
        ConsoleFeedbackCategory, ConsoleFeedbackSeverity, ConsoleFeedbackSource,
    };
    let pane = runtime.workspace().ui.layout.focused;
    assert!(
        !runtime
            .workspace()
            .ui
            .selection_inspector
            .inventory_expanded
    );
    for native in [false, true] {
        if native {
            assert!(runtime.native_select_all());
        } else {
            assert_eq!(runtime.workspace().selection, SelectionTarget::None);
        }
        for _ in 0..2 {
            runtime
                .prepared_scene()
                .expect("current refusal frame prepares");
            let mut wanted = runtime.workspace().selection.clone();
            if let SelectionTarget::Native(view) = &mut wanted {
                view.notice = Some("Expand complete inventory first".into());
            }
            let revision = runtime.renderer.render_session().content_revision();
            let console = runtime.workspace().ui.console.records().count();
            let announcements = runtime.terminal_accessibility.announcement_log.len();
            assert!(runtime.navigate_selection_inspector(InspectorPage::Next));
            // An ordinary unavailable page action changes only the exact notice.
            assert_eq!(runtime.workspace().selection, wanted);
            match &runtime.workspace().selection {
                SelectionTarget::Native(view) => {
                    assert_eq!(view.members, expected);
                    let SelectionSubject::Compound(compound) = &view.subject else {
                        panic!("complete native selection must survive refusal");
                    };
                    assert_eq!(compound.members, expected);
                    assert_eq!(compound.focus, None);
                    assert_eq!(view.focus(), None);
                    assert!(!view.stale);
                    let accessible = runtime.terminal_accessibility.selection.as_ref().unwrap();
                    assert_eq!(accessible.view.as_ref(), view.as_ref());
                    assert_eq!(accessible.pane, Some(pane));
                    assert_eq!(accessible.evaluating, None);
                }
                SelectionTarget::None => {
                    assert!(runtime.terminal_accessibility.selection.is_none())
                }
                _ => panic!("refusal must preserve current selection kind"),
            }
            assert_eq!(
                runtime.workspace().ui.console.records().count(),
                console + 1
            );
            let record = runtime
                .workspace()
                .ui
                .console
                .records()
                .next_back()
                .unwrap();
            assert_eq!(record.message, "Expand complete inventory first");
            assert_eq!(record.category, ConsoleFeedbackCategory::ActionRefusal);
            assert_eq!(record.severity, ConsoleFeedbackSeverity::Error);
            assert_eq!(record.source, ConsoleFeedbackSource::Selection);
            assert_eq!(
                runtime.terminal_accessibility.announcement_log.len(),
                announcements + 1
            );
            let announcement = runtime
                .terminal_accessibility
                .announcement_log
                .back()
                .unwrap();
            assert_eq!(
                announcement.text,
                "Action refused. Error: Expand complete inventory first"
            );
            assert_eq!(
                announcement.priority,
                crate::console_accessibility::AnnouncementPriority::Medium
            );
            // This production revision catches repeated full-state refresh even
            // when the evidence stream subsequently deduplicates identical values.
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
    }
    drop(runtime);
    std::fs::remove_dir_all(root).unwrap();
}
