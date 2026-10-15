//! Read-only native Project preparation and transient Open dialog state.
use crate::{ReviewWorkspaceState, SelectionTarget};
use anyhow::{Context, Result, ensure};
use eda_engine::substrate::{DesignModel, ProjectResolver};
use std::path::{Path, PathBuf};
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LiveReviewRequest {
    pub project_root: PathBuf,
    pub board_file: Option<PathBuf>,
    pub artifact_path: Option<PathBuf>,
    pub net_uuid: Option<String>,
    pub from_anchor_pad_uuid: Option<String>,
    pub to_anchor_pad_uuid: Option<String>,
    pub profile: Option<String>,
    /// Compatibility origin used only to locate a companion schematic.
    pub kicad_board_source: Option<PathBuf>,
}

/// A Project's UUID/revision can match a lawful copy. Session publication must
/// also match the canonical origin root and the consumer's session generation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectOpenOrigin {
    pub root: Option<PathBuf>,
    pub session_generation: u64,
}

impl ProjectOpenOrigin {
    pub fn capture(root: &Path, session_generation: u64) -> Result<Self> {
        Ok(Self {
            root: Some(root.canonicalize()?),
            session_generation,
        })
    }

    pub fn capture_unopened(session_generation: u64) -> Self {
        Self {
            root: None,
            session_generation,
        }
    }

    pub fn matches_unopened(&self, session_generation: u64) -> bool {
        self.root.is_none() && self.session_generation == session_generation
    }

    pub fn matches(&self, root: &Path, session_generation: u64) -> bool {
        self.session_generation == session_generation
            && root
                .canonicalize()
                .is_ok_and(|root| self.root.as_ref() == Some(&root))
    }
}

pub struct PreparedProjectOpen {
    pub root: PathBuf,
    pub workspace: ReviewWorkspaceState,
    pub model: Arc<DesignModel>,
    pub accepted_tip: Option<String>,
}

impl PreparedProjectOpen {
    /// No launch/bootstrap, genesis, migration, terminal or authored mutation.
    /// The native resolver uses its existing operational coordination lease.
    /// Both editors and library projections share one engine-resolved model.
    pub fn prepare(root: &Path, include_review: bool) -> Result<Self> {
        let root = root
            .canonicalize()
            .context("Open Project directory is unavailable")?;
        ensure!(
            root.is_dir(),
            "Open Project requires a native Project directory"
        );
        let request = LiveReviewRequest {
            project_root: root.clone(),
            board_file: None,
            artifact_path: None,
            net_uuid: None,
            from_anchor_pad_uuid: None,
            to_anchor_pad_uuid: None,
            profile: None,
            kicad_board_source: None,
        };
        let accepted_tip = crate::refresh_accepted_transaction_tip(&request)?;
        let (mut workspace, model) =
            crate::workspace_loading::load_workspace(&request, include_review, None)?;
        let model = model.context("Open Project native model is unavailable")?;
        ensure_open_integrity(&model)?;
        workspace.selection = SelectionTarget::None;
        workspace.ui.selection_inspector = Default::default();
        let prepared = Self {
            root,
            workspace,
            model,
            accepted_tip,
        };
        prepared.revalidate()?;
        Ok(prepared)
    }

    /// Final worker admission holds the existing canonical writer exclusion
    /// until the GUI publishes or rejects this exact source snapshot.
    pub fn admit(&self) -> Result<eda_engine::revision::ProjectWriteLease> {
        let lease = eda_engine::revision::ProjectWriteLease::acquire(&self.root)?;
        let current = ProjectResolver::new(&self.root).resolve_with_lease(&lease)?;
        ensure!(
            current == *self.model,
            "Open Project source or history changed during preparation; retry Open"
        );
        ensure_open_integrity(&current)?;
        Ok(lease)
    }

    /// Used by the preparation worker before handing a staged candidate back.
    /// A changed destination is a refusal; preparation does not silently repin.
    pub fn revalidate(&self) -> Result<()> {
        let current = ProjectResolver::new(&self.root).resolve()?;
        ensure!(
            current == *self.model,
            "Open Project source changed during preparation; retry Open"
        );
        let request = &self
            .workspace
            .backing
            .as_ref()
            .context("Open Project backing is unavailable")?
            .request;
        ensure!(
            crate::refresh_accepted_transaction_tip(request)? == self.accepted_tip,
            "Open Project history changed during preparation; retry Open"
        );
        Ok(())
    }
}

/// The resolver may return diagnostics for an inspectable incomplete model.
/// Native Open must not publish that model as a valid replacement Project.
fn ensure_open_integrity(model: &DesignModel) -> Result<()> {
    if let Some(diagnostic) = model.diagnostics.iter().find(|diagnostic| {
        diagnostic.code == "missing_required_shard"
            || diagnostic.code.starts_with("journal_")
            || diagnostic.code == "revision_integrity_unavailable"
    }) {
        anyhow::bail!(
            "Open Project source integrity is unavailable: {}: {}",
            diagnostic.code,
            diagnostic.message
        );
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ProjectOpenFocus {
    #[default]
    Directory,
    Cancel,
    Open,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ProjectOpenDialogState {
    pub open: bool,
    pub directory: String,
    pub busy: bool,
    pub refusal: Option<String>,
    pub focus: ProjectOpenFocus,
}

impl ProjectOpenDialogState {
    /// Shared native provider projection; values never create Project authority.
    pub fn accessibility_nodes(&self) -> Vec<crate::GlobalPreferencesAccessibleNode> {
        use crate::{
            GlobalPreferencesAccessibleNode as Node, GlobalPreferencesAccessibleRole as Role,
        };
        if !self.open {
            return vec![];
        }
        let mut nodes = vec![
            Node {
                id: "open-project-dialog".into(),
                name: "Open Project".into(),
                role: Role::Dialog,
                value: None,
                description:
                    "Open a native Project directory; cancellation preserves the current Project."
                        .into(),
                available: true,
                focused: false,
            },
            Node {
                id: "open-project-directory".into(),
                name: "Project directory".into(),
                role: Role::TextBox,
                value: Some(self.directory.clone()),
                description: if self.busy {
                    "Preparing this directory; editing is unavailable until preparation finishes."
                } else {
                    "Full native Project directory; preserved when Open is refused."
                }
                .into(),
                available: !self.busy,
                focused: self.focus == ProjectOpenFocus::Directory,
            },
            Node {
                id: "open-project-cancel".into(),
                name: "Cancel".into(),
                role: Role::Button,
                value: None,
                description: "Cancel preparation without replacing the current Project.".into(),
                available: true,
                focused: self.focus == ProjectOpenFocus::Cancel,
            },
            Node {
                id: "open-project-submit".into(),
                name: "Open".into(),
                role: Role::Button,
                value: None,
                description: "Validate source and history before publishing this Project.".into(),
                available: self.open_enabled(),
                focused: self.focus == ProjectOpenFocus::Open,
            },
        ];
        if self.busy {
            nodes.push(Node {
                id: "open-project-busy".into(),
                name: "Preparing Project".into(),
                role: Role::Status,
                value: Some(self.directory.clone()),
                description: "Cancel remains available.".into(),
                available: true,
                focused: false,
            });
        }
        if let Some(refusal) = &self.refusal {
            nodes.push(Node {
                id: "open-project-refusal".into(),
                name: "Project not opened".into(),
                role: Role::Status,
                value: Some(refusal.clone()),
                description:
                    "Correct the directory or retry Open; the prior application state is preserved."
                        .into(),
                available: true,
                focused: false,
            });
        }
        nodes
    }

    pub fn open_enabled(&self) -> bool {
        !self.busy && !self.directory.trim().is_empty()
    }

    pub fn move_focus(&mut self, reverse: bool) {
        let order = if self.open_enabled() {
            &[
                ProjectOpenFocus::Directory,
                ProjectOpenFocus::Cancel,
                ProjectOpenFocus::Open,
            ][..]
        } else {
            &[ProjectOpenFocus::Directory, ProjectOpenFocus::Cancel][..]
        };
        let index = order
            .iter()
            .position(|focus| *focus == self.focus)
            .unwrap_or(0);
        self.focus = order[(index + if reverse { order.len() - 1 } else { 1 }) % order.len()];
    }
}

#[cfg(test)]
#[path = "project_open/tests.rs"]
mod tests;
