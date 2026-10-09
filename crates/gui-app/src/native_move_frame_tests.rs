//! Owner-board QA fixture: ordinary App frame loop interleaved with editor input.
//! In-process input events are not physical Wayland/HUMAN qualification.
use super::*;
use std::time::{Duration, Instant};
use winit::platform::wayland::EventLoopBuilderExtWayland;

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
    pending_placement_presented: bool,
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
    fn advance(&mut self, events: &ActiveEventLoop) {
        let runtime = self.app.runtime.as_mut().unwrap();
        if self.logged_stage != Some(self.stage) {
            eprintln!(
                "move-frame stage={} redraw_gap_ms={} selection={:?} preview={}",
                self.stage,
                self.stage_started.elapsed().as_millis(),
                match &runtime.workspace().selection {
                    datum_gui_protocol::SelectionTarget::Native(v) => Some(&v.subject),
                    _ => None,
                },
                runtime.native_edit.preview.is_some()
            );
            self.logged_stage = Some(self.stage);
        }
        self.stage_started = Instant::now();
        match self.stage {
            0 => {
                let component = runtime
                    .workspace()
                    .scene
                    .components
                    .iter()
                    .find(|c| c.component_uuid == "ca340e78-04a5-5307-a024-9c5720a807d2")
                    .or_else(|| {
                        runtime
                            .workspace()
                            .scene
                            .components
                            .iter()
                            .find(|c| !c.locked)
                    })
                    .unwrap()
                    .clone();
                // Use an actual Pad hit on this Footprint, through production mouse dispatch.
                let point = runtime
                    .workspace()
                    .scene
                    .pads
                    .iter()
                    .find(|p| p.component_uuid == component.component_uuid)
                    .and_then(|p| p.center.integer_point())
                    .unwrap_or(component.position);
                let prepared = runtime.prepared_scene().unwrap();
                let v = prepared.scene_viewport;
                let a = (v.x + 20.0, v.y + 20.0);
                let wa = prepared.world_point_at_screen(a.0, a.1).unwrap().0;
                let wb = prepared
                    .world_point_at_screen(a.0 + 100.0, a.1 + 100.0)
                    .unwrap()
                    .0;
                let x = a.0 as f64 + (point.x - wa.x) as f64 * 100.0 / (wb.x - wa.x) as f64;
                let y = a.1 as f64 + (point.y - wa.y) as f64 * 100.0 / (wb.y - wa.y) as f64;
                self.origin = Some((x, y));
                self.mouse(events, x, y, true);
            }
            1 => {
                let datum_gui_protocol::SelectionTarget::Native(view) =
                    &runtime.workspace().selection
                else {
                    panic!("native Object selection required");
                };
                let datum_gui_protocol::SelectionSubject::Object(id) = &view.subject else {
                    panic!("Footprint Object required");
                };
                assert_eq!(
                    id.class,
                    datum_gui_protocol::AuthoredSelectionClass::Footprint
                );
                self.selected_component = Some(id.id.to_string());
                self.original_label = runtime.presented_hits.regions().iter().find_map(|r| {
                    matches!(&r.target, HitTarget::AuthoredObject(target) if target == &runtime.workspace().scene.components.iter().find(|c| c.component_uuid == id.id.to_string()).unwrap().object_id).then_some(r.rect)
                });
                assert!(
                    self.original_label.is_some(),
                    "selected Footprint label must be presented before M"
                );
                self.original_position = Some(
                    runtime
                        .workspace()
                        .scene
                        .components
                        .iter()
                        .find(|c| c.component_uuid == id.id.to_string())
                        .unwrap()
                        .position,
                );
                eprintln!(
                    "move-frame M cursor={:?} origin={:?} tool={:?} components={:?}",
                    runtime.last_cursor_pos,
                    self.origin,
                    runtime.workspace().tool,
                    runtime
                        .workspace()
                        .scene
                        .components
                        .iter()
                        .find(|c| c.component_uuid == "ca340e78-04a5-5307-a024-9c5720a807d2")
                        .map(|c| (&c.component_uuid, &c.object_id, c.locked))
                );
                let started = Instant::now();
                let action = workspace_keyboard::character_action("m", false).unwrap();
                workspace_keyboard::apply(runtime, action);
                eprintln!(
                    "move-frame M_us={} immediate_preview={}",
                    started.elapsed().as_micros(),
                    runtime.native_edit.preview.is_some()
                );
                self.app.request_redraw_if_needed();
            }
            2 => {
                // Record the owner sequence before accommodating the old two-click adapter.
                let (x, y) = self.origin.unwrap();
                self.mouse(events, x + 1.0, y + 1.0, false);
            }
            3 => {
                assert!(
                    runtime.native_edit.preview.is_some(),
                    "M must pick up the selected Footprint without another click"
                );
                self.app.request_redraw_if_needed();
            }
            4..=15 => {
                let (x, y) = self.origin.unwrap();
                if self.stage == 4 && std::env::var_os("DATUM_FIRST_EDIT_POINTER_BURST").is_some() {
                    // Reproduce accumulated native motion in one dispatch batch,
                    // before the next ordinary frame/background round. Finite input,
                    // not a responsiveness budget or OS injection mechanism.
                    let started = Instant::now();
                    for step in 0..192 {
                        let delta = 2.0 + (step % 12) as f64;
                        self.mouse(events, x + delta, y + delta, false);
                    }
                    eprintln!(
                        "move-frame accumulated_pointer_batch_ms={}",
                        started.elapsed().as_millis()
                    );
                }
                let delta = (self.stage - 3) as f64 * 2.0;
                self.mouse(events, x + delta, y + delta, false);
            }
            16 => {
                let preview = runtime
                    .native_edit
                    .preview
                    .as_ref()
                    .expect("moving preview remains active");
                let component = runtime
                    .workspace()
                    .scene
                    .components
                    .iter()
                    .find(|c| c.component_uuid == preview.component.component_uuid)
                    .unwrap();
                assert_ne!(
                    component.position, preview.component.position,
                    "pointer movement must translate the selected Footprint"
                );
                let label = runtime.presented_hits.regions().iter().find(|r| matches!(&r.target, HitTarget::AuthoredObject(id) if id == &preview.component.object_id)).expect("moved Footprint label must be presented");
                let original = self.original_label.unwrap();
                eprintln!(
                    "move-frame presented label original={original:?} moved={:?} pending_frame={}",
                    label.rect,
                    runtime.renderer.render_session().has_pending_frame()
                );
                assert!(
                    (label.rect.x - original.x).abs() > 2.0
                        || (label.rect.y - original.y).abs() > 2.0,
                    "prepared translation alone is insufficient: the actually presented Footprint must move"
                );
                // Cancel another queued motion before any frame can materialize it.
                let (x, y) = self.origin.unwrap();
                self.mouse(events, x + 30.0, y + 30.0, false);
                self.app
                    .runtime
                    .as_mut()
                    .unwrap()
                    .dispatch_session_command(SessionCommand::CancelAuthoringGesture);
                self.app.request_redraw_if_needed();
            }
            17 => {
                if runtime.renderer.render_session().has_pending_frame() {
                    return;
                }
                let original = self.original_label.unwrap();
                let actual = self.selected_label();
                assert!(
                    (actual.x - original.x).abs() < 1.0 && (actual.y - original.y).abs() < 1.0,
                    "Cancel must restore the presented Footprint"
                );
                let runtime = self.app.runtime.as_mut().unwrap();
                assert_eq!(std::fs::read(&self.board_path).unwrap(), self.preimage);
                assert_eq!(
                    runtime
                        .workspace()
                        .scene
                        .components
                        .iter()
                        .find(|c| Some(&c.component_uuid) == self.selected_component.as_ref())
                        .unwrap()
                        .position,
                    self.original_position.unwrap(),
                    "cancelled queued motion must not resurrect on the next frame"
                );
                let (x, y) = self.origin.unwrap();
                self.mouse(events, x, y, false);
                let runtime = self.app.runtime.as_mut().unwrap();
                assert!(workspace_keyboard::apply(
                    runtime,
                    workspace_keyboard::character_action("m", false).unwrap()
                ));
                assert!(
                    runtime.native_edit.preview.is_some(),
                    "M restarts after cancel"
                );
            }
            18 => {
                let (x, y) = self.origin.unwrap();
                self.mouse(events, x + 60.0, y + 30.0, false);
            }
            19 => {
                if runtime.native_move_frame_needed()
                    || runtime.renderer.render_session().has_pending_frame()
                {
                    return;
                }
                // The published Footprint label is an AuthoredObject screen hit.
                // Before the correction it reselected instead of committing Move.
                let target = &runtime
                    .native_edit
                    .preview
                    .as_ref()
                    .unwrap()
                    .component
                    .object_id;
                let region = runtime
                    .presented_hits
                    .regions()
                    .iter()
                    .find(|r| matches!(&r.target, HitTarget::AuthoredObject(id) if id == target))
                    .expect("rendered Footprint must expose its authored label");
                let x = (region.rect.x + region.rect.width * 0.5) as f64;
                let y = (region.rect.y + region.rect.height * 0.5) as f64;
                let click_world = runtime
                    .prepared_scene()
                    .unwrap()
                    .world_point_at_screen(x as f32, y as f32)
                    .unwrap()
                    .0;
                let runtime = self.app.runtime.as_ref().unwrap();
                let destination = runtime
                    .native_edit
                    .preview
                    .as_ref()
                    .unwrap()
                    .destination(click_world, runtime.workspace().authoring.snap)
                    .unwrap();
                self.committing_position = Some(destination);
                self.mouse(events, x, y, true);
                assert_eq!(
                    self.app
                        .runtime
                        .as_ref()
                        .unwrap()
                        .workspace()
                        .scene
                        .components
                        .iter()
                        .find(|c| Some(&c.component_uuid) == self.selected_component.as_ref())
                        .unwrap()
                        .position,
                    destination,
                    "Placement must retain the clicked pose while the canonical edit is pending"
                );
                assert!(
                    self.app.runtime.as_ref().unwrap().native_edit.pending(),
                    "destination click must commit rather than reselect"
                );
            }
            20 => {
                if runtime.native_edit.pending()
                    || runtime.renderer.render_session().has_pending_frame()
                {
                    return;
                }
                assert!(
                    !runtime.native_edit.blocked,
                    "committed refresh must be qualified"
                );
                assert_eq!(
                    runtime
                        .workspace()
                        .scene
                        .components
                        .iter()
                        .find(|c| Some(&c.component_uuid) == self.selected_component.as_ref())
                        .unwrap()
                        .position,
                    self.committing_position.unwrap(),
                    "qualified canonical placement must match the final click"
                );
                eprintln!(
                    "move-frame pending_placement_presented={}",
                    self.pending_placement_presented
                );
                let after = std::fs::read(&self.board_path).unwrap();
                assert_ne!(after, self.preimage);
                self.committed = Some(after);
                self.committed_label = Some(self.selected_label());
                assert_ne!(
                    self.committed_label, self.original_label,
                    "Commit must present the translated Footprint, not just change bytes"
                );
                let runtime = self.app.runtime.as_mut().unwrap();
                eprintln!(
                    "move-frame committed presented_revision={} status={:?}",
                    runtime.workspace().scene.source_revision,
                    runtime.workspace().last_command_status
                );
                assert!(runtime.queue_native_history(false));
            }
            21 => {
                if runtime.native_edit.pending()
                    || runtime.renderer.render_session().has_pending_frame()
                {
                    return;
                }
                assert!(!runtime.native_edit.blocked);
                assert_eq!(
                    std::fs::read(&self.board_path).unwrap(),
                    self.preimage,
                    "Undo restores preimage"
                );
                assert_eq!(
                    self.selected_label(),
                    self.original_label.unwrap(),
                    "Undo must be presented"
                );
                let runtime = self.app.runtime.as_mut().unwrap();
                assert!(runtime.queue_native_history(true));
            }
            22 => {
                if runtime.native_edit.pending()
                    || runtime.renderer.render_session().has_pending_frame()
                {
                    return;
                }
                assert!(!runtime.native_edit.blocked);
                assert_eq!(
                    std::fs::read(&self.board_path).unwrap(),
                    *self.committed.as_ref().unwrap(),
                    "Redo restores committed board"
                );
                assert_eq!(
                    self.selected_label(),
                    self.committed_label.unwrap(),
                    "Redo must be presented"
                );
                let runtime = self.app.runtime.as_mut().unwrap();
                assert!(runtime.queue_native_edit(
                    datum_gui_protocol::native_edit_client::NativeEditAction::Save
                ));
            }
            23 => {
                if runtime.native_edit.pending()
                    || runtime.renderer.render_session().has_pending_frame()
                {
                    return;
                }
                assert!(!runtime.native_edit.blocked);
                assert_eq!(
                    std::fs::read(&self.board_path).unwrap(),
                    *self.committed.as_ref().unwrap()
                );
            }
            _ => {
                runtime.begin_application_terminal_shutdown();
                self.stage = 25;
                return;
            }
        }
        self.stage += 1;
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
        if started.elapsed() > Duration::from_millis(100) {
            eprintln!(
                "move-frame ordinary_round_ms={} production_workspace_pending={pending:?}",
                started.elapsed().as_millis()
            );
        }
        if self.stage < 25
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
        if self.stage == 25
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
        let script_due = (self.stage < 25
            && self.app.runtime.as_ref().is_some_and(|r| {
                !r.native_edit.pending()
                    // At these stages there is no next input yet: require actual
                    // production completion/presentation wakes, not script polling.
                    && (!matches!(self.stage, 17 | 19..=23)
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
        pending_placement_presented: false,
    };
    events.run_app(&mut fixture).unwrap();
    assert_eq!(
        std::fs::read(path).unwrap(),
        fixture.committed.clone().unwrap(),
        "Save leaves committed bytes unchanged"
    );
    assert!(fixture.failure.is_none(), "{:?}", fixture.failure);
    assert_eq!(fixture.stage, 25);
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
