use datum_gui_protocol::{ConsoleFeedbackSource, DockTab, SessionCommand};
use datum_gui_render::HitTarget;

use super::Runtime;

impl Runtime {
    pub(super) fn select_artifact_preview_hit_target(
        &mut self,
        target: &HitTarget,
    ) -> Option<bool> {
        let (command, event) = match target {
            HitTarget::ArtifactPreviewZoomIn => (
                SessionCommand::ZoomArtifactPreviewIn,
                "artifact preview zoom in",
            ),
            HitTarget::ArtifactPreviewZoomOut => (
                SessionCommand::ZoomArtifactPreviewOut,
                "artifact preview zoom out",
            ),
            HitTarget::ArtifactPreviewReset => (
                SessionCommand::ResetArtifactPreviewViewport,
                "artifact preview reset",
            ),
            HitTarget::ToggleArtifactPreviewGeometry => (
                SessionCommand::ToggleArtifactPreviewGeometry,
                "artifact preview geometry toggle",
            ),
            HitTarget::ToggleArtifactPreviewDrills => (
                SessionCommand::ToggleArtifactPreviewDrills,
                "artifact preview drill toggle",
            ),
            _ => return None,
        };
        let handled = self.dispatch_session_command(command);
        if handled {
            self.log_console_echo(ConsoleFeedbackSource::Production, event.to_string());
        }
        Some(handled)
    }

    pub(super) fn handle_artifact_preview_pan_drag(
        &mut self,
        previous: (f32, f32),
        next: (f32, f32),
    ) -> bool {
        let Some(region) = self.presented_hits.regions().iter().rev().find(|region| {
            matches!(region.target, HitTarget::ArtifactPreviewViewport)
                && region.rect.contains(previous.0, previous.1)
        }) else {
            return false;
        };
        let delta_x_ppm =
            (((next.0 - previous.0) / region.rect.width.max(1.0)) * 1_000_000.0).round() as i32;
        let delta_y_ppm =
            (((next.1 - previous.1) / region.rect.height.max(1.0)) * 1_000_000.0).round() as i32;
        self.dispatch_session_command(SessionCommand::PanArtifactPreview {
            delta_x_ppm,
            delta_y_ppm,
        })
    }

    pub(super) fn handle_dock_scroll(&mut self, scroll_lines: f32) -> bool {
        if self.workspace().ui.active_dock_tab != Some(DockTab::Terminal)
            || !scroll_lines.is_finite()
            || scroll_lines.abs() <= 0.01
        {
            return false;
        }
        let total = self.terminal_sessions.active_render_row_count();
        let visible = usize::from(self.terminal_screen_geometry().rows);
        let offset = &mut self.session.workspace_mut().ui.terminal.scroll_offset;
        let before = *offset;
        // Scrollback counts backward from the live screen, one row per wheel
        // event. Share effective bounds without changing terminal row authority.
        let next = datum_gui_viewport::scroll::scrolled_row_offset(
            before,
            total,
            visible,
            -scroll_lines.signum(),
        );
        *offset = next;
        // Compare with stored state, so extent reconciliation is damage too.
        let changed = next != before;
        if changed {
            self.invalidate_frame();
        }
        changed
    }
}

#[cfg(test)]
#[path = "native_terminal_scroll_tests.rs"]
mod native_terminal_scroll_tests;
