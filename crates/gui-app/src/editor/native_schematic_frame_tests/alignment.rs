//! Ordinary App compound alignment through rendered menu, preview and history.
use super::*;
use datum_gui_protocol::native_edit_client::NativeEditAction;
use datum_gui_protocol::{ApplicationFocus, SelectionSubject, SelectionTarget};
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
    WriteProvenance::new("test", CommitSource::Manual, "Native alignment App proof")
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
            project_name: "Body alignment".into(),
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
    let (unit, pin, sheet) = (Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4());
    write(
        &mut model,
        root,
        "datum.library.create",
        json!({"kind":"units","pool_path":"pool","object_id":unit,"fields":{"name":"Passive","pins":{pin.to_string():{"name":"Signal","electrical_type":"Passive"}}}}),
    );
    write(
        &mut model,
        root,
        "datum.schematic.create_sheet",
        json!({"sheet":sheet,"name":"Root"}),
    );
    let mut symbols = Vec::new();
    for (i, (x, y, left, right)) in [
        (0, 0, -1000000, 1000000),
        (10000000, 4000000, -3000000, 2000000),
    ]
    .into_iter()
    .enumerate()
    {
        let library = Uuid::new_v4();
        write(
            &mut model,
            root,
            "datum.library.create",
            json!({"kind":"symbols","pool_path":"pool","object_id":library,"fields":{"name":format!("Body{i}"),"unit":unit,"drawings":[{"Rect":{"min":{"x":left,"y":-500000},"max":{"x":right,"y":500000},"width":100000}}],"pin_anchors":[{"pin":pin,"position":{"x":-2000000,"y":0},"style":{"orientation":"Right","length_nm":1000000,"decoration":"none"}}]}}),
        );
        symbols.push(write(&mut model, root, "datum.schematic.place_symbol", json!({"sheet":sheet,"instance_path":[],"library_symbol":library,"reference":format!("U{}",i+1),"value":"A long caption does not define body bounds","x_nm":x,"y_nm":y})));
    }
    write(
        &mut model,
        root,
        "datum.schematic.draw_wire",
        json!({"sheet":sheet,"instance_path":[],"points":[{"x":-2000000,"y":0},{"x":8000000,"y":4000000}]}),
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

struct AlignmentWorkflow {
    host: Fixture,
    step: usize,
    root: PathBuf,
    output: PathBuf,
    before: BTreeMap<PathBuf, Vec<u8>>,
    saved: BTreeMap<PathBuf, Vec<u8>>,
    journal: usize,
}
impl AlignmentWorkflow {
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
            .expect("Alignment menu control must be presented");
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
            "schematic-alignment stage={} status={:?}",
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
            1 => self.host.world(events, PointNm { x: 0, y: 0 }, true),
            2 => {
                self.host.app.runtime.as_mut().unwrap().modifiers =
                    winit::keyboard::ModifiersState::SHIFT;
                self.host.world(
                    events,
                    PointNm {
                        x: 10000000,
                        y: 4000000,
                    },
                    true,
                );
            }
            3 => {
                let r = self.host.app.runtime.as_mut().unwrap();
                r.modifiers = winit::keyboard::ModifiersState::empty();
                let SelectionTarget::Native(view) = &r.workspace().selection else {
                    panic!("native group selected");
                };
                let SelectionSubject::Compound(compound) = &view.subject else {
                    panic!("complete Symbol compound selected");
                };
                assert_eq!(compound.members.len(), 2);
                assert_eq!(self.snapshot(), self.before);
                self.right_click(events);
            }
            4 | 10 => {
                assert_eq!(
                    r.workspace().ui.marking_menu.as_ref().unwrap().menu_key,
                    "schematic.multiselect"
                );
                if self.step == 4 {
                    self.capture("alignment-context");
                }
                self.menu_slot(events, "NE");
            }
            5 | 11 => self.menu_slot(events, "SUB:0"),
            6 | 12 => {
                if self.step == 6 {
                    self.capture("alignment-horizontal-presets");
                }
                self.menu_slot(events, "SUB:0");
            }
            7 => {
                assert!(
                    !r.presented_hits
                        .regions()
                        .iter()
                        .any(|hit| hit.target == HitTarget::MoveCoordinates),
                    "Fixed alignment must not offer coordinate editing"
                );
                assert!(
                    r.native_edit
                        .preview
                        .as_ref()
                        .unwrap()
                        .alignment_request()
                        .is_some()
                );
                assert_eq!(self.snapshot(), self.before, "Preview must not write");
                self.capture("alignment-preview");
                self.host.world(
                    events,
                    PointNm {
                        x: 15000000,
                        y: 8000000,
                    },
                    false,
                );
            }
            8 => {
                let r = self.host.app.runtime.as_mut().unwrap();
                let preview = r.native_edit.preview.as_ref().unwrap();
                let fixed = preview.reference_position();
                assert_eq!(
                    preview
                        .destination(
                            PointNm {
                                x: 30000000,
                                y: 10000000
                            },
                            r.workspace().authoring.snap
                        )
                        .unwrap(),
                    fixed
                );
                assert!(
                    preview
                        .scene_at(PointNm {
                            x: fixed.x + 1000000,
                            y: fixed.y
                        })
                        .is_err()
                );
                assert!(r.cancel_native_move());
                assert_eq!(
                    self.snapshot(),
                    self.before,
                    "Cancel must restore without writes"
                );
            }
            9 => self.right_click(events),
            13 => {
                assert!(r.native_edit.preview.is_some());
                assert_eq!(self.snapshot(), self.before);
                self.host.world(events, PointNm { x: 0, y: 0 }, true);
            }
            14 => {
                assert!(r.native_edit.preview.is_none());
                assert_eq!(self.poses()["U1"], (0, 0));
                assert_eq!(self.poses()["U2"], (2000000, 4000000));
                assert_eq!(
                    ProjectResolver::new(&self.root)
                        .resolve()
                        .unwrap()
                        .journal
                        .len(),
                    self.journal + 1
                );
                self.capture("alignment-accepted");
                self.history(false);
            }
            15 => {
                assert_eq!(self.poses()["U2"], (10000000, 4000000));
                self.history(true);
            }
            16 => {
                assert_eq!(self.poses()["U2"], (2000000, 4000000));
                assert!(
                    self.host
                        .app
                        .runtime
                        .as_mut()
                        .unwrap()
                        .queue_native_edit(NativeEditAction::Save)
                );
            }
            17 => {
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
            18 => {
                assert_eq!(
                    self.snapshot(),
                    self.saved,
                    "Cold reopen must not write design history"
                );
                assert_eq!(self.poses()["U2"], (2000000, 4000000));
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
            19 => {
                self.capture("alignment-cold");
                events.exit();
            }
            _ => unreachable!(),
        }
        self.step += 1;
        self.host.since = Instant::now();
        self.host.app.request_workspace_redraw();
    }
}

impl ApplicationHandler for AlignmentWorkflow {
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
            self.host.failure = Some(format!("PCB transfer stage {} timed out", self.step));
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
#[ignore = "requires owned fresh DATUM_ALIGNMENT_PROJECT and isolated native Wayland"]
fn ordinary_app_symbol_alignment_context_preview_cancel_history_save_reopen() {
    let root = PathBuf::from(std::env::var_os("DATUM_ALIGNMENT_PROJECT").unwrap());
    let output = PathBuf::from(std::env::var_os("DATUM_ALIGNMENT_OUTPUT").unwrap());
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
    let mut workflow = AlignmentWorkflow {
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
    assert_eq!(workflow.step, 20);
}
