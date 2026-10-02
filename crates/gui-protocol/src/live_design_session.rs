//! Shared workspace command dispatch; one session owns consumer state.
use crate::*;

impl LiveDesignSession {
    pub fn new(workspace: ReviewWorkspaceState) -> Self {
        Self { workspace }
    }

    pub fn workspace(&self) -> &ReviewWorkspaceState {
        &self.workspace
    }

    pub fn workspace_mut(&mut self) -> &mut ReviewWorkspaceState {
        &mut self.workspace
    }

    pub fn apply(&mut self, command: SessionCommand) -> SessionCommandResult {
        match command {
            SessionCommand::SetTool(tool) => {
                if self.workspace.set_tool(tool) {
                    SessionCommandResult {
                        handled: true,
                        events: vec![SessionEvent::ToolChanged(tool), SessionEvent::FrameChanged],
                    }
                } else {
                    SessionCommandResult {
                        handled: false,
                        events: Vec::new(),
                    }
                }
            }
            SessionCommand::BeginAuthoringGesture {
                world,
                target_object_id,
            } => {
                if self
                    .workspace
                    .begin_authoring_gesture(world, target_object_id)
                {
                    SessionCommandResult {
                        handled: true,
                        events: vec![SessionEvent::FrameChanged],
                    }
                } else {
                    SessionCommandResult {
                        handled: false,
                        events: Vec::new(),
                    }
                }
            }
            SessionCommand::PreviewAuthoringGesture {
                world,
                target_object_id,
            } => {
                if self
                    .workspace
                    .preview_authoring_gesture(world, target_object_id)
                {
                    SessionCommandResult {
                        handled: true,
                        events: vec![SessionEvent::FrameChanged],
                    }
                } else {
                    SessionCommandResult {
                        handled: false,
                        events: Vec::new(),
                    }
                }
            }
            SessionCommand::CancelAuthoringGesture => {
                if self.workspace.cancel_authoring_gesture() {
                    SessionCommandResult {
                        handled: true,
                        events: vec![SessionEvent::FrameChanged],
                    }
                } else {
                    SessionCommandResult {
                        handled: false,
                        events: Vec::new(),
                    }
                }
            }
            SessionCommand::SelectReviewAction(action_id) => {
                if self.workspace.select_review_action(&action_id) {
                    SessionCommandResult {
                        handled: true,
                        events: vec![
                            SessionEvent::SelectionChanged(self.workspace.selection.clone()),
                            SessionEvent::FrameChanged,
                        ],
                    }
                } else {
                    SessionCommandResult {
                        handled: false,
                        events: Vec::new(),
                    }
                }
            }
            SessionCommand::SelectAuthoredObject(object_id) => {
                if self.workspace.select_authored_object(&object_id) {
                    SessionCommandResult {
                        handled: true,
                        events: vec![
                            SessionEvent::SelectionChanged(self.workspace.selection.clone()),
                            SessionEvent::FrameChanged,
                        ],
                    }
                } else {
                    SessionCommandResult {
                        handled: false,
                        events: Vec::new(),
                    }
                }
            }
            SessionCommand::SelectCheckFinding(fingerprint) => {
                if self.workspace.select_check_finding(&fingerprint) {
                    SessionCommandResult {
                        handled: true,
                        events: vec![
                            SessionEvent::SelectionChanged(self.workspace.selection.clone()),
                            SessionEvent::FrameChanged,
                        ],
                    }
                } else {
                    SessionCommandResult {
                        handled: false,
                        events: Vec::new(),
                    }
                }
            }
            SessionCommand::ClearSelection => {
                self.workspace.clear_selection();
                SessionCommandResult {
                    handled: true,
                    events: vec![
                        SessionEvent::SelectionChanged(self.workspace.selection.clone()),
                        SessionEvent::FrameChanged,
                    ],
                }
            }
            SessionCommand::SelectPreviousReviewAction => {
                if self.workspace.select_previous_review_action() {
                    SessionCommandResult {
                        handled: true,
                        events: vec![
                            SessionEvent::SelectionChanged(self.workspace.selection.clone()),
                            SessionEvent::FrameChanged,
                        ],
                    }
                } else {
                    SessionCommandResult {
                        handled: false,
                        events: Vec::new(),
                    }
                }
            }
            SessionCommand::SelectNextReviewAction => {
                if self.workspace.select_next_review_action() {
                    SessionCommandResult {
                        handled: true,
                        events: vec![
                            SessionEvent::SelectionChanged(self.workspace.selection.clone()),
                            SessionEvent::FrameChanged,
                        ],
                    }
                } else {
                    SessionCommandResult {
                        handled: false,
                        events: Vec::new(),
                    }
                }
            }
            SessionCommand::ToggleShowAuthored => {
                if self.workspace.toggle_show_authored() {
                    SessionCommandResult {
                        handled: true,
                        events: vec![SessionEvent::FrameChanged],
                    }
                } else {
                    SessionCommandResult {
                        handled: false,
                        events: Vec::new(),
                    }
                }
            }
            SessionCommand::ToggleShowProposed => {
                if self.workspace.toggle_show_proposed() {
                    SessionCommandResult {
                        handled: true,
                        events: vec![SessionEvent::FrameChanged],
                    }
                } else {
                    SessionCommandResult {
                        handled: false,
                        events: Vec::new(),
                    }
                }
            }
            SessionCommand::ToggleShowUnrouted => {
                if self.workspace.toggle_show_unrouted() {
                    let event = if self.workspace.scene.unrouted_primitives.is_empty() {
                        SessionEvent::FrameChanged
                    } else {
                        SessionEvent::SceneChanged
                    };
                    SessionCommandResult {
                        handled: true,
                        events: vec![event],
                    }
                } else {
                    SessionCommandResult {
                        handled: false,
                        events: Vec::new(),
                    }
                }
            }
            SessionCommand::ToggleDimUnrelated => {
                if self.workspace.toggle_dim_unrelated() {
                    let event = if self.workspace.selected_review_action().is_none()
                        && matches!(self.workspace.selection, SelectionTarget::None)
                    {
                        SessionEvent::FrameChanged
                    } else {
                        SessionEvent::SceneChanged
                    };
                    SessionCommandResult {
                        handled: true,
                        events: vec![event],
                    }
                } else {
                    SessionCommandResult {
                        handled: false,
                        events: Vec::new(),
                    }
                }
            }
            SessionCommand::ToggleLayerVisibility(layer_id) => {
                if self.workspace.toggle_layer_visibility(&layer_id) {
                    let event = if layer_visibility_change_is_frame_only(
                        &self.workspace.scene,
                        &layer_id,
                    ) {
                        SessionEvent::FrameChanged
                    } else {
                        SessionEvent::SceneChanged
                    };
                    SessionCommandResult {
                        handled: true,
                        events: vec![event],
                    }
                } else {
                    SessionCommandResult {
                        handled: false,
                        events: Vec::new(),
                    }
                }
            }
            SessionCommand::FocusProductionArtifact(artifact_id) => {
                if self.workspace.focus_production_artifact(&artifact_id) {
                    SessionCommandResult {
                        handled: true,
                        events: vec![SessionEvent::FrameChanged],
                    }
                } else {
                    SessionCommandResult {
                        handled: false,
                        events: Vec::new(),
                    }
                }
            }
            SessionCommand::FocusProductionArtifactFile(path) => {
                if self.workspace.focus_production_artifact_file(&path) {
                    SessionCommandResult {
                        handled: true,
                        events: vec![SessionEvent::FrameChanged],
                    }
                } else {
                    SessionCommandResult {
                        handled: false,
                        events: Vec::new(),
                    }
                }
            }
            command => {
                self.apply_artifact_preview_command(command)
                    .unwrap_or(SessionCommandResult {
                        handled: false,
                        events: Vec::new(),
                    })
            }
        }
    }
}
