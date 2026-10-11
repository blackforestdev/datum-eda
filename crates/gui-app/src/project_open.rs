//! Transactional native Project replacement. Preparation and source
//! revalidation run off the event thread; publication is origin-qualified.
use super::*;
use crate::terminal_session::{PreparedProjectTerminal, ProjectTerminalReservation};
use datum_gui_protocol::{PreparedProjectOpen, ProjectOpenFocus, ProjectOpenOrigin};
use std::sync::mpsc;

#[path = "project_open/prepare.rs"]
mod prepare;
#[path = "project_open/publication.rs"]
mod publication;

struct Answer {
    origin: ProjectOpenOrigin,
    ticket: u64,
    prepared: PreparedProjectOpen,
    terminal: PreparedProjectTerminal,
    context: TerminalLaunchContext,
    _admission: eda_engine::revision::ProjectWriteLease,
}

pub(super) struct OpenWork {
    generation: u64,
    pub(super) session_generation: u64,
    pending: Option<mpsc::Receiver<Result<Answer, String>>>,
    wake: winit::event_loop::EventLoopProxy<()>,
}

impl OpenWork {
    pub(super) fn new(wake: winit::event_loop::EventLoopProxy<()>) -> Self {
        Self {
            generation: 0,
            session_generation: 0,
            pending: None,
            wake,
        }
    }
    fn cancel(&mut self) {
        self.generation = self.generation.wrapping_add(1);
        self.pending = None;
    }
}

impl Runtime {
    pub(super) fn open_project(&mut self) -> bool {
        if self.workspace().ui.global_preferences.open {
            self.close_global_preferences();
        }
        if self.workspace().ui.project_preferences.open {
            self.close_project_preferences();
        }
        self.finish_terminal_split_drag();
        self.finish_dock_resize_drag();
        self.cancel_terminal_tab_drag();
        self.cancel_terminal_text_selection_drag();
        self.divider_drag = None;
        self.terminal_mouse_button = None;
        self.cancel_native_selection_gesture();
        let ui = &mut self.session.workspace_mut().ui;
        ui.active_menu = None;
        ui.active_submenu = None;
        ui.new_project.open = false;
        ui.project_open = Default::default();
        ui.project_open.open = true;
        ui.focus = ApplicationFocus::Overlay;
        self.invalidate_frame();
        true
    }

    pub(super) fn close_project_open(&mut self) -> bool {
        self.project_open.cancel();
        let ui = &mut self.session.workspace_mut().ui;
        ui.project_open.open = false;
        ui.project_open.busy = false;
        ui.focus = ApplicationFocus::Editor(ui.layout.focused);
        self.invalidate_frame();
        true
    }

    pub(super) fn handle_project_open_key(&mut self, event: &winit::event::KeyEvent) -> bool {
        use winit::keyboard::{Key, NamedKey};
        if !self.workspace().ui.project_open.open {
            return false;
        }
        if event.state != winit::event::ElementState::Pressed {
            return true;
        }
        match &event.logical_key {
            Key::Named(NamedKey::Escape) => {
                // The owned host can retire before key-up reaches Main. Keep
                // that release in the shared retiring-overlay key cycle.
                self.menu_keys
                    .consume_escape_release(&event.logical_key, event.state, true);
                self.close_project_open();
            }
            Key::Named(NamedKey::Tab) => {
                let reverse = self.modifiers.shift_key();
                self.session
                    .workspace_mut()
                    .ui
                    .project_open
                    .move_focus(reverse);
            }
            Key::Named(NamedKey::Enter) => {
                if self.workspace().ui.project_open.focus == ProjectOpenFocus::Cancel {
                    self.close_project_open();
                } else {
                    self.begin_project_open();
                }
            }
            Key::Named(NamedKey::Backspace)
                if !self.workspace().ui.project_open.busy
                    && self.workspace().ui.project_open.focus == ProjectOpenFocus::Directory =>
            {
                self.session.workspace_mut().ui.project_open.directory.pop();
            }
            key if !self.workspace().ui.project_open.busy
                && self.workspace().ui.project_open.focus == ProjectOpenFocus::Directory =>
            {
                if let Some(value) =
                    crate::global_preferences_window::dialog_text_input(key, self.modifiers)
                {
                    self.session
                        .workspace_mut()
                        .ui
                        .project_open
                        .directory
                        .push_str(value);
                }
            }
            _ => {}
        }
        self.invalidate_frame();
        true
    }
}
