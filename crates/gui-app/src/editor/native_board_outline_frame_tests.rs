//! Outline delivery through the existing ordinary native App PCB workflow.
//! Canonical fixture setup supplies connectivity, a rule and an unfilled Zone;
//! Outline geometry/history/fill/save are driven through presented UI controls.
use super::*;
use std::sync::Arc;

fn proof() -> WriteProvenance {
    WriteProvenance::new(
        "outline-fixture",
        CommitSource::Test,
        "Explicit downstream PCB fixture",
    )
}
fn rectangle() -> Polygon {
    Polygon::new(vec![
        Point::new(1000000, 1000000),
        Point::new(8000000, 1000000),
        Point::new(8000000, 3000000),
        Point::new(1000000, 3000000),
    ])
}
fn setup_downstream(root: &std::path::Path, net: Uuid) {
    let mut model = ProjectResolver::new(root).resolve().unwrap();
    let rule = datum_gui_protocol::native_edit_client::board_rule::BoardClearanceRule::capture(
        &model, net, 200000, 0,
    )
    .unwrap();
    let prepared = (find_native_write_verb("datum.project.create_rule")
        .unwrap()
        .build)(
        &NativeWriteContext {
            model: &model,
            project_root: root,
        },
        proof(),
        json!({"rule":rule.value().unwrap()}),
    )
    .unwrap();
    commit_prepared(&mut model, root, prepared).unwrap();
    use native::board_routing::zone::{ZoneRequest, capture_zone_basis, prepare_zone};
    let request = ZoneRequest {
        basis: capture_zone_basis(&model, net, 1).unwrap(),
        polygon: Polygon::new(vec![
            Point::new(1500000, 1500000),
            Point::new(7500000, 1500000),
            Point::new(7500000, 2500000),
            Point::new(1500000, 2500000),
        ]),
        priority: 0,
        thermal_relief: false,
        thermal_gap_nm: 0,
        thermal_spoke_width_nm: 0,
        edit_seed: Uuid::new_v4(),
    };
    let prepared = prepare_zone(&model, proof(), &request).unwrap();
    commit_prepared(&mut model, root, prepared).unwrap();
    assert!(
        model
            .materialized_source_shard_value(SourceShardKind::BoardRoot)
            .unwrap()["outline"]["vertices"]
            .as_array()
            .unwrap()
            .is_empty()
    );
}
impl Workflow {
    fn outline(&self) -> serde_json::Value {
        self.model()
            .materialized_source_shard_value(SourceShardKind::BoardRoot)
            .unwrap()["outline"]
            .clone()
    }
    fn outline_key(&mut self, key: NamedKey) {
        assert!(
            self.host
                .app
                .runtime
                .as_mut()
                .unwrap()
                .handle_board_authoring_input(&Key::Named(key), ElementState::Pressed)
        );
    }
    fn outline_point(&mut self, events: &ActiveEventLoop, x: i64, y: i64) {
        self.board(events, PointNm { x, y }, true);
    }
    fn assert_rectangular_downstream(&self, supported: bool) {
        let model = self.model();
        let pad = self
            .host
            .app
            .runtime
            .as_ref()
            .unwrap()
            .workspace()
            .scene
            .pads
            .first()
            .unwrap()
            .pad_uuid
            .parse()
            .unwrap();
        let basis = native::board_routing::route::capture_route_basis(&model, pad, 1).unwrap();
        let stop = native::board_routing::stop::StopContext::capture(&model, &basis);
        if supported {
            let stop = stop.unwrap();
            assert!(stop.outline_defined());
            stop.preview_path(
                &[
                    basis.start,
                    Point::new(basis.start.x + 250000, basis.start.y),
                ],
                100000,
            )
            .unwrap();
        } else {
            assert!(
                stop.err()
                    .unwrap()
                    .to_string()
                    .contains("full-stroke containment")
            );
            use native::board_routing::zone::{FillRequest, capture_board_basis, preview_fills};
            let request = FillRequest {
                basis: capture_board_basis(&model).unwrap(),
                zone: None,
                net: None,
            };
            let fills = preview_fills(&model, &request).unwrap();
            assert_eq!(fills.len(), 1);
            assert_eq!(
                fills[0].state,
                eda_engine::substrate::ZoneFillState::Unsupported
            );
            assert!(fills[0].islands.is_empty());
            assert!(
                fills[0]
                    .provenance
                    .as_deref()
                    .unwrap()
                    .contains("unsupported general clipping")
            );
        }
    }
    fn stale_outline_source(&mut self, name: &str) {
        // External canonical mutation plus a qualified source projection update is
        // the negative control. The retained consumer draft must become stale.
        let mut model = self.model();
        let basis = native::board_routing::zone::capture_board_basis(&model).unwrap();
        let prepared =
            native::board_layout::build_set_board_name(&model, proof(), basis.board, name.into())
                .unwrap();
        commit_prepared(&mut model, &self.root, prepared).unwrap();
        let model = ProjectResolver::new(&self.root).resolve().unwrap();
        let state = self
            .host
            .app
            .runtime
            .as_mut()
            .unwrap()
            .session
            .workspace_mut();
        state.scene.source_revision = model.model_revision.0.clone();
        state.library =
            datum_gui_protocol::library_browser::LibraryBrowser::capture(Arc::new(model));
        assert!(
            !state
                .authoring
                .board_controls
                .as_ref()
                .unwrap()
                .current(state)
        );
        // Match real source publication: the new projection must invalidate
        // retained paint/hits before the next keyboard recovery callback.
        self.host.app.runtime.as_mut().unwrap().invalidate_scene();
        self.saved = self.snapshot();
    }
    pub(super) fn advance_outline(&mut self, events: &ActiveEventLoop) {
        eprintln!("native-board-outline stage={}", self.step);
        let required = match self.step {
            2 | 9 | 17 | 23 => Some(C::Outline),
            31 => Some(C::Fill),
            _ => None,
        };
        if let Some(control) = required
            && !self.ensure_control(events, control)
        {
            return;
        }
        match self.step {
            0 => {
                if self
                    .host
                    .app
                    .runtime
                    .as_ref()
                    .unwrap()
                    .native_selection
                    .is_none()
                {
                    return;
                }
                self.menu("View", "Fit to Board");
            }
            1 => {
                let r = self.host.app.runtime.as_mut().unwrap();
                let layout = r.workspace().ui.layout.clone();
                let scene = r
                    .prepared_scene()
                    .unwrap()
                    .layout
                    .viewport_panes(&layout)
                    .focused_pane()
                    .rect
                    .scene;
                let center = (
                    (scene.x + scene.width / 2.) as f64,
                    (scene.y + scene.height / 2.) as f64,
                );
                self.host.mouse(events, center, false);
                let window = self.host.app.window.as_ref().unwrap().id();
                self.host.app.window_event(
                    events,
                    window,
                    WindowEvent::MouseWheel {
                        device_id: winit::event::DeviceId::dummy(),
                        delta: winit::event::MouseScrollDelta::LineDelta(0., -5.),
                        phase: winit::event::TouchPhase::Moved,
                    },
                );
                self.host
                    .app
                    .runtime
                    .as_mut()
                    .unwrap()
                    .session
                    .workspace_mut()
                    .ui
                    .filters
                    .active_layer_id = None;
            }
            2 => {
                self.saved = self.snapshot();
                self.host.stage = self.model().journal.len();
                self.control(events, C::Outline);
            }
            3 => {
                self.capture("outline-empty");
                self.outline_point(events, 1000000, 1000000);
            }
            4 => self.outline_point(events, 8000000, 3000000),
            5 => self.outline_point(events, 1000000, 3000000),
            6 => self.outline_point(events, 8000000, 1000000),
            7 => self.outline_key(NamedKey::Enter),
            8 => {
                let r = self.host.app.runtime.as_ref().unwrap();
                assert!(
                    r.workspace()
                        .authoring
                        .board_controls
                        .as_ref()
                        .unwrap()
                        .refusal
                        .is_some()
                );
                assert_eq!(
                    r.workspace()
                        .authoring
                        .board_outline
                        .as_ref()
                        .unwrap()
                        .vertices()
                        .len(),
                    4
                );
                assert!(!r.native_edit.pending());
                self.capture("outline-invalid");
                self.outline_key(NamedKey::Escape);
                assert_eq!(self.saved, self.snapshot());
                assert_eq!(self.model().journal.len(), self.host.stage);
            }
            9 => self.control(events, C::Outline),
            10 => self.outline_point(events, 1000000, 1000000),
            11 => self.outline_point(events, 8000000, 1000000),
            12 => self.outline_point(events, 8000000, 3000000),
            13 => self.outline_point(events, 1000000, 3000000),
            14 => {
                self.capture("outline-first-draft");
                self.outline_key(NamedKey::Enter);
                let r = self.host.app.runtime.as_mut().unwrap();
                assert!(r.native_edit.pending());
                assert!(
                    r.workspace()
                        .authoring
                        .board_controls
                        .as_ref()
                        .unwrap()
                        .pending
                );
                assert!(
                    r.workspace()
                        .authoring
                        .board_outline
                        .as_ref()
                        .unwrap()
                        .pending()
                );
                let retained = r.workspace().authoring.clone();
                assert!(r.handle_board_authoring_input(
                    &Key::Named(NamedKey::Escape),
                    ElementState::Pressed
                ));
                assert_eq!(r.workspace().authoring, retained);
            }
            15 => {
                assert_eq!(self.outline(), serde_json::to_value(rectangle()).unwrap());
                assert_eq!(self.model().journal.len(), self.host.stage + 1);
                self.capture("outline-first-committed");
                self.menu("Edit", "Undo");
            }
            16 => {
                let initial: serde_json::Value =
                    serde_json::from_slice(&self.host.preimage).unwrap();
                assert_eq!(self.outline(), initial["outline"]);
                self.menu("Edit", "Redo");
            }
            17 => {
                assert_eq!(self.outline(), serde_json::to_value(rectangle()).unwrap());
                self.control(events, C::Outline);
            }
            18 => self.outline_point(events, 1000000, 1000000),
            19 => self.outline_point(events, 8000000, 1000000),
            20 => self.outline_point(events, 1000000, 3000000),
            21 => {
                self.capture("outline-replacement");
                self.outline_key(NamedKey::Enter);
            }
            22 => {
                assert_eq!(self.outline()["vertices"].as_array().unwrap().len(), 3);
                self.assert_rectangular_downstream(false);
                self.capture("outline-nonrectangular");
                self.menu("Edit", "Undo");
            }
            23 => {
                assert_eq!(self.outline(), serde_json::to_value(rectangle()).unwrap());
                self.control(events, C::Outline);
            }
            24 => self.outline_point(events, 1500000, 1500000),
            25 => self.stale_outline_source("Outline keyboard restart negative control"),
            26 => {
                self.capture("outline-stale");
                let r = self.host.app.runtime.as_mut().unwrap();
                let draft = r.workspace().authoring.clone();
                let layout = r.workspace().ui.layout.clone();
                let focus = r.application_focus();
                let other = r.session.workspace_mut().ui.layout.split_focused_with(
                    PaneContent::Board,
                    datum_gui_protocol::SplitOrientation::Vertical,
                    true,
                );
                r.set_application_focus(datum_gui_protocol::ApplicationFocus::Editor(other));
                assert!(!r.handle_board_authoring_input(
                    &Key::Named(NamedKey::Tab),
                    ElementState::Pressed
                ));
                assert_eq!(r.workspace().authoring, draft);
                r.session.workspace_mut().ui.layout = layout;
                r.set_application_focus(focus);
                // A formerly focused mutation control is never activated stale.
                r.session
                    .workspace_mut()
                    .authoring
                    .board_controls
                    .as_mut()
                    .unwrap()
                    .focus = Some(C::Finish);
                self.outline_key(NamedKey::Enter);
                self.outline_key(NamedKey::Backspace);
                assert_eq!(
                    self.host
                        .app
                        .runtime
                        .as_ref()
                        .unwrap()
                        .workspace()
                        .authoring
                        .board_outline
                        .as_ref()
                        .unwrap()
                        .vertices()
                        .len(),
                    1
                );
                assert_eq!(self.saved, self.snapshot());
                self.outline_key(NamedKey::Tab);
                assert_eq!(
                    self.host
                        .app
                        .runtime
                        .as_ref()
                        .unwrap()
                        .workspace()
                        .authoring
                        .board_controls
                        .as_ref()
                        .unwrap()
                        .focus,
                    Some(C::Outline)
                );
            }
            27 => {
                self.outline_key(NamedKey::Enter);
                let r = self.host.app.runtime.as_ref().unwrap();
                assert!(
                    r.workspace()
                        .authoring
                        .board_controls
                        .as_ref()
                        .unwrap()
                        .current(r.workspace())
                );
                assert!(r.workspace().authoring.board_outline.is_none());
                assert_eq!(self.saved, self.snapshot());
            }
            28 => self.outline_point(events, 1500000, 1500000),
            29 => self.stale_outline_source("Outline keyboard cancel negative control"),
            30 => {
                self.outline_key(NamedKey::Tab);
                self.outline_key(NamedKey::Tab);
                assert_eq!(
                    self.host
                        .app
                        .runtime
                        .as_ref()
                        .unwrap()
                        .workspace()
                        .authoring
                        .board_controls
                        .as_ref()
                        .unwrap()
                        .focus,
                    Some(C::Cancel)
                );
                self.outline_key(NamedKey::Space);
                assert!(
                    self.host
                        .app
                        .runtime
                        .as_ref()
                        .unwrap()
                        .workspace()
                        .authoring
                        .board_controls
                        .is_none()
                );
                assert_eq!(self.saved, self.snapshot());
            }
            31 => {
                self.assert_rectangular_downstream(true);
                self.control(events, C::Fill);
            }
            32 => {
                let model = self.model();
                assert_eq!(model.zone_fills.len(), 1);
                let fill = model.zone_fills.values().next().unwrap();
                assert!(model.current_zone_fill(fill.zone_id).is_some());
                assert!(!fill.islands.is_empty());
                self.menu("Checks", "Run DRC");
            }
            33 => {
                let r = self.host.app.runtime.as_ref().unwrap();
                assert!(r.workspace().checks.is_current(r.workspace()));
                self.capture("outline-rectangle-checks");
                self.menu("File", "Save");
            }
            34 => {
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
            35 => {
                assert_eq!(self.saved, self.snapshot());
                assert_eq!(self.outline(), serde_json::to_value(rectangle()).unwrap());
                let r = self.host.app.runtime.as_ref().unwrap();
                assert!(r.workspace().checks.is_current(r.workspace()));
                let model = self.model();
                assert!(
                    model
                        .current_zone_fill(model.zone_fills.values().next().unwrap().zone_id)
                        .is_some()
                );
                self.capture("outline-cold");
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

pub(super) fn run(outline_only: bool) {
    let (project_var, output_var) = if outline_only {
        ("DATUM_BOARD_OUTLINE_PROJECT", "DATUM_BOARD_OUTLINE_OUTPUT")
    } else {
        ("DATUM_BOARD_TOOL_PROJECT", "DATUM_BOARD_TOOL_OUTPUT")
    };
    let root = PathBuf::from(std::env::var_os(project_var).unwrap());
    let output = PathBuf::from(std::env::var_os(output_var).unwrap());
    std::fs::create_dir_all(&output).unwrap();
    let net = fixture(&root, !outline_only);
    if outline_only {
        setup_downstream(&root, net);
    }
    let model = ProjectResolver::new(&root).resolve().unwrap();
    let path = model
        .source_shards
        .iter()
        .find(|s| s.kind == SourceShardKind::BoardRoot)
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
        net,
        step: 0,
        saved: BTreeMap::new(),
        outline_only,
    };
    events.run_app(&mut workflow).unwrap();
    assert_eq!(workflow.host.failure, None);
    assert_eq!(workflow.step, if outline_only { 36 } else { 30 });
}
#[test]
#[ignore = "requires fresh owned DATUM_BOARD_OUTLINE_PROJECT and isolated native Wayland"]
fn ordinary_app_outline_cancel_first_empty_history_replace_stale_keyboard_save_cold() {
    run(true);
}
