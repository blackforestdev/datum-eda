//! Ordinary App/presented controls and production logical-key consumer proof.
//! Programmatic dispatch is distinct from physical input and HUMAN qualification.
use super::*;
#[path = "native_library_frame_tests/proof_assertions.rs"]
mod proof_assertions;
#[path = "native_library_frame_tests/scenario.rs"]
mod scenario;
use crate::runtime_view_actions::action_evidence::EntrySurface;
use datum_gui_protocol::library_browser::{KINDS, LibraryControl as C};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};
use winit::platform::wayland::EventLoopBuilderExtWayland;

enum Step {
    Open,
    Kind(usize),
    First,
    Search(String),
    Select(String),
    Control(C),
    Field(&'static str, String),
    Choice(&'static str, String),
    Capture(&'static str),
    Snapshot,
    Invariant,
    History(bool),
    Save,
    CheckFootprint(usize),
    CheckFrozen,
    Resize,
    Narrow,
    CheckNarrow,
    Tile,
    Focus(C),
    MissingGeometry,
    RecordPlaced(bool),
    CheckPlaced(bool),
    CheckProcess(usize),
    CheckRemovedPreview,
    VerifyReopen,
    RecordFootprint,
    CheckFootprintValue(bool),
    Reopen,
    KeyControl(C),
    Relationship,
    Refusal,
    RefusalNotice,
}
struct Fixture {
    app: App,
    steps: Vec<Step>,
    stage: usize,
    since: Instant,
    root: PathBuf,
    output: PathBuf,
    snapshot: BTreeMap<String, Vec<u8>>,
    board: Vec<u8>,
    failure: Option<String>,
    proxy: winit::event_loop::EventLoopProxy<()>,
    original_footprint: Option<serde_json::Value>,
    edited_footprint: Option<serde_json::Value>,
    original_revision: Option<u64>,
    expected_package: uuid::Uuid,
    placed_source: Option<(uuid::Uuid, serde_json::Value, Option<u64>)>,
    normal_pane_width: Option<f32>,
}
impl Fixture {
    fn key(&mut self, key: Key) {
        assert!(
            self.app
                .runtime
                .as_mut()
                .unwrap()
                .handle_library_input(&key, ElementState::Pressed)
        );
        self.app.request_workspace_redraw();
    }
    fn focus_field(&mut self, key: &str) {
        let r = self.app.runtime.as_ref().unwrap();
        let index = r
            .workspace()
            .library
            .draft
            .as_ref()
            .unwrap()
            .fields
            .iter()
            .position(|f| f.key == key)
            .unwrap();
        for _ in 0..40 {
            if self.app.runtime.as_ref().unwrap().workspace().library.focus == Some(C::Field(index))
            {
                return;
            }
            self.key(Key::Named(NamedKey::Tab));
        }
        panic!("Field {key} must be keyboard reachable");
    }
    fn control(&mut self, events: &ActiveEventLoop, target: HitTarget) {
        let rect = self
            .app
            .runtime
            .as_ref()
            .unwrap()
            .presented_hits
            .regions()
            .iter()
            .find(|h| h.target == target)
            .unwrap_or_else(|| panic!("stage{} missing presented {target:?}", self.stage))
            .rect;
        assert_eq!(
            self.app
                .runtime
                .as_ref()
                .unwrap()
                .presented_hits
                .hit_test(rect.x + rect.width / 2., rect.y + rect.height / 2.),
            Some(&target),
            "presented pointer target must be unobscured"
        );
        let id = self.app.window.as_ref().unwrap().id();
        self.app.window_event(
            events,
            id,
            WindowEvent::CursorMoved {
                device_id: winit::event::DeviceId::dummy(),
                position: winit::dpi::PhysicalPosition::new(
                    (rect.x + rect.width / 2.) as f64,
                    (rect.y + rect.height / 2.) as f64,
                ),
            },
        );
        for state in [ElementState::Pressed, ElementState::Released] {
            self.app.window_event(
                events,
                id,
                WindowEvent::MouseInput {
                    device_id: winit::event::DeviceId::dummy(),
                    state,
                    button: MouseButton::Left,
                },
            );
        }
    }
    fn advance(&mut self, events: &ActiveEventLoop) {
        let Some(r) = self.app.runtime.as_ref() else {
            return;
        };
        if r.native_edit.pending()
            || r.renderer.render_session().has_pending_frame()
            || !r.surface_transaction.has_presented()
            || self.since.elapsed() < Duration::from_millis(30)
        {
            return;
        }
        assert!(!r.native_edit.blocked, "writer reconciliation must qualify");
        if self.stage >= self.steps.len() {
            events.exit();
            return;
        }
        // Scroll/focus through the same production keyboard path before
        // clicking a control which is outside the currently presented clip.
        let pointer = match &self.steps[self.stage] {
            Step::Control(c) => Some(c.clone()),
            Step::Kind(i) => Some(C::Kind(*i)),
            Step::First => Some(C::Result(r.workspace().library.results()[0])),
            Step::Search(_) => Some(C::Search),
            Step::Select(name) => r
                .workspace()
                .library
                .results()
                .into_iter()
                .find(|i| r.workspace().library.entries[*i].name == *name)
                .map(C::Result),
            _ => None,
        };
        if let Some(control) = pointer {
            if !r
                .presented_hits
                .regions()
                .iter()
                .any(|h| h.target == HitTarget::Library(control.clone()))
            {
                for _ in 0..100 {
                    if self.app.runtime.as_ref().unwrap().workspace().library.focus
                        == Some(control.clone())
                    {
                        break;
                    }
                    self.key(Key::Named(NamedKey::Tab));
                }
                assert_eq!(
                    self.app.runtime.as_ref().unwrap().workspace().library.focus,
                    Some(control)
                );
                self.app.request_workspace_redraw();
                return;
            }
        }
        eprintln!(
            "library ordinary stage{}: status={:?} draft_refusal={:?}",
            self.stage,
            r.workspace().last_command_status,
            r.workspace()
                .library
                .draft
                .as_ref()
                .and_then(|d| d.refusal.as_ref())
        );
        let step = std::mem::replace(&mut self.steps[self.stage], Step::CheckFrozen);
        match step {
            Step::MissingGeometry => {
                let s = self.app.runtime.as_ref().unwrap().workspace();
                let e = s.library.selected(&s.selection).unwrap();
                assert!(
                    e.raw
                        .get("pads")
                        .and_then(|p| p.as_object())
                        .is_some_and(|p| p.is_empty())
                );
                assert_eq!(
                    s.library
                        .process_inspection(&e.kind, &e.raw, 0)
                        .unwrap_err(),
                    "Named pad unavailable"
                );
                assert_eq!(
                    s.ui.console.latest().unwrap().message,
                    "Named pad unavailable"
                );
                assert_eq!(source_bytes(&self.root), self.snapshot);
            }
            Step::Focus(target) => {
                for _ in 0..100 {
                    if self.app.runtime.as_ref().unwrap().workspace().library.focus
                        == Some(target.clone())
                    {
                        break;
                    }
                    self.key(Key::Named(NamedKey::Tab));
                }
                assert_eq!(
                    self.app.runtime.as_ref().unwrap().workspace().library.focus,
                    Some(target)
                );
            }
            Step::RecordFootprint => self.record_footprint(),
            Step::CheckFootprintValue(edited) => self.check_footprint_value(edited),
            Step::RecordPlaced(stack) => self.record_placed(stack),
            Step::CheckPlaced(stack) => self.check_placed(stack),
            Step::CheckProcess(index) => self.check_process(index),
            Step::CheckRemovedPreview => self.check_removed_preview(),
            Step::KeyControl(target) => {
                for _ in 0..80 {
                    if self.app.runtime.as_ref().unwrap().workspace().library.focus
                        == Some(target.clone())
                    {
                        break;
                    }
                    self.key(Key::Named(NamedKey::Tab));
                }
                assert_eq!(
                    self.app.runtime.as_ref().unwrap().workspace().library.focus,
                    Some(target)
                );
                self.key(Key::Named(NamedKey::Enter));
            }
            Step::Relationship => {
                let s = self.app.runtime.as_ref().unwrap().workspace();
                let selected = s.library.selected(&s.selection).unwrap();
                let (relationship, link) = s
                    .library
                    .links(selected)
                    .into_iter()
                    .enumerate()
                    .find(|(_, l)| l.targets.len() == 1)
                    .expect("named relationship");
                let target = link.targets[0];
                for _ in 0..80 {
                    if self.app.runtime.as_ref().unwrap().workspace().library.focus
                        == Some(C::Relationship(relationship))
                    {
                        break;
                    }
                    self.key(Key::Named(NamedKey::Tab));
                }
                assert_eq!(
                    self.app.runtime.as_ref().unwrap().workspace().library.focus,
                    Some(C::Relationship(relationship))
                );
                self.key(Key::Named(NamedKey::Enter));
                assert_eq!(
                    self.app
                        .runtime
                        .as_ref()
                        .unwrap()
                        .workspace()
                        .library
                        .selected(&self.app.runtime.as_ref().unwrap().workspace().selection)
                        .unwrap()
                        .key(),
                    self.app
                        .runtime
                        .as_ref()
                        .unwrap()
                        .workspace()
                        .library
                        .entries[target]
                        .key()
                );
            }
            Step::RefusalNotice => {
                let s = self.app.runtime.as_ref().unwrap().workspace();
                let reason = s.library.draft.as_ref().unwrap().refusal.as_ref().unwrap();
                assert_eq!(&s.ui.console.latest().unwrap().message, reason);
                assert_eq!(source_bytes(&self.root), self.snapshot);
            }
            Step::Refusal => {
                let d = self
                    .app
                    .runtime
                    .as_ref()
                    .unwrap()
                    .workspace()
                    .library
                    .draft
                    .as_ref()
                    .unwrap();
                assert!(d.preview.is_none());
                assert!(d.refusal.as_ref().is_some_and(|s| !s.is_empty()));
                assert_eq!(
                    source_bytes(&self.root),
                    self.snapshot,
                    "displayed refusal cannot write"
                );
            }
            Step::Reopen => {
                let args = <GuiArgs as clap::Parser>::try_parse_from([
                    "datum-gui",
                    "--project-root",
                    self.root.to_str().unwrap(),
                ])
                .unwrap();
                self.app = App::new(args, self.proxy.clone());
                self.app.resumed(events);
            }
            Step::Tile => {
                self.app
                    .runtime
                    .as_mut()
                    .unwrap()
                    .pane_split_focused(datum_gui_protocol::SplitOrientation::Horizontal);
                self.app.request_workspace_redraw();
            }
            Step::Open => {
                assert!(self.app.runtime.as_mut().unwrap().activate_menu_item(
                    "View",
                    "Library Browser",
                    EntrySurface::MenuPointer
                ));
                self.app.request_workspace_redraw();
            }
            Step::Kind(i) => self.control(events, HitTarget::Library(C::Kind(i))),
            Step::First => {
                let i = self
                    .app
                    .runtime
                    .as_ref()
                    .unwrap()
                    .workspace()
                    .library
                    .results()[0];
                self.control(events, HitTarget::Library(C::Result(i)));
            }
            Step::Search(name) => {
                self.control(events, HitTarget::Library(C::Search));
                let count = self
                    .app
                    .runtime
                    .as_ref()
                    .unwrap()
                    .workspace()
                    .library
                    .search
                    .chars()
                    .count();
                for _ in 0..count {
                    self.key(Key::Named(NamedKey::Backspace));
                }
                self.key(Key::Character(name.into()));
            }
            Step::Select(name) => {
                let b = &self.app.runtime.as_ref().unwrap().workspace().library;
                let i = b
                    .results()
                    .into_iter()
                    .find(|i| b.entries[*i].name == name)
                    .expect("named result exists");
                self.control(events, HitTarget::Library(C::Result(i)));
            }
            Step::Control(c) => self.control(events, HitTarget::Library(c)),
            Step::Field(key, value) => {
                self.focus_field(key);
                let n = self
                    .app
                    .runtime
                    .as_ref()
                    .unwrap()
                    .workspace()
                    .library
                    .draft
                    .as_ref()
                    .unwrap()
                    .value(key)
                    .chars()
                    .count();
                for _ in 0..n {
                    self.key(Key::Named(NamedKey::Backspace));
                }
                self.key(Key::Character(value.into()));
            }
            Step::Choice(key, label) => {
                self.focus_field(key);
                let count = self
                    .app
                    .runtime
                    .as_ref()
                    .unwrap()
                    .workspace()
                    .library
                    .draft
                    .as_ref()
                    .unwrap()
                    .fields
                    .iter()
                    .find(|f| f.key == key)
                    .unwrap()
                    .choices
                    .len();
                let mut found = false;
                for _ in 0..=count {
                    let d = self
                        .app
                        .runtime
                        .as_ref()
                        .unwrap()
                        .workspace()
                        .library
                        .draft
                        .as_ref()
                        .unwrap();
                    let f = d.fields.iter().find(|f| f.key == key).unwrap();
                    if f.choices
                        .iter()
                        .any(|c| c.0 == f.text && c.1.starts_with(&label))
                    {
                        found = true;
                        break;
                    }
                    self.key(Key::Named(NamedKey::ArrowRight));
                }
                assert!(found, "named choice {label} is available");
            }
            Step::Capture(label) => {
                if label.ends_with("-preview") {
                    let d = self
                        .app
                        .runtime
                        .as_ref()
                        .unwrap()
                        .workspace()
                        .library
                        .draft
                        .as_ref()
                        .unwrap();
                    assert!(d.preview.is_some());
                    assert!(d.refusal.is_none());
                }
                self.app
                    .runtime
                    .as_mut()
                    .unwrap()
                    .write_visual_screenshot(&self.output.join(format!("{label}.png")))
                    .unwrap();
            }
            Step::Snapshot => self.snapshot = source_bytes(&self.root),
            Step::Invariant => assert_eq!(
                source_bytes(&self.root),
                self.snapshot,
                "preview/cancel must retain all sources and journal"
            ),
            Step::History(redo) => {
                assert!(self.app.runtime.as_mut().unwrap().activate_menu_item(
                    "Edit",
                    if redo { "Redo" } else { "Undo" },
                    EntrySurface::MenuKeyboard
                ));
                self.app.request_workspace_redraw();
            }
            Step::Save => {
                assert!(self.app.runtime.as_mut().unwrap().activate_menu_item(
                    "File",
                    "Save",
                    EntrySurface::MenuKeyboard
                ));
                self.app.request_workspace_redraw();
            }
            Step::CheckFootprint(count) => {
                let b = &self.app.runtime.as_ref().unwrap().workspace().library;
                let e = b
                    .entries
                    .iter()
                    .find(|e| e.kind == "footprints" && e.name == "GUI two pad resistor")
                    .unwrap();
                assert_eq!(e.raw["pads"].as_object().unwrap().len(), count);
            }
            Step::CheckFrozen => assert_eq!(
                std::fs::read(self.root.join("board/board.json")).unwrap(),
                self.board,
                "complete existing placed-user payload and PadIDs remain frozen"
            ),
            Step::Narrow => {
                let r = self.app.runtime.as_mut().unwrap();
                let width = r
                    .presented_hits
                    .regions()
                    .iter()
                    .find(|h| matches!(h.target, HitTarget::Library(C::Scroll { .. })))
                    .unwrap()
                    .rect
                    .width;
                self.normal_pane_width = Some(width);
                r.pane_split_focused(datum_gui_protocol::SplitOrientation::Vertical);
                self.app.request_workspace_redraw();
            }
            Step::CheckNarrow => {
                let r = self.app.runtime.as_ref().unwrap();
                let region = r
                    .presented_hits
                    .regions()
                    .iter()
                    .find(|h| matches!(h.target, HitTarget::Library(C::Scroll { .. })))
                    .unwrap();
                assert!(
                    region.rect.width < self.normal_pane_width.unwrap(),
                    "actual presented Library pane must become narrower"
                );
                eprintln!(
                    "ordinary narrow Library pane: before={} accepted_width={} accepted_height={}",
                    self.normal_pane_width.unwrap(),
                    region.rect.width,
                    region.rect.height
                );
            }
            Step::Resize => {
                let _ = self
                    .app
                    .window
                    .as_ref()
                    .unwrap()
                    .request_inner_size(winit::dpi::PhysicalSize::new(960, 640));
                self.app.request_workspace_redraw();
            }
            Step::VerifyReopen => {
                let model = eda_engine::substrate::ProjectResolver::new(&self.root)
                    .resolve()
                    .unwrap();
                let b = datum_gui_protocol::library_browser::LibraryBrowser::capture(
                    std::sync::Arc::new(model),
                );
                let e = b
                    .entries
                    .iter()
                    .find(|e| e.name == "GUI two pad resistor")
                    .unwrap();
                assert_eq!(e.raw["pads"].as_object().unwrap().len(), 2);
                assert!(b.entries.iter().any(|e| e.name == "GUI resistor copper"));
                assert_eq!(
                    std::fs::read(self.root.join("board/board.json")).unwrap(),
                    self.board
                );
            }
        }
        self.stage += 1;
        self.since = Instant::now();
        self.app.request_workspace_redraw();
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
        if self.since.elapsed() > Duration::from_secs(40) {
            self.failure = Some(format!("stage{} timed out", self.stage));
            events.exit();
            return;
        }
        self.advance(events);
        let bound = self.since + Duration::from_secs(40);
        let due = if self.app.runtime.as_ref().is_some_and(|r| {
            !r.native_edit.pending()
                && !r.renderer.render_session().has_pending_frame()
                && r.surface_transaction.has_presented()
        }) {
            (self.since + Duration::from_millis(30)).min(bound)
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
fn source_bytes(root: &Path) -> BTreeMap<String, Vec<u8>> {
    fn visit(root: &Path, dir: &Path, out: &mut BTreeMap<String, Vec<u8>>) {
        for entry in std::fs::read_dir(dir).unwrap() {
            let p = entry.unwrap().path();
            if p.is_dir() {
                visit(root, &p, out);
            } else if p.extension().is_some_and(|s| s == "json" || s == "jsonl") {
                out.insert(
                    p.strip_prefix(root).unwrap().to_string_lossy().into_owned(),
                    std::fs::read(p).unwrap(),
                );
            }
        }
    }
    let mut out = BTreeMap::new();
    visit(root, root, &mut out);
    out
}
