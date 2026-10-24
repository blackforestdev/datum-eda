//! Content and keyboard ownership identities shared by every pane.
use crate::RevisionPane;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaneContent {
    Board,
    Schematic,
    Library,
    Revision(RevisionPane),
}

impl PaneContent {
    /// The complementary content, used so a split pairs Board with Schematic
    /// (and vice-versa) rather than duplicating the focused pane's content.
    /// Trivial while only two variants exist; revisit when more land.
    pub fn complement(self) -> Self {
        match self {
            PaneContent::Board => PaneContent::Schematic,
            PaneContent::Schematic => PaneContent::Board,
            PaneContent::Library | PaneContent::Revision(_) => PaneContent::Board,
        }
    }
}

/// Stable identifier for a leaf pane, allocated monotonically by the owning
/// `WorkspaceLayout`. Ids are never reused within a layout's lifetime.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PaneId(pub u32);

/// The selected interactive surface for the application window.
///
/// Carrying the editor `PaneId` makes editor and terminal selection equally
/// explicit; no renderer or input path may infer focus from dock visibility or
/// from `WorkspaceLayout::focused` alone.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApplicationFocus {
    Navigator,
    Inspector,
    Editor(PaneId),
    Terminal,
    Overlay,
}

impl Default for ApplicationFocus {
    fn default() -> Self {
        Self::Editor(PaneId(0))
    }
}

impl ApplicationFocus {
    pub fn is_terminal(self) -> bool {
        self == Self::Terminal
    }
}
