use super::*;

#[test]
fn journal_replay_recovers_missing_artifact_metadata_generated_evidence() {
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    let artifact_id = Uuid::new_v4();
    let root = temp_project_root("artifact_metadata_missing_promoted_shard");
    write_minimal_project(&root, project_id, board_id);

    let mut model = ProjectResolver::new(&root)
        .resolve()
        .expect("project resolves before artifact metadata");
    let artifact = ArtifactMetadata {
        schema_version: ARTIFACT_METADATA_SCHEMA_VERSION,
        artifact_id,
        kind: ArtifactKind::GerberSet,
        project_id,
        model_revision: model.model_revision.clone(),
        output_job: None,
        variant: None,
        generator_version: "unit-test".to_string(),
        output_dir: Some(PathBuf::from("fab")),
        files: vec![ArtifactFile {
            path: PathBuf::from("fab/board-F_Cu.gbr"),
            sha256: "sha256:5f9fd18a00b8234c45d8e981d96bf609c0c8b3d32a4f58b8c34f277ed111cb9b"
                .to_string(),
        }],
        production_projections: vec![ArtifactProductionProjection {
            projection_kind: "gerber_copper_layer".to_string(),
            projection_contract: "datum.production_projection.gerber_copper_layer.v1".to_string(),
            model_revision: model.model_revision.clone(),
            byte_count: 128,
            sha256: "sha256:28b3adfae87a0db63bb3e0f8bb9ea8f7c6f1f9955b5f7f4188c5bb47a0f5f3f6"
                .to_string(),
        }],
        validation_state: ArtifactValidationState::NotValidated,
    };
    let before_artifact_revision = model.model_revision.clone();
    model
        .commit_journaled(
            &root,
            OperationBatch {
                batch_id: Uuid::new_v5(&project_id, b"set-artifact-metadata"),
                expected_model_revision: Some(model.model_revision.clone()),
                provenance: CommitProvenance {
                    actor: "unit-test".to_string(),
                    source: CommitSource::Test,
                    reason: "record artifact metadata generated evidence".to_string(),
                },
                operations: vec![Operation::SetArtifactMetadata {
                    artifact_id,
                    previous_artifact_metadata: None,
                    artifact_metadata: serde_json::to_value(&artifact)
                        .expect("artifact should serialize"),
                }],
            },
        )
        .expect("artifact metadata should commit");
    assert_eq!(
        model.model_revision, before_artifact_revision,
        "generated evidence must not mutate the authored model revision"
    );

    std::fs::remove_file(root.join(format!(".datum/artifacts/{artifact_id}.json")))
        .expect("promoted artifact metadata should remove");
    let replayed = ProjectResolver::new(&root)
        .resolve()
        .expect("project should recover artifact metadata from journal");
    assert_eq!(replayed.artifact_metadata[&artifact_id], artifact);
    assert!(replayed.source_shards.iter().any(|shard| {
        shard.kind == SourceShardKind::ArtifactMetadata
            && shard.authority == SourceShardAuthority::GeneratedEvidence
            && shard.dirty_state == SourceShardDirtyState::Missing
            && shard.relative_path == format!(".datum/artifacts/{artifact_id}.json")
    }));
}

#[test]
fn journal_replay_deleted_artifact_metadata_suppresses_stale_promoted_evidence() {
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    let artifact_id = Uuid::new_v4();
    let root = temp_project_root("artifact_metadata_deleted_stale_promoted_shard");
    write_minimal_project(&root, project_id, board_id);

    let mut model = ProjectResolver::new(&root)
        .resolve()
        .expect("project resolves before artifact metadata");
    let artifact = ArtifactMetadata {
        schema_version: ARTIFACT_METADATA_SCHEMA_VERSION,
        artifact_id,
        kind: ArtifactKind::GerberSet,
        project_id,
        model_revision: model.model_revision.clone(),
        output_job: None,
        variant: None,
        generator_version: "unit-test".to_string(),
        output_dir: Some(PathBuf::from("fab")),
        files: vec![ArtifactFile {
            path: PathBuf::from("fab/board-F_Cu.gbr"),
            sha256: "sha256:5f9fd18a00b8234c45d8e981d96bf609c0c8b3d32a4f58b8c34f277ed111cb9b"
                .to_string(),
        }],
        production_projections: vec![ArtifactProductionProjection {
            projection_kind: "gerber_copper_layer".to_string(),
            projection_contract: "datum.production_projection.gerber_copper_layer.v1".to_string(),
            model_revision: model.model_revision.clone(),
            byte_count: 128,
            sha256: "sha256:28b3adfae87a0db63bb3e0f8bb9ea8f7c6f1f9955b5f7f4188c5bb47a0f5f3f6"
                .to_string(),
        }],
        validation_state: ArtifactValidationState::NotValidated,
    };
    let before_artifact_revision = model.model_revision.clone();
    model
        .commit_journaled(
            &root,
            OperationBatch {
                batch_id: Uuid::new_v5(&project_id, b"set-artifact-metadata-before-delete"),
                expected_model_revision: Some(model.model_revision.clone()),
                provenance: CommitProvenance {
                    actor: "unit-test".to_string(),
                    source: CommitSource::Test,
                    reason: "record artifact metadata generated evidence before delete".to_string(),
                },
                operations: vec![Operation::SetArtifactMetadata {
                    artifact_id,
                    previous_artifact_metadata: None,
                    artifact_metadata: serde_json::to_value(&artifact)
                        .expect("artifact should serialize"),
                }],
            },
        )
        .expect("artifact metadata should commit");
    assert_eq!(
        model.model_revision, before_artifact_revision,
        "generated evidence set must not mutate the authored model revision"
    );

    let promoted_path = root.join(format!(".datum/artifacts/{artifact_id}.json"));
    let stale_promoted_bytes = std::fs::read(&promoted_path)
        .expect("promoted artifact metadata should exist before delete");
    let before_delete_revision = model.model_revision.clone();
    model
        .commit_journaled(
            &root,
            OperationBatch {
                batch_id: Uuid::new_v5(&project_id, b"delete-artifact-metadata"),
                expected_model_revision: Some(model.model_revision.clone()),
                provenance: CommitProvenance {
                    actor: "unit-test".to_string(),
                    source: CommitSource::Test,
                    reason: "delete artifact metadata generated evidence".to_string(),
                },
                operations: vec![Operation::DeleteArtifactMetadata {
                    artifact_id,
                    artifact_metadata: serde_json::to_value(&artifact)
                        .expect("artifact should serialize"),
                }],
            },
        )
        .expect("artifact metadata delete should commit");
    assert_eq!(
        model.model_revision, before_delete_revision,
        "generated evidence delete must not mutate the authored model revision"
    );
    assert!(
        !promoted_path.exists(),
        "delete operation should remove promoted artifact metadata shard"
    );

    std::fs::write(&promoted_path, stale_promoted_bytes)
        .expect("stale promoted artifact metadata should be restored to prove replay authority");
    let replayed = ProjectResolver::new(&root)
        .resolve()
        .expect("project should resolve with stale promoted artifact metadata");
    assert!(
        !replayed.artifact_metadata.contains_key(&artifact_id),
        "journaled delete must suppress stale promoted generated evidence"
    );
    assert!(!replayed.source_shards.iter().any(|shard| {
        shard.kind == SourceShardKind::ArtifactMetadata
            && shard.relative_path == format!(".datum/artifacts/{artifact_id}.json")
    }));
}
