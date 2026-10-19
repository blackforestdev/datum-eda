//! One accessible browser/menu/naming projection for both native forms.
use super::*;
use crate::{GlobalPreferencesAccessibleNode as Node, GlobalPreferencesAccessibleRole as Role};

impl FolderBrowserState {
    pub fn accessibility_nodes(
        &self,
        focus: Option<FolderBrowserFocus>,
        leaf: Option<&str>,
        enabled: bool,
    ) -> Vec<Node> {
        use FolderBrowserFocus::*;
        let active = if self.menu.is_some() { None } else { focus };
        let mut nodes = vec![];
        let mut add = |id: &str,
                       name: &str,
                       role,
                       value,
                       available,
                       control: Option<FolderBrowserFocus>,
                       description: &str| {
            nodes.push(Node {
                id: format!("folder-navigation-{id}"),
                name: name.into(),
                role,
                value,
                available: enabled && available,
                focused: control.is_some() && control == active,
                description: description.into(),
            });
        };
        add(
            "path",
            "Location",
            Role::TextBox,
            Some(self.path_input.clone()),
            true,
            Some(Path),
            "Current typed Location. Create/Open requires neither Enter nor Browse. Supports selection and ordinary single-line editing.",
        );
        add(
            "browse",
            "Browse for Location",
            Role::Button,
            Some(
                if self.expanded {
                    "expanded"
                } else {
                    "collapsed"
                }
                .into(),
            ),
            true,
            Some(Browse),
            "Shows folder navigation in this same dialog and focuses the list; pressing again refocuses it.",
        );
        if self.expanded {
            add(
                "up",
                "Up one folder",
                Role::Button,
                None,
                true,
                Some(Up),
                "Navigate to the parent folder.",
            );
            add(
                "home",
                "Home folder",
                Role::Button,
                None,
                true,
                Some(Home),
                "Navigate to Home.",
            );
            let selected = self.selected.and_then(|i| self.entries.get(i));
            add(
                "folders",
                &self
                    .suggestions
                    .as_ref()
                    .map(|q| format!("Folder suggestions in {} for {}", q.parent, q.fragment))
                    .unwrap_or_else(|| format!("Files and folders in {}", self.path_input)),
                Role::List,
                Some(format!(
                    "{} entries. Highlighted: {}. Enter navigates into folders; New uses Location as parent.",
                    self.entries.len(),
                    selected.map(|e| e.name.as_str()).unwrap_or("none")
                )),
                !self.loading,
                Some(Folders),
                if self.suggestions.is_some() {
                    "Suggestions never change Location until accepted. In Location, Up/Down select; Tab or Enter accepts. Escape clears selection first. Shift+Tab traverses. Host actions require an exact directory."
                } else {
                    "Read-only navigation. Right-click or Menu opens actions; F2 renames an eligible entry; Ctrl+Shift+N creates a folder explicitly."
                },
            );
        }
        let selected_directory = self
            .selected
            .and_then(|i| self.entries.get(i))
            .is_some_and(BrowserEntry::navigable);
        add(
            "preview",
            if self.mode == FolderBrowserMode::New {
                "New Project folder"
            } else {
                "Project directory"
            },
            Role::Navigation,
            self.preview(leaf).map(|p| {
                if self.mode == FolderBrowserMode::Open
                    && self.expanded
                    && self.ready()
                    && !selected_directory
                {
                    format!("{p} (this folder)")
                } else {
                    p
                }
            }),
            true,
            Some(Preview),
            "Complete exact candidate. Project publication still requires Create/Open and canonical validation.",
        );
        let status = if self.loading {
            "Reading folder…".into()
        } else if let Some(reason) = self.refusal(leaf) {
            reason
        } else if let Some(notice) = &self.action_notice {
            notice.clone()
        } else if let Some(notice) = &self.notice {
            notice.clone()
        } else if let Some(header) = self.suggestion_header() {
            header
        } else if self.entries.is_empty() {
            "No folders here".into()
        } else {
            format!("{} entries", self.entries.len())
        };
        if self.expanded || self.refusal(leaf).is_some() {
            add(
                "status",
                "Folder navigation status",
                Role::Status,
                Some(status),
                true,
                None,
                "Navigation and explicit host action status; completed actions are not rolled back by dialog Cancel.",
            );
        }
        drop(add);
        if self.expanded && self.listing_ready() {
            let rows = self
                .visible_rows
                .iter()
                .copied()
                .chain(self.selected)
                .collect::<std::collections::BTreeSet<_>>();
            for index in rows {
                let Some(entry) = self.entries.get(index) else {
                    continue;
                };
                nodes.push(Node { id: format!("folder-navigation-row-{index}"), name: entry.name.clone(), role: Role::ListItem,
                    value: Some(format!("{}, {} of {}{}", if self.suggestions.is_some() { "folder suggestion" } else { entry.kind_label() }, index+1, self.entries.len(), if self.selected == Some(index) { ", selected" } else { "" })), available: enabled,
                    focused: active == Some(Folders) && self.selected == Some(index),
                    description: if self.suggestions.is_some() { "Enter or double-click accepts this folder suggestion into Location. Selection never changes the Project candidate or publishes a Project." } else if self.mode == FolderBrowserMode::Open && entry.is_project_manifest() { "Enter opens this folder’s Project. Open validates the containing Project before publication." } else if entry.navigable() { "Enter or double-click navigates into this folder. Eligible ordinary entries can be renamed explicitly." }
                        else { "This file/entry is not folder navigation or a Project Open candidate. Eligible ordinary files can be renamed explicitly." }.into() });
            }
        }
        if let Some(draft) = &self.naming {
            nodes.push(Node { id: "folder-name-group".into(), name: draft.title(), role: Role::Group, value: None, available: enabled, focused: false,
                description: "Explicit host naming action. Completed actions persist after Project dialog Cancel.".into() });
            nodes.push(Node { id: "folder-name-input".into(), name: draft.field_name().into(), role: Role::TextBox, value: Some(draft.value.clone()), available: enabled,
                focused: active == Some(NameEditor), description: format!("{} Selected: {}", if draft.busy { "Read-only while the submitted action is pending." } else { "Single-line editor. Enter submits only this naming action; Escape cancels this draft." }, draft.editor.selected_text(&draft.value)) });
            for (id, name, control, available) in [
                ("cancel", "Cancel naming", NameCancel, !draft.busy),
                ("submit", draft.action_label(), NameSubmit, draft.enabled()),
            ] {
                nodes.push(Node { id: format!("folder-name-{id}"), name: name.into(), role: Role::Button, value: None, available: enabled && available,
                    focused: active == Some(control), description: if draft.busy { "Host operation remains pending; Enter and Escape are consumed. Outer dialog Cancel remains available." } else { "Applies only to the explicit host naming draft, never Project publication." }.into() });
            }
            if let Some(reason) = draft.issue() {
                nodes.push(Node {
                    id: "folder-name-refusal".into(),
                    name: "Folder action refusal".into(),
                    role: Role::Status,
                    value: Some(reason),
                    description: "The exact naming draft is retained.".into(),
                    available: true,
                    focused: false,
                });
            }
        }
        if let Some(menu) = &self.menu {
            let title = format!(
                "Actions for {}",
                self.context_entry
                    .as_ref()
                    .map(|e| e.name.as_str())
                    .unwrap_or(&self.path_input)
            );
            nodes.extend(menu.accessibility_nodes("folder-actions-menu", &title, enabled));
        }
        nodes
    }
}
