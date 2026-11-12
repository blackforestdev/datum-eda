//! Ordinary App resistor-block clipboard, multi-object regions and cold history.
use super::*;
use datum_gui_protocol::native_edit_client::NativeEditAction;
use datum_gui_protocol::{ApplicationFocus, SelectionTarget};
use eda_engine::api::native_write::{
    WriteProvenance, commit_prepared,
    genesis::{GenesisSpec, bootstrap_native_project},
    registry::{NativeWriteContext, find_native_write_verb},
};
use eda_engine::substrate::{CommitSource, DesignModel, ProjectResolver, SourceShardKind};
use serde_json::{Value, json};
use std::{collections::BTreeMap, path::Path};
use uuid::Uuid;
fn provenance() -> WriteProvenance {
    WriteProvenance::new(
        "test",
        CommitSource::Manual,
        "Native block clipboard App proof",
    )
}
fn write(model: &mut DesignModel, root: &Path, verb: &str, params: Value) -> Uuid {
    // Match the ordinary writer companion: resolve each request from persisted source.
    *model = ProjectResolver::new(root).resolve().unwrap();
    let p = (find_native_write_verb(verb).unwrap().build)(
        &NativeWriteContext {
            model,
            project_root: root,
        },
        provenance(),
        params,
    )
    .unwrap();
    let id = p.primary_object_id.unwrap_or_default();
    commit_prepared(model, root, p).unwrap();
    *model = ProjectResolver::new(root).resolve().unwrap();
    id
}
fn setup(root: &Path) -> (Vec<Uuid>, PathBuf) {
    bootstrap_native_project(
        root,
        GenesisSpec {
            project_name: "Resistor block clipboard".into(),
            existing_ids: None,
        },
    )
    .unwrap();
    let mut model = ProjectResolver::new(root).resolve().unwrap();
    let eda_engine::ir::units::PreFeatureProjectUnitsMigration::Create { profile, receipt } =
        eda_engine::ir::units::migrate_pre_feature_project_units(None).unwrap()
    else {
        panic!("Fresh fixture requires the canonical units migration");
    };
    let initialize =
        eda_engine::api::native_write::project::build_initialize_project_display_units(
            &model,
            provenance(),
            profile,
            &receipt,
        )
        .unwrap();
    commit_prepared(&mut model, root, initialize).unwrap();
    model = ProjectResolver::new(root).resolve().unwrap();
    let (unit, left, right, library, entity, gate, package, part, sheet) = (
        Uuid::new_v4(),
        Uuid::new_v4(),
        Uuid::new_v4(),
        Uuid::new_v4(),
        Uuid::new_v4(),
        Uuid::new_v4(),
        Uuid::new_v4(),
        Uuid::new_v4(),
        Uuid::new_v4(),
    );
    for (kind, id, fields) in [
        (
            "units",
            unit,
            json!({"name":"Resistor","pins":{left.to_string():{"name":"1","electrical_type":"Passive"},right.to_string():{"name":"2","electrical_type":"Passive"}}}),
        ),
        (
            "symbols",
            library,
            json!({"name":"Resistor","unit":unit,"drawings":[{"Rect":{"min":{"x":-1000000,"y":-500000},"max":{"x":1000000,"y":500000},"width":100000}}],"pin_anchors":[{"pin":left,"number":"1","position":{"x":-2000000,"y":0},"style":{"orientation":"Right","length_nm":1000000,"decoration":"none"}},{"pin":right,"number":"2","position":{"x":2000000,"y":0},"style":{"orientation":"Left","length_nm":1000000,"decoration":"none"}}]}),
        ),
        ("packages", package, json!({"name":"Resistor body"})),
        (
            "entities",
            entity,
            json!({"name":"Resistor","manufacturer":"","prefix":"R","gates":{gate.to_string():{"uuid":gate,"name":"A","unit":unit,"symbol":library}}}),
        ),
        (
            "parts",
            part,
            json!({"manufacturer":"Designer","mpn":"10K","description":"","datasheet":"","value":"10k","entity":entity,"package":package}),
        ),
    ] {
        write(
            &mut model,
            root,
            "datum.library.create",
            json!({"kind":kind,"pool_path":"pool","object_id":id,"fields":fields}),
        );
    }
    write(
        &mut model,
        root,
        "datum.schematic.create_sheet",
        json!({"sheet":sheet,"name":"Root"}),
    );
    let mut symbols = vec![];
    for (reference, x) in [("R1", 0), ("R2", 10000000)] {
        symbols.push(write(&mut model,root,"datum.schematic.place_symbol",json!({"sheet":sheet,"instance_path":[],"library_symbol":library,"reference":reference,"value":"10k","x_nm":x,"y_nm":0})));
    }
    write(
        &mut model,
        root,
        "datum.schematic.draw_wire",
        json!({"sheet":sheet,"instance_path":[],"points":[{"x":2000000,"y":0},{"x":8000000,"y":0}]}),
    );
    write(
        &mut model,
        root,
        "datum.schematic.set_symbol_field",
        json!({"symbol":symbols[0],"instance_path":[],"field":"custom","key":"Tolerance","value":"1%"}),
    );
    let field = eda_engine::substrate::ElectricalSelectionSnapshot::capture(&model)
        .unwrap()
        .schematic_source()
        .unwrap()
        .sheets[&sheet]
        .symbols[&symbols[0]]
        .fields[0]
        .uuid;
    write(
        &mut model,
        root,
        "datum.schematic.set_symbol_field",
        json!({"symbol":symbols[0],"instance_path":[],"field":"custom","field_id":field,"visible":true,"position":{"x":0,"y":2000000}}),
    );
    let path = model
        .source_shards
        .iter()
        .find(|s| s.kind == SourceShardKind::SchematicSheet)
        .unwrap()
        .path
        .clone();
    (symbols, path)
}

struct BlockClipboardWorkflow {
    host: Fixture,
    step: usize,
    root: PathBuf,
    output: PathBuf,
    before: BTreeMap<PathBuf, Vec<u8>>,
    saved: BTreeMap<PathBuf, Vec<u8>>,
    journal: usize,
}
impl BlockClipboardWorkflow {
    fn snapshot(&self) -> BTreeMap<PathBuf, Vec<u8>> {
        let model = ProjectResolver::new(&self.root).resolve().unwrap();
        let mut files: BTreeMap<_, _> = model
            .source_shards
            .iter()
            .map(|s| (s.path.clone(), std::fs::read(&s.path).unwrap()))
            .collect();
        for relative in [
            ".datum/journal/transactions.jsonl",
            ".datum/journal/cursor.json",
        ] {
            let path = self.root.join(relative);
            if path.exists() {
                files.insert(path.clone(), std::fs::read(path).unwrap());
            }
        }
        files
    }
    fn capture(&mut self, name: &str) {
        self.host
            .app
            .runtime
            .as_mut()
            .unwrap()
            .write_visual_screenshot(&self.output.join(format!("{name}.png")))
            .unwrap();
    }
    fn shortcut(&mut self, key: &str) {
        let action = crate::workspace_keyboard::character_action(key, true)
            .expect("Existing clipboard shortcut");
        assert!(crate::workspace_keyboard::apply(
            self.host.app.runtime.as_mut().unwrap(),
            action
        ));
    }
    fn region(&mut self, events: &ActiveEventLoop, lasso: bool) {
        let points = if lasso {
            vec![
                (-4000000, -2000000),
                (-5000000, -2000000),
                (-5000000, 2000000),
                (14000000, 2000000),
                (14000000, -2000000),
                (-4000000, -2000000),
            ]
        } else {
            vec![(-4000000, -2000000), (14000000, 2000000)]
        };
        let r = self.host.app.runtime.as_mut().unwrap();
        r.dispatch_session_command(datum_gui_protocol::SessionCommand::ClearSelection);
        r.set_workspace_tool(crate::WorkspaceTool::Move);
        r.modifiers = winit::keyboard::ModifiersState::SHIFT;
        self.host.world(
            events,
            PointNm {
                x: points[0].0,
                y: points[0].1,
            },
            false,
        );
        let window = self.host.app.window.as_ref().unwrap().id();
        self.host.app.window_event(
            events,
            window,
            WindowEvent::MouseInput {
                device_id: winit::event::DeviceId::dummy(),
                state: ElementState::Pressed,
                button: MouseButton::Left,
            },
        );
        for (x, y) in &points[1..] {
            self.host.world(events, PointNm { x: *x, y: *y }, false);
        }
        self.host.app.window_event(
            events,
            window,
            WindowEvent::MouseInput {
                device_id: winit::event::DeviceId::dummy(),
                state: ElementState::Released,
                button: MouseButton::Left,
            },
        );
        let r = self.host.app.runtime.as_mut().unwrap();
        while r.selection_evaluation.is_some() {
            r.selection_tick = Some(Instant::now());
            r.poll_native_selection_tick();
        }
        r.modifiers = winit::keyboard::ModifiersState::empty();
        let SelectionTarget::Native(view) = &r.workspace().selection else {
            panic!("Region must select circuit block")
        };
        assert_eq!(view.members.len(), 3);
        assert_eq!(
            view.members
                .iter()
                .filter(|m| m.class == datum_gui_protocol::AuthoredSelectionClass::Symbol)
                .count(),
            2
        );
        assert_eq!(
            view.members
                .iter()
                .filter(|m| m.class == datum_gui_protocol::AuthoredSelectionClass::Wire)
                .count(),
            1
        );
        assert!(r.native_edit.preview.is_none());
    }
    fn right_click(&mut self, events: &ActiveEventLoop) {
        self.host.world(events, PointNm { x: 0, y: 0 }, false);
        let window = self.host.app.window.as_ref().unwrap().id();
        for state in [ElementState::Pressed, ElementState::Released] {
            self.host.app.window_event(
                events,
                window,
                WindowEvent::MouseInput {
                    device_id: winit::event::DeviceId::dummy(),
                    state,
                    button: MouseButton::Right,
                },
            );
        }
    }
    fn menu_slot(&mut self, events: &ActiveEventLoop, slot: &str) {
        let target = self
            .host
            .app
            .runtime
            .as_ref()
            .unwrap()
            .presented_hits
            .regions()
            .iter()
            .find_map(|h| match &h.target {
                target @ HitTarget::MarkingMenuItem { slot: found, .. } if found == slot => {
                    Some(target.clone())
                }
                _ => None,
            })
            .expect("Clipboard context command must be presented");
        self.host.control(events, target);
    }
    fn poses(&self) -> BTreeMap<String, (i64, i64)> {
        let model = ProjectResolver::new(&self.root).resolve().unwrap();
        let snapshot = eda_engine::substrate::ElectricalSelectionSnapshot::capture(&model).unwrap();
        snapshot
            .schematic_source()
            .unwrap()
            .sheets
            .values()
            .flat_map(|s| s.symbols.values())
            .map(|s| (s.reference.clone(), (s.position.x, s.position.y)))
            .collect()
    }
    fn history(&mut self, redo: bool) {
        let r = self.host.app.runtime.as_mut().unwrap();
        let tip = r
            .workspace()
            .supervision
            .journal
            .accepted_transaction_tip
            .as_ref()
            .unwrap()
            .parse()
            .unwrap();
        assert!(r.queue_native_edit(if redo {
            NativeEditAction::Redo { expected_tip: tip }
        } else {
            NativeEditAction::Undo { expected_tip: tip }
        }));
    }
    fn advance(&mut self, events: &ActiveEventLoop) {
        let Some(r) = self.host.app.runtime.as_ref() else {
            return;
        };
        if !r.surface_transaction.has_presented()
            || r.renderer.render_session().has_pending_frame()
            || r.native_edit.pending()
            || r.native_move_frame_needed()
            || self.host.since.elapsed() < Duration::from_millis(20)
        {
            return;
        }
        eprintln!(
            "schematic-block-clipboard stage={} status={:?}",
            self.step,
            r.workspace().last_command_status
        );
        match self.step {
            0 => {
                let r = self.host.app.runtime.as_mut().unwrap();
                if !r.native_edit_available() || r.native_selection.is_none() {
                    return;
                }
                let pane = r
                    .workspace()
                    .ui
                    .layout
                    .leaves()
                    .into_iter()
                    .find(|p| {
                        r.workspace().ui.layout.content_for(*p) == Some(PaneContent::Schematic)
                    })
                    .unwrap();
                r.session.workspace_mut().ui.layout.focused = pane;
                r.set_application_focus(ApplicationFocus::Editor(pane));
                r.fit_camera();
                self.before = self.snapshot();
                self.journal = ProjectResolver::new(&self.root)
                    .resolve()
                    .unwrap()
                    .journal
                    .len();
            }
            1 => {
                self.region(events, false);
                assert_eq!(self.snapshot(), self.before);
            }
            2 => {
                self.region(events, true);
                self.capture("block-selected");
            }
            3 => {
                self.shortcut("c");
                assert_eq!(self.snapshot(), self.before);
            }
            4 | 6 => {
                self.shortcut("v");
                self.host.world(events, PointNm { x: 0, y: 6000000 }, false);
            }
            5 => {
                assert_eq!(
                    r.workspace()
                        .schematic_authoring
                        .design_placement
                        .as_ref()
                        .unwrap()
                        .symbol_definitions
                        .len(),
                    2
                );
                assert_eq!(self.snapshot(), self.before);
                self.capture("block-paste-preview");
                assert!(
                    self.host
                        .app
                        .runtime
                        .as_mut()
                        .unwrap()
                        .cancel_native_schematic()
                );
                assert_eq!(self.snapshot(), self.before);
            }
            7 => {
                assert_eq!(self.snapshot(), self.before);
                self.host.world(events, PointNm { x: 0, y: 6000000 }, true);
            }
            8 => {
                assert_eq!(self.poses().len(), 4);
                assert_eq!(self.poses()["R3"], (0, 6000000));
                assert_eq!(self.poses()["R4"], (10000000, 6000000));
                self.capture("block-pasted");
                self.region(events, false);
            }
            9 => {
                self.right_click(events);
            }
            10 => {
                self.menu_slot(events, "W");
                self.host
                    .world(events, PointNm { x: 0, y: -6000000 }, false);
            }
            11 => {
                self.capture("block-duplicate-preview");
                self.host.world(events, PointNm { x: 0, y: -6000000 }, true);
            }
            12 => {
                assert_eq!(self.poses().len(), 6);
                self.history(false);
            }
            13 => {
                assert_eq!(self.poses().len(), 4);
                self.history(true);
            }
            14 => {
                assert_eq!(self.poses().len(), 6);
                assert!(
                    self.host
                        .app
                        .runtime
                        .as_mut()
                        .unwrap()
                        .queue_native_edit(NativeEditAction::Save)
                );
            }
            15 => {
                self.saved = self.snapshot();
                let args = <GuiArgs as clap::Parser>::try_parse_from([
                    "datum-gui",
                    "--project-root",
                    self.root.to_str().unwrap(),
                ])
                .unwrap();
                let proxy = self.host.app.terminal_event_proxy.clone();
                self.host.app = App::new(args, proxy);
                self.host.app.resumed(events);
            }
            16 => {
                assert_eq!(self.snapshot(), self.saved);
                assert_eq!(self.poses().len(), 6);
                let model = ProjectResolver::new(&self.root).resolve().unwrap();
                assert_eq!(model.component_instances.len(), 6);
                assert!(
                    model
                        .component_instances
                        .values()
                        .all(|ci| ci.placed_package_refs.is_empty())
                );
                let source =
                    eda_engine::substrate::ElectricalSelectionSnapshot::capture(&model).unwrap();
                let sheets = source.schematic_source().unwrap();
                assert_eq!(
                    sheets.sheets.values().map(|s| s.wires.len()).sum::<usize>(),
                    3
                );
                let owners: Vec<_> = sheets
                    .sheets
                    .values()
                    .flat_map(|s| s.symbols.values())
                    .filter(|s| !s.fields.is_empty())
                    .collect();
                assert_eq!(owners.len(), 3);
                let mut ids = std::collections::BTreeSet::new();
                for symbol in owners {
                    let field = &symbol.fields[0];
                    assert_eq!(field.key, "Tolerance");
                    assert_eq!(field.value, "1%");
                    assert!(field.visible);
                    assert!(ids.insert(field.uuid));
                    assert_eq!(
                        field.position,
                        Some(eda_engine::ir::geometry::Point::new(
                            symbol.position.x,
                            symbol.position.y + 2000000
                        ))
                    );
                }
                let r = self.host.app.runtime.as_mut().unwrap();
                let pane = r
                    .workspace()
                    .ui
                    .layout
                    .leaves()
                    .into_iter()
                    .find(|p| {
                        r.workspace().ui.layout.content_for(*p) == Some(PaneContent::Schematic)
                    })
                    .unwrap();
                r.session.workspace_mut().ui.layout.focused = pane;
                r.set_application_focus(ApplicationFocus::Editor(pane));
                r.fit_camera();
            }
            17 => {
                self.capture("block-cold");
                events.exit();
            }
            _ => unreachable!(),
        }
        self.step += 1;
        self.host.since = Instant::now();
        self.host.app.request_workspace_redraw();
    }
}

impl ApplicationHandler for BlockClipboardWorkflow {
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
            self.host.failure = Some(format!("Block clipboard stage {} timed out", self.step));
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
#[ignore = "requires owned fresh DATUM_BLOCK_CLIPBOARD_PROJECT and isolated native Wayland"]
fn ordinary_app_block_copy_paste_duplicate_regions_history_save_reopen() {
    let root = PathBuf::from(std::env::var_os("DATUM_BLOCK_CLIPBOARD_PROJECT").unwrap());
    let output = PathBuf::from(std::env::var_os("DATUM_BLOCK_CLIPBOARD_OUTPUT").unwrap());
    std::fs::create_dir_all(&output).unwrap();
    let (_, path) = setup(&root);
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
        preimage: std::fs::read(&path).unwrap(),
        path,
        wired: None,
        failure: None,
        wire_only: false,
    };
    let mut workflow = BlockClipboardWorkflow {
        host,
        step: 0,
        root,
        output,
        before: BTreeMap::new(),
        saved: BTreeMap::new(),
        journal: 0,
    };
    events.run_app(&mut workflow).unwrap();
    assert!(
        workflow.host.failure.is_none(),
        "{:?}",
        workflow.host.failure
    );
    assert_eq!(workflow.step, 18);
}
