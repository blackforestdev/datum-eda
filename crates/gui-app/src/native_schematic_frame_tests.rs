//! Finite schematic delivery through ordinary Wayland App input/frame dispatch.
//! Programmatic events are not physical Wayland/HUMAN qualification.
use super::*;
use crate::runtime_view_actions::action_evidence::EntrySurface;
use std::{
    path::PathBuf,
    time::{Duration, Instant},
};
use winit::platform::wayland::EventLoopBuilderExtWayland;

struct Fixture {
    app: App,
    stage: usize,
    since: Instant,
    path: PathBuf,
    preimage: Vec<u8>,
    wired: Option<Vec<u8>>,
    failure: Option<String>,
    wire_only: bool,
}
impl Fixture {
    fn mouse(&mut self, events: &ActiveEventLoop, at: (f64, f64), click: bool) {
        let r = self.app.runtime.as_ref().unwrap();
        eprintln!(
            "schematic-frame input stage={} at={at:?} click={click} hit={:?} editor_active={} picker={} status={:?}",
            self.stage,
            r.presented_hits.hit_test(at.0 as f32, at.1 as f32),
            r.workspace().schematic_authoring.active(),
            r.workspace().schematic_authoring.picker,
            r.workspace().last_command_status
        );
        eprintln!(
            "schematic-frame mapping={:?} position={:?}",
            r.renderer
                .render_session()
                .prepared()
                .and_then(|p| p.world_point_at_screen(at.0 as f32, at.1 as f32)),
            r.workspace().schematic_authoring.position
        );
        let window = self.app.window.as_ref().unwrap().id();
        self.app.window_event(
            events,
            window,
            WindowEvent::CursorMoved {
                device_id: winit::event::DeviceId::dummy(),
                position: winit::dpi::PhysicalPosition::new(at.0, at.1),
            },
        );
        if click {
            for state in [ElementState::Pressed, ElementState::Released] {
                self.app.window_event(
                    events,
                    window,
                    WindowEvent::MouseInput {
                        device_id: winit::event::DeviceId::dummy(),
                        state,
                        button: MouseButton::Left,
                    },
                );
            }
        }
    }
    fn control(&mut self, events: &ActiveEventLoop, target: HitTarget) {
        let r = self.app.runtime.as_ref().unwrap();
        let hit = r
            .presented_hits
            .regions()
            .iter()
            .find(|h| h.target == target)
            .expect("control must actually be presented")
            .rect;
        self.mouse(
            events,
            (
                (hit.x + hit.width / 2.) as f64,
                (hit.y + hit.height / 2.) as f64,
            ),
            true,
        );
    }
    fn world(&mut self, events: &ActiveEventLoop, point: PointNm, click: bool) {
        let r = self.app.runtime.as_mut().unwrap();
        let layout = r.workspace().ui.layout.clone();
        let p = r.prepared_scene().unwrap();
        let viewport = p
            .layout
            .viewport_panes(&layout)
            .panes
            .iter()
            .find(|pane| pane.content == PaneContent::Schematic)
            .expect("schematic pane")
            .rect
            .scene;
        let x = viewport.x + 20.;
        let y = viewport.y + 20.;
        let a = p.world_point_at_screen(x, y).unwrap().0;
        let b = p.world_point_at_screen(x + 100., y + 100.).unwrap().0;
        let at = (
            (x + (point.x - a.x) as f32 * 100. / (b.x - a.x) as f32) as f64,
            (y + (point.y - a.y) as f32 * 100. / (b.y - a.y) as f32) as f64,
        );
        self.mouse(events, at, click);
    }
    fn menu(&mut self, label: &str) {
        assert!(self.app.runtime.as_mut().unwrap().activate_menu_item(
            "Place",
            label,
            EntrySurface::MenuPointer
        ));
        self.app.request_workspace_redraw();
    }
    fn next(&mut self) {
        eprintln!(
            "schematic-frame stage={} elapsed_ms={}",
            self.stage,
            self.since.elapsed().as_millis()
        );
        self.stage += 1;
        self.since = Instant::now();
        self.app.request_workspace_redraw();
    }
    fn advance(&mut self, events: &ActiveEventLoop) {
        let Some(r) = self.app.runtime.as_ref() else {
            return;
        };
        if r.native_edit.pending() {
            return;
        }
        assert!(!r.native_edit.blocked, "canonical reply must qualify");
        if self.since.elapsed() < Duration::from_millis(20)
            || !r.surface_transaction.has_presented()
        {
            return;
        }
        if r.renderer.render_session().has_pending_frame() || r.presented_hits.regions().is_empty()
        {
            return;
        }
        match self.stage {
            0 => {
                let r = self.app.runtime.as_mut().unwrap();
                let pane = r
                    .workspace()
                    .ui
                    .layout
                    .leaves()
                    .into_iter()
                    .find(|pane| {
                        r.workspace().ui.layout.content_for(*pane) == Some(PaneContent::Schematic)
                    })
                    .unwrap();
                r.session.workspace_mut().ui.layout.focused = pane;
                r.set_application_focus(datum_gui_protocol::ApplicationFocus::Editor(pane));
                r.invalidate_frame();
                self.next();
                if self.wire_only {
                    self.stage = 10;
                }
            }
            1 => {
                self.menu("Symbol");
                self.next();
            }
            2 => {
                self.control(events, HitTarget::SchematicPlace);
                self.next();
            }
            3 => {
                self.world(
                    events,
                    PointNm {
                        x: -4_000_000,
                        y: 0,
                    },
                    false,
                );
                self.next();
            }
            4 => {
                assert_eq!(
                    std::fs::read(&self.path).unwrap(),
                    self.preimage,
                    "preview never writes"
                );
                self.control(events, HitTarget::SchematicCancel);
                assert_eq!(
                    std::fs::read(&self.path).unwrap(),
                    self.preimage,
                    "cancel never writes"
                );
                self.menu("Symbol");
                self.next();
            }
            5 => {
                self.control(events, HitTarget::SchematicPlace);
                self.next();
            }
            6 => {
                self.world(
                    events,
                    PointNm {
                        x: -4_000_000,
                        y: 0,
                    },
                    true,
                );
                self.next();
            }
            7 => {
                eprintln!(
                    "schematic-frame placement status={:?} refusal={:?}",
                    self.app
                        .runtime
                        .as_ref()
                        .unwrap()
                        .workspace()
                        .last_command_status,
                    self.app
                        .runtime
                        .as_ref()
                        .unwrap()
                        .workspace()
                        .schematic_authoring
                        .refusal
                );
                let e = &self
                    .app
                    .runtime
                    .as_ref()
                    .unwrap()
                    .workspace()
                    .schematic_authoring;
                assert_eq!(
                    e.source.as_ref().unwrap().sheets[&e.sheet.unwrap()]
                        .symbols
                        .len(),
                    1
                );
                // The first real symbol becomes the fit basis. Zoom out through
                // ordinary wheel dispatch so the next placement is on canvas.
                self.world(
                    events,
                    PointNm {
                        x: -4_000_000,
                        y: 0,
                    },
                    false,
                );
                let id = self.app.window.as_ref().unwrap().id();
                for _ in 0..6 {
                    self.app.window_event(
                        events,
                        id,
                        WindowEvent::MouseWheel {
                            device_id: winit::event::DeviceId::dummy(),
                            delta: winit::event::MouseScrollDelta::LineDelta(0., -3.),
                            phase: winit::event::TouchPhase::Moved,
                        },
                    );
                }
                self.menu("Symbol");
                self.next();
            }
            8 => {
                self.control(events, HitTarget::SchematicPlace);
                self.next();
            }
            9 => {
                self.world(events, PointNm { x: 4_000_000, y: 0 }, true);
                self.next();
            }
            10 => {
                let e = &self
                    .app
                    .runtime
                    .as_ref()
                    .unwrap()
                    .workspace()
                    .schematic_authoring;
                let sheet = &e.source.as_ref().unwrap().sheets[&e.sheet.unwrap()];
                assert_eq!(sheet.symbols.len(), if self.wire_only { 1 } else { 2 });
                assert!(
                    sheet
                        .symbols
                        .values()
                        .all(|s| s.pins.len() == 5 && s.part.is_some())
                );
                // Existing library terminals deliberately do not lie on this
                // user grid. Pickup must retain exact source pin coordinates.
                self.app
                    .runtime
                    .as_mut()
                    .unwrap()
                    .session
                    .workspace_mut()
                    .authoring
                    .snap
                    .grid_nm = 400_000;
                self.menu("Wire");
                self.next();
            }
            11 => {
                self.world(
                    events,
                    PointNm {
                        x: if self.wire_only {
                            1_500_000
                        } else {
                            -2_500_000
                        },
                        y: 0,
                    },
                    true,
                );
                self.next();
            }
            12 => {
                self.world(
                    events,
                    PointNm {
                        x: if self.wire_only { 3_000_000 } else { 0 },
                        y: 0,
                    },
                    true,
                );
                self.next();
            }
            13 => {
                self.world(
                    events,
                    PointNm {
                        x: if self.wire_only { 3_000_000 } else { 0 },
                        y: -500_000,
                    },
                    true,
                );
                self.next();
            }
            14 => {
                self.world(
                    events,
                    PointNm {
                        x: if self.wire_only {
                            -1_500_000
                        } else {
                            2_500_000
                        },
                        y: -500_000,
                    },
                    true,
                );
                self.next();
            }
            15 => {
                self.control(events, HitTarget::SchematicFinishWire);
                self.next();
            }
            16 => {
                let e = &self
                    .app
                    .runtime
                    .as_ref()
                    .unwrap()
                    .workspace()
                    .schematic_authoring;
                assert_eq!(
                    e.source.as_ref().unwrap().sheets[&e.sheet.unwrap()]
                        .wires
                        .len(),
                    3
                );
                self.wired = Some(std::fs::read(&self.path).unwrap());
                self.app
                    .runtime
                    .as_mut()
                    .unwrap()
                    .queue_native_history(false);
                self.next();
            }
            17 => {
                let e = &self
                    .app
                    .runtime
                    .as_ref()
                    .unwrap()
                    .workspace()
                    .schematic_authoring;
                assert!(
                    e.source.as_ref().unwrap().sheets[&e.sheet.unwrap()]
                        .wires
                        .is_empty()
                );
                self.app
                    .runtime
                    .as_mut()
                    .unwrap()
                    .queue_native_history(true);
                self.next();
            }
            18 => {
                assert_eq!(
                    std::fs::read(&self.path).unwrap(),
                    *self.wired.as_ref().unwrap(),
                    "redo restores complete source bytes"
                );
                self.app.runtime.as_mut().unwrap().queue_native_edit(
                    datum_gui_protocol::native_edit_client::NativeEditAction::Save,
                );
                self.next();
            }
            19 => {
                assert_eq!(
                    std::fs::read(&self.path).unwrap(),
                    *self.wired.as_ref().unwrap(),
                    "Save does not mutate authored bytes"
                );
                let r = self.app.runtime.as_ref().unwrap();
                let request = &r.workspace().backing.as_ref().unwrap().request;
                let model = eda_engine::substrate::ProjectResolver::new(&request.project_root)
                    .resolve()
                    .unwrap();
                let snapshot =
                    eda_engine::substrate::ElectricalSelectionSnapshot::capture(&model).unwrap();
                let source = snapshot.schematic_source().unwrap();
                let sheet = source.sheets.values().next().unwrap();
                let pin = |reference: &str, name: &str| {
                    sheet
                        .symbols
                        .values()
                        .find(|s| s.reference == reference)
                        .unwrap()
                        .pins
                        .iter()
                        .find(|p| p.name == name)
                        .unwrap()
                        .uuid
                };
                let members = snapshot
                    .schematic_run(
                        snapshot.revision(),
                        &eda_engine::substrate::ElectricalOccurrence {
                            class: "pins".into(),
                            source_id: pin("U1", "OUT"),
                            instance_path: Vec::new(),
                        },
                    )
                    .unwrap();
                assert!(
                    members.owned.iter().any(
                        |m| m.source_id == pin(if self.wire_only { "U1" } else { "U2" }, "IN-")
                    ),
                    "intended terminals must share authoritative physical membership after reopen"
                );
                #[cfg(feature = "visual")]
                self.app
                    .runtime
                    .as_mut()
                    .unwrap()
                    .write_visual_screenshot(
                        &std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                            .join("../../target/nsd-native-connected.png"),
                    )
                    .unwrap();
                self.next();
                events.exit();
            }
            _ => events.exit(),
        }
    }
}
impl ApplicationHandler for Fixture {
    fn resumed(&mut self, events: &ActiveEventLoop) {
        self.app.resumed(events);
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
        self.app.window_event(events, id, event);
    }
    fn about_to_wait(&mut self, events: &ActiveEventLoop) {
        self.app.about_to_wait(events);
        if self
            .app
            .window
            .as_ref()
            .is_some_and(|w| self.app.frames.rendering_failed(w.id()))
        {
            self.failure = Some("ordinary frame coordinator paused".into());
            events.exit();
            return;
        }
        if self.since.elapsed() > Duration::from_secs(30) {
            self.failure = Some(format!("stage {} failed to finish", self.stage));
            events.exit();
            return;
        }
        self.advance(events);
        // Reuse the existing move replay's scheduling rule: a next scripted
        // input adds its own wake, while writes and presentation must complete
        // through ordinary worker/device wakes. Retain the native deadline.
        let bound = self.since + Duration::from_secs(30);
        let due = if self.app.runtime.as_ref().is_some_and(|r| {
            !r.native_edit.pending()
                && !r.renderer.render_session().has_pending_frame()
                && r.surface_transaction.has_presented()
        }) {
            (self.since + Duration::from_millis(20)).min(bound)
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
#[ignore = "requires owned DATUM_SCHEMATIC_PROJECT prepared with canonical setup and native Wayland"]
fn ordinary_app_symbol_preview_cancel_place_wire_history_save_reopen() {
    run(false);
}
#[test]
#[ignore = "requires owned canonical library-backed U1 seed at the origin and native Wayland"]
fn ordinary_app_existing_symbol_wire_history_save_reopen() {
    run(true);
}
fn run(wire_only: bool) {
    let root = std::env::var("DATUM_SCHEMATIC_PROJECT").unwrap();
    let index: serde_json::Value = serde_json::from_slice(
        &std::fs::read(PathBuf::from(&root).join("schematic/schematic.json")).unwrap(),
    )
    .unwrap();
    let relative = index["sheets"]
        .as_object()
        .unwrap()
        .values()
        .next()
        .unwrap()
        .as_str()
        .unwrap();
    let path = PathBuf::from(&root).join("schematic").join(relative);
    let args =
        <GuiArgs as clap::Parser>::try_parse_from(["datum-gui", "--project-root", &root]).unwrap();
    let events = EventLoop::<()>::with_user_event()
        .with_wayland()
        .with_any_thread(true)
        .build()
        .unwrap();
    let mut fixture = Fixture {
        app: App::new(args, events.create_proxy()),
        stage: 0,
        since: Instant::now(),
        preimage: std::fs::read(&path).unwrap(),
        path,
        wired: None,
        failure: None,
        wire_only,
    };
    events.run_app(&mut fixture).unwrap();
    assert!(fixture.failure.is_none(), "{:?}", fixture.failure);
    assert_eq!(fixture.stage, 20);
}
