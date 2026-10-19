//! Shared New/Open context actions; host operations never publish a Project.
use super::*;
use crate::host_filesystem::{self as host, Command, EntryKind, Identity, Outcome, Request};
use datum_gui_protocol::popup_menu::{PopupItem, PopupMenu};
use datum_gui_protocol::{BrowserAction, BrowserEntryKind, NamingDraft, NamingPurpose};

pub(super) struct PendingAction {
    ticket: u64,
    scope: u64,
    parent: String,
    parent_identity: (u64, u64),
    create: bool,
}
impl Runtime {
    pub(crate) fn folder_host_actions_pending(&self) -> bool {
        self.folder_browser
            .host_service
            .as_ref()
            .is_some_and(|service| service.busy())
    }
    pub(super) fn refresh_host_protection(&mut self) {
        if self.folder_browser.host_service.is_some() {
            let protection = self
                .terminal_sessions
                .host_path_protection(self.workspace());
            self.folder_browser
                .host_service
                .as_mut()
                .unwrap()
                .set_protection(protection);
        }
    }
    pub(super) fn host_action_unavailable(&self) -> Option<String> {
        if self.terminal_sessions.project_preparation_pending() {
            Some("Wait for Project preparation and terminal cleanup before using host folder actions.".into())
        } else if self.folder_host_actions_pending() {
            Some("Wait for the submitted folder action to finish.".into())
        } else {
            None
        }
    }
    pub(crate) fn open_folder_actions_menu(
        &mut self,
        index: Option<usize>,
        anchor: (f32, f32),
    ) -> bool {
        self.folder_browser.last_click = None;
        self.folder_browser.editor_drag = None;
        self.folder_browser.qualification = None;
        let unavailable = self.host_action_unavailable();
        let Some(state) = self.folder_browser.state.as_mut().filter(|s| s.expanded) else {
            return false;
        };
        if state.naming.is_some() {
            return true;
        }
        if self.workspace().ui.project_open.busy {
            return true;
        }
        let state = self.folder_browser.state.as_mut().unwrap();
        let entry = index.and_then(|i| state.entries.get(i)).cloned();
        state.selected = index.filter(|i| *i < state.entries.len());
        let read_reason = (!state.ready() || state.parent_identity.is_none())
            .then(|| "Browse a readable current Location before using folder actions.".to_owned());
        let common = unavailable.or(read_reason);
        let rename_reason = common.clone().or_else(|| {
            (!entry.as_ref().is_some_and(|e| e.renameable())).then(|| {
                "Select a regular folder or file to rename it; link entries are not renamed.".into()
            })
        });
        state.context_entry = entry;
        state.menu = Some(PopupMenu::new(
            anchor,
            vec![
                PopupItem {
                    label: "New Folder".into(),
                    shortcut: Some("Ctrl+Shift+N".into()),
                    action: BrowserAction::NewFolder,
                    unavailable: common,
                },
                PopupItem {
                    label: "Rename…".into(),
                    shortcut: Some("F2".into()),
                    action: BrowserAction::Rename,
                    unavailable: rename_reason,
                },
            ],
        ));
        let create_reason = self
            .host_action_unavailable()
            .or_else(|| self.folder_qualification_reason(BrowserAction::NewFolder));
        let rename_reason = self
            .host_action_unavailable()
            .or_else(|| self.folder_qualification_reason(BrowserAction::Rename));
        let menu = self
            .folder_browser
            .state
            .as_mut()
            .unwrap()
            .menu
            .as_mut()
            .unwrap();
        menu.items[0].unavailable = create_reason;
        menu.items[1].unavailable = rename_reason;
        menu.edge(false);
        self.clear_folder_qualification_intent();
        self.set_folder_navigation_focus(Focus::Folders);
        self.refresh_folder_browser();
        true
    }
    pub(super) fn start_folder_naming(&mut self, action: BrowserAction) -> bool {
        self.folder_browser.last_click = None;
        if let Some(reason) = self.host_action_unavailable() {
            self.folder_input_refusal(reason);
            return true;
        }
        let Some(state) = self.folder_browser.state.as_ref() else {
            return false;
        };
        if state.menu.is_none() {
            let index = state.selected;
            self.open_folder_actions_menu(index, (180.0, 240.0));
            self.defer_folder_naming(action);
            return true;
        }
        if let Some(reason) = self.folder_qualification_reason(action) {
            self.defer_folder_naming(action);
            self.folder_input_refusal(reason);
            self.refresh_folder_browser();
            return true;
        }
        let scope = self.folder_browser.scope;
        let Some(state) = self.folder_browser.state.as_mut() else {
            return false;
        };
        let popup_entry = state.menu.take().and_then(|_| state.context_entry.take());
        state.context_entry = None;
        if !state.expanded || !state.ready() || state.parent_identity.is_none() {
            return true;
        }
        let entry = if action == BrowserAction::Rename {
            let entry =
                popup_entry.or_else(|| state.selected.and_then(|i| state.entries.get(i)).cloned());
            let Some(entry) = entry.filter(|e| e.renameable()) else {
                return true;
            };
            Some(entry)
        } else {
            None
        };
        state.naming = Some(NamingDraft::new(
            state.path_input.clone(),
            state.parent_identity.unwrap(),
            scope,
            entry,
        ));
        state.input_refusal = None;
        self.set_folder_navigation_focus(Focus::NameEditor);
        self.refresh_folder_browser();
        true
    }
    pub(super) fn cancel_folder_naming(&mut self) -> bool {
        let Some(state) = &mut self.folder_browser.state else {
            return false;
        };
        if state.naming.as_ref().is_some_and(|d| d.busy) {
            return true;
        }
        state.naming = None;
        self.set_folder_navigation_focus(Focus::Folders);
        self.refresh_folder_browser();
        true
    }
    pub(super) fn submit_folder_naming(&mut self) -> bool {
        if let Some(reason) = self.host_action_unavailable() {
            self.folder_input_refusal(reason);
            return true;
        }
        let Some(draft) = self
            .folder_browser
            .state
            .as_ref()
            .and_then(|s| s.naming.as_ref())
            .cloned()
        else {
            return false;
        };
        if !draft.enabled() {
            return true;
        }
        if self.folder_browser.host_service.is_none() {
            let wake = self.folder_browser.wake.clone();
            match host::Service::new(move || {
                let _ = wake.send_event(());
            }) {
                Ok(service) => self.folder_browser.host_service = Some(service),
                Err(error) => {
                    self.folder_input_refusal(format!("Folder action worker unavailable: {error}"));
                    return true;
                }
            }
        }
        self.refresh_host_protection();
        let command = match &draft.purpose {
            NamingPurpose::CreateDirectory => Command::CreateDirectory {
                leaf: draft.value.clone(),
            },
            NamingPurpose::Rename(entry) => {
                let Some((device, inode)) = entry.identity else {
                    return true;
                };
                let kind = match entry.kind {
                    BrowserEntryKind::Directory => EntryKind::Directory,
                    BrowserEntryKind::File => EntryKind::File,
                    _ => return true,
                };
                Command::Rename {
                    old_leaf: entry.name.clone(),
                    new_leaf: draft.value.clone(),
                    source: Identity {
                        device,
                        inode,
                        kind,
                    },
                }
            }
        };
        let parent = match entered_path(&draft.parent) {
            Ok(parent) => parent,
            Err(error) => {
                self.folder_input_refusal(error);
                return true;
            }
        };
        let request = Request {
            scope: draft.scope,
            parent,
            parent_identity: Identity {
                device: draft.parent_identity.0,
                inode: draft.parent_identity.1,
                kind: EntryKind::Directory,
            },
            command,
        };
        match self
            .folder_browser
            .host_service
            .as_mut()
            .unwrap()
            .submit(request)
        {
            Ok(ticket) => {
                self.folder_browser.host_pending = Some(PendingAction {
                    ticket,
                    scope: draft.scope,
                    parent: draft.parent,
                    parent_identity: draft.parent_identity,
                    create: matches!(draft.purpose, NamingPurpose::CreateDirectory),
                });
                let state = self.folder_browser.state.as_mut().unwrap();
                let draft = state.naming.as_mut().unwrap();
                draft.busy = true;
                draft.refusal = None;
            }
            Err(error) => self.folder_input_refusal(error),
        }
        self.refresh_folder_browser();
        true
    }
    pub(super) fn poll_host_actions(&mut self) -> bool {
        let Some(completion) = self
            .folder_browser
            .host_service
            .as_mut()
            .and_then(host::Service::receive)
        else {
            return false;
        };
        let Some(pending) = self.folder_browser.host_pending.take() else {
            return true;
        };
        let current = self.folder_browser.scope == completion.scope
            && pending.scope == completion.scope
            && pending.ticket == completion.ticket;
        let mut rescan = None;
        let naming_focus = matches!(
            self.folder_navigation_focus(),
            Some(Focus::NameEditor | Focus::NameCancel | Focus::NameSubmit)
        );
        let retained_draft = current
            && self
                .folder_browser
                .state
                .as_ref()
                .is_some_and(|s| s.naming.is_some());
        let message = match completion.outcome {
            Outcome::Committed(change) | Outcome::Unchanged(change) => {
                let message = if change.old_leaf.as_deref() == Some(change.new_leaf.as_str()) {
                    format!("Name unchanged: {} in {}.", change.new_leaf, pending.parent)
                } else {
                    format!(
                        "{} {} in {}.{}",
                        if pending.create {
                            "Created folder"
                        } else {
                            "Renamed to"
                        },
                        change.new_leaf,
                        pending.parent,
                        change
                            .warning
                            .as_ref()
                            .map(|s| format!(" {s}"))
                            .unwrap_or_default()
                    )
                };
                if current && let Some(state) = &mut self.folder_browser.state {
                    state.naming = None;
                    state.action_notice = Some(message.clone());
                    if state.parent_identity == Some(pending.parent_identity) {
                        rescan = Some((state.path_input.clone(), change.new_leaf, change.identity));
                    }
                }
                message
            }
            Outcome::Refused(reason) => {
                let message = format!(
                    "{reason} Nothing was {}.{}",
                    if pending.create {
                        "created"
                    } else {
                        "renamed or replaced"
                    },
                    if retained_draft {
                        " Your naming draft is kept."
                    } else {
                        ""
                    }
                );
                if current
                    && let Some(draft) = self
                        .folder_browser
                        .state
                        .as_mut()
                        .and_then(|s| s.naming.as_mut())
                {
                    draft.busy = false;
                    draft.refusal = Some(message.clone());
                }
                message
            }
            Outcome::Indeterminate(reason) => {
                let message = format!(
                    "{reason} Inspect Location before retrying.{}",
                    if retained_draft {
                        " Your naming draft is kept."
                    } else {
                        ""
                    }
                );
                if current
                    && let Some(draft) = self
                        .folder_browser
                        .state
                        .as_mut()
                        .and_then(|s| s.naming.as_mut())
                {
                    draft.busy = false;
                    draft.refusal = Some(message.clone());
                }
                message
            }
        };
        self.log_console_echo(ConsoleFeedbackSource::Viewport, message.clone());
        self.terminal_accessibility.announce_console(
            crate::console_accessibility::AccessibilityAnnouncement {
                text: message,
                priority: crate::console_accessibility::AnnouncementPriority::High,
            },
        );
        if let Some((path, leaf, identity)) = rescan
            && let Ok(path) = entered_path(&path)
        {
            self.read_folder_navigation(path, false);
            self.folder_browser.select_after_scan = Some((leaf, identity));
            if naming_focus {
                self.set_folder_navigation_focus(Focus::Folders);
            }
        }
        self.refresh_folder_browser();
        true
    }
    pub(super) fn folder_input_refusal(&mut self, message: String) {
        if let Some(state) = &mut self.folder_browser.state {
            if let Some(draft) = &mut state.naming {
                draft.refusal = Some(message);
            } else {
                state.input_refusal = Some(message);
            }
        }
    }
}

impl Runtime {
    pub(super) fn activate_folder_action_hit(&mut self, target: &HitTarget) -> bool {
        match target {
            HitTarget::FolderBrowserMenuItem(index) => {
                let action = self
                    .folder_browser
                    .state
                    .as_ref()
                    .and_then(|s| s.menu.as_ref())
                    .and_then(|m| m.activate(*index));
                if let Some(action) = action {
                    self.start_folder_naming(action);
                }
                true
            }
            HitTarget::FolderBrowser(Focus::NameCancel) => {
                self.set_folder_navigation_focus(Focus::NameCancel);
                self.cancel_folder_naming()
            }
            HitTarget::FolderBrowser(Focus::NameSubmit) => {
                self.set_folder_navigation_focus(Focus::NameSubmit);
                self.submit_folder_naming()
            }
            _ => false,
        }
    }
    pub(crate) fn finish_folder_popup_dismissal(&mut self) -> bool {
        std::mem::take(&mut self.folder_browser.dismiss_release)
    }
    pub(crate) fn folder_popup_pointer(
        &mut self,
        target: Option<&HitTarget>,
        pressed: bool,
    ) -> bool {
        if !pressed && self.finish_folder_popup_dismissal() {
            return true;
        }
        if !self
            .folder_browser
            .state
            .as_ref()
            .is_some_and(|s| s.menu.is_some())
        {
            return false;
        }
        if let Some(HitTarget::FolderBrowserMenuItem(index)) = target {
            if !pressed {
                self.activate_folder_action_hit(&HitTarget::FolderBrowserMenuItem(*index));
            }
        } else if pressed {
            let state = self.folder_browser.state.as_mut().unwrap();
            state.menu = None;
            state.context_entry = None;
            self.folder_browser.qualification = None;
            self.folder_browser.dismiss_release = true;
            self.set_folder_navigation_focus(Focus::Folders);
            self.refresh_folder_browser();
        }
        true
    }
    pub(crate) fn hover_folder_popup(&mut self, target: Option<&HitTarget>) -> bool {
        let Some(menu) = self
            .folder_browser
            .state
            .as_mut()
            .and_then(|s| s.menu.as_mut())
        else {
            return false;
        };
        if let Some(HitTarget::FolderBrowserMenuItem(index)) = target
            && menu.activate(*index).is_some()
            && menu.focused != Some(*index)
        {
            menu.focused = Some(*index);
            self.refresh_folder_browser();
            return true;
        }
        false
    }
    pub(crate) fn handle_folder_actions_key(&mut self, event: &KeyEvent) -> bool {
        if event.state == ElementState::Pressed {
            self.folder_browser.last_click = None;
        }
        if event.state != ElementState::Pressed {
            return false;
        }
        let Some(state) = &self.folder_browser.state else {
            return false;
        };
        if state.menu.is_some() {
            match &event.logical_key {
                Key::Named(NamedKey::Escape | NamedKey::Tab) => {
                    let state = self.folder_browser.state.as_mut().unwrap();
                    state.menu = None;
                    state.context_entry = None;
                    self.folder_browser.qualification = None;
                    self.set_folder_navigation_focus(Focus::Folders);
                }
                Key::Named(NamedKey::ArrowUp | NamedKey::ArrowDown) => self
                    .folder_browser
                    .state
                    .as_mut()
                    .unwrap()
                    .menu
                    .as_mut()
                    .unwrap()
                    .step(event.logical_key == Key::Named(NamedKey::ArrowUp)),
                Key::Named(NamedKey::Home | NamedKey::End) => self
                    .folder_browser
                    .state
                    .as_mut()
                    .unwrap()
                    .menu
                    .as_mut()
                    .unwrap()
                    .edge(event.logical_key == Key::Named(NamedKey::End)),
                Key::Named(NamedKey::Enter | NamedKey::Space) => {
                    if let Some(action) = state.menu.as_ref().unwrap().activate_focused() {
                        self.start_folder_naming(action);
                    }
                }
                _ => {}
            }
            self.refresh_folder_browser();
            return true;
        }
        let focus = self.folder_navigation_focus();
        if let Some(draft) = &state.naming {
            if matches!(
                focus,
                Some(Focus::NameEditor | Focus::NameCancel | Focus::NameSubmit)
            ) {
                if draft.busy
                    && matches!(
                        event.logical_key,
                        Key::Named(NamedKey::Enter | NamedKey::Escape | NamedKey::Space)
                    )
                {
                    return true;
                }
                if event.logical_key == Key::Named(NamedKey::Enter)
                    || event.logical_key == Key::Named(NamedKey::Space)
                        && focus != Some(Focus::NameEditor)
                {
                    if focus == Some(Focus::NameCancel) {
                        self.cancel_folder_naming();
                    } else {
                        self.submit_folder_naming();
                    }
                    return true;
                }
            }
            if !draft.busy && event.logical_key == Key::Named(NamedKey::Escape) {
                return self.cancel_folder_naming();
            }
            if event.logical_key == Key::Named(NamedKey::Tab) {
                let reverse = self.modifiers.shift_key();
                let next = match (focus, reverse) {
                    (Some(Focus::Folders), false) => Some(Focus::NameEditor),
                    (Some(Focus::NameEditor), true) => Some(Focus::Folders),
                    (Some(Focus::NameEditor), false) => Some(if draft.busy {
                        Focus::Preview
                    } else {
                        Focus::NameCancel
                    }),
                    (Some(Focus::NameCancel), true) => Some(Focus::NameEditor),
                    (Some(Focus::NameCancel), false) => Some(if draft.enabled() {
                        Focus::NameSubmit
                    } else {
                        Focus::Preview
                    }),
                    (Some(Focus::NameSubmit), true) => Some(Focus::NameCancel),
                    (Some(Focus::NameSubmit), false) => Some(Focus::Preview),
                    (Some(Focus::Preview), true) => Some(if draft.enabled() {
                        Focus::NameSubmit
                    } else if draft.busy {
                        Focus::NameEditor
                    } else {
                        Focus::NameCancel
                    }),
                    _ => None,
                };
                if let Some(next) = next {
                    self.set_folder_navigation_focus(next);
                    self.refresh_folder_browser();
                    return true;
                }
            }
        }
        if !state.expanded {
            return false;
        }
        let new_folder = self.modifiers.control_key()
            && self.modifiers.shift_key()
            && matches!(&event.logical_key,Key::Character(s)if s.eq_ignore_ascii_case("n"));
        let rename = event.logical_key == Key::Named(NamedKey::F2);
        let menu = event.logical_key == Key::Named(NamedKey::ContextMenu)
            || self.modifiers.shift_key() && event.logical_key == Key::Named(NamedKey::F10);
        if new_folder || rename || menu {
            if state.naming.is_some() {
                return true;
            }
            if menu {
                let index = state.selected;
                return self.open_folder_actions_menu(index, (180.0, 240.0));
            }
            return self.start_folder_naming(if new_folder {
                BrowserAction::NewFolder
            } else {
                BrowserAction::Rename
            });
        }
        false
    }
}
