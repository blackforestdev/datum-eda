//! Qualified markers follow the same actual source occurrence as scoped counts.
use super::*;
use std::sync::Arc;

#[test]
fn marker_frame_never_aliases_repeated_source_occurrences() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../test-harness/testdata/selection/s5a_v1/replay-inputs/native/s5a-c03-u5-semantic-v1/variants/semantic-scope");
    let model = Arc::new(
        eda_engine::substrate::ProjectResolver::new(&root)
            .resolve()
            .unwrap(),
    );
    let before: Vec<_> = model
        .source_shards
        .iter()
        .map(|shard| std::fs::read(root.join(&shard.path)).unwrap())
        .collect();
    let request = crate::LiveReviewRequest {
        project_root: root.clone(),
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
    let children: Vec<_> = state
        .schematic_documents
        .rows
        .iter()
        .filter(|row| !row.key.instance_path.is_empty())
        .map(|row| row.key.clone())
        .collect();
    assert_eq!(children.len(), 2);
    for key in &children {
        state
            .schematic_documents
            .prepare_scene(key, &root, &model)
            .unwrap();
    }
    let pane_a = state
        .schematic_documents
        .open_beside(&mut state.ui.layout, &children[0])
        .unwrap();
    let pane_b = state
        .schematic_documents
        .open_beside(&mut state.ui.layout, &children[1])
        .unwrap();
    let a = state.schematic_documents.scene_for_pane(pane_a).unwrap();
    let b = state.schematic_documents.scene_for_pane(pane_b).unwrap();
    let wire = state
        .schematic_documents
        .source_sheet(&children[0])
        .unwrap()
        .wires
        .keys()
        .next()
        .unwrap();
    let mut finding = CheckFindingSummary {
        fingerprint: "scope-control-only".into(),
        domain: "erc".into(),
        primary_target: serde_json::json!({"kind":"object_uuid", "id":wire, "instance_path":children[0].instance_path}),
        ..Default::default()
    };
    let checks = CheckRunReviewState {
        project_id: Some(model.project.project_id.to_string()),
        model_revision: Some(model.model_revision.0.clone()),
        persisted: true,
        findings: vec![finding.clone()],
        ..Default::default()
    };
    let index = |scene| {
        MarkerProjectionIndex::capture_qualified(
            scene,
            &checks,
            crate::PaneContent::Schematic,
            |_| true,
            Some(&state.schematic_documents),
        )
    };
    assert!(index(a).anchor(&finding).is_ok());
    assert!(index(b).anchor(&finding).is_err());
    let root_scene = state.schematic_scene.as_ref().unwrap();
    let root_key = root_scene.schematic_document.as_ref().unwrap();
    let root_wire = state
        .schematic_documents
        .source_sheet(root_key)
        .unwrap()
        .wires
        .keys()
        .next()
        .unwrap();
    let root_finding = CheckFindingSummary {
        primary_target: serde_json::json!({"kind":"object_uuid", "id":root_wire}),
        ..finding.clone()
    };
    let root_checks = CheckRunReviewState {
        findings: vec![root_finding.clone()],
        ..checks.clone()
    };
    assert!(
        MarkerProjectionIndex::capture_qualified(
            root_scene,
            &root_checks,
            crate::PaneContent::Schematic,
            |_| true,
            Some(&state.schematic_documents)
        )
        .anchor(&root_finding)
        .is_ok(),
        "a proven unique root source needs no invented path"
    );
    finding
        .primary_target
        .as_object_mut()
        .unwrap()
        .remove("instance_path");
    assert!(index(a).anchor(&finding).is_err());
    assert!(index(b).anchor(&finding).is_err());
    finding.primary_target["instance_path"] = serde_json::json!(children[0].instance_path);
    finding.payload = serde_json::json!({"instance_path":children[1].instance_path,"location":{"x_nm":0,"y_nm":0}});
    assert!(
        index(a).anchor(&finding).is_err(),
        "producing location cannot bypass conflicting occurrence identity"
    );
    let mut foreign = a.clone();
    foreign.source_revision.push_str("-stale");
    assert!(index(&foreign).anchor(&finding).is_err());
    assert!(matches!(state.selection, crate::SelectionTarget::None));
    for (shard, bytes) in model.source_shards.iter().zip(before) {
        assert_eq!(std::fs::read(root.join(&shard.path)).unwrap(), bytes);
    }
}
