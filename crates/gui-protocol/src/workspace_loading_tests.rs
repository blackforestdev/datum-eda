//! Compare complete fresh-basis projections with the existing independent loaders
//! on the existing owner-board copy; no new synthetic project or latency budget.
use super::*;
use crate::tests::{unique_project_root, write_json, write_minimal_native_project};
use eda_engine::substrate::{CommitProvenance, CommitSource, Operation, OperationBatch};
use serde_json::json;
use uuid::Uuid;

#[test]
#[ignore = "requires DATUM_FIRST_EDIT_PROJECT private copy and matching CLI"]
fn fresh_native_workspace_preserves_existing_projections() {
    let root = PathBuf::from(std::env::var("DATUM_FIRST_EDIT_PROJECT").unwrap());
    let request = LiveReviewRequest {
        project_root: root.clone(),
        board_file: None,
        kicad_board_source: None,
        artifact_path: None,
        net_uuid: None,
        from_anchor_pad_uuid: None,
        to_anchor_pad_uuid: None,
        profile: None,
    };
    let before = std::fs::read(root.join("board/board.json")).unwrap();
    let (state, basis) = load_board_editor_workspace_with_model(&request).unwrap();
    assert!(
        production_is_empty(&basis),
        "existing first-edit design has no production records"
    );
    let cli = cli_prefix().unwrap();
    assert_eq!(
        state.production,
        load_production_status(&cli, &request).unwrap()
    );
    assert_eq!(
        state.checks,
        check_runs::loading::load(&cli, &request).unwrap()
    );
    assert_eq!(
        state.source_shards,
        refresh_source_shard_status(&request).unwrap()
    );
    let (scene, _) = load_scene_from_engine(&request).unwrap();
    assert_eq!(
        serde_json::to_value(&state.scene).unwrap(),
        serde_json::to_value(scene).unwrap()
    );
    assert_eq!(
        serde_json::to_value(&state.schematic_scene).unwrap(),
        serde_json::to_value(
            native_schematic_scene::project_native_schematic_scene(&basis, &root).unwrap()
        )
        .unwrap()
    );
    assert_eq!(state.supervision.model_revision, basis.model_revision.0);
    assert_eq!(
        state.supervision.journal.accepted_transaction_tip,
        eda_engine::revision::transaction_tip(&basis.journal).map(|id| id.to_string())
    );
    assert_eq!(
        std::fs::read(root.join("board/board.json")).unwrap(),
        before
    );
}

#[test]
fn native_board_scene_loads_resolver_materialized_board_state() {
    let root = unique_project_root("datum-gui-engine-scene");
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    let text_id = Uuid::new_v4();
    write_minimal_native_project(&root, project_id, board_id);
    let mut model = ProjectResolver::new(&root)
        .resolve()
        .expect("native fixture should resolve");
    model
        .commit_journaled(
            &root,
            OperationBatch {
                batch_id: Uuid::new_v4(),
                expected_model_revision: Some(model.model_revision.clone()),
                provenance: CommitProvenance {
                    actor: "gui-protocol-test".to_string(),
                    source: CommitSource::Test,
                    reason: "place board text for resolver-backed GUI scene".to_string(),
                },
                operations: vec![Operation::CreateBoardText {
                    text_id,
                    text: json!({
                        "uuid": text_id,
                        "text": "Resolver Truth",
                        "position": { "x": 1_000_000, "y": 2_000_000 },
                        "rotation": 0,
                        "layer": 37,
                        "height_nm": 1_000_000
                    }),
                }],
            },
        )
        .expect("board text should commit through substrate");
    write_json(
        &root.join("board/board.json"),
        json!({
            "schema_version": 1,
            "uuid": board_id,
            "name": "GUI Engine Scene Demo Board",
            "stackup": { "layers": [] },
            "outline": { "vertices": [], "closed": true },
            "packages": {},
            "component_silkscreen": {},
            "component_pads": {},
            "pads": {},
            "tracks": {},
            "vias": {},
            "zones": {},
            "nets": {},
            "net_classes": {},
            "dimensions": [],
            "texts": [],
            "keepouts": {}
        }),
    );

    let (scene, board_path) = load_scene_from_engine(&LiveReviewRequest {
        project_root: root.clone(),
        board_file: None,
        artifact_path: None,
        net_uuid: None,
        from_anchor_pad_uuid: None,
        to_anchor_pad_uuid: None,
        profile: None,
        kicad_board_source: None,
    })
    .expect("resolver-backed native scene should load");

    assert_eq!(board_path, root.join("board/board.json"));
    assert_eq!(scene.project_uuid, project_id.to_string());
    assert_eq!(scene.board_uuid, board_id.to_string());
    assert_eq!(scene.source_revision, model.model_revision.0);
    assert!(
        scene
            .board_texts
            .iter()
            .any(|text| text.text_uuid == text_id.to_string() && text.text == "Resolver Truth"),
        "native GUI scene should reflect journal-materialized board text, not stale promoted board JSON"
    );
    let snapshot = load_gui_supervision_snapshot(
        &LiveReviewRequest {
            project_root: root.clone(),
            board_file: None,
            artifact_path: None,
            net_uuid: None,
            from_anchor_pad_uuid: None,
            to_anchor_pad_uuid: None,
            profile: None,
            kicad_board_source: None,
        },
        &scene,
        &ProductionStatus::default(),
        &SourceShardStatusSummary::default(),
        &CheckRunReviewState::default(),
        Some(&model),
    )
    .expect("supervision snapshot should load from resolver");

    assert_eq!(snapshot.contract, GUI_SUPERVISION_SNAPSHOT_CONTRACT);
    assert!(snapshot.read_only);
    assert_eq!(snapshot.project_uuid, project_id.to_string());
    assert_eq!(snapshot.model_revision, model.model_revision.0);
    assert_eq!(snapshot.journal.applied_transaction_count, 1);
    let expected_transaction_tip = model
        .journal
        .first()
        .map(|transaction| transaction.transaction_id.to_string());
    assert_eq!(
        snapshot.journal.accepted_transaction_tip.as_deref(),
        expected_transaction_tip.as_deref()
    );
    assert_eq!(snapshot.journal.projection_omitted_count, 0);
    assert_eq!(snapshot.journal.projection.len(), 1);
    let projected = &snapshot.journal.projection[0];
    assert_eq!(projected.journal_ordinal, 1);
    assert_eq!(projected.transaction_kind, "normal");
    assert_eq!(projected.commit_source, "test");
    assert_eq!(
        projected.reason,
        "place board text for resolver-backed GUI scene"
    );
    assert_eq!(projected.operation_count, 1);
    assert_eq!(projected.created_count, 1);
    assert_eq!(projected.modified_count, 0);
    assert_eq!(projected.deleted_count, 0);
    assert_eq!(snapshot.scene.board_text_count, scene.board_texts.len());
}
