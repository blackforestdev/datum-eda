//! Component selection identity, shared by retained geometry and frame overlays.
//! Resolve aliases without allocating a formatted ID for every candidate.

use super::*;

pub(super) fn selected_component_uuid<'a>(
    scene: &'a BoardReviewSceneV1,
    state: &ReviewWorkspaceState,
) -> Option<&'a str> {
    let SelectionTarget::AuthoredObject(object_id) = &state.selection else {
        return None;
    };
    let selected_alias = object_id.strip_prefix("component:");
    scene.components.iter().find_map(|component| {
        ((&component.object_id == object_id)
            || (selected_alias == Some(component.component_uuid.as_str())))
        .then_some(component.component_uuid.as_str())
    })
}

pub(super) fn component_is_selection_related(
    component_uuid: &str,
    scene: &BoardReviewSceneV1,
    state: &ReviewWorkspaceState,
) -> bool {
    selected_component_uuid(scene, state).is_some_and(|selected| selected == component_uuid)
}

pub(super) fn component_is_selection_active(
    component_uuid: &str,
    scene: &BoardReviewSceneV1,
    state: &ReviewWorkspaceState,
) -> bool {
    component_is_selection_related(component_uuid, scene, state)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn component_object_and_legacy_alias_select_the_same_component() {
        let mut state = datum_gui_protocol::load_fixture_workspace_state();
        let component = state
            .scene
            .components
            .first()
            .expect("real fixture component")
            .clone();
        for id in [
            component.object_id.clone(),
            format!("component:{}", component.component_uuid),
        ] {
            state.selection = SelectionTarget::AuthoredObject(id);
            assert_eq!(
                selected_component_uuid(&state.scene, &state),
                Some(component.component_uuid.as_str())
            );
            assert!(component_is_selection_active(
                &component.component_uuid,
                &state.scene,
                &state
            ));
            assert!(!component_is_selection_related(
                "missing-component",
                &state.scene,
                &state
            ));
        }
    }

    #[test]
    fn empty_and_unmatched_selections_do_not_select_a_component() {
        let mut state = datum_gui_protocol::load_fixture_workspace_state();
        for selection in [
            SelectionTarget::None,
            SelectionTarget::AuthoredObject("component:missing".into()),
        ] {
            state.selection = selection;
            assert_eq!(selected_component_uuid(&state.scene, &state), None);
        }
    }
}
