//! Qualified markers follow the same actual source occurrence as scoped counts.
use super::*;
use std::sync::Arc;

#[test]
fn native_drc_markers_require_actual_copper_basis_and_preserve_erc_qualification() {
    use eda_engine::api::native_write::{
        WriteProvenance,
        board_routing::zone::{FillRequest, capture_board_basis, prepare_fill},
        commit_prepared,
    };
    use eda_engine::substrate::{CommitSource, ProjectResolver};
    let root = std::env::temp_dir().join(format!("datum-drc-marker-basis-{}", Uuid::new_v4()));
    crate::known_good_demo::write_known_good_demo_project(&root).unwrap();
    let mut model = ProjectResolver::new(&root).resolve().unwrap();
    let revision = model.model_revision.clone();
    let receipt = model.native_drc_zone_copper_basis().unwrap();
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
        crate::workspace_loading::project_board_editor_workspace(&request, Arc::new(model.clone()))
            .unwrap();
    assert!(state.scene.tracks.iter().any(|track| {
        track.source_object_uuid == "00000000-0000-0000-0000-00000000c20d"
            && !track.layer_id.is_empty()
    }));
    // Finding payload is a projection control, not analysis proof. Its source
    // Track and changing generated-fill basis are actual native project data.
    // The legacy demo Pad has no declared copper layers and cannot supply a
    // qualified visible anchor; the Track has an explicit projected layer.
    let finding = CheckFindingSummary {
        fingerprint: "native-basis-marker-control".into(),
        source: "drc".into(),
        domain: "drc".into(),
        primary_target: serde_json::json!({"kind":"object_uuid","id":"00000000-0000-0000-0000-00000000c20d"}),
        ..Default::default()
    };
    state.checks = CheckRunReviewState {
        check_run_id: Some(Uuid::new_v4().to_string()),
        project_id: Some(model.project.project_id.to_string()),
        model_revision: Some(revision.0.clone()),
        profile_id: Some("drc".into()),
        persisted: true,
        native_drc_zone_copper_basis: Some(receipt.clone()),
        findings: vec![finding.clone()],
        ..Default::default()
    };
    let anchored = |state: &crate::ReviewWorkspaceState| {
        let result = MarkerProjectionIndex::capture_native(
            &state.scene,
            state,
            crate::PaneContent::Board,
            |_| true,
            Some(&state.schematic_documents),
        )
        .anchor(&finding);
        if let Err(reason) = result {
            eprintln!(
                "marker refused: {reason}; report qualification: {:?}",
                state.checks.qualification(state)
            );
        }
        result.is_ok()
    };
    assert!(anchored(&state));
    assert!(
        MarkerProjectionIndex::capture(
            &state.scene,
            &state.checks,
            crate::PaneContent::Board,
            |_| true
        )
        .anchor(&finding)
        .is_err(),
        "scene-only stamps cannot qualify native DRC copper"
    );
    state.checks.native_drc_zone_copper_basis = None;
    assert!(!anchored(&state));
    let mut unknown = receipt.clone();
    unknown["version"] = serde_json::json!(2);
    state.checks.native_drc_zone_copper_basis = Some(unknown);
    assert!(!anchored(&state));
    state.checks.native_drc_zone_copper_basis = Some(receipt.clone());
    let provenance = || {
        WriteProvenance::new(
            "marker-test",
            CommitSource::Test,
            "Explicit native fill input change",
        )
    };
    let fill = FillRequest {
        basis: capture_board_basis(&model).unwrap(),
        zone: Some("00000000-0000-0000-0000-00000000c211".parse().unwrap()),
        net: None,
    };
    let prepared = prepare_fill(&model, provenance(), &fill).unwrap();
    commit_prepared(&mut model, &root, prepared).unwrap();
    model = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(model.model_revision, revision);
    assert_ne!(model.native_drc_zone_copper_basis().unwrap(), receipt);
    state.library = crate::library_browser::LibraryBrowser::capture(Arc::new(model.clone()));
    assert!(!anchored(&state));
    state.checks.profile_id = Some("erc".into());
    assert!(
        anchored(&state),
        "ERC retains its existing authored-source qualification"
    );
    state.checks.profile_id = Some("drc".into());
    model
        .commit_journal_undo(&root, provenance().into())
        .unwrap();
    model = ProjectResolver::new(&root).resolve().unwrap();
    state.library = crate::library_browser::LibraryBrowser::capture(Arc::new(model));
    assert!(
        anchored(&state),
        "exact restored input basis qualifies the recorded result again"
    );
    std::fs::remove_dir_all(root).unwrap();
}

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

#[test]
fn physical_standards_findings_have_board_markers_and_scoped_counts() {
    let root = std::env::temp_dir().join(format!("datum-physical-markers-{}", Uuid::new_v4()));
    crate::known_good_demo::write_known_good_demo_project(&root).unwrap();
    let model = Arc::new(
        eda_engine::substrate::ProjectResolver::new(&root)
            .resolve()
            .unwrap(),
    );
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
    let target = crate::AuthoredSelectionIdentity {
        class: crate::AuthoredSelectionClass::Track,
        instance_path: vec![],
        id: "00000000-0000-0000-0000-00000000c20d".parse().unwrap(),
    };
    // The legacy demo Pad has no declared copper layers; use an actual Track
    // with an explicit projected layer so this control has a qualified anchor.
    assert!(
        state
            .scene
            .tracks
            .iter()
            .any(|track| track.source_object_uuid == target.id.to_string()
                && !track.layer_id.is_empty())
    );
    let authority =
        crate::selection_resolution::NativeSelectionResolution::capture_shared(model.clone())
            .unwrap();
    let view = crate::selection_view::build_selection_view(
        &model,
        &authority,
        crate::SelectionSubject::Object(target.clone()),
        &state.scene,
        state.schematic_scene.as_ref(),
        |_| true,
    )
    .unwrap();
    state.selection = crate::SelectionTarget::Native(Box::new(view));
    // Presentation controls over a real selected board object; CLI/App proof
    // separately establishes physical rule production and persisted history.
    state.checks = CheckRunReviewState {
        check_run_id: Some(Uuid::new_v4().to_string()),
        project_id: Some(model.project.project_id.to_string()),
        model_revision: Some(model.model_revision.0.clone()),
        profile_id: Some("drc".into()),
        persisted: true,
        native_drc_zone_copper_basis: Some(model.native_drc_zone_copper_basis().unwrap()),
        finding_count: 3,
        findings: [
            "track_width_below_min",
            "via_hole_out_of_range",
            "via_annular_below_min",
        ]
        .into_iter()
        .map(|code| CheckFindingSummary {
            fingerprint: code.into(),
            code: code.into(),
            source: "drc".into(),
            domain: "standards".into(),
            severity: "error".into(),
            status: "active".into(),
            primary_target: serde_json::json!({"kind":"object_uuid","id":target.id}),
            ..Default::default()
        })
        .collect(),
        coverage: vec![crate::CheckRunCoverageSummary {
            domain: "drc".into(),
            status: "evaluated".into(),
            target_scope: "board".into(),
            ..Default::default()
        }],
        ..Default::default()
    };
    let assert_count = |state: &crate::ReviewWorkspaceState, expected: &str| {
        let context = crate::inspector_context::InspectorContext::capture(state);
        assert!(
            context
                .sections
                .iter()
                .flat_map(|s| &s.fields)
                .any(|f| f.label == "Scoped DRC"
                    && context.field_text(f, usize::MAX).starts_with(expected))
        );
    };
    assert_count(&state, "3 errors");
    let index = MarkerProjectionIndex::capture_native(
        &state.scene,
        &state,
        crate::PaneContent::Board,
        |_| true,
        Some(&state.schematic_documents),
    );
    for finding in &state.checks.findings {
        let anchor = index.anchor(finding);
        assert!(
            anchor.is_ok(),
            "{anchor:?}; qualification={:?}; projected track={:?}",
            state.checks.qualification(&state),
            state
                .scene
                .tracks
                .iter()
                .find(|p| p.source_object_uuid == target.id.to_string())
        );
        let mut nonphysical = finding.clone();
        nonphysical.source = "zone_fill".into();
        assert!(index.anchor(&nonphysical).is_err());
        let mut foreign = finding.clone();
        foreign.primary_target["instance_path"] = serde_json::json!([Uuid::new_v4()]);
        assert!(index.anchor(&foreign).is_err());
    }
    state.checks.findings[0].status = "waived".into();
    state.checks.findings[1].status = "accepted_deviation".into();
    assert_count(&state, "1 errors · 0 warnings · 0 infos · 2 suppressed");
    state.checks.findings[2].source = "zone_fill".into();
    assert_count(&state, "0 errors · 0 warnings · 0 infos · 2 suppressed");
    std::fs::remove_dir_all(root).unwrap();
}
