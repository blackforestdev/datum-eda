use super::*;
use eda_engine::substrate::{
    ARTIFACT_METADATA_SCHEMA_VERSION, ArtifactFile, ArtifactKind, ArtifactMetadata,
    ArtifactValidationState, CommitProvenance, CommitSource, Operation, OperationBatch,
    ProjectResolver, SourceShardAuthority, SourceShardKind,
};

fn unique_project_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{label}-{}", Uuid::new_v4()))
}

fn place_unfilled_zone(root: &Path) {
    let class_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "place-board-net-class",
            root.to_str().unwrap(),
            "--name",
            "Default",
            "--clearance-nm",
            "150000",
            "--track-width-nm",
            "200000",
            "--via-drill-nm",
            "300000",
            "--via-diameter-nm",
            "600000",
        ])
        .expect("CLI should parse"),
    )
    .expect("place board net class should succeed");
    let class_report: serde_json::Value =
        serde_json::from_str(&class_output).expect("class output should parse");
    let net_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "place-board-net",
            root.to_str().unwrap(),
            "--name",
            "GND",
            "--class",
            class_report["net_class_uuid"].as_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("place board net should succeed");
    let net_report: serde_json::Value =
        serde_json::from_str(&net_output).expect("net output should parse");
    execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "place-board-zone",
            root.to_str().unwrap(),
            "--net",
            net_report["net_uuid"].as_str().unwrap(),
            "--vertex",
            "0:0",
            "--vertex",
            "1000:0",
            "--vertex",
            "1000:1000",
            "--layer",
            "1",
            "--priority",
            "2",
            "--thermal-gap-nm",
            "0",
            "--thermal-spoke-width-nm",
            "0",
        ])
        .expect("CLI should parse"),
    )
    .expect("place board zone should succeed");
    execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "place-board-zone",
            root.to_str().unwrap(),
            "--net",
            net_report["net_uuid"].as_str().unwrap(),
            "--vertex",
            "2000:0",
            "--vertex",
            "3000:0",
            "--vertex",
            "3000:1000",
            "--layer",
            "1",
            "--priority",
            "1",
            "--thermal-gap-nm",
            "0",
            "--thermal-spoke-width-nm",
            "0",
        ])
        .expect("CLI should parse"),
    )
    .expect("place second board zone should succeed");
}

fn assert_unlinked_artifact_evidence_replays_from_journal(
    root: &Path,
    authored_revision: eda_engine::substrate::ModelRevision,
    entry: &serde_json::Value,
) {
    let artifact_id = Uuid::parse_str(
        entry["artifact_id"]
            .as_str()
            .expect("artifact id should serialize"),
    )
    .expect("artifact id should parse");
    let artifact_run_id = Uuid::parse_str(
        entry["artifact_run"]["run_id"]
            .as_str()
            .expect("artifact run id should serialize"),
    )
    .expect("artifact run id should parse");
    let artifact_path = root.join(format!(".datum/artifacts/{artifact_id}.json"));
    let artifact_run_path = root.join(format!(".datum/artifact_runs/{artifact_run_id}.json"));
    assert!(artifact_path.is_file());
    assert!(artifact_run_path.is_file());

    std::fs::remove_file(&artifact_path).expect("promoted artifact metadata should remove");
    std::fs::remove_file(&artifact_run_path).expect("promoted artifact run should remove");

    let replayed = ProjectResolver::new(root)
        .resolve()
        .expect("project should recover generated artifact evidence from journal");
    assert_eq!(
        replayed.model_revision, authored_revision,
        "generated artifact evidence must not mutate authored model revision"
    );
    assert_eq!(
        replayed.artifact_metadata[&artifact_id].schema_version, 1,
        "artifact metadata should replay after promoted shard deletion"
    );
    assert_eq!(
        replayed.artifact_runs[&artifact_run_id].schema_version, 1,
        "artifact run should replay after promoted shard deletion"
    );
    assert!(replayed.source_shards.iter().any(|shard| {
        shard.kind == SourceShardKind::ArtifactMetadata
            && shard.authority == SourceShardAuthority::GeneratedEvidence
            && shard.relative_path == format!(".datum/artifacts/{artifact_id}.json")
            && shard.schema_version == Some(1)
    }));
    assert!(replayed.source_shards.iter().any(|shard| {
        shard.kind == SourceShardKind::ArtifactRun
            && shard.authority == SourceShardAuthority::GeneratedEvidence
            && shard.relative_path == format!(".datum/artifact_runs/{artifact_run_id}.json")
            && shard.schema_version == Some(1)
    }));
}

fn commit_artifact_metadata_pair_for_latest_order(
    root: &Path,
    older_artifact_id: Uuid,
    newer_artifact_id: Uuid,
) {
    commit_test_artifact_metadata(root, older_artifact_id);
    bump_project_revision_for_artifact_latest_order(root);
    commit_test_artifact_metadata(root, newer_artifact_id);
}

fn commit_test_artifact_metadata(root: &Path, artifact_id: Uuid) {
    let mut model = ProjectResolver::new(root)
        .resolve()
        .expect("project resolves before artifact metadata commit");
    let artifact = ArtifactMetadata {
        schema_version: ARTIFACT_METADATA_SCHEMA_VERSION,
        artifact_id,
        kind: ArtifactKind::GerberSet,
        project_id: model.project.project_id,
        model_revision: model.model_revision.clone(),
        output_job: None,
        variant: None,
        generator_version: "cli-test".to_string(),
        output_dir: Some(PathBuf::from("fab")),
        files: vec![ArtifactFile {
            path: PathBuf::from(format!("fab/{artifact_id}.gbr")),
            sha256: "sha256:5f9fd18a00b8234c45d8e981d96bf609c0c8b3d32a4f58b8c34f277ed111cb9b"
                .to_string(),
        }],
        production_projections: Vec::new(),
        validation_state: ArtifactValidationState::NotValidated,
    };
    model
        .commit_journaled(
            root,
            OperationBatch {
                batch_id: Uuid::new_v4(),
                expected_model_revision: Some(model.model_revision.clone()),
                provenance: CommitProvenance {
                    actor: "cli-test".to_string(),
                    source: CommitSource::Cli,
                    reason: "record artifact latest-order evidence".to_string(),
                },
                operations: vec![Operation::SetArtifactMetadata {
                    artifact_id,
                    previous_artifact_metadata: None,
                    artifact_metadata: serde_json::to_value(&artifact)
                        .expect("artifact metadata should serialize"),
                }],
            },
        )
        .expect("artifact metadata should commit");
}

fn bump_project_revision_for_artifact_latest_order(root: &Path) {
    let mut model = ProjectResolver::new(root)
        .resolve()
        .expect("project resolves before revision bump");
    model
        .commit_journaled(
            root,
            OperationBatch {
                batch_id: Uuid::new_v4(),
                expected_model_revision: Some(model.model_revision.clone()),
                provenance: CommitProvenance {
                    actor: "cli-test".to_string(),
                    source: CommitSource::Cli,
                    reason: "bump model revision for artifact latest-order evidence".to_string(),
                },
                operations: vec![Operation::SetProjectName {
                    project_id: model.project.project_id,
                    name: "Artifact Latest Order Demo Renamed".to_string(),
                }],
            },
        )
        .expect("project revision bump should commit");
}

#[test]
fn unlinked_artifact_generate_records_artifact_run_evidence() {
    let root = unique_project_root("datum-eda-cli-artifact-run");
    create_native_project(&root, Some("Artifact Run Demo".to_string()))
        .expect("initial scaffold should succeed");
    let authored_revision = ProjectResolver::new(&root)
        .resolve()
        .expect("project should resolve before artifact generation")
        .model_revision;
    let output_dir = root.join("generated-unlinked-artifacts");

    let generated = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "artifact",
            "generate",
            root.to_str().unwrap(),
            "--output-dir",
            output_dir.to_str().unwrap(),
            "--include",
            "bom",
            "--prefix",
            "Unlinked A",
        ])
        .expect("CLI should parse"),
    )
    .expect("unlinked artifact generate should succeed");
    let report: serde_json::Value =
        serde_json::from_str(&generated).expect("artifact generate JSON");
    let entry = &report["generated"][0];
    assert_eq!(entry["include"], "bom");
    assert_eq!(entry["output_job_run"], serde_json::Value::Null);
    assert_eq!(entry["artifact_run"]["schema_version"], 1);
    assert_eq!(entry["artifact_run"]["status"], "succeeded");
    assert_eq!(entry["artifact_run"]["run_sequence"], 1);
    assert_eq!(entry["artifact_run"]["artifact_id"], entry["artifact_id"]);
    assert!(std::path::Path::new(entry["artifact_run_path"].as_str().unwrap()).is_file());
    assert_eq!(entry["report"]["output_job_run"], serde_json::Value::Null);
    assert_eq!(entry["report"]["artifact_run"]["schema_version"], 1);
    assert_eq!(entry["report"]["artifact_run"]["status"], "succeeded");
    assert_eq!(entry["report"]["artifact_run"]["run_sequence"], 1);
    assert_eq!(
        entry["report"]["artifact_run"]["artifact_id"],
        entry["artifact_id"]
    );
    assert!(std::path::Path::new(entry["report"]["artifact_run_path"].as_str().unwrap()).is_file());

    let artifact_id = entry["artifact_id"].as_str().unwrap();
    let shown = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "artifact",
            "show",
            root.to_str().unwrap(),
            "--artifact",
            artifact_id,
        ])
        .expect("CLI should parse"),
    )
    .expect("artifact show should succeed");
    let shown: serde_json::Value = serde_json::from_str(&shown).expect("artifact show JSON");
    assert_eq!(shown["run_count"], 1);
    assert_eq!(shown["latest_run"]["schema_version"], 1);
    assert_eq!(shown["latest_run"]["artifact_id"], artifact_id);
    assert_eq!(shown["latest_run"]["run_sequence"], 1);

    let listed = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "artifact",
            "list",
            root.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("artifact list should succeed");
    let listed: serde_json::Value = serde_json::from_str(&listed).expect("artifact list JSON");
    assert_eq!(listed["artifact_run_count"], 1);
    assert_eq!(listed["latest_artifact_id"], artifact_id);
    assert_eq!(
        listed["latest_artifact_run_id"],
        entry["artifact_run"]["run_id"]
    );
    assert_eq!(listed["latest_output_job_run_id"], serde_json::Value::Null);
    assert_eq!(listed["artifact_runs"][0]["schema_version"], 1);
    assert_eq!(listed["artifact_runs"][0]["artifact_id"], artifact_id);
    assert_unlinked_artifact_evidence_replays_from_journal(&root, authored_revision, entry);

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn artifact_list_latest_artifact_uses_revision_not_uuid_order() {
    let root = unique_project_root("datum-eda-cli-artifact-list-latest-order");
    create_native_project(&root, Some("Artifact Latest Order Demo".to_string()))
        .expect("native project should be created");
    let older_artifact_id = Uuid::parse_str("ffffffff-ffff-ffff-ffff-ffffffffffff").unwrap();
    let newer_artifact_id = Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap();
    commit_artifact_metadata_pair_for_latest_order(&root, older_artifact_id, newer_artifact_id);

    let listed = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "artifact",
            "list",
            root.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("artifact list should succeed");
    let listed: serde_json::Value = serde_json::from_str(&listed).expect("artifact list JSON");
    assert_eq!(listed["artifact_count"], 2);
    assert_eq!(listed["latest_artifact_id"], newer_artifact_id.to_string());

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn artifact_generate_include_blocks_active_release_check_errors() {
    let root = unique_project_root("datum-eda-cli-artifact-run-check-gate");
    create_native_project(&root, Some("Artifact Run Check Gate Demo".to_string()))
        .expect("initial scaffold should succeed");
    place_unfilled_zone(&root);
    let output_dir = root.join("blocked-artifacts");

    let error = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "artifact",
            "generate",
            root.to_str().unwrap(),
            "--output-dir",
            output_dir.to_str().unwrap(),
            "--include",
            "bom",
            "--prefix",
            "Blocked",
        ])
        .expect("CLI should parse"),
    )
    .expect_err("artifact generate should be blocked by release check gate");
    let message = format!("{error:#}");
    assert!(message.contains("release check gate failed"));
    assert!(message.contains("2 active error code(s)"));
    assert!(message.contains("zone_fill_unfilled"));
    assert!(
        !output_dir.exists(),
        "release gate should block before artifact files are written"
    );

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_export_gerber_set_blocks_active_release_check_errors() {
    let root = unique_project_root("datum-eda-cli-gerber-export-check-gate");
    create_native_project(&root, Some("Gerber Export Check Gate Demo".to_string()))
        .expect("initial scaffold should succeed");
    place_unfilled_zone(&root);
    let output_dir = root.join("blocked-gerbers");

    let error = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "export-gerber-set",
            root.to_str().unwrap(),
            "--output-dir",
            output_dir.to_str().unwrap(),
            "--prefix",
            "Blocked",
        ])
        .expect("CLI should parse"),
    )
    .expect_err("gerber export should be blocked by release check gate");
    let message = format!("{error:#}");
    assert!(message.contains("release check gate failed"));
    assert!(message.contains("2 active error code(s)"));
    assert!(message.contains("zone_fill_unfilled"));
    assert!(!output_dir.exists());

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn artifact_export_manufacturing_set_blocks_active_release_check_errors() {
    let root = unique_project_root("datum-eda-cli-manufacturing-export-check-gate");
    create_native_project(
        &root,
        Some("Manufacturing Export Check Gate Demo".to_string()),
    )
    .expect("initial scaffold should succeed");
    place_unfilled_zone(&root);
    let output_dir = root.join("blocked-manufacturing");

    let error = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "artifact",
            "export-manufacturing-set",
            root.to_str().unwrap(),
            "--output-dir",
            output_dir.to_str().unwrap(),
            "--prefix",
            "Blocked",
        ])
        .expect("CLI should parse"),
    )
    .expect_err("manufacturing export should be blocked by release check gate");
    let message = format!("{error:#}");
    assert!(message.contains("release check gate failed"));
    assert!(message.contains("2 active error code(s)"));
    assert!(message.contains("zone_fill_unfilled"));
    assert!(!output_dir.exists());

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_export_manufacturing_set_blocks_active_release_check_errors() {
    let root = unique_project_root("datum-eda-cli-project-manufacturing-export-check-gate");
    create_native_project(
        &root,
        Some("Project Manufacturing Export Check Gate Demo".to_string()),
    )
    .expect("initial scaffold should succeed");
    place_unfilled_zone(&root);
    let output_dir = root.join("blocked-project-manufacturing");

    let error = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "export-manufacturing-set",
            root.to_str().unwrap(),
            "--output-dir",
            output_dir.to_str().unwrap(),
            "--prefix",
            "Blocked",
        ])
        .expect("CLI should parse"),
    )
    .expect_err("project manufacturing export should be blocked by release check gate");
    let message = format!("{error:#}");
    assert!(message.contains("release check gate failed"));
    assert!(message.contains("2 active error code(s)"));
    assert!(message.contains("zone_fill_unfilled"));
    assert!(!output_dir.exists());

    let _ = std::fs::remove_dir_all(&root);
}
