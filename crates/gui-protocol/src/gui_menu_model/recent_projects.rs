//! Device-local recent destinations projected into the existing File submenu.
use super::{GuiMenuItem, GuiMenuModel, default_gui_menu_model};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::{path::PathBuf, sync::Arc};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecentProject {
    pub name: String,
    pub path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RecentProjects {
    entries: Vec<RecentProject>,
    cached: Option<Arc<GuiMenuModel>>,
}

impl RecentProjects {
    pub fn from_entries(entries: Vec<RecentProject>) -> Result<Self> {
        let mut recent = Self::default();
        recent.update(entries)?;
        Ok(recent)
    }

    pub fn entries(&self) -> &[RecentProject] {
        &self.entries
    }

    /// Rebuild once per changed list, never for a render or keyboard event.
    pub fn update(&mut self, entries: Vec<RecentProject>) -> Result<bool> {
        if self.entries == entries {
            return Ok(false);
        }
        let cached = if entries.is_empty() {
            None
        } else {
            let mut model = default_gui_menu_model()?.clone();
            let file = model
                .menubar
                .iter_mut()
                .find(|menu| menu.menu == "File")
                .context("File menu is unavailable")?;
            let rows = file
                .submenus
                .get_mut("Recent projects")
                .context("Recent projects submenu is unavailable")?;
            *rows = entries
                .iter()
                .enumerate()
                .map(|(index, entry)| GuiMenuItem {
                    label: format!("{} — {}", entry.name, entry.path.display()),
                    icon: Some("folder-open".into()),
                    shortcut: None,
                    destructive: false,
                    verb: None,
                    gui_local: Some(format!("project.open_recent.{index}")),
                    not_built: None,
                    submenu: None,
                    requires_project: false,
                })
                .collect();
            Some(Arc::new(model))
        };
        self.entries = entries;
        self.cached = cached;
        Ok(true)
    }
}

pub fn workspace_gui_menu_model(state: &crate::ReviewWorkspaceState) -> Result<&GuiMenuModel> {
    match &state.ui.recent_projects.cached {
        Some(model) => Ok(model),
        None => default_gui_menu_model(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gui_menu_model::{
        accessibility::menu_accessibility_nodes,
        action_registry::{GuiLocalHandler, gui_local_handler},
    };

    #[test]
    fn recent_projects_use_distinct_paths_and_exact_actions_in_accessible_menu() {
        let entries: Vec<_> = ["/designs/one", "/designs/two"]
            .into_iter()
            .map(|path| RecentProject {
                name: "Amplifier".into(),
                path: path.into(),
            })
            .collect();
        let mut state = crate::load_fixture_workspace_state();
        state.project_unopened = true;
        state.ui.recent_projects = RecentProjects::from_entries(entries.clone()).unwrap();
        state.ui.active_menu = Some("File".into());
        state.ui.active_submenu = Some("Recent projects".into());
        state.ui.focus = crate::ApplicationFocus::Overlay;
        let model = workspace_gui_menu_model(&state).unwrap();
        let file = model
            .menubar
            .iter()
            .find(|menu| menu.menu == "File")
            .unwrap();
        let rows = file.submenus["Recent projects"].clone();
        for (index, row) in rows.iter().enumerate() {
            assert_eq!(
                gui_local_handler(row.gui_local.as_deref().unwrap()),
                Some(GuiLocalHandler::OpenRecentProject(index))
            );
            assert!(row.label.contains(entries[index].path.to_str().unwrap()));
            assert!(row.unavailable_reason(&state).is_none());
        }
        let nodes = menu_accessibility_nodes(model, &state);
        assert!(
            nodes
                .iter()
                .any(|n| n.name == rows[0].label && n.available && n.focused)
        );
        assert!(
            nodes
                .iter()
                .any(|n| n.name == rows[1].label && n.available && !n.focused)
        );
        assert_eq!(gui_local_handler("project.open_recent.-1"), None);
        assert_eq!(gui_local_handler("project.open_recent.01"), None);
        assert_eq!(gui_local_handler("project.open_recent.0.extra"), None);
        let original = state.ui.recent_projects.cached.clone().unwrap();
        assert!(!state.ui.recent_projects.update(entries).unwrap());
        assert!(Arc::ptr_eq(
            &original,
            state.ui.recent_projects.cached.as_ref().unwrap()
        ));
        assert!(state.ui.recent_projects.update(Vec::new()).unwrap());
        assert!(rows[0].unavailable_reason(&state).is_some());
    }

    #[test]
    fn recent_projects_empty_list_is_truthful_and_inert() {
        let mut state = crate::load_fixture_workspace_state();
        state.ui.active_menu = Some("File".into());
        state.ui.active_submenu = Some("Recent projects".into());
        let model = workspace_gui_menu_model(&state).unwrap();
        let nodes = menu_accessibility_nodes(model, &state);
        let empty = nodes
            .iter()
            .find(|n| n.name == "No recent projects")
            .unwrap();
        assert!(!empty.available);
        assert!(empty.description.contains("No Projects have been opened"));
    }
}
