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
    pub selected_witness: Option<String>,
    pub issuance_armed: bool,
    pub screen_reader_announcement: Option<String>,
}

impl RevisionWorkspaceUiState {
    pub fn announce_open(&mut self, surface: RevisionSurface) {
        self.issuance_armed = false;
        self.screen_reader_announcement =
            Some(format!("{} opened beside the workspace", surface.label()));
    }

    pub fn announce_close(&mut self) {
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
