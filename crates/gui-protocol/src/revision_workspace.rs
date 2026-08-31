/// The complete and exclusive REV-I10 visible-surface inventory.
pub const REVISION_VISIBLE_DISPOSITIONS: [&str; 6] = [
    "revision-navigator",
    "release-readiness",
    "impact-summary",
    "change-hierarchy",
    "release-confirmation",
    "reproduction-evidence",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RevisionSurface {
    Release,
    Impact,
    Change,
    Evidence,
}

/// One selectable row in the permanent Revision Navigator. Row identity is
/// deliberately distinct from pane content: Releases and Controlled Documents
/// currently share the cohesive Release projection but remain separate,
/// single-select Navigator entries.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RevisionNavEntry {
    Changes,
    Baselines,
    Releases,
    ControlledDocuments,
    Evidence,
}

impl RevisionNavEntry {
    pub const ALL: [Self; 5] = [
        Self::Changes,
        Self::Baselines,
        Self::Releases,
        Self::ControlledDocuments,
        Self::Evidence,
    ];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Changes => "Changes",
            Self::Baselines => "Baselines",
            Self::Releases => "Releases",
            Self::ControlledDocuments => "Controlled Documents",
            Self::Evidence => "Evidence",
        }
    }

    pub const fn surface(self) -> RevisionSurface {
        match self {
            Self::Changes => RevisionSurface::Change,
            Self::Baselines => RevisionSurface::Impact,
            Self::Releases | Self::ControlledDocuments => RevisionSurface::Release,
            Self::Evidence => RevisionSurface::Evidence,
        }
    }
}

/// Conventional local-entry menu anchored to a secondary click in the
/// Navigator. Screen coordinates are consumer state and never authority.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RevisionNavContextMenu {
    pub entry: RevisionNavEntry,
    pub anchor_x_px: i32,
    pub anchor_y_px: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RevisionNavContextAction {
    OpenBeside,
    Properties,
    ShowImpact,
    PrepareRevision,
    CompareToBaseline,
}

/// Content identity for a real recursively tiled Revision pane. Witness is a
/// separate pane so the summary and canonical tree remain visible together.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RevisionPane {
    Surface(RevisionSurface),
    Witness,
}

/// Session presentation over resolver-owned revision truth. It is deliberately
/// incapable of carrying an authority record or a Design mutation.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RevisionWorkspaceUiState {
    pub selected_entry: Option<RevisionNavEntry>,
    pub context_menu: Option<RevisionNavContextMenu>,
    pub selected_witness: Option<String>,
    pub issuance_armed: bool,
    pub screen_reader_announcement: Option<String>,
}

impl RevisionWorkspaceUiState {
    pub fn announce_open(&mut self, surface: RevisionSurface) {
        self.context_menu = None;
        self.issuance_armed = false;
        self.screen_reader_announcement =
            Some(format!("{} opened beside the workspace", surface.label()));
    }

    pub fn announce_close(&mut self) {
        self.context_menu = None;
        self.issuance_armed = false;
        self.screen_reader_announcement =
            Some("Revision pane closed; focus returned to Navigator".to_owned());
    }
}

impl RevisionSurface {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Release => "Release",
            Self::Impact => "Impact",
            Self::Change => "Change",
            Self::Evidence => "Evidence",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn visible_inventory_is_closed_and_bidirectional() {
        assert_eq!(REVISION_VISIBLE_DISPOSITIONS.len(), 6);
        assert_eq!(REVISION_VISIBLE_DISPOSITIONS[0], "revision-navigator");
        assert_eq!(REVISION_VISIBLE_DISPOSITIONS[5], "reproduction-evidence");
        assert_eq!(
            [
                RevisionSurface::Release,
                RevisionSurface::Impact,
                RevisionSurface::Change,
                RevisionSurface::Evidence
            ]
            .map(RevisionSurface::label),
            ["Release", "Impact", "Change", "Evidence"]
        );
    }

    #[test]
    fn projection_carries_no_authority_or_design_mutation() {
        let mut state = RevisionWorkspaceUiState::default();
        state.announce_open(RevisionSurface::Release);
        assert!(!state.issuance_armed);
        state.issuance_armed = true;
        state.announce_close();
        assert!(!state.issuance_armed);
    }
}
