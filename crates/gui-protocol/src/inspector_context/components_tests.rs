//! Explicit read-only controls over the already admitted native proof input.
use super::*;
use std::sync::Arc;

fn field(state: &ReviewWorkspaceState, label: &str) -> String {
    InspectorContext::capture(state)
        .sections
        .iter()
        .flat_map(|s| &s.fields)
        .find_map(|f| {
            (f.label == label).then(|| match &f.value {
                InspectorValue::Text(v) => v.to_string(),
                InspectorValue::Json(v) => v.to_string(),
                InspectorValue::SelectionField(v) => v.to_string(),
            })
        })
        .unwrap_or_default()
}

#[test]
#[ignore = "requires explicit DATUM_INSPECTOR_TEST_PROJECT admitted read-only input"]
fn admitted_sources_fail_closed_without_fallback_and_pin_uses_terminal() {
    let root = std::path::PathBuf::from(
        std::env::var_os("DATUM_INSPECTOR_TEST_PROJECT").expect("explicit admitted input"),
    );
    let model = Arc::new(
        eda_engine::substrate::ProjectResolver::new(&root)
            .resolve()
            .unwrap(),
    );
    assert_eq!(
        model.model_revision.0,
        "e165c7950ddbcb0d5ac1329aae2672382bab32130536be1d3d98c125eaa88fd3"
    );
    let request = crate::LiveReviewRequest {
        project_root: root,
        board_file: None,
        artifact_path: None,
        net_uuid: None,
        from_anchor_pad_uuid: None,
        to_anchor_pad_uuid: None,
        profile: None,
        kicad_board_source: None,
    };
    let mut state =
        crate::workspace_loading::project_board_editor_workspace(&request, model.clone()).unwrap();
    let authority =
        crate::selection_resolution::NativeSelectionResolution::capture_shared(model.clone())
            .unwrap();
    let id = crate::AuthoredSelectionIdentity {
        class: C::Footprint,
        instance_path: Vec::new(),
        id: "78000000-0000-4000-8000-000000000002".parse().unwrap(),
    };
    let view = crate::selection_view::build_selection_view(
        &model,
        &authority,
        crate::SelectionSubject::Object(id.clone()),
        &state.scene,
        state.schematic_scene.as_ref(),
        |_| true,
    )
    .unwrap();
    assert_eq!(view.members.len(), 1);
    state.selection = SelectionTarget::Native(Box::new(view));
    assert_eq!(
        field(&state, "Object UUID"),
        "60000000-0000-4000-8000-000000000001"
    );
    assert!(field(&state, "Library object").contains("SOIC-8_NARROW_IPC7351B_B"));
    assert_eq!(field(&state, "Approval"), "Not recorded");
    let positive = state.clone();
    if let SelectionTarget::Native(view) = &mut state.selection {
        view.as_mut()
            .values
            .get_mut(&id)
            .unwrap()
            .remove("library_land_pattern_sources");
    }
    assert!(field(&state, "Footprint source").starts_with("Unavailable"));
    assert_eq!(field(&state, "Object UUID"), "");
    for stale in [false, true] {
        state = positive.clone();
        let entry = state
            .library
            .entries
            .iter()
            .position(|e| {
                e.kind == "footprints" && e.id.to_string() == "60000000-0000-4000-8000-000000000001"
            })
            .unwrap();
        if stale {
            state.library.entries[entry].revision = Some(999);
        } else {
            state.library.entries.remove(entry);
        }
        assert!(field(&state, "Footprint source").starts_with("Unavailable"));
        assert_eq!(field(&state, "Object UUID"), "");
        assert!(field(&state, "Placed Part").contains("90000000-0000-4000-8000-000000000001"));
    }
    state = positive;
    let scene = state.schematic_scene.as_ref().unwrap();
    let finding = state
        .checks
        .findings
        .iter()
        .find(|f| f.primary_target["id"] == "78000000-0000-4000-8000-000000000071")
        .unwrap();
    let index = |scene: &crate::BoardReviewSceneV1, visible: bool| {
        crate::check_runs::marker::MarkerProjectionIndex::capture_qualified(
            scene,
            &state.checks,
            crate::PaneContent::Schematic,
            |_| visible,
            Some(&state.schematic_documents),
        )
    };
    let expected = crate::PointNm {
        x: -10_000_000,
        y: 1_500_000,
    };
    assert!(
        scene
            .board_graphics
            .iter()
            .filter(|g| g.source_object_uuid == "78000000-0000-4000-8000-000000000071")
            .count()
            > 1
    );
    assert_eq!(index(scene, true).anchor(finding), Ok(expected));
    assert!(index(scene, false).anchor(finding).is_err());
    let mut missing = scene.clone();
    missing
        .board_graphics
        .retain(|g| g.source_object_uuid != "78000000-0000-4000-8000-000000000071");
    assert!(index(&missing, true).anchor(finding).is_err());
    let mut collision = scene.clone();
    let mut graphic = collision
        .board_graphics
        .iter()
        .find(|g| g.source_object_uuid == "78000000-0000-4000-8000-000000000071")
        .unwrap()
        .clone();
    graphic.object_kind = "schematic_wire".into();
    collision.board_graphics.push(graphic);
    assert_eq!(
        index(&collision, true).anchor(finding),
        Err("Target has ambiguous source identities")
    );
    let mut foreign = scene.clone();
    foreign.source_revision.push_str("-stale");
    assert!(index(&foreign, true).anchor(finding).is_err());
}
