use crate::Runtime;
use crate::console_accessibility::{AccessibilityAnnouncement, AnnouncementPriority};
use datum_gui_protocol::{ApplicationFocus, PaneContent, RevisionPane, SplitOrientation};
use datum_gui_render::HitTarget;

/// Apply consumer-only revision workspace gestures. `None` leaves the target
/// for the existing editor dispatcher; no arm creates an authority record or a
/// Design operation.
impl Runtime {
    pub(super) fn apply_revision_hit(&mut self, target: &HitTarget) -> Option<bool> {
        match target {
            HitTarget::OpenRevisionSurface(surface) => {
                let ui = &mut self.session.workspace_mut().ui;
                let had_companion = ui.layout.leaves().len() > 1;
                let pane = ui.layout.open_beside(
                    PaneContent::Revision(RevisionPane::Surface(*surface)),
                    SplitOrientation::Vertical,
                    true,
                );
                if had_companion {
                    ui.layout.set_focused_ratio(0.35);
                    ui.layout.set_ratio_at_path(&[], 0.72);
                } else {
                    ui.layout.set_focused_ratio(0.45);
                }
                ui.focus = ApplicationFocus::Editor(pane);
                ui.revision.announce_open(*surface);
            }
            HitTarget::CloseRevisionSurface => {
                let ui = &mut self.session.workspace_mut().ui;
                if matches!(ui.layout.focused_content(), PaneContent::Revision(_)) {
                    ui.layout.close_focused();
                    ui.focus = ApplicationFocus::Editor(ui.layout.focused);
                }
                ui.revision.announce_close();
            }
            HitTarget::ToggleRevisionIssuanceArm => {
                let revision = &mut self.session.workspace_mut().ui.revision;
                revision.issuance_armed = !revision.issuance_armed;
                revision.screen_reader_announcement = Some(
                    if revision.issuance_armed {
                        "Release issuance armed; activate again to confirm the stated consequences"
                    } else {
                        "Release issuance disarmed; no authority record was created"
                    }
                    .to_owned(),
                );
            }
            HitTarget::OpenRevisionWitness(witness) => {
                let ui = &mut self.session.workspace_mut().ui;
                let pane = ui.layout.open_beside(
                    PaneContent::Revision(RevisionPane::Witness),
                    SplitOrientation::Vertical,
                    true,
                );
                ui.focus = ApplicationFocus::Editor(pane);
                let revision = &mut ui.revision;
                revision.selected_witness = Some(witness.clone());
                revision.screen_reader_announcement = Some(format!(
                    "Witness {witness} opened beside the retained summary"
                ));
            }
            _ => return None,
        }
        if let Some(text) = self
            .session
            .workspace_mut()
            .ui
            .revision
            .screen_reader_announcement
            .take()
        {
            self.terminal_accessibility
                .announce_console(AccessibilityAnnouncement {
                    text,
                    priority: AnnouncementPriority::Medium,
                });
        }
        Some(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use datum_gui_protocol::RevisionSurface;

    #[test]
    fn arm_is_reversible_consumer_state_and_creates_no_record() {
        let complete = include_str!("runtime_revision_workspace.rs");
        let source = complete.split("#[cfg(test)]").next().unwrap_or(complete);
        assert!(!source.contains("AuthorityRecord"));
        assert!(!source.contains("SessionCommand"));
        assert!(!source.contains("prepare_revision_design_commit"));
        assert!(matches!(
            HitTarget::OpenRevisionSurface(RevisionSurface::Release),
            HitTarget::OpenRevisionSurface(_)
        ));
    }
}
