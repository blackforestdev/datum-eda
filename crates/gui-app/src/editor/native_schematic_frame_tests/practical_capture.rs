//! Ordinary GUI revision of a native amplifier; fixture setup is public API work.
use super::*;
#[path = "practical_capture_fixture.rs"]
mod circuit;
use circuit::Amplifier;
use datum_gui_protocol::native_edit_client::NativeEditAction;
use datum_gui_protocol::{ApplicationFocus, SelectionSubject, SelectionTarget, SessionCommand};
use eda_engine::substrate::{ElectricalSelectionSnapshot, ProjectResolver};
use std::collections::BTreeMap;

struct CaptureWorkflow {
    host: Fixture,
    amp: Amplifier,
    output: PathBuf,
    step: usize,
    before: BTreeMap<PathBuf, Vec<u8>>,
    saved: BTreeMap<PathBuf, Vec<u8>>,
    fault: String,
    run: String,
    wires: usize,
}
impl CaptureWorkflow {
    fn snapshot(&self) -> BTreeMap<PathBuf, Vec<u8>> {
        let model = ProjectResolver::new(&self.amp.root).resolve().unwrap();
        let mut files: BTreeMap<_, _> = model
            .source_shards
            .iter()
            .map(|s| (s.path.clone(), std::fs::read(&s.path).unwrap()))
            .collect();
        for name in [
            ".datum/journal/transactions.jsonl",
            ".datum/journal/cursor.json",
        ] {
            let path = self.amp.root.join(name);
            files.insert(path.clone(), std::fs::read(path).unwrap());
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
    fn menu(&mut self, group: &str, label: &str) {
        assert!(self.host.app.runtime.as_mut().unwrap().activate_menu_item(
            group,
            label,
            EntrySurface::MenuPointer
        ));
    }
    fn poses(&self) -> BTreeMap<String, (i64, i64)> {
        let model = ProjectResolver::new(&self.amp.root).resolve().unwrap();
        let snapshot = ElectricalSelectionSnapshot::capture(&model).unwrap();
        snapshot
            .schematic_source()
            .unwrap()
            .sheets
            .values()
            .flat_map(|s| s.symbols.values())
            .map(|s| (s.reference.clone(), (s.position.x, s.position.y)))
            .collect()
    }
    fn wire_count(&self) -> usize {
        let model = ProjectResolver::new(&self.amp.root).resolve().unwrap();
        let snapshot = ElectricalSelectionSnapshot::capture(&model).unwrap();
        snapshot
            .schematic_source()
            .unwrap()
            .sheets
            .values()
            .map(|s| s.wires.len())
            .sum()
    }
    fn navigate_child(&mut self, events: &ActiveEventLoop) -> bool {
        let r = self.host.app.runtime.as_ref().unwrap();
        if datum_gui_protocol::AuthoringViewBasis::capture_qualified_schematic(
            r.workspace(),
            r.workspace().ui.layout.focused,
        )
        .is_ok_and(|b| {
            b.source == self.amp.child
                && b.document
                    .as_ref()
                    .is_some_and(|d| d.instance_path == [self.amp.instance])
        }) {
            return true;
        }
        let row = r
            .presented_hits
            .regions()
            .iter()
            .find_map(|h| match &h.target {
                target @ HitTarget::SchematicDocumentRow { document, .. }
                    if document.source_sheet == self.amp.child =>
                {
                    Some(target.clone())
                }
                _ => None,
            });
        if let Some(row) = row {
            self.host.control(events, row.clone());
            self.host.control(events, row);
        } else {
            let expand = r
                .presented_hits
                .regions()
                .iter()
                .find_map(|h| match &h.target {
                    target @ HitTarget::SchematicDocumentExpand { document, .. }
                        if document.source_sheet == self.amp.parent =>
                    {
                        Some(target.clone())
                    }
                    _ => None,
                });
            if let Some(expand) = expand {
                self.host.control(events, expand);
            }
        }
        false
    }
    fn focus_and_fit(&mut self) {
        let r = self.host.app.runtime.as_mut().unwrap();
        let pane = r.workspace().ui.layout.focused;
        r.set_application_focus(ApplicationFocus::Editor(pane));
        if r.workspace().ui.layout.zoomed != Some(pane) {
            self.menu("View", "Maximize Pane");
        }
        self.host.app.runtime.as_mut().unwrap().fit_camera();
    }
    fn context(&mut self, events: &ActiveEventLoop) {
        self.host.world(
            events,
            PointNm {
                x: 10000000,
                y: 10000000,
            },
            false,
        );
        let id = self.host.app.window.as_ref().unwrap().id();
        for state in [ElementState::Pressed, ElementState::Released] {
            self.host.app.window_event(
                events,
                id,
                WindowEvent::MouseInput {
                    device_id: winit::event::DeviceId::dummy(),
                    state,
                    button: MouseButton::Right,
                },
            );
        }
    }
    fn slot(&mut self, events: &ActiveEventLoop, slot: &str) {
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
            .expect("Presented alignment command");
        self.host.control(events, target);
    }
    fn region(&mut self, events: &ActiveEventLoop) {
        let r = self.host.app.runtime.as_mut().unwrap();
        r.dispatch_session_command(SessionCommand::ClearSelection);
        r.set_workspace_tool(crate::WorkspaceTool::Move);
        r.modifiers = winit::keyboard::ModifiersState::SHIFT;
        self.host.world(
            events,
            PointNm {
                x: 8000000,
                y: 6000000,
            },
            false,
        );
        let window = self.host.app.window.as_ref().unwrap().id();
        self.host.app.window_event(
            events,
            window,
            WindowEvent::MouseInput {
                device_id: winit::event::DeviceId::dummy(),
                state: ElementState::Pressed,
                button: MouseButton::Left,
            },
        );
        self.host.world(
            events,
            PointNm {
                x: 12000000,
                y: 34000000,
            },
            false,
        );
        self.host.app.window_event(
            events,
            window,
            WindowEvent::MouseInput {
                device_id: winit::event::DeviceId::dummy(),
                state: ElementState::Released,
                button: MouseButton::Left,
            },
        );
        let r = self.host.app.runtime.as_mut().unwrap();
        while r.selection_evaluation.is_some() {
            r.selection_tick = Some(Instant::now());
            r.poll_native_selection_tick();
        }
        r.modifiers = winit::keyboard::ModifiersState::empty();
        let SelectionTarget::Native(view) = &r.workspace().selection else {
            panic!("Native bias block selected")
        };
        assert!(
            self.amp
                .bias
                .iter()
                .all(|id| view.members.iter().any(|m| m.id == *id))
        );
        assert_eq!(
            view.members
                .iter()
                .filter(|m| m.class == datum_gui_protocol::AuthoredSelectionClass::Symbol)
                .count(),
            2
        );
    }
    fn shortcut(&mut self, key: &str) {
        let action = crate::workspace_keyboard::character_action(key, true).unwrap();
        assert!(crate::workspace_keyboard::apply(
            self.host.app.runtime.as_mut().unwrap(),
            action
        ));
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
    fn current_checks(&self) {
        let r = self.host.app.runtime.as_ref().unwrap();
        let model = ProjectResolver::new(&self.amp.root).resolve().unwrap();
        assert!(r.workspace().checks.persisted);
        assert_eq!(
            r.workspace().checks.model_revision.as_deref(),
            Some(model.model_revision.0.as_str())
        );
    }
    fn fault_finding(&self) -> Option<String> {
        self.host
            .app
            .runtime
            .as_ref()
            .unwrap()
            .workspace()
            .checks
            .findings
            .iter()
            .find(|f| {
                f.code == "unconnected_component_pin"
                    && serde_json::to_string(f)
                        .unwrap()
                        .contains(&self.amp.fault_pin.to_string())
            })
            .map(|f| f.fingerprint.clone())
    }
    fn advance(&mut self, events: &ActiveEventLoop) {
        let Some(r) = self.host.app.runtime.as_ref() else {
            return;
        };
        if !r.surface_transaction.has_presented()
            || r.renderer.render_session().has_pending_frame()
            || r.native_edit.pending()
            || r.check_work.pending()
            || r.native_move_frame_needed()
            || self.host.since.elapsed() < Duration::from_millis(20)
        {
            return;
        }
        eprintln!(
            "practical-capture stage={} status={:?}",
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
                self.focus_and_fit();
                self.wires = self.wire_count();
            }
            1 => self.capture("amplifier-root"),
            2 => {
                if !self.navigate_child(events) {
                    return;
                }
            }
            3 => self.focus_and_fit(),
            4 => {
                self.before = self.snapshot();
                self.capture("amplifier-child-before");
                self.host.world(
                    events,
                    PointNm {
                        x: 10000000,
                        y: 10000000,
                    },
                    true,
                );
            }
            5 => {
                self.host.app.runtime.as_mut().unwrap().modifiers =
                    winit::keyboard::ModifiersState::SHIFT;
                self.host.world(
                    events,
                    PointNm {
                        x: 11000000,
                        y: 30000000,
                    },
                    true,
                );
                self.host.app.runtime.as_mut().unwrap().modifiers =
                    winit::keyboard::ModifiersState::empty();
            }
            6 => self.context(events),
            7 => self.slot(events, "NE"),
            8 => self.slot(events, "SUB:0"),
            9 => {
                let SelectionTarget::Native(view) = &r.workspace().selection else {
                    panic!("Expected native bias selection")
                };
                let base = datum_gui_protocol::camera_scene_for_pane(
                    r.workspace(),
                    r.workspace().ui.layout.focused,
                )
                .unwrap();
                std::fs::write(self.output.join("alignment-basis.json"), serde_json::to_vec_pretty(&serde_json::json!({
                    "native_model":r.native_selection.as_ref().unwrap().model.model_revision.0,
                    "view":view.revision.0,"scene":base.source_revision,
                    "native_project":r.native_selection.as_ref().unwrap().model.project.project_id,"view_project":view.project,
                    "document":base.schematic_document
                })).unwrap()).unwrap();
                self.slot(events, "SUB:0");
            }
            10 => {
                assert!(r.native_edit.preview.is_some(), "Alignment preview refused");
                assert_eq!(self.snapshot(), self.before);
                self.capture("bias-align-preview");
            }
            11 => self.host.world(
                events,
                PointNm {
                    x: 10000000,
                    y: 10000000,
                },
                true,
            ),
            12 => {
                assert_eq!(self.poses()["R2"], (10000000, 30000000));
                self.capture("amplifier-aligned");
                self.region(events);
                self.shortcut("c");
                self.shortcut("-");
            }
            13 => {
                self.shortcut("v");
                self.host.world(
                    events,
                    PointNm {
                        x: 40000000,
                        y: 10000000,
                    },
                    false,
                );
            }
            14 => self.capture("bias-copy-preview"),
            15 => self.host.world(
                events,
                PointNm {
                    x: 40000000,
                    y: 10000000,
                },
                true,
            ),
            16 => {
                let poses = self.poses();
                assert_eq!(poses.len(), 13);
                assert_eq!(poses["R6"], (40000000, 10000000));
                assert_eq!(poses["R7"], (40000000, 30000000));
                self.history(false);
            }
            17 => {
                assert_eq!(self.poses().len(), 11);
                self.history(true);
            }
            18 => {
                assert_eq!(self.poses().len(), 13);
                self.history(false);
            }
            19 => {
                assert_eq!(self.poses().len(), 11);
                self.capture("amplifier-before-erc");
                self.menu("Checks", "Run ERC");
            }
            20 => {
                self.current_checks();
                self.fault = self.fault_finding().unwrap_or_else(|| {
                    panic!(
                        "Expected real emitter resistor finding: {:?}",
                        r.workspace().checks.findings
                    )
                });
                self.run = r.workspace().checks.check_run_id.clone().unwrap();
            }
            21 => self
                .host
                .control(events, HitTarget::CheckFinding(self.fault.clone())),
            22 => {
                assert!(
                    matches!(&r.workspace().selection,SelectionTarget::Native(v) if matches!(&v.subject,SelectionSubject::Diagnostic(id) if id==&self.fault))
                );
                self.capture("erc-finding");
                self.host.app.runtime.as_mut().unwrap().fit_camera();
                self.menu("Place", "Wire");
            }
            23 => self.host.world(
                events,
                PointNm {
                    x: 10000000,
                    y: 38000000,
                },
                true,
            ),
            24 => self.host.world(
                events,
                PointNm {
                    x: 20000000,
                    y: 38000000,
                },
                true,
            ),
            25 => self.host.world(
                events,
                PointNm {
                    x: 20000000,
                    y: 32000000,
                },
                true,
            ),
            26 => {
                if r.workspace().schematic_authoring.vertices.len() >= 2 {
                    self.host.control(events, HitTarget::SchematicFinishWire);
                }
            }
            27 => {
                assert_eq!(self.wire_count(), self.wires + 2);
                let model = ProjectResolver::new(&self.amp.root).resolve().unwrap();
                let snapshot = ElectricalSelectionSnapshot::capture(&model).unwrap();
                let resistor = &snapshot.schematic_source().unwrap().sheets[&self.amp.child]
                    .symbols[&self.amp.emitter];
                assert_eq!(
                    resistor
                        .pins
                        .iter()
                        .find(|p| p.uuid == self.amp.fault_pin)
                        .unwrap()
                        .position,
                    eda_engine::ir::geometry::Point::new(20000000, 32000000)
                );
                assert_ne!(
                    r.workspace().checks.model_revision.as_deref(),
                    Some(model.model_revision.0.as_str())
                );
                self.capture("erc-stale");
                self.host
                    .app
                    .runtime
                    .as_mut()
                    .unwrap()
                    .cancel_native_schematic();
                self.history(false);
            }
            28 => {
                assert_eq!(self.wire_count(), self.wires);
                self.history(true);
            }
            29 => {
                assert_eq!(self.wire_count(), self.wires + 2);
                self.menu("Checks", "Run ERC");
            }
            30 => {
                self.current_checks();
                assert!(self.fault_finding().is_none());
                assert_ne!(
                    r.workspace().checks.check_run_id.as_deref(),
                    Some(self.run.as_str())
                );
                self.capture("erc-repaired");
            }
            31 => {
                self.current_checks();
                assert!(self.fault_finding().is_none());
                let checks = &r.workspace().checks;
                std::fs::write(
                    self.output.join("erc-after-fix.json"),
                    serde_json::to_vec_pretty(checks).unwrap(),
                )
                .unwrap();
                assert!(
                    self.host
                        .app
                        .runtime
                        .as_mut()
                        .unwrap()
                        .queue_native_edit(NativeEditAction::Save)
                );
            }
            32 => {
                self.saved = self.snapshot();
                let args = <GuiArgs as clap::Parser>::try_parse_from([
                    "datum-gui",
                    "--project-root",
                    self.amp.root.to_str().unwrap(),
                ])
                .unwrap();
                let proxy = self.host.app.terminal_event_proxy.clone();
                self.host.app = App::new(args, proxy);
                self.host.app.resumed(events);
            }
            33 => {
                assert_eq!(self.snapshot(), self.saved);
                assert_eq!(self.poses().len(), 11);
                assert_eq!(self.poses()["R2"], (10000000, 30000000));
                assert_eq!(self.wire_count(), self.wires + 2);
            }
            34 => {
                if !self.navigate_child(events) {
                    return;
                }
            }
            35 => {
                self.focus_and_fit();
                self.menu("Checks", "Run ERC");
            }
            36 => {
                self.current_checks();
                assert!(self.fault_finding().is_none());
                self.capture("amplifier-cold");
                events.exit();
            }
            _ => unreachable!(),
        }
        self.step += 1;
        self.host.since = Instant::now();
        self.host.app.request_workspace_redraw();
    }
}
impl ApplicationHandler for CaptureWorkflow {
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
            self.host.failure = Some(format!("Amplifier stage {} timed out", self.step));
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
#[ignore = "requires owned fresh DATUM_PRACTICAL_CAPTURE_PROJECT and isolated native Wayland"]
fn ordinary_app_amplifier_root_child_group_revision_erc_fix_history_save_reopen() {
    let root = PathBuf::from(std::env::var_os("DATUM_PRACTICAL_CAPTURE_PROJECT").unwrap());
    let output = PathBuf::from(std::env::var_os("DATUM_PRACTICAL_CAPTURE_OUTPUT").unwrap());
    std::fs::create_dir_all(&output).unwrap();
    let amp = circuit::setup(&root);
    let path = amp.source_file.clone();
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
    let mut flow = CaptureWorkflow {
        host,
        amp,
        output,
        step: 0,
        before: BTreeMap::new(),
        saved: BTreeMap::new(),
        fault: String::new(),
        run: String::new(),
        wires: 0,
    };
    events.run_app(&mut flow).unwrap();
    assert!(flow.host.failure.is_none(), "{:?}", flow.host.failure);
    assert_eq!(flow.step, 37);
}
