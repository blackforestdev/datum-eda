//! Owner-board QA fixture: ordinary App frame loop interleaved with editor input.
//! In-process input events are not physical Wayland/HUMAN qualification.
use super::*;
use std::time::{Duration, Instant};
use winit::platform::wayland::EventLoopBuilderExtWayland;
#[path = "native_move_frame_repeat.rs"]
mod repeat;
#[path = "native_move_frame_steps.rs"]
mod steps;

struct Fixture {
    app: App,
    stage: usize,
    origin: Option<(f64, f64)>,
    started: Instant,
    stage_started: Instant,
    failure: Option<String>,
    board_path: std::path::PathBuf,
    preimage: Vec<u8>,
    committed: Option<Vec<u8>>,
    logged_stage: Option<usize>,
    original_position: Option<PointNm>,
    selected_component: Option<String>,
    original_label: Option<datum_gui_render::RectPx>,
    committed_label: Option<datum_gui_render::RectPx>,
    committing_position: Option<PointNm>,
    commit_started: Option<Instant>,
    pending_placement_presented: bool,
    post_move_component: Option<String>,
    first_committed_position: Option<PointNm>,
}
impl Fixture {
    fn mouse(&mut self, events: &ActiveEventLoop, x: f64, y: f64, click: bool) {
        let window = self.app.window.as_ref().unwrap().id();
        let started = Instant::now();
        self.app.window_event(
            events,
            window,
            WindowEvent::CursorMoved {
                device_id: winit::event::DeviceId::dummy(),
                position: winit::dpi::PhysicalPosition::new(x, y),
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
        eprintln!(
            "move-frame stage={} input_us={} click={click}",
            self.stage,
            started.elapsed().as_micros()
        );
    }
    fn selected_label(&self) -> datum_gui_render::RectPx {
        let runtime = self.app.runtime.as_ref().unwrap();
        let component = runtime
            .workspace()
            .scene
            .components
            .iter()
            .find(|c| Some(&c.component_uuid) == self.selected_component.as_ref())
            .unwrap();
        runtime.presented_hits.regions().iter().find(|r| matches!(&r.target, HitTarget::AuthoredObject(id) if id == &component.object_id)).expect("selected Footprint must have a presented label").rect
    }
}
impl ApplicationHandler for Fixture {
    fn resumed(&mut self, events: &ActiveEventLoop) {
        self.app.resumed(events);
    }
    fn window_event(&mut self, events: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        // The owner can keep using the desktop. Scripted input enters App directly;
        // unrelated physical pointer events must not overwrite its cursor reference.
        if matches!(
            event,
            WindowEvent::CursorMoved { .. }
                | WindowEvent::MouseInput { .. }
                | WindowEvent::CursorLeft { .. }
                | WindowEvent::KeyboardInput { .. }
                | WindowEvent::MouseWheel { .. }
        ) {
            return;
        }
        let redraw = matches!(event, WindowEvent::RedrawRequested);
        let started = Instant::now();
        self.app.window_event(events, id, event);
        if redraw {
            eprintln!(
                "move-frame native_redraw_us={}",
                started.elapsed().as_micros()
            );
            if self.app.frames.rendering_failed(id) {
                self.failure = Some("ordinary frame coordinator paused rendering".into());
                events.exit();
            }
        }
    }
    fn about_to_wait(&mut self, events: &ActiveEventLoop) {
        let pending = self.app.runtime.as_ref().map(|r| {
            (
                r.terminal_production_refresh_pending,
                r.terminal_workspace_refresh_pending,
            )
        });
        let started = Instant::now();
        self.app.about_to_wait(events);
        if self
            .app
            .window
            .as_ref()
            .is_some_and(|w| self.app.frames.rendering_failed(w.id()))
        {
            self.failure = Some("ordinary frame coordinator paused rendering".into());
            events.exit();
            return;
        }
        if self.stage == 20 {
            let runtime = self.app.runtime.as_ref().unwrap();
            if runtime.native_edit.pending()
                && !runtime.renderer.render_session().has_pending_frame()
            {
                let position = runtime
                    .workspace()
                    .scene
                    .components
                    .iter()
                    .find(|c| Some(&c.component_uuid) == self.selected_component.as_ref())
                    .unwrap()
                    .position;
                assert_eq!(
                    Some(position),
                    self.committing_position,
                    "pending placement must not return to its origin"
                );
                let label = self.selected_label();
                assert_ne!(
                    Some(label),
                    self.original_label,
                    "pending placement must be presented away from its origin"
                );
                if !self.pending_placement_presented {
                    eprintln!(
                        "move-frame pending placement presented at={position:?} label={label:?} status={:?}",
                        runtime.workspace().last_command_status
                    );
                }
                self.pending_placement_presented = true;
            }
        }
        if self.stage == 20
            && self.post_move_component.is_none()
            && self.app.runtime.as_ref().is_some_and(|r| {
                r.native_edit.pending() && !r.renderer.render_session().has_pending_frame()
            })
        {
            self.select_while_commit_pending(events);
        }
        if started.elapsed() > Duration::from_millis(100) {
            eprintln!(
                "move-frame ordinary_round_ms={} production_workspace_pending={pending:?}",
                started.elapsed().as_millis()
            );
        }
        if self.stage < 34
            && self.stage_started.elapsed() >= Duration::from_millis(20)
            && self.app.runtime.as_ref().is_some_and(|r| {
                r.surface_transaction.has_presented()
                    && r.renderer.render_session().prepared().is_some()
            })
        {
            self.advance(events);
        }
        if self.started.elapsed() > Duration::from_secs(90) {
            self.failure = Some(format!(
                "fixture stopped at stage {} without completion",
                self.stage
            ));
            events.exit();
        }
        if self.stage == 34
            && self
                .app
                .runtime
                .as_ref()
                .is_some_and(|r| r.application_terminal_shutdown_complete())
        {
            events.exit();
        }
        // Retain the ordinary native scheduler. Only an actual next script input
        // adds a wake; pending edits rely on production worker/queue wakes.
        let script_due = (self.stage < 34
            && self.app.runtime.as_ref().is_some_and(|r| {
                !r.native_edit.pending()
                    // At these stages there is no next input yet: require actual
                    // production completion/presentation wakes, not script polling.
                    && (!matches!(self.stage, 17 | 19..=23 | 26..=27 | 29..=33)
                        || (!r.renderer.render_session().has_pending_frame()
                            && !r.native_move_frame_needed()))
                    && r.surface_transaction.has_presented()
                    && r.renderer.render_session().prepared().is_some()
            }))
        .then_some(self.stage_started + Duration::from_millis(20));
        let bound = self.started + Duration::from_secs(90);
        let due = script_due.map_or(bound, |due| due.min(bound));
        use winit::event_loop::ControlFlow;
        events.set_control_flow(match events.control_flow() {
            ControlFlow::Poll => ControlFlow::Poll,
            ControlFlow::Wait => ControlFlow::WaitUntil(due),
            ControlFlow::WaitUntil(native) => ControlFlow::WaitUntil(native.min(due)),
        });
    }
}

#[test]
#[ignore = "requires owned DATUM_FIRST_EDIT_PROJECT copy and native Wayland; focused ordinary edit QA"]
fn ordinary_app_select_m_move_commit_history_reopen() {
    let root = std::env::var("DATUM_FIRST_EDIT_PROJECT").unwrap();
    let path = std::path::Path::new(&root).join("board/board.json");
    let before = std::fs::read(&path).unwrap();
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
        origin: None,
        started: Instant::now(),
        stage_started: Instant::now(),
        failure: None,
        board_path: path.clone(),
        preimage: before.clone(),
        committed: None,
        logged_stage: None,
        original_position: None,
        selected_component: None,
        original_label: None,
        committed_label: None,
        committing_position: None,
        commit_started: None,
        pending_placement_presented: false,
        post_move_component: None,
        first_committed_position: None,
    };
    events.run_app(&mut fixture).unwrap();
    assert_eq!(
        std::fs::read(path).unwrap(),
        fixture.committed.clone().unwrap(),
        "Save leaves committed bytes unchanged"
    );
    assert!(fixture.failure.is_none(), "{:?}", fixture.failure);
    assert_eq!(fixture.stage, 34);
    let reopened = load_board_editor_workspace_state(
        &fixture
            .app
            .runtime
            .as_ref()
            .unwrap()
            .workspace()
            .backing
            .as_ref()
            .unwrap()
            .request,
    )
    .unwrap();
    assert_eq!(
        reopened.scene.source_revision,
        fixture
            .app
            .runtime
            .as_ref()
            .unwrap()
            .workspace()
            .scene
            .source_revision,
        "reopen resolves saved revision"
    );
}
