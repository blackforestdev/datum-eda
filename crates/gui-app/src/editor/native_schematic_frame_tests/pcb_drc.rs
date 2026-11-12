//! One ordinary native DRC report/rerun workflow. Public API circuit/transfer
//! preparation is fixture setup, not claimed as GUI schematic or PCB editing.
use super::*;
use datum_gui_protocol::CheckProducerProfile;
use eda_engine::api::native_write::{
    self as native, WriteProvenance, commit_prepared,
    registry::{NativeWriteContext, find_native_write_verb},
    schematic_board_transfer::{TransferChoice, TransferRequest, plan_transfer},
};
use eda_engine::substrate::{
    CommitSource, DesignModel, Operation, ProjectResolver, SourceShardKind,
};
use serde_json::json;
use std::collections::BTreeMap;
use uuid::Uuid;

fn prepare_connected_board(root: &std::path::Path) {
    super::board_transfer::setup(root);
    let mut model = ProjectResolver::new(root).resolve().unwrap();
    let provenance = || {
        WriteProvenance::new(
            "pcb-drc-fixture",
            CommitSource::Test,
            "Connected board DRC fixture",
        )
    };
    // The reused legacy fixture predates explicit Working Units. Prepare its
    // versioned factory receipt canonically before observing the GUI; this is
    // fixture setup, not an implicit application migration or GUI-edit claim.
    if let eda_engine::ir::units::PreFeatureProjectUnitsMigration::Create { profile, receipt } =
        eda_engine::ir::units::migrate_pre_feature_project_units(None).unwrap()
    {
        let prepared = native::project::build_initialize_project_display_units(
            &model,
            provenance(),
            profile,
            &receipt,
        )
        .unwrap();
        commit_prepared(&mut model, root, prepared).unwrap();
    }
    let source = model
        .materialized_source_shard_value(SourceShardKind::SchematicRoot)
        .unwrap();
    let sheet = source["sheets"]
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
        origin: eda_engine::ir::geometry::Point::new(2_000_000, 2_000_000),
        pitch_nm: 5_000_000,
        layer: 1,
    };
    let plan = plan_transfer(&model, &request).unwrap();
    request.choices = plan
        .components
        .iter()
        .map(|c| {
            assert_eq!(c.map_options.len(), 1);
            TransferChoice {
                component_instance: c.component_instance.unwrap(),
                pin_pad_map: c.map_options[0].pin_pad_map,
            }
        })
        .collect();
    let prepared = (find_native_write_verb("datum.pcb.prepare_footprint_sources")
        .unwrap()
        .build)(
        &NativeWriteContext {
            model: &model,
            project_root: root,
        },
        provenance(),
        json!({"board":plan.board_id}),
    )
    .unwrap();
    commit_prepared(&mut model, root, prepared).unwrap();
    let prepared =
        native::schematic_board_transfer::prepare_transfer(&model, provenance(), &request).unwrap();
    commit_prepared(&mut model, root, prepared).unwrap();
}

struct Workflow {
    host: Fixture,
    root: PathBuf,
    output: PathBuf,
    step: usize,
    authored: BTreeMap<PathBuf, Vec<u8>>,
    saved: BTreeMap<PathBuf, Vec<u8>>,
    journal_before: usize,
    fingerprint: String,
    run: String,
}
impl Workflow {
    fn scroll_inspector(&mut self, events: &ActiveEventLoop, pages: f32) {
        let r = self.host.app.runtime.as_ref().unwrap();
        let (rect, total) = r
            .presented_hits
            .regions()
            .iter()
            .find_map(|h| match h.target {
                HitTarget::InspectorContextScroll { total, .. } => Some((h.rect, total)),
                _ => None,
            })
            .expect("actual scrollable Inspector must be presented");
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
                delta: winit::event::MouseScrollDelta::LineDelta(0., -(total as f32) * pages),
                phase: winit::event::TouchPhase::Moved,
            },
        );
    }
    fn model(&self) -> DesignModel {
        ProjectResolver::new(&self.root).resolve().unwrap()
    }
    fn snapshot(&self) -> BTreeMap<PathBuf, Vec<u8>> {
        let model = self.model();
        let mut files: BTreeMap<_, _> = model
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
    fn assert_authored_unchanged(&self) {
        for (path, bytes) in &self.authored {
            assert_eq!(&std::fs::read(path).unwrap(), bytes, "{}", path.display());
        }
        let model = self.model();
        assert_eq!(model.journal.len(), self.journal_before + 1);
        assert!(
            model.journal[self.journal_before..]
                .iter()
                .flat_map(|t| &t.operations)
                .all(|op| matches!(op, Operation::SetCheckRun { .. }))
        );
    }
    fn menu(&mut self, group: &str, label: &str) {
        assert!(self.host.app.runtime.as_mut().unwrap().activate_menu_item(
            group,
            label,
            EntrySurface::MenuPointer
        ));
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
    fn current_report(&self) {
        let model = self.model();
        let checks = &self.host.app.runtime.as_ref().unwrap().workspace().checks;
        assert!(checks.persisted);
        assert_eq!(checks.profile_id.as_deref(), Some("drc"));
        assert_eq!(
            checks.project_id.as_deref(),
            Some(model.project.project_id.to_string().as_str())
        );
        assert_eq!(
            checks.model_revision.as_deref(),
            Some(model.model_revision.0.as_str())
        );
        assert_eq!(checks.check_run_id.as_deref(), Some(self.run.as_str()));
        assert!(
            checks
                .findings
                .iter()
                .any(|f| f.fingerprint == self.fingerprint
                    && f.domain == "drc"
                    && f.code == "connectivity_unrouted_net")
        );
        let id = Uuid::parse_str(&self.run).unwrap();
        assert_eq!(model.check_runs[&id].profile_id, "drc");
        self.assert_authored_unchanged();
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
        eprintln!(
            "pcb-drc-native stage={} producer={:?}",
            self.step,
            r.workspace().check_producer.phase
        );
        match self.step {
            0 => {
                if !r.native_edit_available() || r.native_selection.is_none() {
                    return;
                }
                self.menu("Checks", "Run DRC");
            }
            1 => {
                let checks = &r.workspace().checks;
                assert_eq!(checks.profile_id.as_deref(), Some("drc"));
                self.run = checks.check_run_id.clone().unwrap();
                self.fingerprint = checks
                    .findings
                    .iter()
                    .find(|f| f.code == "connectivity_unrouted_net" && f.domain == "drc")
                    .expect("real unrouted native board finding")
                    .fingerprint
                    .clone();
                self.current_report();
                std::fs::write(
                    self.output.join("drc-report.json"),
                    serde_json::to_vec_pretty(checks).unwrap(),
                )
                .unwrap();
                self.capture("drc-report");
            }
            2 => self.scroll_inspector(events, 0.5),
            3 => {
                use datum_gui_protocol::inspector_context::{InspectorContext, InspectorSectionId};
                let context = InspectorContext::capture(r.workspace());
                let section = context
                    .sections
                    .iter()
                    .find(|s| s.id == InspectorSectionId::Checks)
                    .unwrap();
                let finding = r
                    .workspace()
                    .checks
                    .findings
                    .iter()
                    .find(|f| f.fingerprint == self.fingerprint)
                    .unwrap();
                assert!(section.fields.iter().any(|f| f.label == "Finding message"
                    && context.field_text(f, usize::MAX) == finding.message));
                assert!(section.fields.iter().any(|f| f.label == "Finding target"
                    && context.field_text(f, usize::MAX) == finding.target_label().unwrap()));
                self.current_report();
                self.capture("drc-finding");
                self.scroll_inspector(events, 1.0);
            }
            4 => {
                let target = r
                    .presented_hits
                    .regions()
                    .iter()
                    .find_map(|h| match &h.target {
                        HitTarget::InspectorRunCheck(_, CheckProducerProfile::Drc) => {
                            Some(h.target.clone())
                        }
                        _ => None,
                    })
                    .expect("profile-qualified Rerun DRC control must be presented");
                self.capture("drc-rerun");
                self.host.control(events, target);
            }
            5 => {
                self.current_report();
                assert_eq!(
                    r.workspace()
                        .check_producer
                        .request
                        .as_ref()
                        .unwrap()
                        .profile,
                    CheckProducerProfile::Drc
                );
                self.menu("File", "Save");
            }
            6 => {
                self.current_report();
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
            7 => {
                let report = serde_json::to_vec_pretty(&r.workspace().checks).unwrap();
                self.current_report();
                assert_eq!(
                    self.snapshot(),
                    self.saved,
                    "cold reopening and inspection write no canonical sources"
                );
                self.capture("drc-cold");
                std::fs::write(self.output.join("cold-report.json"), report).unwrap();
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
        if self
            .host
            .app
            .window
            .as_ref()
            .is_some_and(|w| self.host.app.frames.rendering_failed(w.id()))
        {
            self.host.failure = Some("Native frame coordinator paused".into());
            events.exit();
            return;
        }
        if self.host.since.elapsed() > Duration::from_secs(30) {
            self.host.failure = Some(format!("PCB DRC stage {} timed out", self.step));
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

#[test]
#[ignore = "requires owned fresh DATUM_PCB_DRC_PROJECT and isolated native Wayland"]
fn ordinary_app_native_pcb_drc_finding_rerun_save_cold() {
    let root = PathBuf::from(std::env::var_os("DATUM_PCB_DRC_PROJECT").unwrap());
    let output = PathBuf::from(std::env::var_os("DATUM_PCB_DRC_OUTPUT").unwrap());
    std::fs::create_dir_all(&output).unwrap();
    prepare_connected_board(&root);
    let model = ProjectResolver::new(&root).resolve().unwrap();
    let authored = model
        .source_shards
        .iter()
        .filter(|s| s.kind != SourceShardKind::CheckRun)
        .map(|s| (s.path.clone(), std::fs::read(&s.path).unwrap()))
        .collect();
    let path = model
        .source_shards
        .iter()
        .find(|s| s.kind == SourceShardKind::SchematicSheet)
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
        authored,
        saved: BTreeMap::new(),
        journal_before: model.journal.len(),
        fingerprint: String::new(),
        run: String::new(),
    };
    events.run_app(&mut workflow).unwrap();
    assert!(
        workflow.host.failure.is_none(),
        "{:?}",
        workflow.host.failure
    );
    assert_eq!(workflow.step, 8);
}
