//! Component-first capture through the existing ordinary App/frame workflow.
//! Synthetic QA package geometry proves connectivity, not a manufactured device pinout.
use super::*;
use datum_gui_protocol::{
    board_transfer::BoardTransferAction as Transfer, library_browser::LibraryControl as C,
};
use eda_engine::{
    api::native_write::{
        self as native, WriteProvenance, commit_prepared,
        registry::{NativeWriteContext, find_native_write_verb},
    },
    substrate::{CommitSource, DesignModel, ProjectResolver, SourceShardKind, SourceShardTaxon},
};
use serde_json::{Value, json};
use std::{collections::BTreeMap, path::Path};
use uuid::Uuid;

fn provenance() -> WriteProvenance {
    WriteProvenance::new(
        "component-test",
        CommitSource::Manual,
        "Component-first native workflow",
    )
}
fn write(model: &mut DesignModel, root: &Path, verb: &str, params: Value) {
    let prepared = (find_native_write_verb(verb).unwrap().build)(
        &NativeWriteContext {
            model,
            project_root: root,
        },
        provenance(),
        params,
    )
    .unwrap();
    commit_prepared(model, root, prepared).unwrap();
}
fn pool(model: &DesignModel, taxon: SourceShardTaxon) -> Value {
    let shard = model
        .source_shards
        .iter()
        .find(|s| s.taxon == Some(taxon))
        .unwrap();
    model
        .materialized_source_shard_value_by_relative_path(&shard.relative_path)
        .unwrap()
}
fn snapshot(root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    let model = ProjectResolver::new(root).resolve().unwrap();
    let mut result: BTreeMap<_, _> = model
        .source_shards
        .iter()
        .map(|s| (s.path.clone(), std::fs::read(&s.path).unwrap()))
        .collect();
    for name in [
        ".datum/journal/transactions.jsonl",
        ".datum/journal/cursor.json",
    ] {
        let path = root.join(name);
        if path.exists() {
            result.insert(path.clone(), std::fs::read(path).unwrap());
        }
    }
    result
}
fn setup(source: &Path, target: &Path) -> Uuid {
    board_transfer::setup(source);
    let mut a = ProjectResolver::new(source).resolve().unwrap();
    let mut part = pool(&a, SourceShardTaxon::PoolPart);
    let map = pool(&a, SourceShardTaxon::PoolPinPadMap);
    part["default_pin_pad_map"] = map["uuid"].clone();
    let id: Uuid = serde_json::from_value(part["uuid"].clone()).unwrap();
    use native::library::{PoolLibraryObjectTarget as Target, PoolLibraryOperationSpec as Spec};
    let prepared = native::library::build_pool_library_write(
        &a,
        provenance(),
        Some("pool"),
        vec![Spec::Set {
            target: Target::new("pool", "parts", id),
            object: part,
        }],
    )
    .unwrap();
    commit_prepared(&mut a, source, prepared).unwrap();
    native::genesis::bootstrap_native_project(
        target,
        native::genesis::GenesisSpec {
            project_name: "Component capture target".into(),
            existing_ids: None,
        },
    )
    .unwrap();
    let mut b = ProjectResolver::new(target).resolve().unwrap();
    let mut specs = Vec::new();
    for (kind, taxon) in [
        ("units", SourceShardTaxon::PoolUnit),
        ("symbols", SourceShardTaxon::PoolSymbol),
    ] {
        let object = pool(&a, taxon);
        specs.push(Spec::Create {
            target: Target::new(
                "pool",
                kind,
                serde_json::from_value(object["uuid"].clone()).unwrap(),
            ),
            object,
        });
    }
    let prepared =
        native::library::build_pool_library_write(&b, provenance(), Some("pool"), specs).unwrap();
    commit_prepared(&mut b, target, prepared).unwrap();
    let sheet = Uuid::new_v4();
    write(
        &mut b,
        target,
        "datum.schematic.create_sheet",
        json!({"sheet":sheet,"name":"Root"}),
    );
    let symbol = pool(&a, SourceShardTaxon::PoolSymbol)["uuid"].clone();
    for (reference, x) in [("U1", 0), ("U2", 10000000)] {
        write(
            &mut b,
            target,
            "datum.schematic.place_symbol",
            json!({"sheet":sheet,"instance_path":[],"library_symbol":symbol,"reference":reference,"value":"Provisional value","x_nm":x,"y_nm":0}),
        );
    }
    write(
        &mut b,
        target,
        "datum.schematic.draw_wire",
        json!({"sheet":sheet,"instance_path":[],"points":[{"x":-2000000,"y":0},{"x":-2000000,"y":3000000},{"x":8000000,"y":3000000},{"x":8000000,"y":0}]}),
    );
    id
}

struct Workflow {
    host: Fixture,
    source: PathBuf,
    target: PathBuf,
    output: PathBuf,
    part: Uuid,
    stage: usize,
    original: Value,
    source_bytes: BTreeMap<PathBuf, Vec<u8>>,
    before_acquisition: BTreeMap<PathBuf, Vec<u8>>,
    cancelled_preview: bool,
}
impl Workflow {
    fn model(&self) -> DesignModel {
        ProjectResolver::new(&self.target).resolve().unwrap()
    }
    fn sheet(&self) -> Value {
        serde_json::from_slice(&std::fs::read(&self.host.path).unwrap()).unwrap()
    }
    fn capture(&mut self, label: &str) {
        self.host
            .app
            .runtime
            .as_mut()
            .unwrap()
            .write_visual_screenshot(&self.output.join(format!("{label}.png")))
            .unwrap();
    }
    fn menu(&mut self, menu: &str, label: &str) {
        assert!(self.host.app.runtime.as_mut().unwrap().activate_menu_item(
            menu,
            label,
            EntrySurface::MenuPointer
        ));
    }
    fn advance(&mut self, events: &ActiveEventLoop) {
        let Some(r) = self.host.app.runtime.as_ref() else {
            return;
        };
        if r.native_edit.pending()
            || !r.surface_transaction.has_presented()
            || r.renderer.render_session().has_pending_frame()
            || self.host.since.elapsed() < Duration::from_millis(30)
        {
            return;
        }
        assert!(!r.native_edit.blocked);
        eprintln!(
            "component-native stage={} status={:?}",
            self.stage,
            r.workspace().last_command_status
        );
        match self.stage {
            0 => {
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
                r.set_application_focus(datum_gui_protocol::ApplicationFocus::Editor(pane));
                self.host.menu("Component");
            }
            1 => {
                assert!(r.workspace().schematic_authoring.components.active);
                assert!(
                    r.workspace()
                        .schematic_authoring
                        .picker_choices()
                        .is_empty()
                );
                assert!(r.component_picker_open());
                assert_eq!(
                    r.workspace().ui.layout.focused_content(),
                    PaneContent::Schematic
                );
                self.capture("empty-complete-component-picker");
            }
            2 => {
                assert!(
                    r.workspace()
                        .library
                        .acquisition
                        .as_ref()
                        .unwrap()
                        .is_native_part()
                );
                if r.workspace().library.acquisition.as_ref().unwrap().path
                    == self.source.to_str().unwrap()
                {
                    assert_eq!(
                        r.workspace().ui.layout.focused_content(),
                        PaneContent::Schematic
                    );
                    self.stage = 3;
                    return;
                }
                self.host
                    .control(events, HitTarget::Library(C::AcquirePath));
                let path = self.source.to_str().unwrap().to_owned();
                assert!(
                    self.host
                        .app
                        .runtime
                        .as_mut()
                        .unwrap()
                        .handle_library_input(&Key::Character(path.into()), ElementState::Pressed)
                );
                self.host
                    .control(events, HitTarget::Library(C::AcquireLoad));
            }
            3 => {
                let a = r.workspace().library.acquisition.as_ref().unwrap();
                if a.loading || !a.loaded {
                    return;
                }
                let row = a
                    .sources
                    .iter()
                    .position(|s| s.name == self.part.to_string())
                    .unwrap();
                self.host
                    .control(events, HitTarget::Library(C::AcquireResult(row)));
            }
            4 => {
                let a = r.workspace().library.acquisition.as_ref().unwrap();
                if a.loading || a.selected_native_part().is_none() {
                    return;
                }
                assert_eq!(a.selected_native_part(), Some(self.part));
                let p = a.preview.as_ref().unwrap().native_part.as_ref().unwrap();
                assert!(!p.mapping_labels().unwrap().is_empty());
                self.capture("complete-component-preview");
                if !self.cancelled_preview {
                    self.host.control(events, HitTarget::Library(C::Cancel));
                    self.cancelled_preview = true;
                    self.stage = 29;
                } else {
                    self.host
                        .control(events, HitTarget::Library(C::AcquireImport));
                }
            }
            5 => {
                assert!(
                    r.component_picker_open(),
                    "Adoption returns to the same picker"
                );
                assert_eq!(
                    self.sheet(),
                    self.original,
                    "Acquisition must not place or rewrite Symbols"
                );
                for (path, bytes) in &self.before_acquisition {
                    if path
                        .components()
                        .any(|c| c.as_os_str() == "units" || c.as_os_str() == "symbols")
                    {
                        assert_eq!(
                            &std::fs::read(path).unwrap(),
                            bytes,
                            "Identical source dependencies must be reused"
                        );
                    }
                }
                assert!(
                    r.workspace()
                        .schematic_authoring
                        .picker_choices()
                        .iter()
                        .any(|c| c
                            .component
                            .as_ref()
                            .is_some_and(|c| c.part.part == self.part && c.value == "Signal"))
                );
                self.capture("adopted-library-separate-placement");
            }
            6 => {
                self.capture("value-package-component-picker");
                self.host
                    .control(events, HitTarget::Library(C::AcquireImport));
            }
            7 => {
                if r.workspace().library.acquisition.is_some() {
                    return;
                }
                assert!(r.workspace().schematic_authoring.active());
                self.host.world(
                    events,
                    PointNm {
                        x: 5000000,
                        y: 8000000,
                    },
                    false,
                );
                self.stage = 31;
            }
            32 => {
                self.capture("component-placement-ghost");
                self.host.world(
                    events,
                    PointNm {
                        x: 5000000,
                        y: 8000000,
                    },
                    true,
                );
                self.stage = 7;
            }
            8 => {
                assert_eq!(self.sheet()["symbols"].as_object().unwrap().len(), 3);
                self.host
                    .app
                    .runtime
                    .as_mut()
                    .unwrap()
                    .cancel_native_schematic();
                self.menu("Edit", "Undo");
            }
            9 => {
                assert_eq!(self.sheet(), self.original);
                self.menu("Edit", "Redo");
            }
            10 => {
                assert_eq!(self.sheet()["symbols"].as_object().unwrap().len(), 3);
                self.menu("Edit", "Undo");
            }
            11 | 15 => {
                assert_eq!(self.sheet()["wires"], self.original["wires"]);
                self.host.world(
                    events,
                    PointNm {
                        x: if self.stage == 11 { 0 } else { 10000000 },
                        y: 0,
                    },
                    true,
                );
            }
            12 | 16 => {
                assert!(
                    self.host
                        .app
                        .runtime
                        .as_mut()
                        .unwrap()
                        .begin_schematic_assignment()
                );
            }
            13 | 17 => {
                let a = r
                    .workspace()
                    .schematic_authoring
                    .assignment_draft
                    .as_ref()
                    .unwrap()
                    .assignment
                    .as_ref()
                    .unwrap();
                assert_eq!(a.choice().unwrap().part, self.part);
                self.capture(if self.stage == 13 {
                    "wired-component-assignment"
                } else {
                    "second-component-assignment"
                });
                self.host.control(events, HitTarget::SchematicLabelPreview);
            }
            14 | 18 => {
                let review = r
                    .workspace()
                    .schematic_authoring
                    .delete_review
                    .as_ref()
                    .unwrap();
                if !review.ready() {
                    return;
                }
                self.capture(if self.stage == 14 {
                    "review-component-assignment"
                } else {
                    "review-second-assignment"
                });
                assert!(
                    self.host
                        .app
                        .runtime
                        .as_mut()
                        .unwrap()
                        .handle_schematic_review_input(
                            &Key::Named(NamedKey::Tab),
                            ElementState::Pressed,
                        )
                );
            }
            19 => {
                let now = self.sheet();
                assert_eq!(now["wires"], self.original["wires"]);
                for (id, original) in self.original["symbols"].as_object().unwrap() {
                    let mut expected = original.clone();
                    for key in ["part", "entity", "gate", "value"] {
                        expected[key] = now["symbols"][id][key].clone();
                    }
                    assert_eq!(
                        now["symbols"][id], expected,
                        "Assignment must preserve IDs, pins, pose and other fields"
                    );
                    assert_eq!(now["symbols"][id]["value"], "Signal");
                    assert_ne!(now["symbols"][id]["value"], original["value"]);
                }
                self.menu("Edit", "Transfer Schematic to PCB…");
            }
            20 => {
                let review = r.workspace().ui.board_transfer.as_ref().unwrap();
                assert_eq!(review.plan.ready_count(), 2);
                assert!(review.request.choices.is_empty());
                assert!(review.plan.components.iter().all(|c| c.component_choice));
                assert!(
                    !r.presented_hits.regions().iter().any(|h| matches!(
                        h.target,
                        HitTarget::BoardTransfer(Transfer::ChooseMap(_))
                    ))
                );
                let before = review.request.clone();
                self.capture("component-default-transfer");
                self.host
                    .app
                    .runtime
                    .as_mut()
                    .unwrap()
                    .handle_board_transfer(Transfer::ChooseMap(0));
                assert_eq!(
                    self.host
                        .app
                        .runtime
                        .as_ref()
                        .unwrap()
                        .workspace()
                        .ui
                        .board_transfer
                        .as_ref()
                        .unwrap()
                        .request,
                    before
                );
                self.host
                    .control(events, HitTarget::BoardTransfer(Transfer::PrepareSources));
            }
            21 => {
                self.host
                    .control(events, HitTarget::BoardTransfer(Transfer::Accept));
            }
            22 => {
                let b = self
                    .model()
                    .materialized_source_shard_value(SourceShardKind::BoardRoot)
                    .unwrap();
                assert_eq!(b["packages"].as_object().unwrap().len(), 2);
                assert_eq!(self.sheet()["wires"], self.original["wires"]);
                self.menu("File", "Save");
            }
            23 => {
                let model = self.model();
                let electrical =
                    eda_engine::substrate::ElectricalSelectionSnapshot::capture(&model).unwrap();
                let terminal = electrical
                    .authored_sources()
                    .unwrap()
                    .into_iter()
                    .find(|o| o.occurrence.class == "pins")
                    .unwrap()
                    .occurrence;
                let net = electrical
                    .net_for_origin(&model.model_revision, &terminal)
                    .unwrap();
                assert_eq!(net.owned.iter().filter(|o| o.class == "pads").count(), 2);
                assert_eq!(snapshot(&self.source), self.source_bytes);
                let args = <GuiArgs as clap::Parser>::try_parse_from([
                    "datum-gui",
                    "--project-root",
                    self.target.to_str().unwrap(),
                ])
                .unwrap();
                let proxy = self.host.app.terminal_event_proxy.clone();
                self.host.app = App::new(args, proxy);
                self.host.app.resumed(events);
            }
            24 => {
                assert_eq!(r.workspace().scene.pads.len(), 2);
                assert!(
                    r.workspace()
                        .scene
                        .pads
                        .iter()
                        .all(|pad| pad.net_uuid.is_some())
                );
                assert_eq!(
                    r.workspace().scene.pads[0].net_uuid,
                    r.workspace().scene.pads[1].net_uuid
                );
                assert_eq!(snapshot(&self.source), self.source_bytes);
                self.capture("component-board-after-save");
                std::fs::write(self.output.join("result.json"), serde_json::to_vec_pretty(&json!({"status":"PASS","scope":"ordinary native acquisition/cancel, component placement/history, wired assignment, declared transfer and cold App","terminal_stage":25})).unwrap()).unwrap();
                events.exit();
            }
            30 => {
                assert!(r.workspace().library.acquisition.is_none());
                assert_eq!(snapshot(&self.target), self.before_acquisition);
                assert_eq!(snapshot(&self.source), self.source_bytes);
                self.host.menu("Component");
            }
            31 => {
                self.stage = 1;
            }
            _ => unreachable!(),
        }
        self.stage += 1;
        self.host.since = Instant::now();
        self.host.app.request_workspace_redraw();
    }
}
impl ApplicationHandler for Workflow {
    fn resumed(&mut self, e: &ActiveEventLoop) {
        self.host.app.resumed(e);
    }
    fn window_event(&mut self, e: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        if !matches!(
            event,
            WindowEvent::CursorMoved { .. }
                | WindowEvent::MouseInput { .. }
                | WindowEvent::KeyboardInput { .. }
                | WindowEvent::MouseWheel { .. }
                | WindowEvent::CursorLeft { .. }
        ) {
            self.host.app.window_event(e, id, event);
        }
    }
    fn about_to_wait(&mut self, e: &ActiveEventLoop) {
        self.host.app.about_to_wait(e);
        if self.host.since.elapsed() > Duration::from_secs(30)
            || self
                .host
                .app
                .window
                .as_ref()
                .is_some_and(|w| self.host.app.frames.rendering_failed(w.id()))
        {
            self.host.failure = Some(format!(
                "component stage {} timed out or rendering paused",
                self.stage
            ));
            e.exit();
            return;
        }
        self.advance(e);
        let due = Instant::now() + Duration::from_millis(20);
        use winit::event_loop::ControlFlow;
        e.set_control_flow(match e.control_flow() {
            ControlFlow::Poll => ControlFlow::Poll,
            ControlFlow::Wait => ControlFlow::WaitUntil(due),
            ControlFlow::WaitUntil(native) => ControlFlow::WaitUntil(native.min(due)),
        });
    }
}
#[test]
#[ignore = "requires fresh owned DATUM_COMPONENT_PROJECT/SOURCE/OUTPUT and isolated Wayland"]
fn ordinary_component_acquisition_capture_assignment_and_declared_transfer() {
    let target = PathBuf::from(std::env::var_os("DATUM_COMPONENT_PROJECT").unwrap());
    let source = PathBuf::from(std::env::var_os("DATUM_COMPONENT_SOURCE").unwrap());
    let output = PathBuf::from(std::env::var_os("DATUM_COMPONENT_OUTPUT").unwrap());
    assert!(
        !source.exists() && !target.exists(),
        "Fresh owned fixture directories required"
    );
    std::fs::create_dir_all(&output).unwrap();
    let part = setup(&source, &target);
    let index: Value =
        serde_json::from_slice(&std::fs::read(target.join("schematic/schematic.json")).unwrap())
            .unwrap();
    let path = target.join("schematic").join(
        index["sheets"]
            .as_object()
            .unwrap()
            .values()
            .next()
            .unwrap()
            .as_str()
            .unwrap(),
    );
    let preimage = std::fs::read(&path).unwrap();
    let original = serde_json::from_slice(&preimage).unwrap();
    let args = <GuiArgs as clap::Parser>::try_parse_from([
        "datum-gui",
        "--project-root",
        target.to_str().unwrap(),
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
        path,
        preimage,
        wired: None,
        failure: None,
        wire_only: false,
    };
    let source_bytes = snapshot(&source);
    let before_acquisition = snapshot(&target);
    let mut workflow = Workflow {
        host,
        source,
        target,
        output,
        part,
        stage: 0,
        original,
        source_bytes,
        before_acquisition,
        cancelled_preview: false,
    };
    events.run_app(&mut workflow).unwrap();
    assert!(
        workflow.host.failure.is_none(),
        "{:?}",
        workflow.host.failure
    );
    assert_eq!(workflow.stage, 25);
}
