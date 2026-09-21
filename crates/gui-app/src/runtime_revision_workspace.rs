use crate::Runtime;
use crate::console_accessibility::{AccessibilityAnnouncement, AnnouncementPriority};
use datum_gui_protocol::{ApplicationFocus, PaneContent, RevisionPane, SplitOrientation};
use datum_gui_render::HitTarget;

/// Apply consumer-only revision workspace gestures. `None` leaves the target
/// for the existing editor dispatcher; no arm creates an authority record or a
/// Design operation.
impl Runtime {
    const MIN_USABLE_PANE_WIDTH_PX: f32 = 280.0;

    pub(super) fn apply_revision_hit(&mut self, target: &HitTarget) -> Option<bool> {
        match target {
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
                let focused_width = self
                    .current_layout()
                    .viewport_panes(&self.workspace().ui.layout)
                    .focused_pane()
                    .rect
                    .frame
                    .width;
                let ui = &mut self.session.workspace_mut().ui;
                let pane = ui.layout.open_beside(
                    PaneContent::Revision(RevisionPane::Witness),
                    if focused_width >= Self::MIN_USABLE_PANE_WIDTH_PX * 2.0 {
                        SplitOrientation::Vertical
                    } else {
                        SplitOrientation::Horizontal
                    },
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

    /// None is outside Layers; Some(false) consumes a local no-op.
    pub(super) fn handle_layer_scroll(&mut self, scroll_lines: f32) -> Option<bool> {
        let (x, y) = self.last_cursor_pos?;
        // Board wheel input must not force Layers preparation.
        if !self.current_layout().left_sidebar.contains(x, y) {
            return None;
        }
        let regions = self.presented_hits.regions();
        if !regions.iter().any(|region| {
            region.target == HitTarget::LayerScrollRegion && region.rect.contains(x, y)
        }) {
            return None;
        }
        let visible = regions
            .iter()
            .filter(|region| matches!(region.target, HitTarget::ToggleLayer(_)))
            .count();
        let total = self.workspace().scene.layers.len();
        let offset = &mut self.session.workspace_mut().ui.filters.layer_scroll_offset;
        let before = datum_gui_viewport::scroll::scrolled_row_offset(*offset, total, visible, 0.0);
        let next =
            datum_gui_viewport::scroll::scrolled_row_offset(before, total, visible, scroll_lines);
        *offset = next;
        let changed = next != before;
        if changed {
            self.invalidate_frame();
        }
        Some(changed)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn arm_is_reversible_consumer_state_and_creates_no_record() {
        let complete = include_str!("runtime_revision_workspace.rs");
        let source = complete.split("#[cfg(test)]").next().unwrap_or(complete);
        assert!(!source.contains("AuthorityRecord"));
        assert!(!source.contains("SessionCommand"));
        assert!(!source.contains("prepare_revision_design_commit"));
    }
}
