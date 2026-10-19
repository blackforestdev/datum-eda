//! Shared New/Open browser coordination. Browsing has no Project writer.
use super::*;
use datum_gui_protocol::{
    FolderBrowserFocus as Focus, FolderBrowserMode as Mode, FolderBrowserState,
};

#[path = "folder_browser_actions.rs"]
mod actions;
#[path = "folder_browser_discovery.rs"]
mod discovery;
#[path = "dialog_text_editor.rs"]
mod editor;
#[path = "folder_browser_input.rs"]
mod input;
#[path = "folder_browser_path_assistance.rs"]
mod path_assistance;
#[path = "folder_browser_qualification.rs"]
mod qualification;
#[path = "folder_browser_scan.rs"]
mod scan;

pub(super) struct FolderBrowser {
    pub state: Option<FolderBrowserState>,
    scanner: Option<scan::Scanner>,
    pending: Option<u64>,
    last_refusal: Option<String>,
    last_announced_destination: Option<String>,
    last_suggestion_announcement: Option<String>,
    last_click: Option<(usize, std::time::Instant)>,
    wake: winit::event_loop::EventLoopProxy<()>,
    host_service: Option<crate::host_filesystem::Service>,
    host_pending: Option<actions::PendingAction>,
    scope: u64,
    select_after_scan: Option<(String, Option<crate::host_filesystem::Identity>)>,
    editor_drag: Option<editor::EditorDrag>,
    dismiss_release: bool,
    inspector: Option<crate::host_filesystem::Inspector>,
    qualification: Option<qualification::Qualification>,
}

/// Local native-browser hit authority. Row indices alone do not identify what
/// was shown when an asynchronous scan replaces the current listing.
#[derive(Clone, PartialEq)]
pub(crate) struct BrowserPresentation {
    scope: u64,
    mode: Mode,
    path: String,
    directory: String,
    parent: Option<(u64, u64)>,
    entries: Vec<datum_gui_protocol::BrowserEntry>,
    expanded: bool,
    ready: bool,
    suggestions: Option<datum_gui_protocol::FolderSuggestions>,
    menu: Option<(
        Vec<datum_gui_protocol::popup_menu::PopupItem<datum_gui_protocol::BrowserAction>>,
        Option<datum_gui_protocol::BrowserEntry>,
    )>,
    naming: Option<datum_gui_protocol::NamingDraft>,
}

impl FolderBrowser {
    pub fn new(wake: winit::event_loop::EventLoopProxy<()>) -> Self {
        Self {
            state: None,
            scanner: None,
            pending: None,
            last_click: None,
            last_refusal: None,
            last_announced_destination: None,
            last_suggestion_announcement: None,
            wake,
            host_service: None,
            host_pending: None,
            scope: 0,
            select_after_scan: None,
            editor_drag: None,
            dismiss_release: false,
            inspector: None,
            qualification: None,
        }
    }

    pub fn cancel(&mut self) {
        self.scope = self.scope.wrapping_add(1);
        self.editor_drag = None;
        self.qualification = None;
        self.select_after_scan = None;
        if let Some(scanner) = &self.scanner {
            scanner.cancel();
        }
        self.pending = None;
        self.last_click = None;
        self.last_refusal = None;
        self.last_announced_destination = None;
        self.last_suggestion_announcement = None;
        self.state = None;
    }
}

impl Runtime {
    pub(crate) fn folder_browser_presentation(&self) -> Option<BrowserPresentation> {
        let state = self.folder_browser.state.as_ref()?;
        Some(BrowserPresentation {
            scope: self.folder_browser.scope,
            mode: state.mode,
            path: state.path_input.clone(),
            directory: state.directory.clone(),
            parent: state.parent_identity,
            entries: state.entries.clone(),
            expanded: state.expanded,
            ready: state.listing_ready(),
            suggestions: state.suggestions.clone(),
            menu: state
                .menu
                .as_ref()
                .map(|m| (m.items.clone(), state.context_entry.clone())),
            naming: state.naming.clone(),
        })
    }
    pub(super) fn folder_navigation_focus(&self) -> Option<Focus> {
        use datum_gui_protocol::{NewProjectFocus, ProjectOpenFocus};
        let ui = &self.session.workspace().ui;
        if ui.new_project.open {
            if let NewProjectFocus::Navigation(focus) = ui.new_project.focus {
                return Some(focus);
            }
        }
        if ui.project_open.open {
            if let ProjectOpenFocus::Navigation(focus) = ui.project_open.focus {
                return Some(focus);
            }
        }
        None
    }
    pub(super) fn set_folder_navigation_focus(&mut self, focus: Focus) {
        if self.folder_navigation_focus() != Some(focus) {
            self.folder_browser.last_click = None;
        }
        let ui = &mut self.session.workspace_mut().ui;
        if ui.new_project.open {
            ui.new_project.focus = datum_gui_protocol::NewProjectFocus::Navigation(focus);
        } else if ui.project_open.open && !ui.project_open.busy {
            ui.project_open.focus = datum_gui_protocol::ProjectOpenFocus::Navigation(focus);
        }
    }
    pub(super) fn refresh_folder_browser(&mut self) {
        self.refresh_host_protection();
        let pending = self
            .folder_browser
            .host_service
            .as_ref()
            .is_some_and(|service| service.busy());
        self.session
            .workspace_mut()
            .ui
            .new_project
            .host_action_pending = pending;
        self.session
            .workspace_mut()
            .ui
            .project_open
            .host_action_pending = pending;
        let Some(state) = &self.folder_browser.state else {
            self.refresh_dialog_state();
            return;
        };
        let selected_suggestion = state
            .suggestions
            .as_ref()
            .and_then(|_| state.selected)
            .and_then(|i| {
                state.entries.get(i).map(|entry| {
                    format!(
                        "{}, folder suggestion, {} of {}, selected",
                        entry.name,
                        i + 1,
                        state.entries.len()
                    )
                })
            });
        let announcement = state
            .suggestion_header()
            .map(|header| match &selected_suggestion {
                Some(selection) => format!("{header}\n{selection}"),
                None => header,
            });
        if announcement != self.folder_browser.last_suggestion_announcement {
            if let Some(text) = selected_suggestion.as_ref().or(announcement.as_ref()) {
                self.terminal_accessibility.announce_console(
                    crate::console_accessibility::AccessibilityAnnouncement {
                        text: text.clone(),
                        priority: crate::console_accessibility::AnnouncementPriority::Medium,
                    },
                );
            }
            self.folder_browser.last_suggestion_announcement = announcement;
        }
        let ui = &mut self.session.workspace_mut().ui;
        sync_location(ui, state);
        let leaf = (state.mode == Mode::New).then(|| ui.new_project.effective_folder_name());
        let refusal = state.refusal(leaf);
        if refusal != self.folder_browser.last_refusal {
            if let Some(message) = &refusal {
                self.terminal_accessibility.announce_console(
                    crate::console_accessibility::AccessibilityAnnouncement {
                        text: message.clone(),
                        priority: crate::console_accessibility::AnnouncementPriority::High,
                    },
                );
            }
            self.folder_browser.last_refusal = refusal;
        }
        let focus_new = ui.new_project.focus;
        let focus_open = ui.project_open.focus;
        let editing = matches!(
            focus_new,
            datum_gui_protocol::NewProjectFocus::ProjectName
                | datum_gui_protocol::NewProjectFocus::FolderName
                | datum_gui_protocol::NewProjectFocus::Navigation(Focus::Path)
        ) && ui.new_project.open
            || focus_open == datum_gui_protocol::ProjectOpenFocus::Navigation(Focus::Path)
                && ui.project_open.open;
        let candidate = state.destination(leaf);
        if !editing && candidate != self.folder_browser.last_announced_destination {
            if let Some(path) = &candidate {
                self.terminal_accessibility.announce_console(
                    crate::console_accessibility::AccessibilityAnnouncement {
                        text: format!(
                            "{}: {path}",
                            if state.mode == Mode::New {
                                "New Project folder"
                            } else {
                                "Selected"
                            }
                        ),
                        priority: crate::console_accessibility::AnnouncementPriority::Medium,
                    },
                );
            }
            self.folder_browser.last_announced_destination = candidate;
        }
        self.refresh_dialog_state();
    }
    pub(super) fn init_project_navigation(&mut self, mode: Mode, input: &str) {
        self.folder_browser.cancel();
        let directory = initial_location(input);
        let text = directory
            .as_ref()
            .ok()
            .and_then(|p| p.to_str())
            .unwrap_or(input)
            .to_owned();
        let mut state = FolderBrowserState::new(mode, text);
        state.expanded = true;
        self.folder_browser.state = Some(state);
        match directory {
            Ok(path) => self.read_folder_navigation(path, false),
            Err(error) => {
                self.folder_browser.state.as_mut().unwrap().error = Some(error);
                self.refresh_folder_browser();
            }
        }
    }
    fn start_folder_navigation(&mut self, directory: PathBuf) {
        self.read_folder_navigation(directory, true);
    }
    pub(super) fn poll_folder_browser(&mut self) -> bool {
        self.refresh_host_protection();
        let host_changed = self.poll_host_actions() | self.poll_folder_qualification();
        if self
            .folder_browser
            .state
            .as_ref()
            .is_some_and(|s| match s.mode {
                Mode::New => !self.workspace().ui.new_project.open,
                Mode::Open => !self.workspace().ui.project_open.open,
            })
        {
            self.folder_browser.cancel();
        }
        if self.folder_browser.poll() {
            if let Some((leaf, identity)) = self.folder_browser.select_after_scan.take()
                && let Some(state) = &mut self.folder_browser.state
                && state.ready()
            {
                state.selected = state.entries.iter().position(|entry| {
                    entry.name == leaf
                        && identity.is_some_and(|id| entry.identity == Some((id.device, id.inode)))
                });
            }
            self.refresh_folder_browser();
            true
        } else {
            host_changed
        }
    }
}

/// The cached form destination is a projection of the one displayed navigator,
/// not an independently editable path. Engine admission remains authoritative.
pub(crate) fn sync_location(
    ui: &mut datum_gui_protocol::WorkspaceUiState,
    state: &FolderBrowserState,
) {
    match state.mode {
        Mode::New if ui.new_project.open => {
            if ui.new_project.use_project_name {
                ui.new_project.folder_name = ui.new_project.project_name.clone();
            }
            let leaf = ui.new_project.effective_folder_name();
            let destination = state
                .preview(Some(leaf))
                .and_then(|s| entered_path(&s).ok())
                .and_then(|p| p.to_str().map(str::to_owned))
                .unwrap_or_default();
            let ready = state
                .destination(Some(leaf))
                .and_then(|s| entered_path(&s).ok())
                .is_some();
            if ui.new_project.destination != destination || ui.new_project.location_ready != ready {
                ui.new_project.clear_creation_refusal();
            }
            ui.new_project.destination = destination;
            ui.new_project.location_ready = ready;
        }
        Mode::Open if ui.project_open.open && !ui.project_open.busy => {
            let destination = state
                .destination(None)
                .and_then(|s| entered_path(&s).ok())
                .and_then(|p| p.to_str().map(str::to_owned))
                .unwrap_or_default();
            ui.project_open.directory = destination;
        }
        _ => {}
    }
}

fn entered_path(input: &str) -> Result<PathBuf, String> {
    if input.is_empty() || input.contains('\0') {
        return Err("Enter a valid Location, or use Browse to choose one.".into());
    }
    let path = if input == "~" || input.starts_with("~/") {
        let home = std::env::var_os("HOME")
            .filter(|v| Path::new(v).is_absolute())
            .ok_or("Home folder is unavailable. Enter an absolute folder path.")?;
        PathBuf::from(home).join(
            input
                .strip_prefix("~/")
                .unwrap_or("")
                .trim_start_matches('/'),
        )
    } else {
        PathBuf::from(input)
    };
    if path.is_absolute() {
        Ok(path)
    } else {
        Ok(std::env::current_dir()
            .map_err(|e| e.to_string())?
            .join(path))
    }
}

fn initial_location(input: &str) -> Result<PathBuf, String> {
    if !input.is_empty() {
        return entered_path(input);
    }
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .filter(|p| p.is_absolute())
        .ok_or_else(|| "Home Location is unavailable. Enter an absolute Location.".into())
}

#[cfg(test)]
#[path = "folder_browser_action_tests.rs"]
mod action_tests;
#[cfg(test)]
#[path = "folder_browser_tests.rs"]
mod tests;
