//! Candidate-bound read-only availability; never owns a mutation completion.
use super::*;
use crate::host_filesystem::{self as host, Command, EntryKind, Identity, Protection, Request};
use datum_gui_protocol::{BrowserAction, BrowserEntry, BrowserEntryKind};

#[derive(Clone, PartialEq, Eq)]
struct Key {
    scope: u64,
    path: String,
    parent: (u64, u64),
    entry: Option<BrowserEntry>,
    protection: Protection,
}
pub(super) struct Qualification {
    key: Key,
    ticket: u64,
    result: Option<host::Availability>,
    deferred: Option<BrowserAction>,
}
#[cfg(test)]
impl Qualification {
    pub(super) fn result_ready(&self) -> bool {
        self.result.is_some()
    }
}
impl Runtime {
    fn qualification_key(&self) -> Option<Key> {
        let state = self
            .folder_browser
            .state
            .as_ref()
            .filter(|s| s.expanded && s.ready())?;
        Some(Key {
            scope: self.folder_browser.scope,
            path: state.path_input.clone(),
            parent: state.parent_identity?,
            entry: if state.menu.is_some() {
                state.context_entry.clone()
            } else {
                state.selected.and_then(|i| state.entries.get(i)).cloned()
            },
            protection: self
                .terminal_sessions
                .host_path_protection(self.workspace()),
        })
    }
    /// Cached results only describe this exact parent, row and session context.
    pub(super) fn folder_qualification_reason(&mut self, action: BrowserAction) -> Option<String> {
        let Some(key) = self.qualification_key() else {
            return Some("Browse a readable current Location before using folder actions.".into());
        };
        if self
            .folder_browser
            .qualification
            .as_ref()
            .is_none_or(|q| q.key != key)
        {
            if let Err(reason) = self.begin_folder_qualification(key) {
                return Some(reason);
            }
        }
        let Some(result) = self
            .folder_browser
            .qualification
            .as_ref()
            .and_then(|q| q.result.as_ref())
        else {
            return Some(
                "Checking this Location and selected entry without opening a Project…".into(),
            );
        };
        match action {
            BrowserAction::NewFolder => result.create.as_ref().err().cloned(),
            BrowserAction::Rename => result
                .rename
                .as_ref()
                .map(|r| r.as_ref().err().cloned())
                .unwrap_or_else(|| {
                    Some("Select a regular folder or file; link entries are not renamed.".into())
                }),
        }
    }
    fn begin_folder_qualification(&mut self, key: Key) -> Result<(), String> {
        if self.folder_browser.inspector.is_none() {
            let wake = self.folder_browser.wake.clone();
            self.folder_browser.inspector = Some(
                host::Inspector::new(move || {
                    let _ = wake.send_event(());
                })
                .map_err(|e| format!("Safety inspection is unavailable: {e}"))?,
            );
        }
        let rename = key.entry.as_ref().filter(|e| e.renameable()).map(|entry| {
            let (device, inode) = entry.identity.unwrap();
            Command::Rename {
                old_leaf: entry.name.clone(),
                new_leaf: entry.name.clone(),
                source: Identity {
                    device,
                    inode,
                    kind: if entry.kind == BrowserEntryKind::Directory {
                        EntryKind::Directory
                    } else {
                        EntryKind::File
                    },
                },
            }
        });
        let parent = Request {
            scope: key.scope,
            parent: entered_path(&key.path)?,
            parent_identity: Identity {
                device: key.parent.0,
                inode: key.parent.1,
                kind: EntryKind::Directory,
            },
            command: Command::CreateDirectory {
                leaf: String::new(),
            },
        };
        let ticket = self.folder_browser.inspector.as_ref().unwrap().request(
            parent,
            rename,
            key.protection.clone(),
        )?;
        self.folder_browser.qualification = Some(Qualification {
            key,
            ticket,
            result: None,
            deferred: None,
        });
        Ok(())
    }
    pub(super) fn clear_folder_qualification_intent(&mut self) {
        if let Some(q) = &mut self.folder_browser.qualification {
            q.deferred = None;
        }
    }
    pub(super) fn defer_folder_naming(&mut self, action: BrowserAction) {
        if let Some(q) = &mut self.folder_browser.qualification
            && q.result.is_none()
        {
            q.deferred = Some(action);
        }
    }
    pub(super) fn poll_folder_qualification(&mut self) -> bool {
        let Some(result) = self
            .folder_browser
            .inspector
            .as_ref()
            .and_then(host::Inspector::receive)
        else {
            return false;
        };
        let key = self.qualification_key();
        let Some(q) = &mut self.folder_browser.qualification else {
            return false;
        };
        if q.ticket != result.ticket {
            return false;
        }
        if key.as_ref() != Some(&q.key) {
            self.folder_browser.qualification = None;
            if let Some(menu) = self
                .folder_browser
                .state
                .as_mut()
                .and_then(|s| s.menu.as_mut())
            {
                for item in &mut menu.items {
                    item.unavailable = Some(
                        "Safety context changed. Reopen this menu to inspect the current entry."
                            .into(),
                    );
                }
                menu.focused = None;
                self.refresh_folder_browser();
                return true;
            }
            return false;
        }
        q.result = Some(result);
        let deferred = q.deferred.take();
        // Refresh only the currently bound popup. Dismissal and changed selection
        // cannot turn a late eligibility answer into an automatic action.
        if self
            .folder_browser
            .state
            .as_ref()
            .is_some_and(|s| s.menu.is_some())
        {
            let create = self
                .host_action_unavailable()
                .or_else(|| self.folder_qualification_reason(BrowserAction::NewFolder));
            let rename = self
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
            menu.items[0].unavailable = create;
            menu.items[1].unavailable = rename;
            if menu.focused.is_none_or(|i| menu.activate(i).is_none()) {
                menu.edge(false);
            }
            if let Some(action) = deferred
                && menu
                    .items
                    .iter()
                    .any(|item| item.action == action && item.unavailable.is_none())
            {
                self.start_folder_naming(action);
            }
            self.refresh_folder_browser();
            return true;
        }
        false
    }
}
