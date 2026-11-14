//! Ordinary App review/Reject/Accept/history/Save/cold on a routed counterpart.
//! Programmatic input is not a physical-input or owner-project QA claim.
use super::*;
use datum_gui_protocol::board_transfer::BoardTransferAction as Transfer;
use datum_gui_protocol::native_edit_client::NativeEditAction;
use eda_engine::{
    api::native_write::{
        self as native, WriteProvenance, commit_prepared,
        registry::{NativeWriteContext, find_native_write_verb},
        schematic_delete_proposal::{
            self as proposals, FieldRowEdit, NativeProposalAction as Action, NativeProposalRequest,
        },
    },
    substrate::{CommitSource, DesignModel, ProjectResolver, SourceShardKind},
};
use serde_json::{Value, json};
use uuid::Uuid;

fn provenance() -> WriteProvenance {
    WriteProvenance::new(
        "native-frame",
        CommitSource::Manual,
        "Prepare PCB field revision fixture",
    )
}
fn write(model: &mut DesignModel, root: &std::path::Path, verb: &str, params: Value) {
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
fn setup(root: &std::path::Path) {
    board_transfer::setup(root);
    let mut model = ProjectResolver::new(root).resolve().unwrap();
    let board_id = serde_json::from_value(
        model
            .materialized_source_shard_value(SourceShardKind::BoardRoot)
            .unwrap()["uuid"]
            .clone(),
    )
    .unwrap();
    let prepared = native::board_layout::source_profile::build_adopt_board_source_profile(
        &model,
        provenance(),
        board_id,
    )
    .unwrap();
    commit_prepared(&mut model, root, prepared).unwrap();
    let source = eda_engine::substrate::ElectricalSelectionSnapshot::capture(&model).unwrap();
    let sheet = *source
        .schematic_source()
        .unwrap()
        .sheets
        .keys()
        .next()
        .unwrap();
    let maps: Vec<_> = model
        .source_shards
        .iter()
        .filter(|s| s.taxon == Some(eda_engine::substrate::SourceShardTaxon::PoolPinPadMap))
        .collect();
    assert_eq!(maps.len(), 1);
    let map = serde_json::from_value(
        model
            .materialized_source_shard_value_by_relative_path(&maps[0].relative_path)
            .unwrap()["uuid"]
            .clone(),
    )
    .unwrap();
    let request = native::schematic_board_transfer::TransferRequest {
        sheet,
        choices: model
            .component_instances
            .values()
            .map(|ci| native::schematic_board_transfer::TransferChoice {
                component_instance: ci.id,
                pin_pad_map: map,
            })
            .collect(),
        edit_seed: Uuid::new_v4(),
        origin: eda_engine::ir::geometry::Point::zero(),
        pitch_nm: 5_000_000,
        layer: 1,
    };
    let prepared =
        native::schematic_board_transfer::prepare_transfer(&model, provenance(), &request).unwrap();
    commit_prepared(&mut model, root, prepared).unwrap();
    let package = model
        .component_instances
        .values()
        .next()
        .unwrap()
        .placed_package_refs[0];
    write(
        &mut model,
        root,
        "datum.pcb.move_component",
        json!({"component":package,"x_nm":2_000_000,"y_nm":3_000_000}),
    );
    model = ProjectResolver::new(root).resolve().unwrap();
    let prepared = native::board_components::build_edit_board_package(
        &model,
        provenance(),
        package,
        native::board_components::BoardPackageEdit::Rotation { rotation: 90 },
    )
    .unwrap();
    commit_prepared(&mut model, root, prepared).unwrap();
    model = ProjectResolver::new(root).resolve().unwrap();
    let board = model
        .materialized_source_shard_value(SourceShardKind::BoardRoot)
        .unwrap();
    let projection =
        eda_engine::substrate::component_pad_source::component_pad_projection(&model, &board)
            .unwrap();
    let pads: Vec<_> = projection.pads.values().cloned().collect();
    assert_eq!(pads.len(), 2);
    write(
        &mut model,
        root,
        "datum.project.create_rule",
        json!({"rule":{
        "uuid":Uuid::new_v4(),"name":"Chosen clearance","scope":{"Net":pads[0].net.unwrap()},
        "priority":0,"enabled":true,"rule_type":"clearance_copper","params":{"min_nm":100_000}}}),
    );
    let basis = native::board_routing::route::capture_route_basis(&model, pads[0].uuid, 1).unwrap();
    write(
        &mut model,
        root,
        "datum.pcb.draw_track",
        json!({"basis":basis,"end_pad":pads[1].uuid,
        "points":[pads[0].position,pads[1].position],"width_nm":200_000,"edit_seed":Uuid::new_v4()}),
    );
    let source = eda_engine::substrate::ElectricalSelectionSnapshot::capture(&model).unwrap();
    let symbol = source.schematic_source().unwrap().sheets[&sheet]
        .symbols
        .values()
        .next()
        .unwrap()
        .uuid;
    let id = Uuid::new_v4();
    for action in [
        Action::CreateFieldBatch {
            proposal_id: id,
            sheet,
            instance_path: vec![],
            edits: vec![FieldRowEdit {
                symbol,
                reference: "AMP1".into(),
                value: "10k".into(),
            }],
        },
        Action::AcceptApply { proposal_id: id },
    ] {
        let request = NativeProposalRequest {
            project_root: root.into(),
            expected_model_revision: model.model_revision.0.clone(),
            expected_transaction_tip: eda_engine::revision::transaction_tip(&model.journal),
            action,
        };
        proposals::execute(&mut model, &request).unwrap();
    }
}

struct Workflow {
    host: Fixture,
    root: PathBuf,
    output: PathBuf,
    step: usize,
    original: Value,
    accepted: Option<Value>,
}
impl Workflow {
    fn model(&self) -> DesignModel {
        ProjectResolver::new(&self.root).resolve().unwrap()
    }
    fn board(&self) -> Value {
        self.model()
            .materialized_source_shard_value(SourceShardKind::BoardRoot)
            .unwrap()
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
    fn history(&mut self, redo: bool) {
        let tip = eda_engine::revision::transaction_tip(&self.model().journal).unwrap();
        self.host
            .app
            .runtime
            .as_mut()
            .unwrap()
            .queue_native_edit(if redo {
                NativeEditAction::Redo { expected_tip: tip }
            } else {
                NativeEditAction::Undo { expected_tip: tip }
            });
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
        assert!(
            !r.native_edit.blocked,
            "ordinary worker response must qualify"
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
            2 | 6 => self
                .host
                .control(events, HitTarget::BoardTransfer(Transfer::ReviewFields)),
            3 | 7 => {
                let review = r
                    .workspace()
                    .schematic_authoring
                    .delete_review
                    .as_ref()
                    .unwrap();
                assert!(review.ready() && review.current(r.workspace()));
                let summary = review.board_field_revision.as_ref().unwrap();
                assert_eq!(summary.summary.changes.len(), 1);
                assert_eq!(summary.summary.changes[0].reference, "AMP1");
                assert_eq!(summary.summary.changes[0].value, "10k");
                assert_eq!(summary.contexts.len(), 1);
                assert!(summary.contexts[0].contains("Part: Designer TRANSFER"));
                assert_eq!(
                    self.board(),
                    self.original,
                    "Draft leaves the board untouched"
                );
                self.capture(if self.step == 3 {
                    "review-before-reject"
                } else {
                    "review-before-accept"
                });
                self.host
                    .app
                    .runtime
                    .as_mut()
                    .unwrap()
                    .handle_schematic_review_input(
                        &winit::keyboard::Key::Named(if self.step == 3 {
                            winit::keyboard::NamedKey::Escape
                        } else {
                            winit::keyboard::NamedKey::Tab
                        }),
                        ElementState::Pressed,
                    );
                assert!(
                    self.host
                        .app
                        .runtime
                        .as_ref()
                        .unwrap()
                        .native_edit
                        .pending()
                );
                if self.step == 3 {
                    self.host
                        .app
                        .runtime
                        .as_mut()
                        .unwrap()
                        .handle_schematic_review_input(
                            &winit::keyboard::Key::Named(winit::keyboard::NamedKey::Escape),
                            ElementState::Pressed,
                        );
                } else {
                    self.host.control(events, HitTarget::SchematicFieldTable(
                        datum_gui_protocol::schematic_authoring::field_table::FieldTableControl::Cancel,
                    ));
                }
                assert!(
                    self.host
                        .app
                        .runtime
                        .as_ref()
                        .unwrap()
                        .workspace()
                        .schematic_authoring
                        .delete_review
                        .is_none()
                );
            }
            4 => {
                assert!(r.workspace().schematic_authoring.delete_review.is_none());
                assert_eq!(self.board(), self.original);
                self.host
                    .control(events, HitTarget::MenuTitle("Edit".into()));
            }
            8 => {
                assert!(
                    r.workspace()
                        .ui
                        .console
                        .records()
                        .any(|record| record.message == "PCB references and values updated")
                );
                assert!(
                    !r.workspace()
                        .ui
                        .console
                        .records()
                        .any(|record| record.message == "Selected objects deleted")
                );
                assert!(
                    r.workspace().schematic_authoring.delete_review.is_none(),
                    "late completion cannot reopen a dismissed review"
                );
                let accepted = self.board();
                assert_ne!(accepted, self.original);
                assert_eq!(accepted["component_pads"], self.original["component_pads"]);
                assert_eq!(accepted["tracks"], self.original["tracks"]);
                assert!(!accepted["tracks"].as_object().unwrap().is_empty());
                self.accepted = Some(accepted);
                self.history(false);
            }
            9 => {
                assert_eq!(self.board(), self.original);
                self.history(true);
            }
            10 => {
                assert_eq!(self.board(), *self.accepted.as_ref().unwrap());
                self.host
                    .app
                    .runtime
                    .as_mut()
                    .unwrap()
                    .queue_native_edit(NativeEditAction::Save);
            }
            11 => {
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
            12 => {
                assert_eq!(self.board(), *self.accepted.as_ref().unwrap());
                let model = self.model();
                assert_eq!(model.component_instances.len(), 2);
                let projection =
                    eda_engine::substrate::component_pad_source::component_pad_projection(
                        &model,
                        &self.board(),
                    )
                    .unwrap();
                assert_eq!(projection.query_pads.len(), 2);
                assert!(
                    projection
                        .query_pads
                        .values()
                        .all(|pad| pad.terminal().net.is_some())
                );
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
            13 => {
                self.capture("pcb-fields-cold");
                events.exit();
            }
            _ => unreachable!(),
        }
        self.step += 1;
        self.host.since = Instant::now();
        self.host.app.request_workspace_redraw();
    }
}
impl ApplicationHandler for Workflow {
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
            self.host.failure = Some(format!("PCB field review stage {} timed out", self.step));
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
#[ignore = "requires owned fresh DATUM_FIELD_REVISION_PROJECT and isolated native Wayland"]
fn ordinary_app_pcb_field_revision_reject_accept_history_save_cold() {
    let root = PathBuf::from(std::env::var_os("DATUM_FIELD_REVISION_PROJECT").unwrap());
    let output = PathBuf::from(std::env::var_os("DATUM_FIELD_REVISION_OUTPUT").unwrap());
    std::fs::create_dir_all(&output).unwrap();
    setup(&root);
    let model = ProjectResolver::new(&root).resolve().unwrap();
    let original = model
        .materialized_source_shard_value(SourceShardKind::BoardRoot)
        .unwrap();
    let path = model
        .source_shards
        .iter()
        .find(|s| s.kind == SourceShardKind::SchematicRoot)
        .unwrap()
        .path
        .clone();
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
    let mut workflow = Workflow {
        host,
        root,
        output,
        step: 0,
        original,
        accepted: None,
    };
    events.run_app(&mut workflow).unwrap();
    assert!(
        workflow.host.failure.is_none(),
        "{:?}",
        workflow.host.failure
    );
    assert_eq!(workflow.step, 14);
    std::fs::write(workflow.output.join("result.json"), serde_json::to_vec_pretty(&json!({
        "status":"PASS", "completed_steps":workflow.step,
        "scope":"Ordinary App synthetic project: reviewed PCB reference/value reject, apply, pending dismissal, Undo/Redo, Save/cold and unchanged pads/routes/net membership. Programmatic events, not owner-project or human qualification."
    })).unwrap()).unwrap();
}
