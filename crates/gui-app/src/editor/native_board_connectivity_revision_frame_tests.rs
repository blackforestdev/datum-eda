//! Ordinary App connectivity review on a native three-component routed circuit.
//! Programmatic input is not physical-input or owner-project qualification.
use super::*;
use datum_gui_protocol::{
    board_transfer::BoardTransferAction as Transfer, native_edit_client::NativeEditAction,
};
use eda_engine::{
    api::native_write::{self as native, WriteProvenance, commit_prepared},
    substrate::{CommitSource, DesignModel, ProjectResolver, SourceShardKind},
};
use serde_json::{Value, json};
use uuid::Uuid;
#[path = "native_board_connectivity_revision_frame_tests/fixture.rs"]
mod fixture;
struct Workflow {
    host: Fixture,
    root: PathBuf,
    output: PathBuf,
    step: usize,
    original: Value,
    accepted: Option<Value>,
    original_identities: Value,
    accepted_identities: Option<Value>,
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
                    .control(events, HitTarget::MenuTitle("View".into()));
            }
            1 => self.host.control(
                events,
                HitTarget::MenuItem {
                    menu: "View".into(),
                    label: "Maximize Pane".into(),
                },
            ),
            2 => {
                assert_eq!(
                    r.workspace().ui.layout.zoomed,
                    Some(r.workspace().ui.layout.focused)
                );
                self.host
                    .control(events, HitTarget::MenuTitle("Edit".into()));
            }
            3 | 7 => self.host.control(
                events,
                HitTarget::MenuItem {
                    menu: "Edit".into(),
                    label: "Transfer Schematic to PCB…".into(),
                },
            ),
            4 | 8 => self.host.control(
                events,
                HitTarget::BoardTransfer(Transfer::ReviewConnectivity),
            ),
            5 | 9 => {
                let review = r
                    .workspace()
                    .schematic_authoring
                    .delete_review
                    .as_ref()
                    .unwrap();
                assert!(review.ready() && review.current(r.workspace()));
                let evidence = review.board_connectivity_revision.as_ref().unwrap();
                let payload = evidence.rows.join("\n");
                for change in &evidence.summary.changes {
                    for id in [
                        change.component_instance,
                        change.terminal.source_id,
                        change.pad,
                        change.previous_logical_net,
                        change.logical_net,
                        change.previous_board_net,
                        change.board_net,
                    ] {
                        assert!(
                            payload.contains(&id.to_string()),
                            "Missing complete source ID {id}"
                        );
                    }
                }
                for copper in &evidence.summary.retained_copper {
                    assert!(payload.contains(&copper.source_id.to_string()));
                }
                use datum_gui_protocol::schematic_authoring::field_table::FieldTableControl as C;
                assert!(
                    r.presented_hits
                        .regions()
                        .iter()
                        .any(|hit| hit.target == HitTarget::SchematicFieldTable(C::Accept)),
                    "Complete review must actually present an enabled Apply control"
                );
                assert!(
                    r.presented_hits
                        .regions()
                        .iter()
                        .any(|hit| hit.target == HitTarget::SchematicFieldTable(C::Cancel))
                );
                assert_eq!(evidence.summary.changes.len(), 1);
                assert_eq!(evidence.summary.zone_fills.len(), 1);
                assert!(!evidence.summary.retained_copper.is_empty());
                assert!(
                    evidence
                        .rows
                        .iter()
                        .any(|row| row.contains("contacts unavailable"))
                );
                assert!(
                    evidence
                        .rows
                        .iter()
                        .any(|row| row.contains("old net assignments"))
                );
                assert_eq!(
                    self.board(),
                    self.original,
                    "Draft leaves the board untouched"
                );
                self.capture(if self.step == 5 {
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
                        &winit::keyboard::Key::Named(if self.step == 5 {
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
                if self.step == 5 {
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
            6 => {
                assert!(r.workspace().schematic_authoring.delete_review.is_none());
                assert_eq!(self.board(), self.original);
                self.host
                    .control(events, HitTarget::MenuTitle("Edit".into()));
            }
            10 => {
                crate::runtime_native_edit::proposal_completion_tests::completed_echo_precedes_workspace_warning(
                    r,"PCB pin/net correspondence updated");
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
                assert_eq!(
                    accepted, self.original,
                    "Correspondence Apply preserves the complete authored BoardRoot"
                );
                assert_eq!(accepted["packages"], self.original["packages"]);
                assert_eq!(accepted["vias"], self.original["vias"]);
                assert_eq!(accepted["zones"], self.original["zones"]);
                self.accepted_identities =
                    Some(serde_json::to_value(&self.model().electrical_identities).unwrap());
                assert_ne!(
                    self.accepted_identities.as_ref().unwrap(),
                    &self.original_identities
                );
                assert_eq!(accepted["tracks"], self.original["tracks"]);
                assert!(!accepted["tracks"].as_object().unwrap().is_empty());
                self.accepted = Some(accepted);
                self.history(false);
            }
            11 => {
                assert_eq!(
                    serde_json::to_value(&self.model().electrical_identities).unwrap(),
                    self.original_identities
                );
                assert_eq!(self.board(), self.original);
                self.history(true);
            }
            12 => {
                assert_eq!(
                    serde_json::to_value(&self.model().electrical_identities).unwrap(),
                    *self.accepted_identities.as_ref().unwrap()
                );
                assert_eq!(self.board(), *self.accepted.as_ref().unwrap());
                self.host
                    .app
                    .runtime
                    .as_mut()
                    .unwrap()
                    .queue_native_edit(NativeEditAction::Save);
            }
            13 => {
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
            14 => {
                assert_eq!(self.board(), *self.accepted.as_ref().unwrap());
                let model = self.model();
                assert_eq!(model.component_instances.len(), 3);
                assert_eq!(
                    serde_json::to_value(&model.electrical_identities).unwrap(),
                    *self.accepted_identities.as_ref().unwrap()
                );
                let projection =
                    eda_engine::substrate::component_pad_source::component_pad_projection(
                        &model,
                        &self.board(),
                    )
                    .unwrap();
                assert_eq!(projection.query_pads.len(), 6);
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
            15 => {
                self.capture("pcb-connectivity-cold");
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
            self.host.failure = Some(format!(
                "PCB connectivity review stage {} timed out",
                self.step
            ));
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
#[ignore = "requires owned fresh DATUM_CONNECTIVITY_REVISION_PROJECT and isolated native Wayland"]
fn ordinary_app_pcb_connectivity_revision_reject_apply_history_save_cold() {
    let root = PathBuf::from(std::env::var_os("DATUM_CONNECTIVITY_REVISION_PROJECT").unwrap());
    let output = PathBuf::from(std::env::var_os("DATUM_CONNECTIVITY_REVISION_OUTPUT").unwrap());
    std::fs::create_dir_all(&output).unwrap();
    let mut circuit = fixture::setup(&root);
    let zone = fixture::add_filled_zone(&mut circuit);
    fixture::rewire(&mut circuit);
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
        root: root.clone(),
        output,
        step: 0,
        original,
        accepted: None,
        original_identities: serde_json::to_value(&model.electrical_identities).unwrap(),
        accepted_identities: None,
    };
    events.run_app(&mut workflow).unwrap();
    assert!(
        workflow.host.failure.is_none(),
        "{:?}",
        workflow.host.failure
    );
    assert_eq!(workflow.step, 16);
    // Explicit native manual repair/refill/check is separate from Apply; no UI-routing claim.
    circuit.model = ProjectResolver::new(&root).resolve().unwrap();
    let unavailable = fixture::connectivity_findings(&circuit);
    assert!(
        unavailable
            .iter()
            .any(|finding| finding.code == "nominal_connectivity_unavailable"
                && finding.objects.contains(&zone)),
        "Unavailable fill must be disclosed against its actual Zone source: {unavailable:?}"
    );
    let identities = circuit.model.electrical_identities.clone();
    fixture::repair(&mut circuit, zone);
    assert!(fixture::connectivity_codes(&circuit).is_empty());
    circuit.model = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(circuit.model.electrical_identities, identities);
    assert!(fixture::connectivity_codes(&circuit).is_empty());
    std::fs::write(workflow.output.join("result.json"), serde_json::to_vec_pretty(&json!({
        "status":"PASS", "completed_steps":workflow.step,
        "scope":"Ordinary App synthetic three-component/six-terminal circuit: existing View > Maximize Pane then complete review Reject/Apply, retained copper, pending dismissal, complete correspondence Undo/Redo, Save/cold. Separate public native explicit route repair/refill/connectivity check and cold verification. Programmatic events, not owner-project or human qualification."
    })).unwrap()).unwrap();
}
