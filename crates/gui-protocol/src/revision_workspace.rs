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

/// Session presentation over resolver-owned revision truth. It is deliberately
/// incapable of carrying an authority record or a Design mutation.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RevisionWorkspaceUiState {
    pub active_surface: Option<RevisionSurface>,
    pub selected_witness: Option<String>,
    pub issuance_armed: bool,
    pub screen_reader_announcement: Option<String>,
}

impl RevisionWorkspaceUiState {
    pub fn open(&mut self, surface: RevisionSurface) {
        self.active_surface = Some(surface);
        self.issuance_armed = false;
        self.screen_reader_announcement =
            Some(format!("{} opened beside the workspace", surface.label()));
    }

    pub fn close(&mut self) {
        self.active_surface = None;
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
        state.open(RevisionSurface::Release);
        assert_eq!(state.active_surface, Some(RevisionSurface::Release));
        assert!(!state.issuance_armed);
        state.issuance_armed = true;
        state.close();
        assert_eq!(state.active_surface, None);
        assert!(!state.issuance_armed);
    }
}
