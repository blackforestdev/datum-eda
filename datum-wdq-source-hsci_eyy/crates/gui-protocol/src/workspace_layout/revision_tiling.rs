//! Revision-specific recursive-tile placement and surface reuse.

use super::*;

impl WorkspaceLayout {
    /// Open content beside the complete existing tile tree. This preserves the
    /// internal Board/Schematic pairing instead of repeatedly subdividing one
    /// focused leaf into unusable slivers.
    pub fn open_beside_root(
        &mut self,
        content: PaneContent,
        orientation: SplitOrientation,
        first_ratio: f32,
        focus_new: bool,
    ) -> PaneId {
        if let Some(id) = find_content(&self.root, content) {
            if focus_new {
                self.focused = id;
                self.zoomed = None;
            }
            return id;
        }
        let new_id = self.alloc_id();
        let prior_root = std::mem::replace(
            &mut self.root,
            PaneNode::Leaf {
                id: new_id,
                content,
            },
        );
        self.root = PaneNode::Split {
            orientation,
            ratio: clamp_ratio(first_ratio),
            first: Box::new(prior_root),
            second: Box::new(PaneNode::Leaf {
                id: new_id,
                content,
            }),
        };
        if focus_new {
            self.focused = new_id;
            self.zoomed = None;
        }
        new_id
    }

    /// Focus and retarget the first Revision surface pane, if one exists.
    /// Navigator activation uses this as its LRU-equivalent eviction policy:
    /// only one top-level Revision surface remains open, while a separately
    /// authorized witness pane may still coexist beside its summary.
    pub fn retarget_first_revision_surface(&mut self, content: PaneContent) -> Option<PaneId> {
        let id = self.leaves().into_iter().find(|id| {
            matches!(
                self.content_for(*id),
                Some(PaneContent::Revision(RevisionPane::Surface(_)))
            )
        })?;
        if let Some(PaneNode::Leaf { content: slot, .. }) = find_leaf_mut(&mut self.root, id) {
            *slot = content;
        }
        self.focused = id;
        self.zoomed = None;
        Some(id)
    }
}
