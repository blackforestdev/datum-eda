//! Source-bound Select All through the ordinary App pointer and rendered menu.
use super::*;
use datum_gui_protocol::{ApplicationFocus, SelectionTarget, selection_menu::Command};

struct MenuWorkflow {
    host: Fixture,
    step: usize,
    root: PathBuf,
    before: std::collections::BTreeMap<PathBuf, Vec<u8>>,
}

impl MenuWorkflow {
    fn open(&mut self, events: &ActiveEventLoop, content: PaneContent) {
        let r = self.host.app.runtime.as_mut().unwrap();
        let layout = r.workspace().ui.layout.clone();
        let pane = r
            .prepared_scene()
            .unwrap()
            .layout
            .viewport_panes(&layout)
            .panes
            .into_iter()
            .find(|pane| pane.content == content)
            .unwrap();
        let rect = pane.rect.scene;
        self.host.mouse(
            events,
            (
                (rect.x + rect.width * 0.5) as f64,
                (rect.y + rect.height * 0.5) as f64,
            ),
            false,
        );
        let id = self.host.app.window.as_ref().unwrap().id();
        for state in [ElementState::Pressed, ElementState::Released] {
            self.host.app.window_event(
                events,
                id,
                WindowEvent::MouseInput {
                    device_id: winit::event::DeviceId::dummy(),
                    state,
                    button: MouseButton::Right,
                },
            );
        }
    }

    fn click_slot(&mut self, events: &ActiveEventLoop, slot: &str) {
        let target = self
            .host
            .app
            .runtime
            .as_ref()
            .unwrap()
            .presented_hits
            .regions()
            .iter()
            .find_map(|region| match &region.target {
                target @ HitTarget::MarkingMenuItem { slot: found, .. } if found == slot => {
                    Some(target.clone())
                }
                _ => None,
            })
            .expect("menu row must be presented and pointer-reachable");
        self.host.control(events, target);
    }

    fn advance(&mut self, events: &ActiveEventLoop) {
        let Some(r) = self.host.app.runtime.as_ref() else {
            return;
        };
        if !r.surface_transaction.has_presented()
            || r.renderer.render_session().has_pending_frame()
            || self.host.since.elapsed() < Duration::from_millis(20)
        {
            return;
        }
        let schematic = self.step < 5;
        match self.step {
            0 => {
                // Start with Board focused, so a schematic right-click must bind its own pane.
                let r = self.host.app.runtime.as_mut().unwrap();
                let board = r
                    .workspace()
                    .ui
                    .layout
                    .leaves()
                    .into_iter()
                    .find(|pane| {
                        r.workspace().ui.layout.content_for(*pane) == Some(PaneContent::Board)
                    })
                    .unwrap();
                r.swap_pane_focus(|layout| layout.focused = board);
            }
            1 | 5 => self.open(
                events,
                if schematic {
                    PaneContent::Schematic
                } else {
                    PaneContent::Board
                },
            ),
            2 | 6 => {
                let menu = r
                    .workspace()
                    .ui
                    .marking_menu
                    .as_ref()
                    .expect("right-click opens menu");
                let selection = menu.selection.as_ref().unwrap();
                assert_eq!(selection.schematic, schematic);
                assert_eq!(
                    selection.source_scene,
                    datum_gui_protocol::camera_scene_for_pane(
                        r.workspace(),
                        r.workspace().ui.layout.focused
                    )
                    .unwrap()
                    .scene_id
                );
                self.click_slot(events, "SELECT");
            }
            3 | 7 => {
                let menu = r
                    .workspace()
                    .ui
                    .marking_menu
                    .as_ref()
                    .unwrap()
                    .selection
                    .as_ref()
                    .unwrap();
                assert!(menu.open);
                let index = menu
                    .items
                    .iter()
                    .position(|item| item.command == Command::AllInDocument)
                    .unwrap();
                assert!(menu.items[index].unavailable.is_none());
                if let Some(dir) = std::env::var_os("DATUM_NATIVE_CAPTURE_OUT") {
                    let path = PathBuf::from(dir).join(if schematic {
                        "schematic-select-all-menu.png"
                    } else {
                        "board-select-all-menu.png"
                    });
                    self.host
                        .app
                        .runtime
                        .as_mut()
                        .unwrap()
                        .write_visual_screenshot(&path)
                        .unwrap();
                }
                self.click_slot(events, &format!("SELECT:{index}"));
            }
            4 | 8 => {
                assert!(r.workspace().ui.marking_menu.is_none());
                assert_eq!(
                    r.application_focus(),
                    ApplicationFocus::Editor(r.workspace().ui.layout.focused)
                );
                let SelectionTarget::Native(view) = &r.workspace().selection else {
                    panic!("global menu action must acquire objects");
                };
                assert!(!view.members.is_empty());
                if schematic {
                    assert!(
                        !view.schematic_keys.is_empty()
                            || view.schematic_documents.iter().any(|p| !p.keys.is_empty()),
                        "selected sheet objects must have a rendered projection"
                    );
                } else {
                    assert!(
                        !view.board_keys.is_empty(),
                        "board menu must acquire board objects"
                    );
                }
                let selected = view.subject.clone();
                let r = self.host.app.runtime.as_mut().unwrap();
                assert!(r.native_select_all());
                assert!(
                    matches!(&r.workspace().selection, SelectionTarget::Native(view) if view.subject == selected),
                    "menu and Ctrl+A must select the same complete document"
                );
                for (path, bytes) in &self.before {
                    assert_eq!(
                        &std::fs::read(path).unwrap(),
                        bytes,
                        "selection wrote {path:?}"
                    );
                }
                if !schematic {
                    events.exit();
                }
            }
            _ => unreachable!(),
        }
        self.step += 1;
        self.host.since = Instant::now();
        self.host.app.request_workspace_redraw();
    }
}

impl ApplicationHandler for MenuWorkflow {
    fn resumed(&mut self, events: &ActiveEventLoop) {
        self.host.app.resumed(events);
    }
    fn window_event(&mut self, events: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        if matches!(
            event,
            WindowEvent::CursorMoved { .. }
                | WindowEvent::MouseInput { .. }
                | WindowEvent::KeyboardInput { .. }
                | WindowEvent::MouseWheel { .. }
                | WindowEvent::CursorLeft { .. }
        ) {
            return;
        }
        self.host.app.window_event(events, id, event);
    }
    fn about_to_wait(&mut self, events: &ActiveEventLoop) {
        self.host.app.about_to_wait(events);
        if self.host.since.elapsed() > Duration::from_secs(30) {
            self.host.failure = Some(format!(
                "context-menu stage {} timed out for {}",
                self.step,
                self.root.display()
            ));
            events.exit();
            return;
        }
        self.advance(events);
        events.set_control_flow(winit::event_loop::ControlFlow::WaitUntil(
            Instant::now() + Duration::from_millis(20),
        ));
    }
}

#[test]
#[ignore = "requires private DATUM_SYMBOL_MOVE_PROJECT with schematic and board geometry and isolated Wayland"]
fn ordinary_app_context_menu_select_all_matches_keyboard_on_sheet_and_board() {
    let root = PathBuf::from(std::env::var_os("DATUM_SYMBOL_MOVE_PROJECT").unwrap());
    let mut before = std::collections::BTreeMap::new();
    for relative in [
        "project.json",
        "board/board.json",
        "schematic/schematic.json",
        ".datum/journal/transactions.jsonl",
        ".datum/journal/cursor.json",
    ] {
        let path = root.join(relative);
        before.insert(path.clone(), std::fs::read(path).unwrap());
    }
    let index: serde_json::Value =
        serde_json::from_slice(&std::fs::read(root.join("schematic/schematic.json")).unwrap())
            .unwrap();
    for relative in index["sheets"].as_object().unwrap().values() {
        let path = root.join("schematic").join(relative.as_str().unwrap());
        before.insert(path.clone(), std::fs::read(path).unwrap());
    }
    let args = <GuiArgs as clap::Parser>::try_parse_from([
        "datum-gui",
        "--project-root",
        root.to_str().unwrap(),
    ])
    .unwrap();
    let events = EventLoop::<()>::with_user_event()
        .with_wayland()
        .with_any_thread(true)
        .build()
        .unwrap();
    let host = Fixture {
        app: App::new(args, events.create_proxy()),
        stage: 0,
        since: Instant::now(),
        path: root.join("project.json"),
        preimage: vec![],
        wired: None,
        failure: None,
        wire_only: false,
    };
    let mut workflow = MenuWorkflow {
        host,
        step: 0,
        root,
        before,
    };
    events.run_app(&mut workflow).unwrap();
    assert!(
        workflow.host.failure.is_none(),
        "{:?}",
        workflow.host.failure
    );
    assert_eq!(workflow.step, 9);
}
