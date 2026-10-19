//! One observed entry and naming-draft model consumed by New and Open.
use crate::single_line_editor::SingleLineEditor;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BrowserEntryKind {
    Directory,
    File,
    Link { directory: bool },
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrowserEntry {
    pub name: String,
    pub kind: BrowserEntryKind,
    /// Observed filesystem identity; absence never authorizes a host rename.
    pub identity: Option<(u64, u64)>,
}
impl BrowserEntry {
    /// An activation shortcut, never proof that the containing Project is valid.
    pub fn is_project_manifest(&self) -> bool {
        self.name == "project.json" && self.kind == BrowserEntryKind::File
    }
    pub fn navigable(&self) -> bool {
        matches!(
            self.kind,
            BrowserEntryKind::Directory | BrowserEntryKind::Link { directory: true }
        )
    }
    pub fn renameable(&self) -> bool {
        self.identity.is_some()
            && matches!(
                self.kind,
                BrowserEntryKind::Directory | BrowserEntryKind::File
            )
    }
    pub fn kind_label(&self) -> &'static str {
        match self.kind {
            BrowserEntryKind::Directory => "folder",
            BrowserEntryKind::File => "file",
            BrowserEntryKind::Link { .. } => "link",
            BrowserEntryKind::Other => "other entry",
        }
    }
    /// An unqualified directory projection is useful before metadata is known;
    /// it can be displayed/navigated but cannot become a Rename request.
    pub fn directory(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            kind: BrowserEntryKind::Directory,
            identity: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BrowserAction {
    NewFolder,
    Rename,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NamingPurpose {
    CreateDirectory,
    Rename(BrowserEntry),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NamingDraft {
    pub purpose: NamingPurpose,
    pub parent: String,
    pub parent_identity: (u64, u64),
    pub scope: u64,
    pub value: String,
    pub editor: SingleLineEditor,
    pub busy: bool,
    pub refusal: Option<String>,
}
impl NamingDraft {
    pub fn new(
        parent: String,
        parent_identity: (u64, u64),
        scope: u64,
        entry: Option<BrowserEntry>,
    ) -> Self {
        let (purpose, value) = match entry {
            Some(entry) => (NamingPurpose::Rename(entry.clone()), entry.name),
            None => (NamingPurpose::CreateDirectory, String::new()),
        };
        let mut editor = SingleLineEditor::at_end(&value);
        if matches!(purpose, NamingPurpose::Rename(_)) {
            editor.select_all(&value);
        }
        Self {
            purpose,
            parent,
            parent_identity,
            scope,
            value,
            editor,
            busy: false,
            refusal: None,
        }
    }
    pub fn enabled(&self) -> bool {
        !self.busy
            && crate::valid_project_folder_name(&self.value)
            && !self.value.chars().any(char::is_control)
    }
    pub fn issue(&self) -> Option<String> {
        self.refusal.clone().or_else(|| {
            (!self.busy && !self.value.is_empty() && !self.enabled()).then(|| {
                "Enter one name without separators, control characters, '.' or '..'.".into()
            })
        })
    }
    pub fn field_name(&self) -> &'static str {
        if matches!(self.purpose, NamingPurpose::CreateDirectory) {
            "New folder name"
        } else {
            "New name"
        }
    }
    pub fn action_label(&self) -> &'static str {
        match (&self.purpose, self.busy) {
            (NamingPurpose::CreateDirectory, false) => "Create Folder",
            (NamingPurpose::CreateDirectory, true) => "Creating…",
            (_, false) => "Rename",
            (_, true) => "Renaming…",
        }
    }
    pub fn title(&self) -> String {
        match &self.purpose {
            NamingPurpose::CreateDirectory => format!("New folder in {}", self.parent),
            NamingPurpose::Rename(entry) => format!("Rename “{}” in {}", entry.name, self.parent),
        }
    }
}

impl From<&str> for BrowserEntry {
    fn from(name: &str) -> Self {
        Self::directory(name)
    }
}
impl From<String> for BrowserEntry {
    fn from(name: String) -> Self {
        Self::directory(name)
    }
}
