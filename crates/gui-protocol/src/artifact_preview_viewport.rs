use super::{
    ProductionArtifactDetail, ReviewWorkspaceState, SessionCommand, SessionCommandResult,
    SessionEvent,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtifactPreviewViewportState {
    pub zoom_ppm: u32,
    pub pan_x_ppm: i32,
    pub pan_y_ppm: i32,
    pub show_geometry: bool,
    pub show_drills: bool,
}

impl Default for ArtifactPreviewViewportState {
    fn default() -> Self {
        Self {
            zoom_ppm: 1_000_000,
            pan_x_ppm: 0,
            pan_y_ppm: 0,
            show_geometry: true,
            show_drills: true,
        }
    }
}

impl ReviewWorkspaceState {
    pub fn focus_production_artifact(&mut self, artifact_id: &str) -> bool {
        let Some(artifact) = self
            .production
            .output_jobs
            .iter()
            .flat_map(|job| job.artifacts.iter())
            .find(|artifact| artifact.artifact_id == artifact_id)
            .cloned()
        else {
            return false;
        };
        let already_focused = self
            .production
            .focused_artifact
            .as_ref()
            .is_some_and(|focused| focused.artifact_id == artifact.artifact_id);
        let focused_file = artifact.files.first().cloned();
        self.production.focused_artifact = Some(ProductionArtifactDetail {
            artifact_id: artifact.artifact_id,
            kind: artifact.kind,
            output_dir: artifact.output_dir,
            validation_state: "not_validated".to_string(),
            file_count: artifact.file_count,
            files: artifact.files,
            focused_file,
            focused_preview: None,
            production_projection_count: artifact.production_projection_count,
            production_projections: artifact.production_projections,
        });
        !already_focused
    }

    pub fn focus_production_artifact_file(&mut self, path: &str) -> bool {
        let Some(artifact) = self.production.focused_artifact.as_mut() else {
            return false;
        };
        let Some(file) = artifact
            .files
            .iter()
            .find(|file| file.path == path)
            .cloned()
        else {
            return false;
        };
        let already_focused = artifact
            .focused_file
            .as_ref()
            .is_some_and(|focused| focused.path == file.path);
        artifact.focused_file = Some(file);
        artifact.focused_preview = None;
        self.ui.artifact_preview = ArtifactPreviewViewportState::default();
        !already_focused
    }

    pub fn zoom_artifact_preview(&mut self, zoom_in: bool) -> bool {
        let current = self.ui.artifact_preview.zoom_ppm;
        let next = if zoom_in {
            ((current as u64 * 6) / 5).min(8_000_000) as u32
        } else {
            ((current as u64 * 5) / 6).max(250_000) as u32
        };
        if next == current {
            return false;
        }
        self.ui.artifact_preview.zoom_ppm = next;
        true
    }

    pub fn reset_artifact_preview_viewport(&mut self) -> bool {
        if self.ui.artifact_preview == ArtifactPreviewViewportState::default() {
            return false;
        }
        self.ui.artifact_preview = ArtifactPreviewViewportState::default();
        true
    }

    pub fn pan_artifact_preview(&mut self, delta_x_ppm: i32, delta_y_ppm: i32) -> bool {
        if delta_x_ppm == 0 && delta_y_ppm == 0 {
            return false;
        }
        let current = &self.ui.artifact_preview;
        let next_x = (current.pan_x_ppm + delta_x_ppm).clamp(-2_000_000, 2_000_000);
        let next_y = (current.pan_y_ppm + delta_y_ppm).clamp(-2_000_000, 2_000_000);
        if next_x == current.pan_x_ppm && next_y == current.pan_y_ppm {
            return false;
        }
        self.ui.artifact_preview.pan_x_ppm = next_x;
        self.ui.artifact_preview.pan_y_ppm = next_y;
        true
    }
}

impl super::LiveDesignSession {
    pub(crate) fn apply_artifact_preview_command(
        &mut self,
        command: SessionCommand,
    ) -> Option<SessionCommandResult> {
        let handled = match command {
            SessionCommand::ZoomArtifactPreviewIn => self.workspace.zoom_artifact_preview(true),
            SessionCommand::ZoomArtifactPreviewOut => self.workspace.zoom_artifact_preview(false),
            SessionCommand::PanArtifactPreview {
                delta_x_ppm,
                delta_y_ppm,
            } => self
                .workspace
                .pan_artifact_preview(delta_x_ppm, delta_y_ppm),
            SessionCommand::ResetArtifactPreviewViewport => {
                self.workspace.reset_artifact_preview_viewport()
            }
            SessionCommand::ToggleArtifactPreviewGeometry => {
                self.workspace.ui.artifact_preview.show_geometry =
                    !self.workspace.ui.artifact_preview.show_geometry;
                true
            }
            SessionCommand::ToggleArtifactPreviewDrills => {
                self.workspace.ui.artifact_preview.show_drills =
                    !self.workspace.ui.artifact_preview.show_drills;
                true
            }
            _ => return None,
        };
        Some(SessionCommandResult {
            handled,
            events: if handled {
                vec![SessionEvent::FrameChanged]
            } else {
                Vec::new()
            },
        })
    }
}
