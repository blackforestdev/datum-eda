//! Ordinary App review/transfer/history with authenticated native setup.
use super::*;
use datum_gui_protocol::board_transfer::BoardTransferAction as Action;
use eda_engine::api::native_write::{
    self as native, WriteProvenance, commit_prepared,
    registry::{NativeWriteContext, find_native_write_verb},
};
use eda_engine::substrate::{
    CommitSource, DesignModel, ElectricalSelectionSnapshot, ProjectResolver, SourceShardKind,
};
use serde_json::{Value, json};
use std::collections::BTreeMap;
use uuid::Uuid;
fn provenance() -> WriteProvenance {
    WriteProvenance::new(
        "transfer-test",
        CommitSource::Manual,
        "Initial connected PCB transfer",
    )
}
fn write(model: &mut DesignModel, root: &std::path::Path, verb: &str, params: Value) -> Uuid {
    let p = (find_native_write_verb(verb).unwrap().build)(
        &NativeWriteContext {
            model,
            project_root: root,
        },
        provenance(),
        params,
    )
    .unwrap();
    let id = p.primary_object_id.unwrap_or(Uuid::nil());
    commit_prepared(model, root, p).unwrap();
    id
}
pub(super) fn setup(root: &std::path::Path) {
    let _report = native::genesis::bootstrap_native_project(
        &root,
        native::genesis::GenesisSpec {
            project_name: "Connected transfer".into(),
            existing_ids: None,
        },
    )
    .unwrap();
    let mut model = ProjectResolver::new(&root).resolve().unwrap();
    let (unit, pin, symbol, entity, gate, package, stack, footprint, pad, part, map, sheet) = (
        Uuid::new_v4(),
        Uuid::new_v4(),
        Uuid::new_v4(),
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
            json!({"name":"Passive pin","pins":{pin.to_string():{"name":"Signal","electrical_type":"Passive"}}}),
        ),
        (
            "symbols",
            symbol,
            json!({"name":"Transfer body","unit":unit,"drawings":[{"Rect":{"min":{"x":-500000,"y":-500000},"max":{"x":500000,"y":500000},"width":100000}}],"pin_anchors":[{"pin":pin,"position":{"x":-2000000,"y":0},"style":{"orientation":"Right","length_nm":1000000,"decoration":"none"}}]}),
        ),
        ("packages", package, json!({"name":"Package"})),
        (
            "padstacks",
            stack,
            json!({"name":"Copper","aperture":{"circle":{"diameter_nm":1000000}}}),
        ),
        (
            "footprints",
            footprint,
            json!({"name":"One pad","package":package,"pads":{pad.to_string():{"uuid":pad,"name":"1","position":{"x":0,"y":0},"padstack":stack,"layer":1}}}),
        ),
        (
            "entities",
            entity,
            json!({"name":"Entity","manufacturer":"Designer","prefix":"U","gates":{gate.to_string():{"uuid":gate,"name":"G","unit":unit,"symbol":symbol}}}),
        ),
        (
            "parts",
            part,
            json!({"manufacturer":"Designer","mpn":"TRANSFER","description":"","datasheet":"","value":"Signal","entity":entity,"package":package,"default_footprint":footprint}),
        ),
        (
            "pin_pad_maps",
            map,
            json!({"part":part,"footprint":footprint,"mappings":{pad.to_string():{"gate":gate,"pin":pin}}}),
        ),
    ] {
        write(
            &mut model,
            &root,
            "datum.library.create",
            json!({"kind":kind,"object_id":id,"pool_path":"pool","fields":fields}),
        );
    }
    write(
        &mut model,
        &root,
        "datum.schematic.create_sheet",
        json!({"sheet":sheet,"name":"Root"}),
    );
    for (reference, x) in [("U1", 0), ("U2", 10000000)] {
        let intent = native::schematic_library::part_placement::PartGateSelection::capture(
            &model, part, gate, symbol,
        )
        .unwrap();
        write(
            &mut model,
            &root,
            "datum.schematic.place_symbol",
            json!({"sheet":sheet,"instance_path":[],"library_symbol":symbol,"explicit_part":intent,"reference":reference,"value":"Signal","x_nm":x,"y_nm":0}),
        );
    }
    write(
        &mut model,
        &root,
        "datum.schematic.draw_wire",
        json!({"sheet":sheet,"instance_path":[],"points":[{"x":-2000000,"y":0},{"x":8000000,"y":0}]}),
    );
}

struct TransferWorkflow {
    host: Fixture,
    step: usize,
    root: PathBuf,
    output: PathBuf,
    preimage: BTreeMap<PathBuf, Vec<u8>>,
    journal: usize,
}
impl TransferWorkflow {
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
    fn capture(&mut self, label: &str) {
        self.host
            .app
            .runtime
            .as_mut()
            .unwrap()
            .write_visual_screenshot(&self.output.join(format!("{label}.png")))
            .unwrap();
    }
    fn control(&mut self, events: &ActiveEventLoop, action: Action) {
        self.host.control(events, HitTarget::BoardTransfer(action));
    }
    fn model(&self) -> DesignModel {
        ProjectResolver::new(&self.root).resolve().unwrap()
    }
    fn count(&self) -> usize {
        self.model()
            .materialized_source_shard_value(SourceShardKind::BoardRoot)
            .unwrap()["packages"]
            .as_object()
            .unwrap()
            .len()
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
            "board-transfer-native stage={} status={:?}",
            self.step,
            r.workspace().last_command_status
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
                    .find(|p| {
                        r.workspace().ui.layout.content_for(*p) == Some(PaneContent::Schematic)
                    })
                    .unwrap();
                r.session.workspace_mut().ui.layout.focused = pane;
                r.set_application_focus(datum_gui_protocol::ApplicationFocus::Editor(pane));
                self.preimage = self.snapshot();
                self.journal = self.model().journal.len();
                self.host
                    .control(events, HitTarget::MenuTitle("Edit".into()));
            }
            1 | 5 => self.host.control(
                events,
                HitTarget::MenuItem {
                    menu: "Edit".into(),
                    label: "Transfer Schematic to PCB…".into(),
                },
            ),
            2 => {
                let review = r.workspace().ui.board_transfer.as_ref().unwrap();
                assert_eq!(review.plan.ready_count(), 0);
                assert!(review.plan.requires_profile_adoption);
                assert!(review.plan.components.iter().all(|c| c.reason.is_some()));
                self.capture("transfer-unresolved");
                self.control(events, Action::ChooseMap(0));
            }
            3 => {
                self.control(events, Action::Cancel);
                assert!(
                    self.host
                        .app
                        .runtime
                        .as_ref()
                        .unwrap()
                        .workspace()
                        .ui
                        .board_transfer
                        .is_none()
                );
                assert_eq!(self.snapshot(), self.preimage);
            }
            4 => self
                .host
                .control(events, HitTarget::MenuTitle("Edit".into())),
            6 => self.control(events, Action::ChooseMap(0)),
            7 => self.control(events, Action::ChooseMap(1)),
            8 => {
                assert_eq!(
                    r.workspace()
                        .ui
                        .board_transfer
                        .as_ref()
                        .unwrap()
                        .plan
                        .ready_count(),
                    2
                );
                assert_eq!(self.snapshot(), self.preimage);
                self.capture("transfer-ready-unprepared");
                self.control(events, Action::PrepareSources);
            }
            9 => {
                let review = r.workspace().ui.board_transfer.as_ref().unwrap();
                assert!(!review.plan.requires_profile_adoption);
                assert_eq!(self.count(), 0);
                assert_eq!(self.model().journal.len(), self.journal + 1);
                self.capture("transfer-ready");
                self.control(events, Action::Accept);
            }
            10 => {
                assert_eq!(self.count(), 2);
                let board = self
                    .model()
                    .materialized_source_shard_value(SourceShardKind::BoardRoot)
                    .unwrap();
                assert!(
                    board["packages"]
                        .as_object()
                        .unwrap()
                        .values()
                        .all(|p| p["value"] == "Signal")
                );
                assert_eq!(self.model().journal.len(), self.journal + 2);
                let review = r.workspace().ui.board_transfer.as_ref().unwrap();
                assert_eq!(review.plan.ready_count(), 0);
                assert!(review.plan.components.iter().all(|c| c.status==eda_engine::api::native_write::schematic_board_transfer::TransferStatus::Existing));
                self.preimage = self.snapshot();
                self.capture("transfer-existing");
                self.control(events, Action::Cancel);
            }
            11 => {
                assert_eq!(self.snapshot(), self.preimage);
                self.host
                    .app
                    .runtime
                    .as_mut()
                    .unwrap()
                    .queue_native_history(false);
            }
            12 => {
                assert_eq!(self.count(), 0);
                self.host
                    .app
                    .runtime
                    .as_mut()
                    .unwrap()
                    .queue_native_history(true);
            }
            13 => {
                assert_eq!(self.count(), 2);
                self.host.app.runtime.as_mut().unwrap().queue_native_edit(
                    datum_gui_protocol::native_edit_client::NativeEditAction::Save,
                );
            }
            14 => {
                let model = self.model();
                let snapshot = ElectricalSelectionSnapshot::capture(&model).unwrap();
                let terminal = snapshot
                    .authored_sources()
                    .unwrap()
                    .into_iter()
                    .find(|o| o.occurrence.class == "pins")
                    .unwrap()
                    .occurrence;
                let net = snapshot
                    .net_for_origin(&model.model_revision, &terminal)
                    .unwrap();
                assert_eq!(net.owned.iter().filter(|o| o.class == "pads").count(), 2);
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
            15 => {
                assert_eq!(self.count(), 2);
                let r = self.host.app.runtime.as_mut().unwrap();
                let pane = r
                    .workspace()
                    .ui
                    .layout
                    .leaves()
                    .into_iter()
                    .find(|p| r.workspace().ui.layout.content_for(*p) == Some(PaneContent::Board))
                    .unwrap();
                r.session.workspace_mut().ui.layout.focused = pane;
                r.set_application_focus(datum_gui_protocol::ApplicationFocus::Editor(pane));
                r.fit_camera();
            }
            16 => {
                self.preimage = self.snapshot();
                let layer = r
                    .workspace()
                    .scene
                    .layers
                    .iter()
                    .find(|layer| layer.layer_id == "L1")
                    .unwrap();
                let visible = r
                    .workspace()
                    .ui
                    .filters
                    .layer_visibility
                    .get("L1")
                    .copied()
                    .unwrap_or(layer.visible_by_default);
                assert!(
                    layer.visible_by_default,
                    "Native copper must default visible"
                );
                assert!(
                    visible,
                    "Cold native copper must be visible without a layer toggle"
                );
            }
            17 => {
                assert!(
                    r.workspace()
                        .ui
                        .filters
                        .layer_visibility
                        .get("L1")
                        .copied()
                        .unwrap_or(false)
                );
                assert_eq!(
                    self.snapshot(),
                    self.preimage,
                    "Layer visibility is consumer-only"
                );
                let scene = &r.workspace().scene;
                assert_eq!(scene.pads.len(), 2);
                assert!(
                    scene.pads.iter().all(|pad| pad.layer_id == "L1"
                        && pad.copper_layer_ids.iter().any(|id| id == "L1"))
                );
                assert!(scene.pads.iter().all(|p| p.net_uuid.is_some()));
                assert_eq!(scene.pads[0].net_uuid, scene.pads[1].net_uuid);
                assert_eq!(scene.unrouted_primitives.len(), 1);
                std::fs::write(
                    self.output.join("cold-board-scene.json"),
                    serde_json::to_vec_pretty(scene).unwrap(),
                )
                .unwrap();
                self.capture("transfer-board-cold");
                events.exit();
            }
            _ => unreachable!(),
        }
        self.step += 1;
        self.host.since = Instant::now();
        self.host.app.request_workspace_redraw();
    }
}
impl ApplicationHandler for TransferWorkflow {
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
#[ignore = "requires owned fresh DATUM_TRANSFER_PROJECT and isolated native Wayland"]
fn ordinary_app_initial_connected_transfer_review_cancel_commit_history_save_reopen() {
    let root = PathBuf::from(std::env::var_os("DATUM_TRANSFER_PROJECT").unwrap());
    let output = PathBuf::from(std::env::var_os("DATUM_TRANSFER_OUTPUT").unwrap());
    std::fs::create_dir_all(&output).unwrap();
    setup(&root);
    let index: Value =
        serde_json::from_slice(&std::fs::read(root.join("schematic/schematic.json")).unwrap())
            .unwrap();
    let path = root.join("schematic").join(
        index["sheets"]
            .as_object()
            .unwrap()
            .values()
            .next()
            .unwrap()
            .as_str()
            .unwrap(),
    );
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
    let mut workflow = TransferWorkflow {
        host,
        step: 0,
        root,
        output,
        preimage: BTreeMap::new(),
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
