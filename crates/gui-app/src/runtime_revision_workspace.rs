use crate::console_accessibility::{AccessibilityAnnouncement, AnnouncementPriority};
use crate::{Runtime, append_gui_diagnostic_line};
use datum_gui_protocol::{
    ApplicationFocus, PaneContent, RevisionNavContextAction, RevisionNavContextMenu,
    RevisionNavEntry, RevisionPane, RevisionSurface, SplitOrientation,
};
use datum_gui_render::HitTarget;

fn revision_click_opens(
    prior: Option<(RevisionNavEntry, std::time::Instant)>,
    entry: RevisionNavEntry,
    now: std::time::Instant,
) -> bool {
    prior.is_some_and(|(prior_entry, at)| {
        prior_entry == entry && now.duration_since(at) <= Runtime::REVISION_DOUBLE_CLICK_WINDOW
    })
}

/// Apply consumer-only revision workspace gestures. `None` leaves the target
/// for the existing editor dispatcher; no arm creates an authority record or a
/// Design operation.
impl Runtime {
    const REVISION_DOUBLE_CLICK_WINDOW: std::time::Duration = std::time::Duration::from_millis(500);
    const MIN_USABLE_PANE_WIDTH_PX: f32 = 280.0;

    pub(super) fn apply_revision_hit(&mut self, target: &HitTarget) -> Option<bool> {
        match target {
            HitTarget::SelectRevisionNavEntry(entry) => self.select_revision_entry(*entry),
            HitTarget::RevisionNavContextAction(action) => {
                self.apply_revision_context_action(*action)
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

    fn select_revision_entry(&mut self, entry: RevisionNavEntry) {
        let now = std::time::Instant::now();
        let opens = revision_click_opens(self.revision_last_click, entry, now);
        self.revision_last_click = (!opens).then_some((entry, now));
        let revision = &mut self.session.workspace_mut().ui.revision;
        revision.selected_entry = Some(entry);
        revision.context_menu = None;
        revision.screen_reader_announcement = Some(if opens {
            format!("{} activated; opening beside", entry.label())
        } else {
            format!(
                "{} selected; double-click or press Enter to open beside",
                entry.label()
            )
        });
        if opens {
            self.open_revision_surface(entry.surface());
        } else {
            self.invalidate_frame();
        }
    }

    pub(super) fn open_selected_revision(&mut self) -> bool {
        let Some(entry) = self.workspace().ui.revision.selected_entry else {
            return false;
        };
        self.revision_last_click = None;
        self.open_revision_surface(entry.surface());
        true
    }

    fn open_revision_surface(&mut self, surface: RevisionSurface) {
        let content = PaneContent::Revision(RevisionPane::Surface(surface));
        let total_width = self.current_layout().viewport.width;
        let ui = &mut self.session.workspace_mut().ui;
        let pane = ui
            .layout
            .retarget_first_revision_surface(content)
            .unwrap_or_else(|| {
                let orientation = if total_width >= Self::MIN_USABLE_PANE_WIDTH_PX * 3.0 {
                    SplitOrientation::Vertical
                } else {
                    SplitOrientation::Horizontal
                };
                ui.layout.open_beside_root(content, orientation, 0.66, true)
            });
        ui.focus = ApplicationFocus::Editor(pane);
        ui.revision.announce_open(surface);
        self.invalidate_frame();
    }

    pub(super) fn open_revision_context_menu_at_cursor(&mut self) -> bool {
        let Some((x, y)) = self.last_cursor_pos else {
            return false;
        };
        let entry = match self.prepared_scene().hit_test(x, y).cloned() {
            Some(HitTarget::SelectRevisionNavEntry(entry)) => entry,
            _ => return false,
        };
        let ui = &mut self.session.workspace_mut().ui;
        ui.active_menu = None;
        ui.marking_menu = None;
        ui.revision.selected_entry = Some(entry);
        ui.revision.context_menu = Some(RevisionNavContextMenu {
            entry,
            anchor_x_px: x.round() as i32,
            anchor_y_px: y.round() as i32,
        });
        self.set_application_focus(ApplicationFocus::Overlay);
        self.invalidate_frame();
        true
    }

    pub(super) fn revision_context_menu_active(&self) -> bool {
        self.workspace().ui.revision.context_menu.is_some()
    }

    pub(super) fn dismiss_revision_context_menu(&mut self) -> bool {
        if self
            .session
            .workspace_mut()
            .ui
            .revision
            .context_menu
            .take()
            .is_none()
        {
            return false;
        }
        let pane = self.workspace().ui.layout.focused;
        self.set_application_focus(ApplicationFocus::Editor(pane));
        self.invalidate_frame();
        true
    }

    fn apply_revision_context_action(&mut self, action: RevisionNavContextAction) {
        let entry = self
            .workspace()
            .ui
            .revision
            .context_menu
            .map(|menu| menu.entry)
            .or(self.workspace().ui.revision.selected_entry);
        self.dismiss_revision_context_menu();
        let Some(entry) = entry else {
            return;
        };
        match action {
            RevisionNavContextAction::OpenBeside => self.open_revision_surface(entry.surface()),
            RevisionNavContextAction::Properties => {
                self.session.workspace_mut().ui.revision.selected_entry = Some(entry);
                self.invalidate_frame();
            }
            RevisionNavContextAction::ShowImpact | RevisionNavContextAction::CompareToBaseline => {
                self.open_revision_surface(RevisionSurface::Impact)
            }
            RevisionNavContextAction::PrepareRevision => {
                self.open_revision_surface(RevisionSurface::Change)
            }
        }
    }

    pub(super) fn handle_layer_scroll(&mut self, scroll_lines: f32) -> bool {
        if scroll_lines.abs() <= 0.01 {
            return false;
        }
        let Some((x, y)) = self.last_cursor_pos else {
            return false;
        };
        let over_layers = self.prepared_scene().hit_regions.iter().any(|region| {
            region.target == HitTarget::LayerScrollRegion && region.rect.contains(x, y)
        });
        if !over_layers {
            return false;
        }
        let layer_count = self.workspace().scene.layers.len();
        let offset = &mut self.session.workspace_mut().ui.filters.layer_scroll_offset;
        let step = scroll_lines.abs().ceil() as usize;
        *offset = if scroll_lines < 0.0 {
            offset
                .saturating_add(step)
                .min(layer_count.saturating_sub(1))
        } else {
            offset.saturating_sub(step)
        };
        self.invalidate_frame();
        true
    }

    pub(super) fn run_revision_nav_smoke(&mut self) -> anyhow::Result<()> {
        let target = HitTarget::SelectRevisionNavEntry(RevisionNavEntry::Baselines);
        let region = self
            .prepared_scene()
            .hit_regions
            .iter()
            .find(|region| region.target == target)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("Baselines Navigator hit region is missing"))?;
        self.last_cursor_pos = Some((
            region.rect.x + region.rect.width * 0.5,
            region.rect.y + region.rect.height * 0.5,
        ));
        let leaves_before = self.workspace().ui.layout.leaves().len();
        let select_started = std::time::Instant::now();
        anyhow::ensure!(self.handle_primary_click(), "single click was not handled");
        let select_elapsed = select_started.elapsed();
        anyhow::ensure!(
            self.workspace().ui.layout.leaves().len() == leaves_before,
            "single click opened a pane instead of selecting only"
        );
        anyhow::ensure!(
            self.workspace().ui.revision.selected_entry == Some(RevisionNavEntry::Baselines),
            "single click did not establish exclusive Baselines selection"
        );

        let open_started = std::time::Instant::now();
        anyhow::ensure!(self.handle_primary_click(), "double click was not handled");
        let open_elapsed = open_started.elapsed();
        anyhow::ensure!(
            self.workspace().ui.layout.leaves().len() == leaves_before + 1,
            "double click did not open exactly one sibling pane"
        );
        self.session.workspace_mut().ui.filters.layer_scroll_offset = usize::MAX;
        self.invalidate_frame();
        let measurement = format!(
            "Navigator input select={}us double-click-open={}us retained-resolves={}",
            select_elapsed.as_micros(),
            open_elapsed.as_micros(),
            datum_gui_render::retained_scene_resolve_count()
        );
        append_gui_diagnostic_line(&measurement);
        eprintln!("[datum-revision-input] {measurement}");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use datum_gui_protocol::RevisionNavEntry;

    #[test]
    fn arm_is_reversible_consumer_state_and_creates_no_record() {
        let complete = include_str!("runtime_revision_workspace.rs");
        let source = complete.split("#[cfg(test)]").next().unwrap_or(complete);
        assert!(!source.contains("AuthorityRecord"));
        assert!(!source.contains("SessionCommand"));
        assert!(!source.contains("prepare_revision_design_commit"));
        assert!(matches!(
            HitTarget::SelectRevisionNavEntry(RevisionNavEntry::Releases),
            HitTarget::SelectRevisionNavEntry(_)
        ));
    }

    #[test]
    fn one_click_selects_and_only_same_row_double_click_opens() {
        let at = std::time::Instant::now();
        assert!(!revision_click_opens(None, RevisionNavEntry::Changes, at));
        assert!(!revision_click_opens(
            Some((RevisionNavEntry::Baselines, at)),
            RevisionNavEntry::Changes,
            at + std::time::Duration::from_millis(100)
        ));
        assert!(revision_click_opens(
            Some((RevisionNavEntry::Changes, at)),
            RevisionNavEntry::Changes,
            at + std::time::Duration::from_millis(499)
        ));
        assert!(!revision_click_opens(
            Some((RevisionNavEntry::Changes, at)),
            RevisionNavEntry::Changes,
            at + std::time::Duration::from_millis(501)
        ));
    }
}
