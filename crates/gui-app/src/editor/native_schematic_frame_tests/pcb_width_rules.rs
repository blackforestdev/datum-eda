//! Ordinary App DRC for authored width, correction, rerun, save and cold history.
//! Fixture operations use the public canonical API, not GUI rule/routing editing.
use super::*;
use eda_engine::substrate::{DesignModel, Operation, ProjectResolver, SourceShardKind};
use std::collections::BTreeMap;
use uuid::Uuid;
#[path = "pcb_width_rules/fixture.rs"]
mod fixture;

struct Workflow {
    host: Fixture,
    root: PathBuf,
    output: PathBuf,
    track: eda_engine::board::Track,
    peer: eda_engine::board::Track,
    step: usize,
    old_run: Uuid,
    run: Uuid,
    saved: BTreeMap<PathBuf, Vec<u8>>,
}
impl Workflow {
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
    fn menu(&mut self, group: &str, label: &str) {
        assert!(self.host.app.runtime.as_mut().unwrap().activate_menu_item(
            group,
            label,
            EntrySurface::MenuPointer
        ));
    }
    fn capture(&mut self, name: &str) {
        let r = self.host.app.runtime.as_mut().unwrap();
        r.write_visual_screenshot(&self.output.join(format!("{name}.png")))
            .unwrap();
        std::fs::write(
            self.output.join(format!("{name}.json")),
            serde_json::to_vec_pretty(&r.workspace().checks).unwrap(),
        )
        .unwrap();
    }
    fn reopen(&mut self, events: &ActiveEventLoop) {
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
    fn assert_report(&self, corrected: bool) {
        let r = self.host.app.runtime.as_ref().unwrap();
        let checks = &r.workspace().checks;
        assert!(checks.persisted && checks.is_current(r.workspace()));
        assert_eq!(checks.profile_id.as_deref(), Some("drc"));
        assert_eq!(
            checks.check_run_id.as_deref(),
            Some(self.run.to_string().as_str())
        );
        let finding = checks
            .findings
            .iter()
            .find(|f| f.code == "track_width_below_min");
        if corrected {
            assert!(finding.is_none());
        } else {
            let finding = finding.unwrap();
            assert_eq!(finding.severity, "error");
            assert!(finding.message.contains("200000nm") && finding.message.contains("500000nm"));
            assert_eq!(finding.primary_target["id"], self.track.uuid.to_string());
        }
        let clearance = checks
            .findings
            .iter()
            .filter(|f| f.code == "clearance_copper")
            .collect::<Vec<_>>();
        assert_eq!(clearance.len(), usize::from(!corrected));
        if let Some(finding) = clearance.first() {
            assert_eq!(finding.severity, "error");
            assert!(finding.message.contains("200000nm"));
        }
        let model = self.model();
        if corrected {
            assert!(
                model.check_runs[&self.old_run]
                    .findings
                    .iter()
                    .any(|f| f.code == "clearance_copper")
            );
            assert!(
                !model.check_runs[&self.run]
                    .findings
                    .iter()
                    .any(|f| f.code == "clearance_copper")
            );
        }
        assert_eq!(
            checks.model_revision.as_deref(),
            Some(model.model_revision.0.as_str())
        );
        if corrected {
            assert_ne!(self.run, self.old_run);
            assert!(
                model.check_runs[&self.old_run]
                    .findings
                    .iter()
                    .any(|f| f.code == "track_width_below_min" && f.severity == "error")
            );
            assert!(
                !model.check_runs[&self.run]
                    .findings
                    .iter()
                    .any(|f| f.code == "track_width_below_min")
            );
            assert_ne!(
                model.check_runs[&self.old_run].model_revision,
                model.check_runs[&self.run].model_revision
            );
        }
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
        eprintln!("width-rule-app stage={}", self.step);
        match self.step {
            0 => {
                if !r.native_edit_available() || r.native_selection.is_none() {
                    return;
                }
                self.menu("View", "Fit to Board");
            }
            1 => self.menu("Checks", "Run DRC"),
            2 => {
                self.run = r
                    .workspace()
                    .checks
                    .check_run_id
                    .as_ref()
                    .unwrap()
                    .parse()
                    .unwrap();
                self.old_run = self.run;
                self.assert_report(false);
                self.capture("width-report");
                self.menu("View", "Fit to Board");
            }
            3 => {
                let finding = r
                    .workspace()
                    .checks
                    .findings
                    .iter()
                    .find(|f| f.code == "track_width_below_min")
                    .unwrap();
                self.host
                    .control(events, HitTarget::CheckFinding(finding.fingerprint.clone()));
            }
            4 => {
                let view = datum_gui_protocol::inspector_context::finding::capture(r.workspace())
                    .unwrap()
                    .unwrap();
                assert_eq!(view.finding.code, "track_width_below_min");
                assert_eq!(view.currentness, "Current persisted result");
                self.assert_report(false);
                self.capture("width-selected");
            }
            5 => {
                let finding = r
                    .workspace()
                    .checks
                    .findings
                    .iter()
                    .find(|f| f.code == "clearance_copper")
                    .unwrap();
                let fingerprint = finding.fingerprint.clone();
                self.capture("clearance-report");
                self.host
                    .control(events, HitTarget::CheckFinding(fingerprint));
            }
            6 => {
                let view = datum_gui_protocol::inspector_context::finding::capture(r.workspace())
                    .unwrap()
                    .unwrap();
                assert_eq!(view.finding.code, "clearance_copper");
                assert_eq!(view.currentness, "Current persisted result");
                self.assert_report(false);
                self.capture("clearance-selected");
                self.menu("File", "Save");
            }
            7 => {
                self.saved = self.snapshot();
                self.reopen(events);
            }
            8 => {
                self.assert_report(false);
                assert_eq!(
                    self.snapshot(),
                    self.saved,
                    "cold inspection does not mutate"
                );
                self.capture("width-cold");
                fixture::correct(&self.root, &mut self.track, &mut self.peer);
                self.reopen(events);
            }
            9 => {
                assert!(!r.workspace().checks.is_current(r.workspace()));
                self.capture("width-correction-stale");
                self.menu("Checks", "Run DRC");
            }
            10 => {
                self.run = r
                    .workspace()
                    .checks
                    .check_run_id
                    .as_ref()
                    .unwrap()
                    .parse()
                    .unwrap();
                self.assert_report(true);
                self.capture("width-corrected");
                self.menu("File", "Save");
            }
            11 => {
                self.saved = self.snapshot();
                self.reopen(events);
            }
            12 => {
                self.assert_report(true);
                assert_eq!(self.snapshot(), self.saved);
                self.capture("width-corrected-cold");
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
            self.host.failure = Some(format!("Width-rule App stage {} timed out", self.step));
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
#[ignore = "requires fresh owned DATUM_WIDTH_RULE_PROJECT and isolated native Wayland"]
fn ordinary_app_authored_width_drc_correction_rerun_save_cold_history() {
    let root = PathBuf::from(std::env::var_os("DATUM_WIDTH_RULE_PROJECT").unwrap());
    let output = PathBuf::from(std::env::var_os("DATUM_WIDTH_RULE_OUTPUT").unwrap());
    std::fs::create_dir_all(&output).unwrap();
    let (track, peer) = fixture::prepare(&root);
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
        track,
        peer,
        step: 0,
        old_run: Uuid::nil(),
        run: Uuid::nil(),
        saved: BTreeMap::new(),
    };
    events.run_app(&mut workflow).unwrap();
    assert!(
        workflow.host.failure.is_none(),
        "{:?}",
        workflow.host.failure
    );
    assert_eq!(workflow.step, 13);
}
