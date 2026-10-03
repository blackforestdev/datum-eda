//! Shared read-only relationship paint queries; native authored material stays canonical.
use super::*;
pub(super) fn unrouted_matches_active_action(
    unrouted: &UnroutedPrimitive,
    state: &ReviewWorkspaceState,
) -> bool {
    if immediate_selection_workspace(state) {
        return false;
    }
    state
        .selected_review_action()
        .is_some_and(|action| action.net_uuid == unrouted.net_uuid)
}

pub(super) fn unrouted_base_color(
    scene: &BoardReviewSceneV1,
    unrouted: &UnroutedPrimitive,
) -> [f32; 3] {
    scene
        .net_display
        .iter()
        .find(|entry| entry.net_uuid == unrouted.net_uuid)
        .map(|entry| entry.airwire_color_rgb)
        .unwrap_or(UNROUTED_BASE)
}

pub(super) fn proposal_preview_affected_ids(state: &ReviewWorkspaceState) -> Vec<&str> {
    proposal_preview_ids(state).collect()
}
pub(super) fn proposal_preview_ids(
    state: &ReviewWorkspaceState,
) -> impl Iterator<Item = &str> + Clone {
    state
        .production
        .proposals
        .iter()
        .filter(|_| !immediate_selection_workspace(state))
        .filter_map(|proposal| proposal.preview.as_ref())
        .flat_map(|preview| preview.affected_objects.iter().map(String::as_str))
}

pub(super) fn source_object_matches_preview(
    affected_ids: &[&str],
    object_id: &str,
    source_object_uuid: &str,
) -> bool {
    affected_ids
        .iter()
        .any(|affected| *affected == object_id || *affected == source_object_uuid)
}

pub(super) fn component_matches_preview(
    component_uuid: &str,
    scene: &BoardReviewSceneV1,
    affected_ids: &[&str],
) -> bool {
    scene.components.iter().any(|component| {
        component.component_uuid == component_uuid
            && source_object_matches_preview(
                affected_ids,
                &component.object_id,
                &component.source_object_uuid,
            )
    })
}
