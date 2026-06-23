use datum_gui_protocol::{DockTab, SessionCommand};
use datum_gui_render::{HitTarget, PreparedScene};

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
            self.log_review_event(event.to_string());
        }
        Some(handled)
    }

    pub(super) fn handle_outputs_scroll(&mut self, scroll_lines: f32) -> bool {
        if self.workspace().ui.active_dock_tab != Some(DockTab::Outputs) {
            return false;
        }
        let command = if scroll_lines > 0.0 {
            SessionCommand::ZoomArtifactPreviewIn
        } else {
            SessionCommand::ZoomArtifactPreviewOut
        };
        self.dispatch_session_command(command)
    }

    pub(super) fn handle_artifact_preview_pan_drag(
        &mut self,
        prepared: &PreparedScene,
        previous: (f32, f32),
        next: (f32, f32),
    ) -> bool {
        let Some(region) = prepared.hit_regions.iter().rev().find(|region| {
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
        let Some(active) = self.workspace().ui.active_dock_tab else {
            return false;
        };
        let delta = if scroll_lines > 0.0 { 1_usize } else { 0_usize };
        let ui = &mut self.session.workspace_mut().ui;
        match active {
            DockTab::Terminal => {
                if scroll_lines > 0.0 {
                    let max = ui.terminal.lines.len();
                    ui.terminal.scroll_offset = (ui.terminal.scroll_offset + delta).min(max);
                } else {
                    ui.terminal.scroll_offset = ui.terminal.scroll_offset.saturating_sub(1);
                }
            }
            DockTab::Assistant => {
                if scroll_lines > 0.0 {
                    let max = ui.assistant.transcript.len();
                    ui.assistant.scroll_offset = (ui.assistant.scroll_offset + delta).min(max);
                } else {
                    ui.assistant.scroll_offset = ui.assistant.scroll_offset.saturating_sub(1);
                }
            }
            DockTab::Outputs => return self.handle_outputs_scroll(scroll_lines),
        }
        self.invalidate_frame();
        true
    }
}
