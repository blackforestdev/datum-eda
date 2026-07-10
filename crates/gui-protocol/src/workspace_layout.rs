//! Workspace consumer-state cluster: the dock/filter/console/marking-menu bag
//! plus the pane-tiling tree (decision 021, workspace pane tiling).
//!
//! Doctrine: everything in this module is CONSUMER / WORKSPACE state — the same
//! class as window layout, hover, or selection. It is NEVER journaled: pane
//! layout, focus, zoom, split ratios, and the console/marking-menu bags do not
//! enter `commit()`/the design journal and are not typed design Operations. They
//! project over the resolved model; they never mutate it.

use crate::{ArtifactPreviewViewportState, TerminalLaneState};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DockTab {
    Terminal,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceFilterState {
    pub show_authored: bool,
    pub show_proposed: bool,
    pub show_unrouted: bool,
    pub dim_unrelated: bool,
    pub active_layer_id: Option<String>,
    pub layer_visibility: BTreeMap<String, bool>,
}

/// The invisible console sink for GUI-action narration — the AutoCAD/Eagle
/// command-echo lane (fit board, layer toggle, selection, view zoom, ...).
///
/// Doctrine: GUI-action echoes are NOT terminal output. The integrated PTY
/// terminal is a real shell that GUI actions must never write to. The correct
/// visible home for these echoes is the editor command console, which does not
/// exist yet (it is downstream of the authoring write-path). Until that surface
/// is built, narration lands here in an invisible model-only sink — never on the
/// PTY, never on the terminal display buffer.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConsoleLaneState {
    pub lines: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WorkspaceUiState {
    pub active_dock_tab: Option<DockTab>,
    pub active_menu: Option<String>,
    pub marking_menu: Option<MarkingMenuState>,
    pub dock_height_px: u32,
    pub hovered_object_id: Option<String>,
    pub filters: WorkspaceFilterState,
    pub terminal: TerminalLaneState,
    pub console: ConsoleLaneState,
    pub artifact_preview: ArtifactPreviewViewportState,
    pub layout: WorkspaceLayout,
}

impl WorkspaceUiState {
    /// Append a GUI-action narration echo to the invisible console sink.
    ///
    /// Mirrors the terminal lane's 240-line cap but never touches the PTY lane:
    /// this is a model-only field with no visible surface yet.
    pub fn push_console_line(&mut self, line: String) {
        self.console.lines.push(line);
        if self.console.lines.len() > 240 {
            let overflow = self.console.lines.len() - 240;
            self.console.lines.drain(0..overflow);
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MarkingMenuState {
    pub menu_key: String,
    pub target_object_id: Option<String>,
    pub anchor_x_px: i32,
    pub anchor_y_px: i32,
    pub preview_slot: Option<String>,
    pub gesture_dx_px: i32,
    pub gesture_dy_px: i32,
}

// ---------------------------------------------------------------------------
// Pane tiling tree (decision 021).
//
// The workspace viewport is a recursive binary tile tree: a node is either a
// Leaf (one pane) or a Split (two child nodes, an orientation, and a ratio).
// Binary + nesting reproduces every layout a tiling WM can make. `zoomed` is a
// transient maximize state layered over the tree; it never destroys the tree.
//
// This is consumer/workspace state — layout, focus, split ratio, and zoom are
// the same class as window layout or selection. It is NEVER journaled and is
// not a typed design Operation; it only projects over the resolved model.
// ---------------------------------------------------------------------------

/// Which way a split divides its two children.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SplitOrientation {
    Horizontal,
    Vertical,
}

/// What a pane shows (its `(document, view)` projection over the model).
///
/// Only Board and Schematic exist today. Footprint, Symbol, Datasheet, 3D, and
/// CheckReport are future variants — they land (greyed/disabled in the "Fill
/// pane with →" menu until the corresponding surfaces exist) as those editors
/// come online. Do not add them until their surfaces are real.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaneContent {
    Board,
    Schematic,
}

/// Stable identifier for a leaf pane, allocated monotonically by the owning
/// `WorkspaceLayout`. Ids are never reused within a layout's lifetime.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PaneId(pub u32);

/// A node in the recursive tile tree: a leaf pane or a binary split.
#[derive(Debug, Clone, PartialEq)]
pub enum PaneNode {
    Leaf {
        id: PaneId,
        content: PaneContent,
    },
    Split {
        orientation: SplitOrientation,
        ratio: f32,
        first: Box<PaneNode>,
        second: Box<PaneNode>,
    },
}

/// A named layout preset for the View-menu "Layout presets" action.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkspacePreset {
    /// One Board leaf filling the whole workspace.
    Single,
    /// A vertical Board|Schematic split at ratio 0.5, Board focused.
    BoardSchematic,
}

/// The workspace pane tree plus its transient focus/zoom state.
///
/// The DEFAULT layout reproduces today's hard-coded look: a Board+Schematic
/// vertical split at ratio 0.5 with the Board leaf focused. This keeps the
/// foundation commit behavior-neutral — the render side still uses its own
/// hard-coded split; nothing changes visually until a later commit repoints the
/// renderer at this tree.
#[derive(Debug, Clone, PartialEq)]
pub struct WorkspaceLayout {
    /// Root of the recursive tile tree.
    pub root: PaneNode,
    /// The focused leaf — owns the Inspector, tools, and active-editor menus.
    pub focused: PaneId,
    /// Transient maximize state; `Some(id)` fills the workspace with that leaf
    /// and hides the rest without destroying the tree.
    pub zoomed: Option<PaneId>,
    /// Monotonic counter for allocating fresh `PaneId`s; never rewinds.
    next_id: u32,
}

impl Default for WorkspaceLayout {
    fn default() -> Self {
        Self::board_schematic()
    }
}

/// Split ratios clamp to this range so a pane can never collapse to nothing.
pub const PANE_RATIO_MIN: f32 = 0.1;
pub const PANE_RATIO_MAX: f32 = 0.9;

fn clamp_ratio(ratio: f32) -> f32 {
    ratio.clamp(PANE_RATIO_MIN, PANE_RATIO_MAX)
}

impl WorkspaceLayout {
    /// A single Board leaf filling the workspace.
    pub fn single() -> Self {
        let board = PaneId(0);
        WorkspaceLayout {
            root: PaneNode::Leaf {
                id: board,
                content: PaneContent::Board,
            },
            focused: board,
            zoomed: None,
            next_id: 1,
        }
    }

    /// A vertical Board|Schematic split at ratio 0.5, Board focused — today's
    /// hard-coded two-pane look.
    pub fn board_schematic() -> Self {
        let board = PaneId(0);
        let schematic = PaneId(1);
        WorkspaceLayout {
            root: PaneNode::Split {
                orientation: SplitOrientation::Vertical,
                ratio: 0.5,
                first: Box::new(PaneNode::Leaf {
                    id: board,
                    content: PaneContent::Board,
                }),
                second: Box::new(PaneNode::Leaf {
                    id: schematic,
                    content: PaneContent::Schematic,
                }),
            },
            focused: board,
            zoomed: None,
            next_id: 2,
        }
    }

    fn alloc_id(&mut self) -> PaneId {
        let id = PaneId(self.next_id);
        self.next_id += 1;
        id
    }

    /// The focused leaf node (always a `PaneNode::Leaf`).
    pub fn focused_leaf(&self) -> &PaneNode {
        find_leaf(&self.root, self.focused)
            .expect("focused pane id must always resolve to a leaf in the tree")
    }

    /// The content of the focused leaf.
    pub fn focused_content(&self) -> PaneContent {
        match self.focused_leaf() {
            PaneNode::Leaf { content, .. } => *content,
            PaneNode::Split { .. } => unreachable!("focused_leaf always returns a leaf"),
        }
    }

    /// All leaf ids in deterministic in-order (first-before-second) walk order.
    pub fn leaves(&self) -> Vec<PaneId> {
        let mut out = Vec::new();
        collect_leaves(&self.root, &mut out);
        out
    }

    /// Split the focused leaf into a Split whose children are the old leaf
    /// (first) and a fresh leaf (second) inheriting the old content. Focus
    /// stays on the original leaf.
    pub fn split_focused(&mut self, orientation: SplitOrientation) {
        let new_id = self.alloc_id();
        let focused = self.focused;
        if let Some(slot) = find_leaf_mut(&mut self.root, focused) {
            let content = match slot {
                PaneNode::Leaf { content, .. } => *content,
                PaneNode::Split { .. } => return,
            };
            let old = std::mem::replace(
                slot,
                PaneNode::Leaf {
                    id: focused,
                    content,
                },
            );
            *slot = PaneNode::Split {
                orientation,
                ratio: 0.5,
                first: Box::new(old),
                second: Box::new(PaneNode::Leaf {
                    id: new_id,
                    content,
                }),
            };
        }
        // Focus is unchanged: `focused` still names the old leaf inside `first`.
    }

    /// Remove the focused leaf; its sibling reclaims the space (the parent Split
    /// collapses to the sibling). Focus moves to a deterministic neighbor (the
    /// in-order first leaf of the reclaimed sibling). Closing the last remaining
    /// leaf is a no-op — the tree is never empty.
    pub fn close_focused(&mut self) {
        if matches!(self.root, PaneNode::Leaf { .. }) {
            return;
        }
        if let Some(new_focus) = collapse_leaf(&mut self.root, self.focused) {
            self.focused = new_focus;
        }
        self.clear_stale_zoom();
    }

    /// Move focus to the next leaf in `leaves()` order, wrapping.
    pub fn focus_next(&mut self) {
        let leaves = self.leaves();
        if leaves.is_empty() {
            return;
        }
        let idx = leaves.iter().position(|id| *id == self.focused).unwrap_or(0);
        self.focused = leaves[(idx + 1) % leaves.len()];
    }

    /// Move focus to the previous leaf in `leaves()` order, wrapping.
    pub fn focus_prev(&mut self) {
        let leaves = self.leaves();
        if leaves.is_empty() {
            return;
        }
        let idx = leaves.iter().position(|id| *id == self.focused).unwrap_or(0);
        self.focused = leaves[(idx + leaves.len() - 1) % leaves.len()];
    }

    /// The "Fill focused pane with →" action: retarget only the focused leaf.
    pub fn set_focused_content(&mut self, content: PaneContent) {
        let focused = self.focused;
        if let Some(PaneNode::Leaf { content: slot, .. }) = find_leaf_mut(&mut self.root, focused) {
            *slot = content;
        }
    }

    /// Toggle maximize on the focused leaf: set `zoomed` to the focused id, or
    /// clear it if the focused leaf is already zoomed. Reversible; never mutates
    /// the tree.
    pub fn toggle_zoom(&mut self) {
        if self.zoomed == Some(self.focused) {
            self.zoomed = None;
        } else {
            self.zoomed = Some(self.focused);
        }
    }

    /// Set the split ratio of the focused leaf's parent split, clamped to
    /// `[PANE_RATIO_MIN, PANE_RATIO_MAX]`. No-op if the focused leaf is the root.
    pub fn set_focused_ratio(&mut self, ratio: f32) {
        let clamped = clamp_ratio(ratio);
        let focused = self.focused;
        set_parent_ratio(&mut self.root, focused, clamped);
    }

    /// Replace the whole layout with a named preset.
    pub fn apply_preset(&mut self, preset: WorkspacePreset) {
        *self = match preset {
            WorkspacePreset::Single => Self::single(),
            WorkspacePreset::BoardSchematic => Self::board_schematic(),
        };
    }

    /// Drop `zoomed` if it names a leaf that no longer exists.
    fn clear_stale_zoom(&mut self) {
        if let Some(z) = self.zoomed {
            if !self.leaves().contains(&z) {
                self.zoomed = None;
            }
        }
    }
}

fn collect_leaves(node: &PaneNode, out: &mut Vec<PaneId>) {
    match node {
        PaneNode::Leaf { id, .. } => out.push(*id),
        PaneNode::Split { first, second, .. } => {
            collect_leaves(first, out);
            collect_leaves(second, out);
        }
    }
}

fn find_leaf(node: &PaneNode, id: PaneId) -> Option<&PaneNode> {
    match node {
        PaneNode::Leaf { id: leaf_id, .. } => (*leaf_id == id).then_some(node),
        PaneNode::Split { first, second, .. } => {
            find_leaf(first, id).or_else(|| find_leaf(second, id))
        }
    }
}

fn find_leaf_mut(node: &mut PaneNode, id: PaneId) -> Option<&mut PaneNode> {
    match node {
        PaneNode::Leaf { id: leaf_id, .. } => {
            if *leaf_id == id {
                Some(node)
            } else {
                None
            }
        }
        PaneNode::Split { first, second, .. } => {
            if let Some(found) = find_leaf_mut(first, id) {
                return Some(found);
            }
            find_leaf_mut(second, id)
        }
    }
}

/// The in-order first leaf id of a subtree.
fn first_leaf_id(node: &PaneNode) -> PaneId {
    match node {
        PaneNode::Leaf { id, .. } => *id,
        PaneNode::Split { first, .. } => first_leaf_id(first),
    }
}

/// Collapse the Split that directly parents leaf `target`, replacing it with the
/// sibling subtree. Returns the new focus id (in-order first leaf of the
/// reclaimed sibling) if a collapse happened.
fn collapse_leaf(node: &mut PaneNode, target: PaneId) -> Option<PaneId> {
    let PaneNode::Split { first, second, .. } = node else {
        return None;
    };
    let first_is_target = matches!(first.as_ref(), PaneNode::Leaf { id, .. } if *id == target);
    let second_is_target = matches!(second.as_ref(), PaneNode::Leaf { id, .. } if *id == target);
    if first_is_target {
        let sibling = std::mem::replace(second.as_mut(), placeholder());
        let new_focus = first_leaf_id(&sibling);
        *node = sibling;
        return Some(new_focus);
    }
    if second_is_target {
        let sibling = std::mem::replace(first.as_mut(), placeholder());
        let new_focus = first_leaf_id(&sibling);
        *node = sibling;
        return Some(new_focus);
    }
    if let Some(found) = collapse_leaf(first, target) {
        return Some(found);
    }
    collapse_leaf(second, target)
}

/// Set the ratio of the Split that directly parents leaf `target`.
fn set_parent_ratio(node: &mut PaneNode, target: PaneId, ratio: f32) -> bool {
    let PaneNode::Split {
        first,
        second,
        ratio: slot,
        ..
    } = node
    else {
        return false;
    };
    let direct = matches!(first.as_ref(), PaneNode::Leaf { id, .. } if *id == target)
        || matches!(second.as_ref(), PaneNode::Leaf { id, .. } if *id == target);
    if direct {
        *slot = ratio;
        return true;
    }
    if set_parent_ratio(first, target, ratio) {
        return true;
    }
    set_parent_ratio(second, target, ratio)
}

/// A throwaway node used only as the momentary hole in a `mem::replace` during
/// tree surgery; always immediately overwritten or dropped.
fn placeholder() -> PaneNode {
    PaneNode::Leaf {
        id: PaneId(u32::MAX),
        content: PaneContent::Board,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn content_of(layout: &WorkspaceLayout, id: PaneId) -> PaneContent {
        match find_leaf(&layout.root, id) {
            Some(PaneNode::Leaf { content, .. }) => *content,
            _ => panic!("expected leaf {id:?}"),
        }
    }

    #[test]
    fn default_equals_board_schematic_preset() {
        let mut applied = WorkspaceLayout::single();
        applied.apply_preset(WorkspacePreset::BoardSchematic);
        assert_eq!(WorkspaceLayout::default(), applied);
        // And it reproduces today's look: vertical Board|Schematic, 0.5, Board focused.
        let default = WorkspaceLayout::default();
        assert_eq!(default.leaves(), vec![PaneId(0), PaneId(1)]);
        assert_eq!(default.focused_content(), PaneContent::Board);
        match default.root {
            PaneNode::Split {
                orientation, ratio, ..
            } => {
                assert_eq!(orientation, SplitOrientation::Vertical);
                assert_eq!(ratio, 0.5);
            }
            _ => panic!("default root must be a split"),
        }
    }

    #[test]
    fn split_produces_two_leaves_and_keeps_focus() {
        let mut layout = WorkspaceLayout::single();
        assert_eq!(layout.leaves().len(), 1);
        let focused_before = layout.focused;
        layout.split_focused(SplitOrientation::Horizontal);
        let leaves = layout.leaves();
        assert_eq!(leaves.len(), 2);
        // Focus stays on the original leaf.
        assert_eq!(layout.focused, focused_before);
        assert!(leaves.contains(&focused_before));
        // New leaf inherits the focused content.
        assert_eq!(layout.focused_content(), PaneContent::Board);
        assert_eq!(content_of(&layout, leaves[1]), PaneContent::Board);
    }

    #[test]
    fn close_collapses_and_sibling_reclaims() {
        let mut layout = WorkspaceLayout::board_schematic();
        // Focus the Schematic leaf, then close it: Board must reclaim the whole space.
        layout.focused = PaneId(1);
        layout.close_focused();
        assert_eq!(layout.leaves(), vec![PaneId(0)]);
        assert!(matches!(layout.root, PaneNode::Leaf { id, .. } if id == PaneId(0)));
        // Focus lands on a valid, existing leaf.
        assert!(layout.leaves().contains(&layout.focused));
        assert_eq!(layout.focused, PaneId(0));
    }

    #[test]
    fn close_last_leaf_is_noop() {
        let mut layout = WorkspaceLayout::single();
        let before = layout.clone();
        layout.close_focused();
        assert_eq!(layout, before);
        assert_eq!(layout.leaves().len(), 1);
    }

    #[test]
    fn close_focus_moves_to_deterministic_neighbor() {
        // Board split into [Board | Schematic]; focus the Board(0) leaf and close it.
        let mut layout = WorkspaceLayout::board_schematic();
        layout.focused = PaneId(0);
        layout.close_focused();
        // Sibling (Schematic) reclaims; focus deterministically lands on it.
        assert_eq!(layout.leaves(), vec![PaneId(1)]);
        assert_eq!(layout.focused, PaneId(1));
    }

    #[test]
    fn focus_cycles_in_order_and_wraps() {
        let mut layout = WorkspaceLayout::single();
        layout.split_focused(SplitOrientation::Vertical); // ids: 0 (focused), 1
        layout.focused = PaneId(0);
        let order = layout.leaves();
        assert_eq!(order, vec![PaneId(0), PaneId(1)]);

        layout.focus_next();
        assert_eq!(layout.focused, PaneId(1));
        layout.focus_next(); // wraps
        assert_eq!(layout.focused, PaneId(0));
        layout.focus_prev(); // wraps backward
        assert_eq!(layout.focused, PaneId(1));
        layout.focus_prev();
        assert_eq!(layout.focused, PaneId(0));
    }

    #[test]
    fn set_content_changes_only_focused_leaf() {
        let mut layout = WorkspaceLayout::board_schematic();
        layout.focused = PaneId(0);
        layout.set_focused_content(PaneContent::Schematic);
        assert_eq!(content_of(&layout, PaneId(0)), PaneContent::Schematic);
        // The other leaf is untouched.
        assert_eq!(content_of(&layout, PaneId(1)), PaneContent::Schematic);
        // (Both Schematic now — but pane 1 was Schematic before and pane 0 changed.)
        layout.set_focused_content(PaneContent::Board);
        assert_eq!(content_of(&layout, PaneId(0)), PaneContent::Board);
        assert_eq!(content_of(&layout, PaneId(1)), PaneContent::Schematic);
    }

    #[test]
    fn zoom_is_reversible_and_preserves_tree() {
        let mut layout = WorkspaceLayout::board_schematic();
        let tree_before = layout.root.clone();
        let leaves_before = layout.leaves();
        assert_eq!(layout.zoomed, None);

        layout.toggle_zoom();
        assert_eq!(layout.zoomed, Some(layout.focused));
        // Tree and leaves are untouched by zoom.
        assert_eq!(layout.root, tree_before);
        assert_eq!(layout.leaves(), leaves_before);

        layout.toggle_zoom();
        assert_eq!(layout.zoomed, None);
        assert_eq!(layout.root, tree_before);
        assert_eq!(layout.leaves(), leaves_before);
    }

    #[test]
    fn close_clears_stale_zoom() {
        let mut layout = WorkspaceLayout::board_schematic();
        layout.focused = PaneId(1);
        layout.toggle_zoom();
        assert_eq!(layout.zoomed, Some(PaneId(1)));
        layout.close_focused(); // removes leaf 1, which was zoomed
        assert_eq!(layout.zoomed, None);
    }

    #[test]
    fn presets_produce_expected_trees() {
        let mut layout = WorkspaceLayout::default();
        layout.apply_preset(WorkspacePreset::Single);
        assert_eq!(layout, WorkspaceLayout::single());
        assert_eq!(layout.leaves(), vec![PaneId(0)]);
        assert_eq!(layout.focused_content(), PaneContent::Board);

        layout.apply_preset(WorkspacePreset::BoardSchematic);
        assert_eq!(layout, WorkspaceLayout::board_schematic());
        assert_eq!(layout.leaves(), vec![PaneId(0), PaneId(1)]);
        assert_eq!(layout.focused, PaneId(0));
    }

    #[test]
    fn ratio_writes_clamp_to_range() {
        let mut layout = WorkspaceLayout::board_schematic();
        layout.focused = PaneId(0);

        layout.set_focused_ratio(5.0);
        assert_eq!(root_ratio(&layout), PANE_RATIO_MAX);

        layout.set_focused_ratio(-1.0);
        assert_eq!(root_ratio(&layout), PANE_RATIO_MIN);

        layout.set_focused_ratio(0.42);
        assert_eq!(root_ratio(&layout), 0.42);
    }

    #[test]
    fn pane_ids_are_unique_and_monotonic() {
        let mut layout = WorkspaceLayout::single();
        let mut seen = vec![layout.focused];
        for _ in 0..5 {
            let before = layout.next_id;
            layout.split_focused(SplitOrientation::Vertical);
            assert_eq!(layout.next_id, before + 1, "counter advances by exactly one");
            // The id just handed out equals the pre-alloc counter value.
            let newest = PaneId(before);
            assert!(!seen.contains(&newest), "ids must be unique: {newest:?}");
            assert!(layout.leaves().contains(&newest), "new leaf must be in the tree");
            seen.push(newest);
        }
        // Ids handed out were 1,2,3,4,5 in order (0 was the initial leaf).
        assert_eq!(seen, vec![
            PaneId(0),
            PaneId(1),
            PaneId(2),
            PaneId(3),
            PaneId(4),
            PaneId(5),
        ]);
    }

    fn root_ratio(layout: &WorkspaceLayout) -> f32 {
        match layout.root {
            PaneNode::Split { ratio, .. } => ratio,
            _ => panic!("root must be a split"),
        }
    }
}
