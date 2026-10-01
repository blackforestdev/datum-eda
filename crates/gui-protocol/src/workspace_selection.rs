//! Existing application selection entry owner, extracted for the S5A migration.

use crate::{ReviewWorkspaceState, RouteProposalActionPayload, SelectionTarget};

impl ReviewWorkspaceState {
    pub fn selected_review_action(&self) -> Option<&RouteProposalActionPayload> {
        self.review
            .proposal_actions
            .iter()
            .find(|action| action.action_id == self.active_review_target_id)
    }

    pub fn select_previous_review_action(&mut self) -> bool {
        let Some(index) = self
            .review
            .proposal_actions
            .iter()
            .position(|action| action.action_id == self.active_review_target_id)
        else {
            return false;
        };
        if index == 0 {
            return false;
        }
        let action_id = self.review.proposal_actions[index - 1].action_id.clone();
        self.select_review_action(&action_id)
    }

    pub fn select_next_review_action(&mut self) -> bool {
        let Some(index) = self
            .review
            .proposal_actions
            .iter()
            .position(|action| action.action_id == self.active_review_target_id)
        else {
            return false;
        };
        if index + 1 >= self.review.proposal_actions.len() {
            return false;
        }
        let action_id = self.review.proposal_actions[index + 1].action_id.clone();
        self.select_review_action(&action_id)
    }

    pub fn select_review_action(&mut self, action_id: &str) -> bool {
        if self
            .review
            .proposal_actions
            .iter()
            .any(|action| action.action_id == action_id)
        {
            self.active_review_target_id = action_id.to_string();
            self.selection = SelectionTarget::ReviewAction(action_id.to_string());
            true
        } else {
            false
        }
    }

    pub fn select_authored_object(&mut self, object_id: &str) -> bool {
        let normalized_object_id = object_id
            .strip_prefix("component:")
            .and_then(|component_uuid| {
                self.scene
                    .components
                    .iter()
                    .find(|component| component.component_uuid == component_uuid)
                    .map(|component| component.object_id.as_str())
            })
            .unwrap_or(object_id);
        let exists = self
            .scene
            .components
            .iter()
            .any(|c| c.object_id == normalized_object_id)
            || self
                .scene
                .pads
                .iter()
                .any(|p| p.object_id == normalized_object_id)
            || self
                .scene
                .tracks
                .iter()
                .any(|t| t.object_id == normalized_object_id)
            || self
                .scene
                .vias
                .iter()
                .any(|v| v.object_id == normalized_object_id)
            || self
                .scene
                .zones
                .iter()
                .any(|z| z.object_id == normalized_object_id)
            || self
                .scene
                .board_graphics
                .iter()
                .any(|g| g.object_id == normalized_object_id)
            || self
                .scene
                .outline
                .iter()
                .any(|outline| outline.object_id == normalized_object_id)
            || self
                .scene
                .board_texts
                .iter()
                .any(|t| t.object_id == normalized_object_id);
        if exists {
            self.selection = SelectionTarget::AuthoredObject(normalized_object_id.to_string());
            true
        } else {
            false
        }
    }

    pub fn select_check_finding(&mut self, fingerprint: &str) -> bool {
        if fingerprint.is_empty() {
            return false;
        }
        let exists = self
            .checks
            .findings
            .iter()
            .any(|finding| finding.fingerprint == fingerprint);
        if exists {
            self.selection = SelectionTarget::CheckFinding(fingerprint.to_string());
            true
        } else {
            false
        }
    }

    pub fn clear_selection(&mut self) {
        self.selection = SelectionTarget::None;
    }
}
