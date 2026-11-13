//! Ordinary App PCB tool proof. Circuit/transfer/outline are canonical fixture setup;
//! Route/Rule/Zone/Fill actions below use presented GUI controls and shared input.
use super::*;
use datum_gui_protocol::board_authoring_controls::{BoardControl as C, BoardField as F};
use eda_engine::api::native_write::{
    self as native, WriteProvenance, commit_prepared,
    registry::{NativeWriteContext, find_native_write_verb},
    schematic_board_transfer::{TransferChoice, TransferRequest, plan_transfer},
};
use eda_engine::ir::geometry::{Point, Polygon};
use eda_engine::substrate::{CommitSource, DesignModel, ProjectResolver, SourceShardKind};
use serde_json::json;
use std::collections::BTreeMap;
use uuid::Uuid;
use winit::keyboard::{Key, NamedKey};

fn fixture(root: &std::path::Path, with_outline: bool) -> Uuid {
    super::board_transfer::setup(root);
    let proof = || {
        WriteProvenance::new(
            "board-controls-fixture",
            CommitSource::Test,
            "Representative connected board fixture",
        )
    };
    let mut model = ProjectResolver::new(root).resolve().unwrap();
    if let eda_engine::ir::units::PreFeatureProjectUnitsMigration::Create { profile, receipt } =
        eda_engine::ir::units::migrate_pre_feature_project_units(None).unwrap()
    {
        let p = native::project::build_initialize_project_display_units(
            &model,
            proof(),
            profile,
            &receipt,
        )
        .unwrap();
        commit_prepared(&mut model, root, p).unwrap();
    }
    let schematic = model
        .materialized_source_shard_value(SourceShardKind::SchematicRoot)
        .unwrap();
    let sheet = schematic["sheets"]
        .as_object()
        .unwrap()
        .keys()
        .next()
        .unwrap()
        .parse()
        .unwrap();
    let mut request = TransferRequest {
        sheet,
        choices: vec![],
        edit_seed: Uuid::new_v4(),
        origin: Point::new(2000000, 2000000),
        pitch_nm: 5000000,
        layer: 1,
    };
    let plan = plan_transfer(&model, &request).unwrap();
    request.choices = plan
        .components
        .iter()
        .map(|c| TransferChoice {
            component_instance: c.component_instance.unwrap(),
            pin_pad_map: c.map_options[0].pin_pad_map,
        })
        .collect();
    let p = (find_native_write_verb("datum.pcb.prepare_footprint_sources")
        .unwrap()
        .build)(
        &NativeWriteContext {
            model: &model,
            project_root: root,
        },
        proof(),
        json!({"board":plan.board_id}),
    )
    .unwrap();
    commit_prepared(&mut model, root, p).unwrap();
    let p = native::schematic_board_transfer::prepare_transfer(&model, proof(), &request).unwrap();
    commit_prepared(&mut model, root, p).unwrap();
    if with_outline {
        let p = native::board_layout::build_set_board_outline(
            &model,
            proof(),
            plan.board_id,
            &Polygon::new(vec![
                Point::new(0, 0),
                Point::new(10000000, 0),
                Point::new(10000000, 6000000),
                Point::new(0, 6000000),
            ]),
        )
        .unwrap();
        commit_prepared(&mut model, root, p).unwrap();
    }
    let board = model
        .materialized_source_shard_value(SourceShardKind::BoardRoot)
        .unwrap();
    assert!(board["tracks"].as_object().unwrap().is_empty());
    assert!(board["zones"].as_object().unwrap().is_empty());
    board["nets"]
        .as_object()
        .unwrap()
        .keys()
        .next()
        .unwrap()
        .parse()
        .unwrap()
}
struct Workflow {
    host: Fixture,
    root: PathBuf,
    output: PathBuf,
    net: Uuid,
    step: usize,
    saved: BTreeMap<PathBuf, Vec<u8>>,
    outline_only: bool,
}
impl Workflow {
    fn model(&self) -> DesignModel {
        ProjectResolver::new(&self.root).resolve().unwrap()
    }
    fn count(&self, key: &str) -> usize {
        self.model()
            .materialized_source_shard_value(SourceShardKind::BoardRoot)
            .unwrap()[key]
            .as_object()
            .unwrap()
            .len()
    }
    fn snapshot(&self) -> BTreeMap<PathBuf, Vec<u8>> {
        let mut files: BTreeMap<_, _> = self
            .model()
            .source_shards
            .iter()
            .map(|s| (s.path.clone(), std::fs::read(&s.path).unwrap()))
            .collect();
        for path in [
            eda_engine::substrate::transaction_journal_path(&self.root),
            self.root.join(".datum/journal/cursor.json"),
        ] {
            if path.exists() {
                files.insert(path.clone(), std::fs::read(path).unwrap());
            }
        }
        files
    }
    fn control(&mut self, events: &ActiveEventLoop, control: C) {
        self.host
            .control(events, HitTarget::BoardAuthoring(control));
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
    fn field(&mut self, events: &ActiveEventLoop, field: F, value: &str) {
        self.control(events, C::Field(field));
        let r = self.host.app.runtime.as_mut().unwrap();
        // The shared logical-key input controller is the actual production core;
        // these programmatic keys do not claim physical/HUMAN observation.
        assert!(r.handle_shared_editor_input(&Key::Named(NamedKey::End), ElementState::Pressed));
        for _ in 0..80 {
            assert!(r.handle_shared_editor_input(
                &Key::Named(NamedKey::Backspace),
                ElementState::Pressed
            ));
        }
        assert!(r.handle_shared_editor_input(&Key::Character(value.into()), ElementState::Pressed));
        let form = r.workspace().authoring.board_controls.as_ref().unwrap();
        assert_eq!(form.field(field).text, value);
    }
    fn board(&mut self, events: &ActiveEventLoop, point: PointNm, click: bool) {
        let r = self.host.app.runtime.as_mut().unwrap();
        let layout = r.workspace().ui.layout.clone();
        let prepared = r.prepared_scene().unwrap();
        let viewport = prepared
            .layout
            .viewport_panes(&layout)
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
        let pointer = (
            (x + (point.x - a.x) as f32 * 100. / (b.x - a.x) as f32) as f64,
            (y + (point.y - a.y) as f32 * 100. / (b.y - a.y) as f32) as f64,
        );
        assert!(
            matches!(
                prepared.world_point_at_screen(pointer.0 as f32, pointer.1 as f32),
                Some((_, datum_gui_render::SceneSurface::Board))
            ),
            "Board proof point {point:?} projects outside the Board viewport"
        );
        self.host.mouse(events, pointer, click);
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
    fn menu(&mut self, menu: &str, label: &str) {
        assert!(self.host.app.runtime.as_mut().unwrap().activate_menu_item(
            menu,
            label,
            EntrySurface::MenuPointer
        ));
    }
    fn advance(&mut self, events: &ActiveEventLoop) {
        let Some(r) = &self.host.app.runtime else {
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
        if self.outline_only {
            return self.advance_outline(events);
        }
        eprintln!("native-board-controls stage={}", self.step);
        let required = match self.step {
            1 => Some(C::Route),
            2 => Some(C::Field(F::Width)),
            3 => Some(C::Field(F::Clearance)),
            4 => Some(C::Field(F::RulePriority)),
            5 => Some(C::ApplyRule),
            8 => Some(C::Finish),
            11 => Some(C::Zone),
            12 => Some(C::Field(F::Net)),
            13 => Some(C::Field(F::Priority)),
            14 => Some(C::Field(F::ThermalGap)),
            15 => Some(C::Field(F::SpokeWidth)),
            16 => Some(C::Thermal),
            21 => Some(C::Finish),
            25 => Some(C::Fill),
            _ => None,
        };
        if let Some(control) = required
            && !self.ensure_control(events, control)
        {
            return;
        }
        let r = self.host.app.runtime.as_ref().unwrap();
        match self.step {
            0 => {
                if r.native_selection.is_none() {
                    return;
                }
                self.menu("View", "Fit to Board");
            }
            1 => self.control(events, C::Route),
            2 => {
                let r = self.host.app.runtime.as_mut().unwrap();
                let layout = r.workspace().ui.layout.clone();
                let focus = r.application_focus();
                let draft = r.workspace().authoring.clone();
                let tool = r.workspace().tool;
                let pane = r.session.workspace_mut().ui.layout.split_focused_with(
                    PaneContent::Board,
                    datum_gui_protocol::SplitOrientation::Vertical,
                    true,
                );
                assert_ne!(pane, layout.focused);
                r.set_application_focus(datum_gui_protocol::ApplicationFocus::Editor(pane));
                assert!(!r.handle_board_authoring_input(
                    &Key::Named(NamedKey::Escape),
                    ElementState::Pressed
                ));
                assert_eq!(r.workspace().authoring, draft);
                assert_eq!(r.workspace().tool, tool);
                r.session.workspace_mut().ui.layout = layout;
                r.set_application_focus(focus);
                self.capture("route-setup");
                self.field(events, F::Width, "0.25 mm");
                self.board(
                    events,
                    PointNm {
                        x: 2000000,
                        y: 2000000,
                    },
                    true,
                );
            }
            3 => {
                assert!(
                    r.workspace()
                        .authoring
                        .board_route
                        .as_ref()
                        .unwrap()
                        .basis()
                        .clearance
                        .is_none()
                );
                self.capture("route-missing-clearance");
                self.field(events, F::Clearance, "0.2 mm");
            }
            4 => self.field(events, F::RulePriority, "18446744073709551615"),
            5 => {
                self.capture("route-rule-configured");
                self.control(events, C::ApplyRule);
            }
            6 => {
                let model = self.model();
                let rules = model
                    .materialized_source_shard_value(SourceShardKind::RulesRoot)
                    .unwrap();
                assert!(
                    rules["rules"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .any(|v| v["priority"].as_u64() == Some(u64::MAX)
                            && v["params"]["min_nm"] == 200000)
                );
                self.board(
                    events,
                    PointNm {
                        // Centre on the visible edge: the 0.25mm copper stroke
                        // extends outside the outline without an off-canvas pointer.
                        x: 5000000,
                        y: 0,
                    },
                    false,
                );
            }
            7 => {
                let refusal = r
                    .workspace()
                    .authoring
                    .board_controls
                    .as_ref()
                    .unwrap()
                    .refusal
                    .as_deref()
                    .expect("outside copper must produce a Stop refusal");
                assert!(
                    refusal.contains("copper extends outside the board outline"),
                    "{refusal}"
                );
                assert_eq!(self.count("tracks"), 0);
                self.capture("route-stop");
                self.board(
                    events,
                    PointNm {
                        x: 7000000,
                        y: 2000000,
                    },
                    true,
                );
            }
            8 => {
                self.capture("route-preview");
                self.control(events, C::Finish);
            }
            9 => {
                assert_eq!(self.count("tracks"), 1);
                self.capture("route-authored");
                self.menu("Edit", "Undo");
            }
            10 => {
                assert_eq!(self.count("tracks"), 0);
                self.menu("Edit", "Redo");
            }
            11 => {
                assert_eq!(self.count("tracks"), 1);
                self.control(events, C::Zone);
            }
            12 => self.field(events, F::Net, &self.net.to_string()),
            13 => self.field(events, F::Priority, "0"),
            14 => self.field(events, F::ThermalGap, "0 nm"),
            15 => self.field(events, F::SpokeWidth, "0 nm"),
            16 => {
                let r = self.host.app.runtime.as_mut().unwrap();
                assert!(r.handle_board_authoring_input(
                    &Key::Named(NamedKey::Tab),
                    ElementState::Pressed
                ));
                let controls = r.workspace().authoring.board_controls.as_ref().unwrap();
                assert_eq!(controls.focus, Some(C::Thermal));
                assert!(r.handle_board_authoring_input(
                    &Key::Named(NamedKey::Space),
                    ElementState::Pressed
                ));
                let controls = r.workspace().authoring.board_controls.as_ref().unwrap();
                assert_eq!(controls.thermal, Some(false));
            }
            17 => self.board(
                events,
                PointNm {
                    x: 500000,
                    y: 500000,
                },
                true,
            ),
            18 => self.board(
                events,
                PointNm {
                    x: 9500000,
                    y: 500000,
                },
                true,
            ),
            19 => self.board(
                events,
                PointNm {
                    x: 9500000,
                    y: 5500000,
                },
                true,
            ),
            20 => self.board(
                events,
                PointNm {
                    x: 500000,
                    y: 5500000,
                },
                true,
            ),
            21 => {
                self.capture("zone-draft");
                self.control(events, C::Finish);
            }
            22 => {
                assert_eq!(self.count("zones"), 1);
                let model = self.model();
                assert_eq!(model.zone_fills.len(), 1);
                let fill = model.zone_fills.values().next().unwrap();
                let board = model
                    .materialized_source_shard_value(SourceShardKind::BoardRoot)
                    .unwrap();
                assert!(board["zones"].get(fill.zone_id.to_string()).is_some());
                assert_eq!(fill.state, eda_engine::substrate::ZoneFillState::Unfilled);
                assert!(fill.islands.is_empty() && fill.provenance.is_none());
                assert!(model.current_zone_fill(fill.zone_id).is_none());
                self.board(
                    events,
                    PointNm {
                        x: 500000,
                        y: 3000000,
                    },
                    true,
                );
            }
            23 => {
                let datum_gui_protocol::SelectionTarget::Native(view) = &r.workspace().selection
                else {
                    return;
                };
                let datum_gui_protocol::SelectionSubject::Object(identity) = &view.subject else {
                    return;
                };
                if identity.class != datum_gui_protocol::AuthoredSelectionClass::Zone || view.stale
                {
                    return;
                }
                let board = self
                    .model()
                    .materialized_source_shard_value(SourceShardKind::BoardRoot)
                    .unwrap();
                assert!(board["zones"].get(identity.id.to_string()).is_some());
                assert_eq!(
                    datum_gui_protocol::board_zones::captured_zone_clearance(view)
                        .unwrap()
                        .minimum_nm,
                    200000
                );
                assert_eq!(
                    view.values[identity]["zone_boundary"]["vertices"]
                        .as_array()
                        .unwrap()
                        .len(),
                    4
                );
                for field in ["thermal_gap_nm", "thermal_spoke_width_nm"] {
                    assert_eq!(view.values[identity][field].as_i64(), Some(0));
                    assert_eq!(view.field_display_preview(field, 100), "0.000mm");
                }
                self.capture("zone-unfilled");
                self.menu("Edit", "Undo");
            }
            24 => {
                assert_eq!(self.count("zones"), 0);
                self.menu("Edit", "Redo");
            }
            25 => {
                assert_eq!(self.count("zones"), 1);
                self.control(events, C::Fill);
            }
            26 => {
                let model = self.model();
                assert_eq!(model.zone_fills.len(), 1);
                let fill = model.zone_fills.values().next().unwrap();
                assert!(model.current_zone_fill(fill.zone_id).is_some());
                assert!(!fill.islands.is_empty());
                self.capture("zone-filled");
                self.menu("Checks", "Run DRC");
            }
            27 => {
                assert!(r.workspace().checks.is_current(r.workspace()));
                self.capture("board-drc");
                self.menu("File", "Save");
            }
            28 => {
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
            29 => {
                assert_eq!(self.saved, self.snapshot());
                assert_eq!(self.count("tracks"), 1);
                assert_eq!(self.count("zones"), 1);
                assert!(r.workspace().checks.is_current(r.workspace()));
                let model = self.model();
                let fill = model.zone_fills.values().next().unwrap();
                assert!(model.current_zone_fill(fill.zone_id).is_some());
                self.capture("board-cold");
                self.step += 1;
                events.exit();
                return;
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
            self.host.failure = Some(format!("PCB controls stage {} timed out", self.step));
            events.exit();
            return;
        }
        self.advance(events);
        let bound = self.host.since + Duration::from_secs(30);
        let due = if self.host.app.runtime.as_ref().is_some_and(|r| {
            !r.native_edit.pending()
                && !r.check_work.pending()
                && !r.renderer.render_session().has_pending_frame()
                && r.surface_transaction.has_presented()
        }) {
            (self.host.since + Duration::from_millis(30)).min(bound)
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
#[path = "native_board_outline_frame_tests.rs"]
mod outline;
#[test]
#[ignore = "requires fresh owned DATUM_BOARD_TOOL_PROJECT and isolated native Wayland"]
fn ordinary_app_native_route_rule_zone_fill_history_save_cold() {
    outline::run(false);
}
