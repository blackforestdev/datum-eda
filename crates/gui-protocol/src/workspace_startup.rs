//! Application-owned unopened shell: no resolver, model, root or genesis.
use crate::*;

impl ReviewWorkspaceState {
    pub fn unopened() -> Self {
        let scene = BoardReviewSceneV1 {
            kind: String::new(),
            version: 1,
            scene_id: String::new(),
            project_uuid: String::new(),
            project_name: String::new(),
            board_uuid: String::new(),
            board_name: String::new(),
            units: String::new(),
            source_revision: String::new(),
            pad_expansion_setup: Default::default(),
            bounds: SceneBounds {
                min_x: 0,
                min_y: 0,
                max_x: 1,
                max_y: 1,
            },
            layers: vec![],
            outline: vec![],
            components: vec![],
            component_graphics: vec![],
            component_texts: vec![],
            pads: vec![],
            tracks: vec![],
            vias: vec![],
            zones: vec![],
            board_graphics: vec![],
            board_texts: vec![],
            board_text_geometries: vec![],
            glyph_mesh_assets: vec![],
            unrouted_primitives: vec![],
            net_display: vec![],
            proposal_overlay_primitives: vec![],
            review_primitives: vec![],
        };
        let review = RouteProposalReviewPayload {
            action: String::new(),
            review_source: String::new(),
            status: String::new(),
            explanation: String::new(),
            project_root: None,
            artifact_path: None,
            kind: None,
            source_version: None,
            version: None,
            project_uuid: None,
            producer_revision: None,
            source_fingerprint: None,
            producer_proposal: None,
            project_name: None,
            net_uuid: None,
            from_anchor_pad_uuid: None,
            to_anchor_pad_uuid: None,
            selection_profile: None,
            selection_rule: None,
            selected_candidate: None,
            selected_policy: None,
            contract: String::new(),
            actions: 0,
            draw_track_actions: 0,
            selected_path_bend_count: 0,
            selected_path_point_count: 0,
            selected_path_segment_count: 0,
            segment_evidence: vec![],
            proposal_actions: vec![],
        };
        let mut state = Self::new(scene, review);
        state.project_unopened = true;
        state.ui.terminal.status = "No Project open · no terminal session".to_owned();
        state.active_review_target_id.clear();
        state.ui.layout = WorkspaceLayout::single();
        state
    }
}

#[cfg(test)]
#[path = "workspace_startup_tests.rs"]
mod tests;
