//! Navigator rows and existing pane/camera ownership remain separate identities.
use super::*;
use crate::{PaneContent, SplitOrientation, WorkspaceLayout};

impl SchematicNavigation {
    pub fn project(&self) -> Option<Uuid> {
        self.project
    }
}

impl SchematicDocuments {
    pub fn navigation(&self) -> SchematicNavigation {
        SchematicNavigation {
            project: self.project,
            panes: self.panes.clone(),
            inspected: self.inspected.clone(),
            expanded: self.expanded.clone(),
            scroll_offset: self.scroll_offset,
        }
    }
    pub fn restore_views(
        &mut self,
        navigation: &SchematicNavigation,
        layout: &WorkspaceLayout,
        root: &std::path::Path,
        model: &eda_engine::substrate::DesignModel,
    ) -> anyhow::Result<()> {
        if navigation.project.is_none() {
            return Ok(());
        }
        anyhow::ensure!(
            navigation.project == self.project && self.project == Some(model.project.project_id),
            "Document navigation belongs to another Project"
        );
        self.panes = navigation
            .panes
            .iter()
            .filter(|(pane, _)| layout.content_for(**pane).is_some())
            .map(|(pane, key)| (*pane, key.clone()))
            .collect();
        self.inspected = navigation.inspected.clone();
        self.expanded = navigation.expanded.clone();
        self.scroll_offset = navigation.scroll_offset;
        let keys: BTreeSet<_> = self
            .panes
            .iter()
            .filter(|(pane, _)| layout.content_for(**pane) == Some(PaneContent::Schematic))
            .map(|(_, key)| key.clone())
            .collect();
        for key in keys {
            if self.row(&key).is_some() {
                self.prepare_scene(&key, root, model)?;
            }
        }
        Ok(())
    }

    pub fn bind_initial_panes(&mut self, layout: &WorkspaceLayout) {
        let Some(root) = self
            .rows
            .iter()
            .find(|row| row.parent.is_none())
            .map(|row| row.key.clone())
        else {
            return;
        };
        if self.primary.is_none() {
            self.primary = Some(root.clone());
        }
        for pane in layout.leaves() {
            if layout.content_for(pane) == Some(PaneContent::Schematic) {
                self.panes.entry(pane).or_insert_with(|| root.clone());
            }
        }
    }

    /// The caller prepares this exact scene first. Publication cannot create an
    /// empty/fallback pane, and never receives or edits the authored selection.
    pub fn open_beside(
        &mut self,
        layout: &mut WorkspaceLayout,
        key: &SchematicDocumentKey,
    ) -> Option<PaneId> {
        if self.row(key).is_none() || !self.scenes.contains_key(key) {
            return None;
        }
        if let Some(pane) = self.panes.iter().find_map(|(pane, current)| {
            (current == key && layout.content_for(*pane) == Some(PaneContent::Schematic))
                .then_some(*pane)
        }) {
            layout.focused = pane;
            layout.zoomed = None;
            return Some(pane);
        }
        let pane =
            layout.split_focused_with(PaneContent::Schematic, SplitOrientation::Vertical, true);
        self.panes.insert(pane, key.clone());
        Some(pane)
    }

    pub fn move_inspection(&mut self, delta: isize) -> bool {
        let rows = self.visible_rows();
        if rows.is_empty() {
            return false;
        }
        let current = self
            .inspected
            .as_ref()
            .and_then(|key| rows.iter().position(|row| &row.key == key))
            .unwrap_or(0);
        let next = current.saturating_add_signed(delta).min(rows.len() - 1);
        let key = rows[next].key.clone();
        self.inspect(&key)
    }
}
