//! Bounded opt-in input receipts, exported only after the controlled workload.
//! No event scheduling, per-event I/O, DRM observation or acceptance inference.
use crate::{App, Runtime};
use anyhow::{Context, Result, ensure};
use serde_json::{Value, json};
use std::path::PathBuf;
use std::time::Instant;
use winit::event::{ElementState, MouseButton, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::window::WindowId;

const LIMIT: usize = 4096;
const HOVER_BYTES: usize = 128;

#[derive(Clone, Copy)]
struct State {
    cursor: Option<[f32; 2]>,
    native_cursor: Option<(f32, f32)>,
    center: [f32; 2],
    zoom: f32,
    pan: bool,
    epoch: u64,
    hover: [u8; HOVER_BYTES],
    hover_len: usize,
    hover_surface: Option<datum_gui_protocol::PaneContent>,
    truncated: bool,
}
impl State {
    fn capture(runtime: &Runtime) -> Self {
        let ui = &runtime.workspace().ui;
        let mut value = Self {
            cursor: ui.cursor_pos.map(|p| [p.x, p.y]),
            native_cursor: runtime.last_cursor_pos,
            center: [runtime.camera.center_x_nm, runtime.camera.center_y_nm],
            zoom: runtime.camera.zoom,
            pan: runtime.pan_gesture.is_active(),
            epoch: runtime.measurements.epoch(),
            hover: [0; HOVER_BYTES],
            hover_len: 0,
            hover_surface: None,
            truncated: false,
        };
        if let Some(hover) = &ui.hovered_object {
            value.hover_len = hover.object_id.len().min(HOVER_BYTES);
            value.hover[..value.hover_len]
                .copy_from_slice(&hover.object_id.as_bytes()[..value.hover_len]);
            value.hover_surface = Some(hover.surface);
            value.truncated = hover.object_id.len() > HOVER_BYTES;
        }
        value
    }
    fn value(&self) -> Value {
        json!({"cursor":self.cursor,"native_cursor":self.native_cursor,
            "camera_center_nm":self.center,"camera_zoom":self.zoom,"pan_active":self.pan,
            "device_epoch":self.epoch,"hover_utf8":std::str::from_utf8(&self.hover[..self.hover_len]).ok(),
            "hover_surface":self.hover_surface.map(|s|format!("{s:?}")),"truncated":self.truncated})
    }
}

struct Record {
    received_ns: u128,
    completed_ns: Option<u128>,
    position: Option<[f64; 2]>,
    button: Option<(u32, bool)>,
    route: &'static str,
    before: State,
    after: Option<State>,
}

pub(crate) struct InputObservation {
    path: PathBuf,
    started: Instant,
    monotonic_origin_ns: u64,
    records: Vec<Record>,
    overflow: bool,
    active: Option<usize>,
}
impl InputObservation {
    pub(crate) fn from_environment() -> Result<Option<Self>> {
        let Some(path) = std::env::var_os("DATUM_INPUT_RECEIPT") else {
            return Ok(None);
        };
        ensure!(
            std::env::var_os("DATUM_MEASUREMENT_SHUTDOWN_SOCKET").is_some(),
            "input receipt requires controlled measurement shutdown"
        );
        let mut records = Vec::new();
        records
            .try_reserve_exact(LIMIT)
            .context("reserve bounded input receipt")?;
        let mut ts = libc::timespec {
            tv_sec: 0,
            tv_nsec: 0,
        };
        // SAFETY: CLOCK_MONOTONIC is valid and ts is writable for the call.
        ensure!(
            unsafe { libc::clock_gettime(libc::CLOCK_MONOTONIC, &mut ts) } == 0,
            "input receipt clock unavailable"
        );
        Ok(Some(Self {
            path: path.into(),
            started: Instant::now(),
            monotonic_origin_ns: ts.tv_sec as u64 * 1_000_000_000 + ts.tv_nsec as u64,
            records,
            overflow: false,
            active: None,
        }))
    }
    fn begin(&mut self, event: &WindowEvent, before: State) -> bool {
        let (position, button) = match event {
            WindowEvent::CursorMoved { position, .. } => (Some([position.x, position.y]), None),
            WindowEvent::MouseInput { button, state, .. } => {
                let button = match button {
                    MouseButton::Left => 1,
                    MouseButton::Middle => 2,
                    MouseButton::Right => 3,
                    MouseButton::Back => 4,
                    MouseButton::Forward => 5,
                    MouseButton::Other(n) => u32::from(*n) + 6,
                };
                (None, Some((button, *state == ElementState::Pressed)))
            }
            _ => return false,
        };
        if self.active.is_some() || self.records.len() == LIMIT {
            self.overflow = true;
            return false;
        }
        self.active = Some(self.records.len());
        self.records.push(Record {
            received_ns: self.started.elapsed().as_nanos(),
            completed_ns: None,
            position,
            button,
            route: "unhandled",
            before,
            after: None,
        });
        true
    }
    pub(crate) fn route(&mut self, route: &'static str) {
        if let Some(index) = self.active {
            self.records[index].route = route;
        }
    }
    fn complete(&mut self, after: Option<State>) {
        if let Some(index) = self.active.take() {
            let record = &mut self.records[index];
            record.completed_ns = Some(self.started.elapsed().as_nanos());
            record.after = after;
        }
    }
    fn complete_receipt(&self) -> bool {
        !self.overflow
            && self.active.is_none()
            && self.records.iter().all(|r| {
                r.completed_ns.is_some()
                    && r.after.is_some_and(|s| !s.truncated)
                    && !r.before.truncated
            })
    }
    fn export(&self, runtime: &Runtime) -> Result<()> {
        let final_state = State::capture(runtime);
        let complete = self.complete_receipt() && !final_state.truncated;
        let state = runtime.workspace();
        let records: Vec<_> = self.records.iter().enumerate().map(|(i,r)|json!({"sequence":i,
            "received_ns":r.received_ns,"completed_ns":r.completed_ns,"position":r.position,
            "button":r.button,"route":r.route,"before":r.before.value(),"after":r.after.map(|s|s.value())})).collect();
        let report = json!({"schema":"datum.input-receipt/v1","pid":std::process::id(),
            "monotonic_origin_ns":self.monotonic_origin_ns,"complete":complete,"overflow":self.overflow,
            "record_limit":LIMIT,"record_storage_bytes":self.records.capacity()*std::mem::size_of::<Record>(),
            "records":records,"final_state":final_state.value(),
            "final_selection":format!("{:?}",state.selection),"final_focus":format!("{:?}",state.ui.focus),
            "final_focused_pane":state.ui.layout.focused.0,
            "scope":"Main native receipt and completed routing, not display acknowledgement or CPU/action acceptance"});
        std::fs::write(&self.path, serde_json::to_vec(&report)?).context("export input receipt")?;
        ensure!(complete, "incomplete or overflowed input receipt");
        Ok(())
    }
}

impl App {
    pub(crate) fn handle_native_window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        let observed = match (&mut self.input_observation, &self.runtime) {
            (Some(observer), Some(runtime)) if runtime.window.id() == window_id => {
                observer.begin(&event, State::capture(runtime))
            }
            _ => false,
        };
        self.dispatch_native_window_event(event_loop, window_id, event);
        if observed {
            self.input_observation
                .as_mut()
                .expect("active observer")
                .complete(self.runtime.as_ref().map(State::capture));
        }
    }
    pub(crate) fn export_input_receipt(&self) -> Result<()> {
        if let Some(observer) = &self.input_observation {
            observer.export(
                self.runtime
                    .as_ref()
                    .context("input receipt runtime absent")?,
            )?;
        }
        Ok(())
    }
}

pub(crate) fn mark(observer: &mut Option<InputObservation>, route: &'static str) {
    if let Some(observer) = observer {
        observer.route(route);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn state() -> State {
        State {
            cursor: None,
            native_cursor: None,
            center: [0.0, 0.0],
            zoom: 1.0,
            pan: false,
            epoch: 1,
            hover: [0; HOVER_BYTES],
            hover_len: 0,
            hover_surface: None,
            truncated: false,
        }
    }
    fn observer() -> InputObservation {
        InputObservation {
            path: PathBuf::new(),
            started: Instant::now(),
            monotonic_origin_ns: 0,
            records: Vec::with_capacity(LIMIT),
            overflow: false,
            active: None,
        }
    }
    fn motion() -> WindowEvent {
        WindowEvent::CursorMoved {
            device_id: winit::event::DeviceId::dummy(),
            position: winit::dpi::PhysicalPosition::new(300.0, 150.0),
        }
    }
    #[test]
    fn duplicate_receipts_and_semantics_are_not_redraw_counts() {
        let mut o = observer();
        for route in ["authoring_hover", "terminal_clipboard_menu", "unhandled"] {
            assert!(o.begin(&motion(), state()));
            assert!(!o.complete_receipt(), "in-flight receipt is incomplete");
            o.route(route);
            let mut after = state();
            after.cursor = Some([300.0, 150.0]);
            o.complete(Some(after));
        }
        assert!(o.complete_receipt());
        assert_eq!(o.records.len(), 3, "do not deduplicate native receipts");
        assert_eq!(
            o.records[2].route, "unhandled",
            "not a semantic acknowledgement"
        );
        assert_eq!(o.records[0].after.unwrap().cursor, Some([300.0, 150.0]));
        assert_eq!(o.records[0].position, o.records[1].position);
        assert!(
            o.records
                .iter()
                .all(|r| r.completed_ns.unwrap() >= r.received_ns)
        );
    }
    #[test]
    fn lost_completion_overflow_nesting_and_truncation_fail_closed() {
        let mut o = observer();
        for _ in 0..LIMIT {
            assert!(o.begin(&motion(), state()));
            o.complete(Some(state()));
        }
        let capacity = o.records.capacity();
        assert!(o.complete_receipt());
        assert!(!o.begin(&motion(), state()));
        assert!(!o.complete_receipt());
        assert_eq!(o.records.capacity(), capacity);
        let mut o = observer();
        assert!(o.begin(&motion(), state()));
        o.complete(None);
        assert!(!o.complete_receipt());
        let mut o = observer();
        assert!(o.begin(&motion(), state()));
        assert!(!o.begin(&motion(), state()));
        o.complete(Some(state()));
        assert!(!o.complete_receipt());
        let mut o = observer();
        let mut truncated = state();
        truncated.truncated = true;
        assert!(o.begin(&motion(), truncated));
        o.complete(Some(state()));
        assert!(!o.complete_receipt());
    }
    #[test]
    #[ignore = "native X11 input receipt conformance with pinned real board; run serially"]
    #[allow(deprecated)]
    fn native_pointer_receipts_follow_completed_dispatch_without_redraw_requirement() {
        use clap::Parser;
        use std::{sync::Arc, time::Duration};
        use winit::platform::{pump_events::EventLoopExtPumpEvents, x11::EventLoopBuilderExtX11};
        let board = std::env::var("DATUM_NATIVE_TEST_BOARD").expect("pinned real board");
        let args = crate::GuiArgs::try_parse_from(["datum-gui", "--board", &board]).unwrap();
        let mut events = winit::event_loop::EventLoop::<()>::with_user_event()
            .with_x11()
            .with_any_thread(true)
            .build()
            .unwrap();
        let wake = events.create_proxy();
        let window = Arc::new(
            events
                .create_window(
                    winit::window::Window::default_attributes()
                        .with_visible(false)
                        .with_inner_size(winit::dpi::PhysicalSize::new(1280, 800)),
                )
                .unwrap(),
        );
        let launch = args.load_launch_state(Some(wake.clone())).unwrap();
        let mut runtime = pollster::block_on(Runtime::new(
            window.clone(),
            launch,
            Some(1.0),
            wake.clone(),
        ))
        .unwrap();
        runtime.session.workspace_mut().ui.active_dock_tab = None;
        runtime.session.workspace_mut().ui.crosshair_style =
            datum_gui_protocol::CrosshairStyle::None;
        runtime.build_terminal_prepared_scene().unwrap();
        let panes = runtime
            .current_layout()
            .viewport_panes(&runtime.workspace().ui.layout);
        let rect = panes
            .panes
            .iter()
            .find(|p| p.content == datum_gui_protocol::PaneContent::Board)
            .unwrap()
            .rect
            .scene;
        let before_selection = format!("{:?}", runtime.workspace().selection);
        let mut app = App::new(args, wake);
        app.window = Some(window.clone());
        app.runtime = Some(runtime);
        let mut observation = observer();
        observation.path = std::env::temp_dir().join(format!(
            "datum-input-conformance-{}.json",
            std::process::id()
        ));
        app.input_observation = Some(observation);
        for x in [rect.x + 10.0, rect.x + 12.0, rect.x + 12.0] {
            let y = rect.y + 10.0;
            let mut pending = Some(WindowEvent::CursorMoved {
                device_id: winit::event::DeviceId::dummy(),
                position: winit::dpi::PhysicalPosition::new(f64::from(x), f64::from(y)),
            });
            events.pump_events(Some(Duration::ZERO), |_, active| {
                if let Some(event) = pending.take() {
                    app.handle_native_window_event(active, window.id(), event);
                }
            });
            assert!(pending.is_none());
            let observed = app
                .input_observation
                .as_ref()
                .unwrap()
                .records
                .last()
                .unwrap();
            assert_eq!(observed.route, "authoring_hover");
            assert_eq!(observed.after.unwrap().cursor, Some([x, y]));
            assert_eq!(observed.after.unwrap().native_cursor, Some((x, y)));
            assert!(!observed.after.unwrap().pan);
        }
        let observation = app.input_observation.as_ref().unwrap();
        assert_eq!(observation.records.len(), 3);
        assert!(observation.complete_receipt());
        let runtime = app.runtime.as_mut().unwrap();
        assert_eq!(
            format!("{:?}", runtime.workspace().selection),
            before_selection
        );
        runtime.begin_application_terminal_shutdown();
        let deadline = Instant::now() + Duration::from_secs(6);
        while !runtime.application_terminal_shutdown_complete() {
            runtime.poll_terminal_output();
            assert!(Instant::now() < deadline);
            std::thread::sleep(Duration::from_millis(10));
        }
        app.export_input_receipt().unwrap();
        let path = &app.input_observation.as_ref().unwrap().path;
        let value: Value = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
        assert_eq!(value["complete"], true);
        assert_eq!(value["records"].as_array().unwrap().len(), 3);
        eprintln!(
            "input receipt conformance:3 supplied native-dispatch motions, repeated coordinate retained, canonical cursor/selection preserved; storage_bytes={}",
            value["record_storage_bytes"]
        );
        std::fs::remove_file(path).unwrap();
    }
}
