//! Shared consumer workspace state; per-editor projections remain configured data.
use crate::*;

#[derive(Debug, Clone, PartialEq)]
pub struct ReviewWorkspaceState {
    /// Explicit application state; unbacked review fixtures remain loaded.
    pub project_unopened: bool,
    pub library: library_browser::LibraryBrowser,
    pub schematic_authoring: schematic_authoring::SchematicAuthoring,
    pub scene: BoardReviewSceneV1,
    /// Companion projection; never a separate design authority.
    pub schematic_scene: Option<BoardReviewSceneV1>,
    pub schematic_documents: crate::schematic_documents::SchematicDocuments,
    pub review: RouteProposalReviewPayload,
    pub production: ProductionStatus,
    pub source_shards: SourceShardStatusSummary,
    pub checks: CheckRunReviewState,
    pub check_producer: crate::CheckProducerState,
    pub inspector_context: crate::inspector_context::InspectorNavigation,
    pub supervision: GuiSupervisionSnapshot,
    pub selection: SelectionTarget,
    pub active_review_target_id: String,
    pub tool: WorkspaceTool,
    pub authoring: AuthoringToolState,
    pub backing: Option<WorkspaceBacking>,
    pub last_command_status: Option<EditorCommandStatus>,
    pub ui: WorkspaceUiState,
}

impl ReviewWorkspaceState {
    pub fn new(scene: BoardReviewSceneV1, review: RouteProposalReviewPayload) -> Self {
        let layer_visibility = scene
            .layers
            .iter()
            .map(|layer| (layer.layer_id.clone(), layer.visible_by_default))
            .collect();
        let active_layer_id = scene
            .layers
            .iter()
            .find(|layer| layer.visible_by_default)
            .or_else(|| scene.layers.first())
            .map(|layer| layer.layer_id.clone());
        let has_review_actions = !review.proposal_actions.is_empty();
        let active_review_target_id = review
            .proposal_actions
            .first()
            .map(|action| action.action_id.clone())
            .unwrap_or_else(|| "no-proposal-action".to_string());
        Self {
            project_unopened: false,
            scene,
            schematic_scene: None,
            schematic_documents: Default::default(),
            schematic_authoring: Default::default(),
            library: Default::default(),
            review,
            production: ProductionStatus::default(),
            source_shards: SourceShardStatusSummary::default(),
            checks: CheckRunReviewState::default(),
            check_producer: Default::default(),
            inspector_context: Default::default(),
            selection: if has_review_actions {
                SelectionTarget::ReviewAction(active_review_target_id.clone())
            } else {
                SelectionTarget::None
            },
            active_review_target_id,
            tool: WorkspaceTool::Select,
            authoring: AuthoringToolState {
                snap: AuthoringSnapState::default(),
                gesture: AuthoringGestureState::idle(WorkspaceTool::Select),
            },
            backing: None,
            last_command_status: None,
            supervision: GuiSupervisionSnapshot::default(),
            ui: WorkspaceUiState::new(WorkspaceFilterState {
                show_authored: true,
                excluded_selection_classes: Default::default(),
                show_proposed: true,
                show_unrouted: true,
                dim_unrelated: has_review_actions,
                active_layer_id,
                layer_visibility,
                layer_scroll_offset: 0,
            }),
        }
    }
}

impl ReviewWorkspaceState {
    pub fn review_rows(&self) -> Vec<ReviewActionRow> {
        self.review
            .proposal_actions
            .iter()
            .map(|action| ReviewActionRow {
                action_id: action.action_id.clone(),
                title: format!(
                    "{} {}",
                    action.proposal_action.to_uppercase(),
                    action.selected_path_segment_index + 1
                ),
                subtitle: format!("LAYER {} {} NM", action.layer, action.width_nm),
            })
            .collect()
    }

    pub fn toggle_show_authored(&mut self) -> bool {
        self.ui.filters.show_authored = !self.ui.filters.show_authored;
        true
    }

    pub fn toggle_show_proposed(&mut self) -> bool {
        self.ui.filters.show_proposed = !self.ui.filters.show_proposed;
        true
    }

    pub fn toggle_show_unrouted(&mut self) -> bool {
        self.ui.filters.show_unrouted = !self.ui.filters.show_unrouted;
        true
    }

    pub fn toggle_dim_unrelated(&mut self) -> bool {
        self.ui.filters.dim_unrelated = !self.ui.filters.dim_unrelated;
        true
    }

    pub fn toggle_layer_visibility(&mut self, layer_id: &str) -> bool {
        let entry = self
            .ui
            .filters
            .layer_visibility
            .entry(layer_id.to_string())
            .or_insert(true);
        *entry = !*entry;
        true
    }

    pub fn selected_segment_evidence(&self) -> Option<&RouteProposalSegmentEvidence> {
        self.selected_review_action().and_then(|action| {
            self.review
                .segment_evidence
                .iter()
                .find(|segment| {
                    segment.layer_segment_index
                        == action.selected_path_layer_segment_index.unwrap_or(0)
                })
                .or_else(|| self.review.segment_evidence.first())
        })
    }

    pub fn review_action_index(&self, action_id: &str) -> Option<usize> {
        self.review
            .proposal_actions
            .iter()
            .position(|action| action.action_id == action_id)
    }
}
