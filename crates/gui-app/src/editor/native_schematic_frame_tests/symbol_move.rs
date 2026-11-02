//! One existing native Project through ordinary App input/frame/writer dispatch.
//! Programmatic native events do not qualify physical Wayland or HUMAN input.
use super::*;
use eda_engine::schematic::{PlacedSymbol, SchematicWire};
use std::collections::BTreeMap;

struct MoveWorkflow {
    host: Fixture,
    step: usize,
    original: Option<PlacedSymbol>,
    moved: Option<PlacedSymbol>,
    original_wires: BTreeMap<uuid::Uuid, SchematicWire>,
    moved_wires: BTreeMap<uuid::Uuid, SchematicWire>,
    require_connected: bool,
    source_before: BTreeMap<PathBuf, Vec<u8>>,
    board_before: Vec<u8>,
    root: PathBuf,
}
impl MoveWorkflow {
    fn wires(&self) -> BTreeMap<uuid::Uuid, SchematicWire> {
        let e = &self
            .host
            .app
            .runtime
            .as_ref()
            .unwrap()
            .workspace()
            .schematic_authoring;
        e.source.as_ref().unwrap().sheets[&e.sheet.unwrap()]
            .wires
            .iter()
            .map(|(id, wire)| (*id, wire.clone()))
            .collect()
    }
    fn symbol(&self) -> &PlacedSymbol {
        let editor = &self
            .host
            .app
            .runtime
            .as_ref()
            .unwrap()
            .workspace()
            .schematic_authoring;
        let original = self.original.as_ref().unwrap();
        &editor.source.as_ref().unwrap().sheets[&editor.sheet.unwrap()].symbols[&original.uuid]
    }
    fn shortcut(&mut self, key: &str, control: bool) {
        let action = crate::workspace_keyboard::character_action(key, control).unwrap();
        assert!(crate::workspace_keyboard::apply(
            self.host.app.runtime.as_mut().unwrap(),
            action
        ));
        self.host.app.request_workspace_redraw();
    }
    fn unchanged(&self) {
        for (path, bytes) in &self.source_before {
            assert_eq!(
                &std::fs::read(path).unwrap(),
                bytes,
                "preview wrote {path:?}"
            );
        }
        assert_eq!(
            std::fs::read(self.root.join("board/board.json")).unwrap(),
            self.board_before
        );
    }
    fn destination_pointer(&self) -> PointNm {
        let pin = &self.original.as_ref().unwrap().pins[0];
        PointNm {
            x: pin.position.x + 1_000_000,
            y: pin.position.y + 1_000_000,
        }
    }
    fn advance(&mut self, events: &ActiveEventLoop) {
        let Some(r) = self.host.app.runtime.as_ref() else {
            return;
        };
        if r.native_edit.pending()
            || !r.surface_transaction.has_presented()
            || r.renderer.render_session().has_pending_frame()
            || self.host.since.elapsed() < Duration::from_millis(20)
        {
            return;
        }
        assert!(
            !r.native_edit.blocked,
            "canonical Symbol response must qualify"
        );
        eprintln!(
            "symbol-move-native step={} status={:?} selection={:?}",
            self.step,
            r.workspace().last_command_status,
            match &r.workspace().selection {
                datum_gui_protocol::SelectionTarget::Native(v) => Some(&v.subject),
                _ => None,
            }
        );
        match self.step {
            0 => {
                let r = self.host.app.runtime.as_mut().unwrap();
                let pane = r
                    .workspace()
                    .ui
                    .layout
                    .leaves()
                    .into_iter()
                    .find(|pane| {
                        r.workspace().ui.layout.content_for(*pane) == Some(PaneContent::Schematic)
                    })
                    .unwrap();
                r.session.workspace_mut().ui.layout.focused = pane;
                r.set_application_focus(datum_gui_protocol::ApplicationFocus::Editor(pane));
                r.fit_camera();
                let editor = &r.workspace().schematic_authoring;
                if editor.source.is_none() || editor.sheet.is_none() {
                    return;
                }
                let basis = editor
                    .move_bases
                    .values()
                    .filter_map(|b| b.as_ref().ok())
                    .find(|b| {
                        Some(b.sheet_id) == editor.sheet
                            && !b.symbol.pins.is_empty()
                            && (!self.require_connected
                                || editor.source.as_ref().unwrap().sheets[&b.sheet_id]
                                    .wires
                                    .values()
                                    .any(|wire| {
                                        b.symbol.pins.iter().any(|pin| {
                                            pin.position == wire.from || pin.position == wire.to
                                        })
                                    }))
                    })
                    .unwrap_or_else(|| {
                        panic!(
                            "existing eligible native Symbol (connected={}) is required: {:?}",
                            self.require_connected, editor.move_bases
                        )
                    });
                self.original = Some(basis.symbol.clone());
                self.original_wires = editor.source.as_ref().unwrap().sheets[&basis.sheet_id]
                    .wires
                    .iter()
                    .map(|(id, wire)| (*id, wire.clone()))
                    .collect();
                r.invalidate_frame();
            }
            1 => {
                super::region_selection::exercise(
                    &mut self.host,
                    events,
                    self.original.as_ref().unwrap(),
                );
                self.unchanged();
                let pin = &self.original.as_ref().unwrap().pins[0];
                self.host.world(
                    events,
                    PointNm {
                        x: pin.position.x,
                        y: pin.position.y,
                    },
                    true,
                );
            }
            2 => {
                let r = self.host.app.runtime.as_ref().unwrap();
                assert!(
                    matches!(&r.workspace().selection, datum_gui_protocol::SelectionTarget::Native(v)
                    if matches!(&v.subject, datum_gui_protocol::SelectionSubject::Object(id)
                        if id.class == datum_gui_protocol::AuthoredSelectionClass::Symbol && id.id == self.original.as_ref().unwrap().uuid))
                );
                self.shortcut("m", false);
            }
            3 => {
                assert!(
                    self.host
                        .app
                        .runtime
                        .as_ref()
                        .unwrap()
                        .native_edit
                        .preview
                        .as_ref()
                        .unwrap()
                        .is_schematic()
                );
                self.host.world(events, self.destination_pointer(), false);
            }
            4 => {
                self.unchanged();
                let r = self.host.app.runtime.as_mut().unwrap();
                let preview = r.native_edit.preview.as_ref().unwrap();
                assert_ne!(
                    r.workspace().schematic_scene,
                    Some(preview.base().clone()),
                    "rendered Symbol preview must move"
                );
                let scene = r.workspace().schematic_scene.as_ref().unwrap();
                for wire in self.original_wires.values().filter(|wire| {
                    self.original
                        .as_ref()
                        .unwrap()
                        .pins
                        .iter()
                        .any(|pin| pin.position == wire.from || pin.position == wire.to)
                }) {
                    let id = wire.uuid.to_string();
                    let before: Vec<_> = preview
                        .base()
                        .board_graphics
                        .iter()
                        .filter(|g| g.source_object_uuid == id)
                        .collect();
                    let after: Vec<_> = scene
                        .board_graphics
                        .iter()
                        .filter(|g| g.source_object_uuid == id)
                        .collect();
                    assert!(
                        !before.is_empty() && !after.is_empty(),
                        "attached wire must retain projected identity"
                    );
                    assert_ne!(
                        before, after,
                        "attached wire must move in the actual preview scene"
                    );
                }
                assert!(r.dispatch_session_command(SessionCommand::CancelAuthoringGesture));
                assert!(r.native_edit.preview.is_none());
            }
            5 => {
                self.unchanged();
                assert_eq!(self.symbol(), self.original.as_ref().unwrap());
                assert_eq!(
                    self.wires(),
                    self.original_wires,
                    "cancel preserves every wire"
                );
                let pin = &self.original.as_ref().unwrap().pins[0];
                self.host.world(
                    events,
                    PointNm {
                        x: pin.position.x,
                        y: pin.position.y,
                    },
                    false,
                );
                self.shortcut("m", false);
            }
            6 => self.host.world(events, self.destination_pointer(), false),
            7 => {
                self.unchanged();
                self.host.world(events, self.destination_pointer(), true);
                assert!(
                    self.host
                        .app
                        .runtime
                        .as_ref()
                        .unwrap()
                        .native_edit
                        .pending(),
                    "click must start canonical asynchronous edit"
                );
            }
            8 => {
                let moved = self.symbol().clone();
                let original = self.original.as_ref().unwrap();
                assert_ne!(moved.position, original.position);
                let dx = moved.position.x - original.position.x;
                let dy = moved.position.y - original.position.y;
                let mut expected = original.clone();
                expected.position = moved.position;
                for pin in &mut expected.pins {
                    pin.position.x += dx;
                    pin.position.y += dy;
                }
                assert_eq!(moved, expected, "complete pin/identity/library translation");
                let mut expected_wires = self.original_wires.clone();
                for wire in expected_wires.values_mut() {
                    for endpoint in [&mut wire.from, &mut wire.to] {
                        if original.pins.iter().any(|pin| pin.position == *endpoint) {
                            endpoint.x += dx;
                            endpoint.y += dy;
                        }
                    }
                }
                self.moved_wires = self.wires();
                assert_eq!(
                    self.moved_wires, expected_wires,
                    "only attached endpoints translate; all wire UUIDs persist"
                );
                self.moved = Some(moved);
                self.shortcut("z", true);
            }
            9 => {
                assert_eq!(
                    self.symbol(),
                    self.original.as_ref().unwrap(),
                    "exact Symbol undo"
                );
                assert_eq!(self.wires(), self.original_wires, "exact wire undo");
                self.shortcut("y", true);
            }
            10 => {
                assert_eq!(
                    self.symbol(),
                    self.moved.as_ref().unwrap(),
                    "exact Symbol redo"
                );
                assert_eq!(self.wires(), self.moved_wires, "exact wire redo");
                self.shortcut("s", true);
            }
            11 => {
                let model = eda_engine::substrate::ProjectResolver::new(&self.root)
                    .resolve()
                    .unwrap();
                let snapshot =
                    eda_engine::substrate::ElectricalSelectionSnapshot::capture(&model).unwrap();
                let original = self.original.as_ref().unwrap();
                let reopened_sheet = snapshot
                    .schematic_source()
                    .unwrap()
                    .sheets
                    .values()
                    .find(|sheet| sheet.symbols.contains_key(&original.uuid))
                    .unwrap();
                let reopened = &reopened_sheet.symbols[&original.uuid];
                let reopened_wires: BTreeMap<_, _> = reopened_sheet
                    .wires
                    .iter()
                    .map(|(id, wire)| (*id, wire.clone()))
                    .collect();
                assert_eq!(reopened_wires, self.moved_wires, "Save/reopen exact wires");
                assert_eq!(
                    reopened,
                    self.moved.as_ref().unwrap(),
                    "Save/reopen exact Symbol"
                );
                assert_eq!(
                    std::fs::read(self.root.join("board/board.json")).unwrap(),
                    self.board_before
                );
                eprintln!(
                    "symbol-move-native PASS preview/cancel/commit/undo/redo/Save/reopen source={} destination={}",
                    serde_json::to_string(original).unwrap(),
                    serde_json::to_string(reopened).unwrap()
                );
                events.exit();
            }
            _ => unreachable!(),
        }
        self.step += 1;
        self.host.since = Instant::now();
        self.host.app.request_workspace_redraw();
    }
}
impl ApplicationHandler for MoveWorkflow {
    fn resumed(&mut self, events: &ActiveEventLoop) {
        self.host.app.resumed(events);
    }
    fn window_event(&mut self, events: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        // Keep the existing finite native replay isolated from physical input.
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
        if self
            .host
            .app
            .window
            .as_ref()
            .is_some_and(|w| self.host.app.frames.rendering_failed(w.id()))
        {
            self.host.failure = Some("ordinary frame coordinator paused".into());
            events.exit();
            return;
        }
        if self.host.since.elapsed() > Duration::from_secs(30) {
            self.host.failure = Some(format!("Symbol Move stage {} timed out", self.step));
            events.exit();
            return;
        }
        self.advance(events);
        let bound = self.host.since + Duration::from_secs(30);
        let due = if self.host.app.runtime.as_ref().is_some_and(|r| {
            !r.native_edit.pending()
                && !r.renderer.render_session().has_pending_frame()
                && r.surface_transaction.has_presented()
        }) {
            (self.host.since + Duration::from_millis(20)).min(bound)
        } else {
            bound
        };
        use winit::event_loop::ControlFlow;
        events.set_control_flow(match events.control_flow() {
            ControlFlow::Poll => ControlFlow::Poll,
            ControlFlow::Wait => ControlFlow::WaitUntil(due),
            ControlFlow::WaitUntil(native) => ControlFlow::WaitUntil(native.min(due)),
        });
    }
}

#[test]
#[ignore = "requires private owner-derived DATUM_SYMBOL_MOVE_PROJECT with existing isolated native Symbol and native Wayland"]
fn ordinary_app_symbol_move_preview_cancel_commit_history_save_reopen() {
    run(false);
}

#[test]
#[ignore = "requires private DATUM_SYMBOL_MOVE_PROJECT with an eligible connected native Symbol and native Wayland"]
fn ordinary_app_connected_symbol_move_preserves_wires_through_preview_and_history() {
    run(true);
}

fn run(require_connected: bool) {
    let root = PathBuf::from(std::env::var_os("DATUM_SYMBOL_MOVE_PROJECT").unwrap());
    let index: serde_json::Value =
        serde_json::from_slice(&std::fs::read(root.join("schematic/schematic.json")).unwrap())
            .unwrap();
    let relative = index["sheets"]
        .as_object()
        .unwrap()
        .values()
        .next()
        .unwrap()
        .as_str()
        .unwrap();
    let path = root.join("schematic").join(relative);
    let mut source_before = BTreeMap::new();
    for relative in [
        "project.json",
        "board/board.json",
        "schematic/schematic.json",
        ".datum/journal/transactions.jsonl",
        ".datum/journal/cursor.json",
    ] {
        let p = root.join(relative);
        source_before.insert(p.clone(), std::fs::read(p).unwrap());
    }
    source_before.insert(path.clone(), std::fs::read(&path).unwrap());
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
        path: path.clone(),
        preimage: std::fs::read(path).unwrap(),
        wired: None,
        failure: None,
        wire_only: false,
    };
    let mut workflow = MoveWorkflow {
        host,
        step: 0,
        original: None,
        moved: None,
        original_wires: BTreeMap::new(),
        moved_wires: BTreeMap::new(),
        require_connected,
        board_before: std::fs::read(root.join("board/board.json")).unwrap(),
        root,
        source_before,
    };
    events.run_app(&mut workflow).unwrap();
    assert!(
        workflow.host.failure.is_none(),
        "{:?}",
        workflow.host.failure
    );
    assert_eq!(workflow.step, 12);
}
