//! Ordinary App repair after an accepted schematic net revision. Fixture setup
//! uses public writers; every copper edit below uses the existing GUI controls.
use super::*;
use datum_gui_protocol::{
    SelectionSubject, SelectionTarget,
    board_authoring_controls::{BoardControl as C, BoardField as F},
};
use std::collections::VecDeque;
use winit::keyboard::{Key, NamedKey};

#[derive(Debug)]
enum Step {
    Focus,
    Menu(&'static str),
    Item(&'static str, &'static str),
    Select(Uuid, PointNm),
    Delete(Uuid),
    Deleted(Uuid),
    PointerDelete(Uuid, PointNm),
    Unsupported,
    Control(C),
    Width,
    Point(PointNm),
    Routes(usize),
    Filled,
    UndoFill,
    RedoFill,
    Report,
    Save,
    Cold,
    Done,
}
struct Repair {
    host: Fixture,
    root: PathBuf,
    output: PathBuf,
    steps: VecDeque<Step>,
    completed: usize,
    expected: Value,
    physical: Value,
    identities: Value,
    zone: Uuid,
    filled: Option<Value>,
    delete_journal_len: Option<usize>,
}
impl Repair {
    fn model(&self) -> DesignModel {
        ProjectResolver::new(&self.root).resolve().unwrap()
    }
    fn board(&self) -> Value {
        self.model()
            .materialized_source_shard_value(SourceShardKind::BoardRoot)
            .unwrap()
    }
    fn invariant(&self) {
        let model = self.model();
        assert_eq!(
            serde_json::to_value(&model.electrical_identities).unwrap(),
            self.identities
        );
        let board = self.board();
        for key in [
            "uuid",
            "packages",
            "component_pads",
            "vias",
            "zones",
            "nets",
            "outline",
        ] {
            assert_eq!(board[key], self.physical[key], "Repair changed {key}");
        }
        assert!(
            self.host
                .app
                .runtime
                .as_ref()
                .unwrap()
                .workspace()
                .ui
                .console
                .records()
                .all(|record| !record
                    .message
                    .contains("Authoring command sent to Terminal"))
        );
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
    fn point(&mut self, events: &ActiveEventLoop, point: PointNm) {
        let r = self.host.app.runtime.as_mut().unwrap();
        let layout = r.workspace().ui.layout.clone();
        let prepared = r.prepared_scene().unwrap();
        let panes = prepared.layout.viewport_panes(&layout);
        let viewport = panes
            .panes
            .iter()
            .find(|p| p.content == PaneContent::Board)
            .unwrap()
            .rect
            .scene;
        let (x, y) = (viewport.x + 20., viewport.y + 20.);
        let a = prepared.world_point_at_screen(x, y).unwrap().0;
        let b = prepared
            .world_point_at_screen(x + 100., y + 100.)
            .unwrap()
            .0;
        let at = (
            (x + (point.x - a.x) as f32 * 100. / (b.x - a.x) as f32) as f64,
            (y + (point.y - a.y) as f32 * 100. / (b.y - a.y) as f32) as f64,
        );
        assert!(matches!(
            prepared.world_point_at_screen(at.0 as f32, at.1 as f32),
            Some((_, datum_gui_render::SceneSurface::Board))
        ));
        self.host.mouse(events, at, true);
    }
    fn ensure_control(&mut self, events: &ActiveEventLoop, control: C) -> bool {
        let r = self.host.app.runtime.as_mut().unwrap();
        let layout = r.workspace().ui.layout.clone();
        let header = r
            .prepared_scene()
            .unwrap()
            .layout
            .viewport_panes(&layout)
            .focused_pane()
            .rect
            .header;
        let hits = r.presented_hits.regions();
        if hits.iter().any(|h| {
            let toolbar = h.rect.height == 25.
                && h.rect.width >= 32.
                && header.contains(h.rect.x, h.rect.y)
                && header.contains(h.rect.x + h.rect.width - 1., h.rect.y + 24.);
            h.target == HitTarget::BoardAuthoring(control) && (h.rect.height >= 29. || toolbar)
        }) {
            return true;
        }
        let scroll = hits.iter().find_map(|hit| match &hit.target {
            HitTarget::BoardAuthoringNavigation {
                offsets,
                total,
                visible,
                ..
            } => offsets
                .iter()
                .find(|(c, _, _)| *c == control)
                .map(|(_, row, span)| (hit.rect, *total, *visible, *row, *span)),
            _ => None,
        });
        if let Some((rect, total, visible, row, span)) = scroll {
            let offset = r.workspace().ui.selection_inspector.scroll_offset;
            let next = datum_gui_viewport::scroll::revealed_row_span_offset(
                offset, total, visible, row, span,
            );
            self.host.mouse(
                events,
                (
                    (rect.x + rect.width / 2.) as f64,
                    (rect.y + rect.height / 2.) as f64,
                ),
                false,
            );
            let window = self.host.app.window.as_ref().unwrap().id();
            self.host.app.window_event(
                events,
                window,
                WindowEvent::MouseWheel {
                    device_id: winit::event::DeviceId::dummy(),
                    delta: winit::event::MouseScrollDelta::LineDelta(
                        0.,
                        offset as f32 - next as f32,
                    ),
                    phase: winit::event::TouchPhase::Moved,
                },
            );
        }
        self.host.app.request_workspace_redraw();
        false
    }
    fn advance(&mut self, events: &ActiveEventLoop) {
        let Some(r) = self.host.app.runtime.as_ref() else {
            return;
        };
        if r.native_edit.pending()
            || r.check_work.pending()
            || !r.surface_transaction.has_presented()
            || r.renderer.render_session().has_pending_frame()
            || self.host.since.elapsed() < Duration::from_millis(30)
        {
            return;
        }
        assert!(!r.native_edit.blocked);
        let required = match self.steps.front().unwrap() {
            Step::Width => Some(C::Field(F::Width)),
            Step::Control(control) => Some(*control),
            _ => None,
        };
        if let Some(control) = required
            && !self.ensure_control(events, control)
        {
            return;
        }
        let r = self.host.app.runtime.as_ref().unwrap();
        let step = self.steps.front().unwrap();
        eprintln!("native-pcb-repair stage={} {step:?}", self.completed);
        match step {
            Step::Focus => {
                if r.native_selection.is_none() {
                    return;
                }
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
            }
            Step::Menu(menu) => self
                .host
                .control(events, HitTarget::MenuTitle((*menu).into())),
            Step::Item(menu, label) => {
                let deleting = *menu == "Edit" && *label == "Delete";
                self.host.control(
                    events,
                    HitTarget::MenuItem {
                        menu: (*menu).into(),
                        label: (*label).into(),
                    },
                );
                if deleting {
                    let r = self.host.app.runtime.as_mut().unwrap();
                    assert!(r.native_edit.pending());
                    assert!(r.delete_selected_board_track());
                    assert!(
                        r.native_edit.pending(),
                        "Repeated Delete must preserve its pending worker"
                    );
                }
            }
            Step::Select(track, point) => {
                assert!(
                    r.workspace()
                        .scene
                        .tracks
                        .iter()
                        .any(|t| t.source_object_uuid == track.to_string())
                );
                self.point(events, *point);
            }
            Step::Point(point) => self.point(events, *point),
            Step::Delete(track) => {
                let SelectionTarget::Native(view) = &r.workspace().selection else {
                    return;
                };
                let SelectionSubject::Object(id) = &view.subject else {
                    return;
                };
                if view.stale || id.id != *track || view.members.len() != 1 {
                    return;
                }
                self.delete_journal_len = Some(self.model().journal.len());
                self.host
                    .control(events, HitTarget::MenuTitle("Edit".into()));
            }
            Step::Deleted(track) => {
                assert_eq!(
                    self.model().journal.len(),
                    self.delete_journal_len.take().unwrap() + 1,
                    "Pending repeated Delete must not add another transaction"
                );
                self.expected["tracks"]
                    .as_object_mut()
                    .unwrap()
                    .remove(&track.to_string())
                    .unwrap();
                assert_eq!(
                    self.board(),
                    self.expected,
                    "Delete must preserve every other object"
                );
                self.invariant();
            }
            Step::PointerDelete(track, point) => {
                assert!(
                    r.workspace()
                        .scene
                        .tracks
                        .iter()
                        .any(|t| t.source_object_uuid == track.to_string())
                );
                self.delete_journal_len = Some(self.model().journal.len());
                let r = self.host.app.runtime.as_mut().unwrap();
                r.set_workspace_tool(WorkspaceTool::Delete);
                assert_eq!(r.workspace().tool, WorkspaceTool::Delete);
                self.point(events, *point);
                assert!(
                    self.host
                        .app
                        .runtime
                        .as_ref()
                        .unwrap()
                        .native_edit
                        .pending(),
                    "Delete tool canvas click must reach the native worker"
                );
            }
            Step::Unsupported => {
                let before = self.board();
                let raw = before["packages"]
                    .as_object()
                    .unwrap()
                    .values()
                    .next()
                    .unwrap();
                let position: eda_engine::ir::geometry::Point =
                    serde_json::from_value(raw["position"].clone()).unwrap();
                let r = self.host.app.runtime.as_mut().unwrap();
                r.set_workspace_tool(WorkspaceTool::Delete);
                assert_eq!(r.workspace().tool, WorkspaceTool::Delete);
                self.point(
                    events,
                    PointNm {
                        x: position.x,
                        y: position.y,
                    },
                );
                let r = self.host.app.runtime.as_mut().unwrap();
                assert!(!r.native_edit.pending());
                assert!(r.set_workspace_tool(WorkspaceTool::Select));
                assert_eq!(
                    self.board(),
                    before,
                    "Unsupported native target must refuse"
                );
                self.capture("obsolete-copper-removed");
            }
            Step::Control(control) => {
                self.host
                    .control(events, HitTarget::BoardAuthoring(*control));
                if *control == C::Fill || *control == C::Finish {
                    assert!(
                        self.host
                            .app
                            .runtime
                            .as_ref()
                            .unwrap()
                            .native_edit
                            .pending(),
                        "Copper must use the canonical native worker"
                    );
                }
            }
            Step::Width => {
                self.host
                    .control(events, HitTarget::BoardAuthoring(C::Field(F::Width)));
                let r = self.host.app.runtime.as_mut().unwrap();
                let count = r
                    .workspace()
                    .authoring
                    .board_controls
                    .as_ref()
                    .unwrap()
                    .field(F::Width)
                    .text
                    .chars()
                    .count();
                assert!(
                    r.handle_shared_editor_input(&Key::Named(NamedKey::End), ElementState::Pressed)
                );
                for _ in 0..count {
                    assert!(r.handle_shared_editor_input(
                        &Key::Named(NamedKey::Backspace),
                        ElementState::Pressed
                    ));
                }
                assert!(r.handle_shared_editor_input(
                    &Key::Character("0.2 mm".into()),
                    ElementState::Pressed
                ));
            }
            Step::Routes(count) => {
                assert_eq!(self.board()["tracks"].as_object().unwrap().len(), *count);
                self.invariant();
            }
            Step::Filled => {
                assert!(self.model().current_zone_fill(self.zone).is_some());
                self.filled =
                    Some(serde_json::to_value(&self.model().zone_fills[&self.zone]).unwrap());
                self.capture("repaired-copper-filled");
            }
            Step::UndoFill => {
                assert!(
                    self.model().current_zone_fill(self.zone).is_none(),
                    "Undo must disclose stale fill"
                );
                self.invariant();
            }
            Step::RedoFill => {
                let model = self.model();
                assert!(model.current_zone_fill(self.zone).is_some());
                assert_eq!(
                    serde_json::to_value(&model.zone_fills[&self.zone]).unwrap(),
                    *self.filled.as_ref().unwrap()
                );
                self.invariant();
            }
            Step::Report => {
                let checks = &r.workspace().checks;
                assert!(checks.persisted && checks.is_current(r.workspace()));
                assert_eq!(checks.profile_id.as_deref(), Some("drc"));
                self.model()
                    .qualify_native_drc_zone_copper_basis(
                        checks.native_drc_zone_copper_basis.as_ref(),
                    )
                    .unwrap();
                let model = self.model();
                let run: Uuid = checks.check_run_id.as_ref().unwrap().parse().unwrap();
                let report = &model.check_runs[&run];
                let violations: Vec<eda_engine::drc::DrcViolation> =
                    serde_json::from_value(report.raw_report["drc"].clone()).unwrap();
                assert!(
                    violations.iter().all(|finding| finding.rule_type
                        != eda_engine::rules::ast::RuleType::Connectivity),
                    "Explicit GUI repair must resolve Connectivity findings: {violations:?}"
                );
                self.capture("repaired-drc-report");
            }
            Step::Save => {
                self.expected = self.board();
                self.host
                    .app
                    .runtime
                    .as_mut()
                    .unwrap()
                    .queue_native_edit(NativeEditAction::Save);
            }
            Step::Cold => {
                assert_eq!(self.board(), self.expected);
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
            Step::Done => {
                self.invariant();
                assert_eq!(self.board(), self.expected);
                assert!(self.model().current_zone_fill(self.zone).is_some());
                let circuit = fixture::Circuit {
                    root: self.root.clone(),
                    model: self.model(),
                    sheet: Uuid::nil(),
                };
                assert!(fixture::connectivity_codes(&circuit).is_empty());
                self.capture("repair-cold-reopen");
                events.exit();
            }
        }
        self.steps.pop_front();
        self.completed += 1;
        self.host.since = Instant::now();
        self.host.app.request_workspace_redraw();
    }
}
impl ApplicationHandler for Repair {
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
                "PCB repair stage {} {:?} timed out",
                self.completed,
                self.steps.front()
            ));
            events.exit();
            return;
        }
        self.advance(events);
        use winit::event_loop::ControlFlow;
        let due = Instant::now() + Duration::from_millis(20);
        events.set_control_flow(match events.control_flow() {
            ControlFlow::Poll => ControlFlow::Poll,
            ControlFlow::Wait => ControlFlow::WaitUntil(due),
            ControlFlow::WaitUntil(native) => ControlFlow::WaitUntil(native.min(due)),
        });
    }
}
fn menu(steps: &mut VecDeque<Step>, title: &'static str, label: &'static str) {
    steps.extend([Step::Menu(title), Step::Item(title, label)]);
}

#[test]
#[ignore = "requires fresh owned DATUM_CONNECTIVITY_REPAIR_PROJECT and isolated compositor"]
fn ordinary_app_native_track_delete_route_refill_drc_history_save_cold() {
    use native::schematic_board_transfer::connectivity_revision as revision;
    use native::schematic_delete_proposal::{
        self as proposals, NativeProposalAction as A, NativeProposalRequest,
    };
    let root = PathBuf::from(std::env::var_os("DATUM_CONNECTIVITY_REPAIR_PROJECT").unwrap());
    let output = PathBuf::from(std::env::var_os("DATUM_CONNECTIVITY_REPAIR_OUTPUT").unwrap());
    std::fs::create_dir_all(&output).unwrap();
    let mut circuit = fixture::setup(&root);
    if let eda_engine::ir::units::PreFeatureProjectUnitsMigration::Create { profile, receipt } =
        eda_engine::ir::units::migrate_pre_feature_project_units(None).unwrap()
    {
        let prepared = native::project::build_initialize_project_display_units(
            &circuit.model,
            fixture::provenance(),
            profile,
            &receipt,
        )
        .unwrap();
        commit_prepared(&mut circuit.model, &root, prepared).unwrap();
    }
    let zone = fixture::add_filled_zone(&mut circuit);
    fixture::rewire(&mut circuit);
    let review = revision::preview(&circuit.model, circuit.sheet).unwrap();
    let proposal_id = Uuid::new_v4();
    for action in [
        A::CreateBoardConnectivityRevision {
            proposal_id,
            sheet: circuit.sheet,
            review,
        },
        A::AcceptApply { proposal_id },
    ] {
        let request = NativeProposalRequest {
            project_root: root.clone(),
            expected_model_revision: circuit.model.model_revision.0.clone(),
            expected_transaction_tip: eda_engine::revision::transaction_tip(&circuit.model.journal),
            action,
        };
        proposals::execute(&mut circuit.model, &request).unwrap();
    }
    let board = circuit
        .model
        .materialized_source_shard_value(SourceShardKind::BoardRoot)
        .unwrap();
    let mut steps = VecDeque::from([Step::Focus]);
    menu(&mut steps, "View", "Maximize Pane");
    menu(&mut steps, "View", "Fit to Board");
    for (index, raw) in board["tracks"].as_object().unwrap().values().enumerate() {
        let t: eda_engine::board::Track = serde_json::from_value(raw.clone()).unwrap();
        let point = PointNm {
            x: (t.from.x + t.to.x) / 2,
            y: (t.from.y + t.to.y) / 2,
        };
        if index < 2 {
            steps.extend([
                Step::Select(t.uuid, point),
                Step::Delete(t.uuid),
                Step::Item("Edit", "Delete"),
                Step::Deleted(t.uuid),
            ]);
        } else {
            steps.extend([Step::PointerDelete(t.uuid, point), Step::Deleted(t.uuid)]);
        }
    }
    steps.push_back(Step::Unsupported);
    let projection = eda_engine::substrate::component_pad_source::component_pad_projection(
        &circuit.model,
        &board,
    )
    .unwrap();
    let mut groups = std::collections::BTreeMap::<Uuid, Vec<_>>::new();
    for pad in projection.pads.values() {
        groups
            .entry(pad.net.unwrap())
            .or_default()
            .push(pad.clone());
    }
    let mut upper = groups.values().find(|p| p.len() == 2).unwrap().clone();
    upper.sort_by_key(|p| p.position.x);
    let lower = groups.values().find(|p| p.len() == 4).unwrap();
    let changed = lower.iter().find(|p| p.position.y > 0).unwrap();
    let mut bottom: Vec<_> = lower.iter().filter(|p| p.position.y < 0).collect();
    bottom.sort_by_key(|p| p.position.x);
    let end = bottom
        .iter()
        .find(|p| p.position.x == changed.position.x)
        .unwrap();
    let point = |p: eda_engine::ir::geometry::Point| PointNm { x: p.x, y: p.y };
    let routes = vec![
        vec![
            upper[0].position,
            eda_engine::ir::geometry::Point::new(upper[0].position.x, 6_000_000),
            eda_engine::ir::geometry::Point::new(upper[1].position.x, 6_000_000),
            upper[1].position,
        ],
        vec![bottom[0].position, bottom[1].position],
        vec![bottom[2].position, bottom[1].position],
        vec![changed.position, end.position],
    ];
    let mut count = 0;
    for points in routes {
        steps.extend([Step::Control(C::Fill), Step::Control(C::Route), Step::Width]);
        count += points.len() - 1;
        steps.extend(points.into_iter().map(|p| Step::Point(point(p))));
        steps.extend([Step::Control(C::Finish), Step::Routes(count)]);
    }
    steps.extend([Step::Control(C::Fill), Step::Filled]);
    menu(&mut steps, "Edit", "Undo");
    steps.push_back(Step::UndoFill);
    menu(&mut steps, "Edit", "Redo");
    steps.push_back(Step::RedoFill);
    menu(&mut steps, "Checks", "Run DRC");
    steps.push_back(Step::Report);
    steps.extend([Step::Save, Step::Cold, Step::Done]);
    let total = steps.len();
    let path = circuit
        .model
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
    let mut repair = Repair {
        host,
        root,
        output,
        steps,
        completed: 0,
        expected: board.clone(),
        physical: board,
        identities: serde_json::to_value(&circuit.model.electrical_identities).unwrap(),
        zone,
        filled: None,
        delete_journal_len: None,
    };
    events.run_app(&mut repair).unwrap();
    assert!(repair.host.failure.is_none(), "{:?}", repair.host.failure);
    assert_eq!(repair.completed, total);
    std::fs::write(repair.output.join("result.json"),serde_json::to_vec_pretty(&json!({
        "status":"PASS","completed_steps":total,
        "scope":"Programmatic ordinary App native Track Delete, explicit routing, refill, current DRC with no Connectivity findings, fill Undo/Redo, Save and cold reopen. Public fixture setup accepted the schematic net revision. No human, owner-project, release-runtime or full DRC-clean claim."
    })).unwrap()).unwrap();
}
