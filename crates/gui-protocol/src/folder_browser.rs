//! Shared read-only directory navigation inside the native New/Open forms.
use std::path::{Component, Path};
#[path = "folder_browser/entries.rs"]
mod entries;
#[path = "folder_browser/suggestions.rs"]
mod suggestions;
pub use entries::*;
pub use suggestions::*;
#[path = "folder_browser/accessibility.rs"]
mod accessibility;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FolderBrowserMode {
    New,
    Open,
}
impl FolderBrowserMode {
    pub fn host_action_wait_reason(self) -> &'static str {
        match self {
            Self::New => "Create becomes available when the folder action finishes.",
            Self::Open => "Open becomes available when the folder action finishes.",
        }
    }
    pub fn host_action_wait_node(self) -> crate::GlobalPreferencesAccessibleNode {
        crate::GlobalPreferencesAccessibleNode {
            id: "project-host-action-pending".into(),name:"Folder action pending".into(),
            role:crate::GlobalPreferencesAccessibleRole::Status,value:Some(self.host_action_wait_reason().into()),
            description:"The submitted host action or its retained completion is pending. Navigation and Cancel remain available. No Project action runs automatically.".into(),available:true,focused:false,
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FolderBrowserFocus {
    Up,
    Home,
    Path,
    Browse,
    Folders,
    Preview,
    NameEditor,
    NameCancel,
    NameSubmit,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FolderBrowserState {
    pub mode: FolderBrowserMode,
    pub expanded: bool,
    pub path_input: String,
    pub directory: String,
    pub suggestions: Option<FolderSuggestions>,
    pub entries: Vec<BrowserEntry>,
    pub parent_identity: Option<(u64, u64)>,
    pub path_editor: crate::single_line_editor::SingleLineEditor,
    pub menu: Option<crate::popup_menu::PopupMenu<BrowserAction>>,
    pub context_entry: Option<BrowserEntry>,
    pub naming: Option<NamingDraft>,
    pub action_notice: Option<String>,
    pub input_refusal: Option<String>,
    pub occupied: std::collections::BTreeSet<String>,
    pub visible_rows: Vec<usize>,
    pub selected: Option<usize>,
    pub loading: bool,
    pub error: Option<String>,
    pub notice: Option<String>,
}

pub fn valid_project_folder_name(leaf: &str) -> bool {
    let mut parts = Path::new(leaf).components();
    !leaf.is_empty()
        && !leaf.contains(['/', '\0'])
        && matches!(parts.next(), Some(Component::Normal(_)))
        && parts.next().is_none()
}

impl FolderBrowserState {
    pub fn new(mode: FolderBrowserMode, directory: String) -> Self {
        let path_editor = crate::single_line_editor::SingleLineEditor::at_end(&directory);
        Self {
            mode,
            expanded: false,
            path_input: directory,
            directory: String::new(),
            suggestions: None,
            entries: vec![],
            parent_identity: None,
            path_editor,
            menu: None,
            context_entry: None,
            naming: None,
            action_notice: None,
            input_refusal: None,
            occupied: Default::default(),
            visible_rows: vec![],
            selected: None,
            loading: false,
            error: None,
            notice: None,
        }
    }
    pub fn ready(&self) -> bool {
        self.suggestions.is_none()
            && !self.loading
            && self.path_input == self.directory
            && !self.directory.is_empty()
            && self.error.is_none()
    }
    /// Enter/double-click delegates to Open's existing containing-root validator.
    pub fn opens_selected_manifest(&self) -> bool {
        self.mode == FolderBrowserMode::Open
            && self.expanded
            && self.ready()
            && self.menu.is_none()
            && self.naming.is_none()
            && self
                .selected
                .and_then(|i| self.entries.get(i))
                .is_some_and(BrowserEntry::is_project_manifest)
    }
    pub fn destination(&self, leaf: Option<&str>) -> Option<String> {
        if self.mode == FolderBrowserMode::New
            && self.ready()
            && self.occupied.contains(leaf.unwrap_or(""))
        {
            return None;
        }
        self.preview(leaf)
    }
    /// New always uses the displayed current folder as parent. A highlighted
    /// row is navigation only; Open can choose that row without entering it.
    pub fn preview(&self, leaf: Option<&str>) -> Option<String> {
        if self.path_input.is_empty() || self.path_input.contains('\0') {
            return None;
        }
        let path = match self.mode {
            FolderBrowserMode::New => {
                let leaf = leaf?;
                if !valid_project_folder_name(leaf) {
                    return None;
                }
                Path::new(&self.path_input).join(leaf)
            }
            FolderBrowserMode::Open => self
                .selected
                .filter(|_| self.expanded && self.ready())
                .and_then(|i| self.entries.get(i))
                .filter(|entry| entry.navigable())
                .map(|entry| Path::new(&self.directory).join(&entry.name))
                .unwrap_or_else(|| Path::new(&self.path_input).to_path_buf()),
        };
        path.to_str().map(str::to_owned)
    }
    pub fn refusal(&self, leaf: Option<&str>) -> Option<String> {
        if let Some(error) = &self.error {
            return Some(error.clone());
        }
        if self.mode == FolderBrowserMode::New
            && let Some(leaf) = leaf
        {
            if !leaf.is_empty() && !valid_project_folder_name(leaf) {
                return Some("Enter one folder name without '/', NUL, '.' or '..'. Correct Project name or uncheck Use project name to override Folder name.".into());
            }
            if self.ready() && self.occupied.contains(leaf) {
                return Some(format!(
                    "{leaf} already exists in Location. Choose another folder name; nothing was created."
                ));
            }
        }
        self.input_refusal.clone()
    }
    /// Shared complete read-only explanation; never part of the destination path.
    pub fn refusal_details(
        &self,
        leaf: Option<&str>,
        form_refusal: Option<&str>,
    ) -> Option<String> {
        let mut message = form_refusal.map(str::to_owned);
        if let Some(reason) = self.refusal(leaf) {
            if message.as_deref() != Some(reason.as_str()) {
                message = Some(message.map(|s| format!("{s} {reason}")).unwrap_or(reason));
            }
        }
        message
    }
    pub fn move_selection(&mut self, delta: isize) {
        self.selected = if self.entries.is_empty() {
            None
        } else {
            Some(match self.selected {
                Some(i) => i.saturating_add_signed(delta).min(self.entries.len() - 1),
                None if delta < 0 => self.entries.len() - 1,
                None => 0,
            })
        };
    }
}

pub fn folder_navigation_focus_order(expanded: bool) -> Vec<FolderBrowserFocus> {
    use FolderBrowserFocus::*;
    let mut order = vec![Path, Browse];
    if expanded {
        order.extend([Up, Home, Folders]);
    }
    order.push(Preview);
    order
}
#[cfg(test)]
#[path = "folder_browser/tests.rs"]
mod tests;
