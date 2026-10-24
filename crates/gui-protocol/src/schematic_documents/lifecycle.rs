//! Qualified document bindings follow the shared recursive pane lifecycle.
use super::*;
use crate::{PaneContent, SplitOrientation, WorkspaceLayout};

impl SchematicDocuments {
    /// Warm document identity is retained across explicit editor fills, including
    /// an unavailable occurrence. A fill must never silently choose another row.
    pub fn bind_initial_view(&mut self, pane: PaneId) {
        if let Some(primary) = self.primary.clone() {
            self.panes.entry(pane).or_insert(primary);
        }
    }

    pub fn split_view(
        &mut self,
        layout: &mut WorkspaceLayout,
        orientation: SplitOrientation,
    ) -> PaneId {
        let source = layout.focused;
        let content = layout.focused_content();
        let pane = layout.split_focused_with(content, orientation, false);
        if let Some(key) = self.panes.get(&source).cloned() {
            self.panes.insert(pane, key);
        }
        pane
    }

    pub fn prune_views(&mut self, layout: &WorkspaceLayout) {
        self.panes
            .retain(|pane, _| layout.content_for(*pane).is_some());
    }

    /// An explicit preset starts fresh views of the captured initial owner.
    pub fn reset_views(&mut self, layout: &WorkspaceLayout) {
        self.panes.clear();
        for pane in layout.leaves() {
            if layout.content_for(pane) == Some(PaneContent::Schematic) {
                self.bind_initial_view(pane);
            }
        }
    }

    pub fn visible_keys(&self, layout: &WorkspaceLayout) -> BTreeSet<SchematicDocumentKey> {
        layout
            .leaves()
            .into_iter()
            .filter(|pane| {
                layout.content_for(*pane) == Some(PaneContent::Schematic)
                    && layout.zoomed.is_none_or(|zoomed| zoomed == *pane)
            })
            .filter_map(|pane| self.panes.get(&pane))
            .cloned()
            .collect()
    }
}
