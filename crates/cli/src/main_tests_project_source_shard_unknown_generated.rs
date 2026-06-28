use super::*;
use eda_engine::substrate::{
    ARTIFACT_METADATA_SCHEMA_VERSION, ARTIFACT_RUN_SCHEMA_VERSION, ArtifactFile, ArtifactKind,
    ArtifactMetadata, ArtifactProductionProjection, ArtifactRun, ArtifactValidationState,
    CHECK_RUN_SCHEMA_VERSION, CheckFinding, CheckRun, CommitProvenance, CommitSource,
    OUTPUT_JOB_RUN_SCHEMA_VERSION, Operation, OperationBatch, OutputJob, OutputJobLogEntry,
    OutputJobLogLevel, OutputJobRun, OutputJobRunLauncher, OutputJobRunProvenance,
    OutputJobRunStatus, PRODUCTION_RECORD_SCHEMA_VERSION, ProjectResolver, SourceShardKind,
    ZoneFillState,
};

fn unique_project_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{}-{}", label, Uuid::new_v4()))
}

fn commit_artifact_metadata(root: &Path) -> Uuid {
    let mut model = ProjectResolver::new(root)
        .resolve()
        .expect("project resolves before artifact metadata");
    let artifact_id = Uuid::new_v4();
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
    model
        .commit_journaled(
            root,
            OperationBatch {
                batch_id: Uuid::new_v4(),
                expected_model_revision: Some(model.model_revision.clone()),
                provenance: CommitProvenance {
                    actor: "cli-test".to_string(),
                    source: CommitSource::Cli,
                    reason: "record unreadable generated evidence shard".to_string(),
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
    artifact_id
}

fn commit_check_run(root: &Path) -> Uuid {
    let mut model = ProjectResolver::new(root)
        .resolve()
        .expect("project resolves before check run");
    let check_run_id = Uuid::new_v4();
    let run = CheckRun {
        schema_version: CHECK_RUN_SCHEMA_VERSION,
        check_run_id,
        project_id: model.project.project_id,
        model_revision: model.model_revision.clone(),
        profile_id: "native-combined".to_string(),
        status: "warning".to_string(),
        summary: serde_json::json!({
            "status": "warning",
            "warnings": 1,
            "errors": 0
        }),
        finding_count: 1,
        findings: vec![CheckFinding {
            finding_id: Uuid::new_v5(&model.project.project_id, b"unknown-check-run-finding"),
            index: 0,
            source: "drc".to_string(),
            code: "process_aperture".to_string(),
            severity: "warning".to_string(),
            fingerprint: "sha256:5f9fd18a00b8234c45d8e981d96bf609c0c8b3d32a4f58b8c34f277ed111cb9b"
                .to_string(),
            domain: "drc".to_string(),
            rule_id: "process_aperture".to_string(),
            standards_basis: None,
            standards_basis_detail: None,
            rule_revision: None,
            import_key: None,
            status: "active".to_string(),
            primary_target: serde_json::json!({ "kind": "pad", "id": "pad-test" }),
            related_targets: Vec::new(),
            message: "mask expansion below profile".to_string(),
            explanation: "mask expansion below profile".to_string(),
            suggested_next_action: None,
            evidence: Vec::new(),
            payload: serde_json::json!({ "detail": "mask expansion below profile" }),
            proposal_refs: Vec::new(),
            proposal_links: Vec::new(),
            waiver_refs: Vec::new(),
            deviation_refs: Vec::new(),
        }],
        proposal_refs: Vec::new(),
        proposal_links: Vec::new(),
        profile_basis: Default::default(),
        coverage: Vec::new(),
        raw_report: serde_json::json!({ "domain": "combined" }),
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
                    reason: "record unreadable check run generated evidence shard".to_string(),
                },
                operations: vec![Operation::SetCheckRun {
                    check_run_id,
                    previous_check_run: None,
                    check_run: serde_json::to_value(&run).expect("check run should serialize"),
                }],
            },
        )
        .expect("check run should commit");
    check_run_id
}

fn commit_artifact_run(root: &Path) -> Uuid {
    let artifact_id = commit_artifact_metadata(root);
    let mut model = ProjectResolver::new(root)
        .resolve()
        .expect("project resolves before artifact run");
    let run_id = Uuid::new_v4();
    let run = ArtifactRun {
        schema_version: ARTIFACT_RUN_SCHEMA_VERSION,
        run_id,
        artifact_id,
        run_sequence: 1,
        project_id: model.project.project_id,
        model_revision: model.model_revision.clone(),
        status: OutputJobRunStatus::Succeeded,
        exit_code: Some(0),
        provenance: Some(OutputJobRunProvenance {
            launcher: OutputJobRunLauncher::GuiTerminal,
            terminal_session_id: Some("terminal-test".to_string()),
            terminal_context_path: Some(PathBuf::from(".datum/terminal-context.json")),
            project_root: Some(root.to_path_buf()),
            source_revision: None,
        }),
        log: vec![OutputJobLogEntry {
            sequence: 1,
            level: OutputJobLogLevel::Info,
            message: "generated artifact evidence".to_string(),
        }],
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
                    reason: "record unreadable artifact run generated evidence shard".to_string(),
                },
                operations: vec![Operation::SetArtifactRun {
                    run_id,
                    previous_artifact_run: None,
                    artifact_run: serde_json::to_value(&run)
                        .expect("artifact run should serialize"),
                }],
            },
        )
        .expect("artifact run should commit");
    run_id
}

fn commit_output_job_run(root: &Path) -> Uuid {
    let mut model = ProjectResolver::new(root)
        .resolve()
        .expect("project resolves before output job");
    let board_id = *model
        .objects
        .iter()
        .find(|(_, object)| {
            object.domain == "board"
                && model.source_shards.iter().any(|shard| {
                    shard.shard_id == object.source_shard_id
                        && shard.kind == SourceShardKind::BoardRoot
                })
        })
        .map(|(object_id, _)| object_id)
        .expect("native project should expose a board root object");
    let output_job_id = Uuid::new_v4();
    let output_job = OutputJob {
        schema_version: PRODUCTION_RECORD_SCHEMA_VERSION,
        id: output_job_id,
        name: "Unknown Output Job Run Evidence".to_string(),
        include: vec![ArtifactKind::GerberSet],
        prefix: "unknown-output-run".to_string(),
        output_dir: None,
        board_or_panel: board_id,
        variant: None,
        manufacturing_plan: None,
        object_revision: eda_engine::substrate::ObjectRevision(0),
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
                    reason: "create output job for unreadable run evidence shard".to_string(),
                },
                operations: vec![Operation::CreateOutputJob {
                    output_job_id,
                    output_job: serde_json::to_value(&output_job)
                        .expect("output job should serialize"),
                }],
            },
        )
        .expect("output job should commit");

    let run_id = Uuid::new_v4();
    let run = OutputJobRun {
        schema_version: OUTPUT_JOB_RUN_SCHEMA_VERSION,
        run_id,
        output_job: output_job_id,
        run_sequence: 1,
        project_id: model.project.project_id,
        model_revision: model.model_revision.clone(),
        status: OutputJobRunStatus::Succeeded,
        artifact_id: None,
        exit_code: Some(0),
        provenance: Some(OutputJobRunProvenance {
            launcher: OutputJobRunLauncher::GuiTerminal,
            terminal_session_id: Some("terminal-test".to_string()),
            terminal_context_path: Some(PathBuf::from(".datum/terminal-context.json")),
            project_root: Some(root.to_path_buf()),
            source_revision: None,
        }),
        log: vec![OutputJobLogEntry {
            sequence: 1,
            level: OutputJobLogLevel::Info,
            message: "generated output job evidence".to_string(),
        }],
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
                    reason: "record unreadable output job run generated evidence shard".to_string(),
                },
                operations: vec![Operation::SetOutputJobRun {
                    run_id,
                    previous_output_job_run: None,
                    output_job_run: serde_json::to_value(&run)
                        .expect("output job run should serialize"),
                }],
            },
        )
        .expect("output job run should commit");
    run_id
}

fn create_simple_fillable_zone(root: &Path) -> Uuid {
    let class_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "place-board-net-class",
            root.to_str().unwrap(),
            "--name",
            "ZoneFillClass",
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
    let net_uuid = net_report["net_uuid"].as_str().unwrap();
    let place_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "place-board-zone",
            root.to_str().unwrap(),
            "--net",
            net_uuid,
            "--vertex",
            "0:0",
            "--vertex",
            "1000000:0",
            "--vertex",
            "1000000:1000000",
            "--vertex",
            "0:1000000",
            "--layer",
            "1",
            "--priority",
            "2",
            "--thermal-relief",
            "false",
            "--thermal-gap-nm",
            "0",
            "--thermal-spoke-width-nm",
            "0",
        ])
        .expect("CLI should parse"),
    )
    .expect("place board zone should succeed");
    let placed: serde_json::Value =
        serde_json::from_str(&place_output).expect("place output should parse");
    Uuid::parse_str(placed["zone_uuid"].as_str().unwrap()).expect("zone UUID should parse")
}

fn fill_zone(root: &Path, zone_id: Uuid) {
    let zone_id = zone_id.to_string();
    execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "fill-zones",
            root.to_str().unwrap(),
            "--zone",
            &zone_id,
        ])
        .expect("CLI should parse"),
    )
    .expect("fill-zones should succeed");
}

fn commit_project_name_after_artifact_metadata(root: &Path) {
    let mut model = ProjectResolver::new(root)
        .resolve()
        .expect("project resolves before follow-up transaction");
    model
        .commit_journaled(
            root,
            OperationBatch {
                batch_id: Uuid::new_v4(),
                expected_model_revision: Some(model.model_revision.clone()),
                provenance: CommitProvenance {
                    actor: "cli-test".to_string(),
                    source: CommitSource::Cli,
                    reason: "force generated evidence replay across later transaction".to_string(),
                },
                operations: vec![Operation::SetProjectName {
                    project_id: model.project.project_id,
                    name: "Generated Evidence Replay Follow-Up".to_string(),
                }],
            },
        )
        .expect("follow-up project-name transaction should commit");
}

fn assert_unknown_generated_shard(
    root: &Path,
    output: &str,
    path: String,
    kind: &str,
    taxon: &str,
) {
    let report: serde_json::Value =
        serde_json::from_str(output).expect("resolve-debug JSON should parse");
    assert!(
        report["source_shards"]
            .as_array()
            .unwrap()
            .iter()
            .any(|shard| {
                shard["path"] == path
                    && shard["kind"] == kind
                    && shard["taxon"] == taxon
                    && shard["authority"] == "GeneratedEvidence"
                    && shard["dirty_state"] == "Unknown"
            }),
        "resolve-debug should expose unreadable generated evidence as Unknown for {}",
        root.display()
    );
}

fn assert_unknown_artifact_metadata(root: &Path, artifact_id: Uuid, output: &str) {
    assert_unknown_generated_shard(
        root,
        output,
        format!(".datum/artifacts/{artifact_id}.json"),
        "ArtifactMetadata",
        "ArtifactMetadata",
    );
}

#[test]
fn project_query_resolve_debug_reports_unknown_generated_evidence_shard() {
    let root = unique_project_root("datum-eda-cli-project-resolve-debug-unknown-shard");
    create_native_project(&root, Some("Resolve Debug Unknown Shard Demo".to_string()))
        .expect("initial scaffold should succeed");
    let artifact_id = commit_artifact_metadata(&root);
    let promoted_path = root.join(format!(".datum/artifacts/{artifact_id}.json"));
    std::fs::remove_file(&promoted_path).expect("promoted artifact metadata should remove");
    std::fs::create_dir(&promoted_path).expect("directory at promoted shard path should create");

    let output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "query",
            root.to_str().unwrap(),
            "resolve-debug",
        ])
        .expect("CLI should parse"),
    )
    .expect("project query resolve-debug should succeed");
    assert_unknown_artifact_metadata(&root, artifact_id, &output);

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_query_resolve_debug_reports_unknown_zone_fill_generated_evidence_shard() {
    let root = unique_project_root("datum-eda-cli-project-resolve-debug-unknown-zone-fill");
    create_native_project(
        &root,
        Some("Resolve Debug Unknown ZoneFill Evidence Demo".to_string()),
    )
    .expect("initial scaffold should succeed");
    let zone_id = create_simple_fillable_zone(&root);
    fill_zone(&root, zone_id);
    let promoted_path = root.join(format!(".datum/zone_fills/{zone_id}.json"));
    std::fs::remove_file(&promoted_path).expect("promoted zone fill should remove");
    std::fs::create_dir(&promoted_path)
        .expect("directory at promoted zone-fill path should create");

    let output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "query",
            root.to_str().unwrap(),
            "resolve-debug",
        ])
        .expect("CLI should parse"),
    )
    .expect("project query resolve-debug should succeed");
    assert_unknown_generated_shard(
        &root,
        &output,
        format!(".datum/zone_fills/{zone_id}.json"),
        "ZoneFill",
        "ZoneFill",
    );

    let model = ProjectResolver::new(&root)
        .resolve()
        .expect("project should still resolve with unreadable promoted zone fill");
    assert_eq!(
        model.zone_fills[&zone_id].state,
        ZoneFillState::Filled,
        "journal-recovered ZoneFill should still populate filled generated evidence"
    );

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_query_resolve_debug_reports_unknown_output_job_run_generated_evidence_shard() {
    let root = unique_project_root("datum-eda-cli-project-resolve-debug-unknown-output-job-run");
    create_native_project(
        &root,
        Some("Resolve Debug Unknown OutputJobRun Evidence Demo".to_string()),
    )
    .expect("initial scaffold should succeed");
    let run_id = commit_output_job_run(&root);
    let promoted_path = root.join(format!(".datum/output_job_runs/{run_id}.json"));
    std::fs::remove_file(&promoted_path).expect("promoted output job run should remove");
    std::fs::create_dir(&promoted_path)
        .expect("directory at promoted output-job-run path should create");

    let output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "query",
            root.to_str().unwrap(),
            "resolve-debug",
        ])
        .expect("CLI should parse"),
    )
    .expect("project query resolve-debug should succeed");
    assert_unknown_generated_shard(
        &root,
        &output,
        format!(".datum/output_job_runs/{run_id}.json"),
        "OutputJobRun",
        "OutputJobRun",
    );

    let model = ProjectResolver::new(&root)
        .resolve()
        .expect("project should still resolve with unreadable promoted output job run");
    assert!(
        model.output_job_runs.contains_key(&run_id),
        "journal-recovered OutputJobRun should still populate resolved evidence"
    );

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_query_resolve_debug_reports_unknown_artifact_run_generated_evidence_shard() {
    let root = unique_project_root("datum-eda-cli-project-resolve-debug-unknown-artifact-run");
    create_native_project(
        &root,
        Some("Resolve Debug Unknown ArtifactRun Evidence Demo".to_string()),
    )
    .expect("initial scaffold should succeed");
    let run_id = commit_artifact_run(&root);
    let promoted_path = root.join(format!(".datum/artifact_runs/{run_id}.json"));
    std::fs::remove_file(&promoted_path).expect("promoted artifact run should remove");
    std::fs::create_dir(&promoted_path)
        .expect("directory at promoted artifact-run path should create");

    let output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "query",
            root.to_str().unwrap(),
            "resolve-debug",
        ])
        .expect("CLI should parse"),
    )
    .expect("project query resolve-debug should succeed");
    assert_unknown_generated_shard(
        &root,
        &output,
        format!(".datum/artifact_runs/{run_id}.json"),
        "ArtifactRun",
        "ArtifactRun",
    );

    let model = ProjectResolver::new(&root)
        .resolve()
        .expect("project should still resolve with unreadable promoted artifact run");
    assert!(
        model.artifact_runs.contains_key(&run_id),
        "journal-recovered ArtifactRun should still populate resolved evidence"
    );

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_query_resolve_debug_reports_unknown_check_run_generated_evidence_shard() {
    let root = unique_project_root("datum-eda-cli-project-resolve-debug-unknown-check-run");
    create_native_project(
        &root,
        Some("Resolve Debug Unknown CheckRun Evidence Demo".to_string()),
    )
    .expect("initial scaffold should succeed");
    let check_run_id = commit_check_run(&root);
    let promoted_path = root.join(format!(".datum/check_runs/{check_run_id}.json"));
    std::fs::remove_file(&promoted_path).expect("promoted check run should remove");
    std::fs::create_dir(&promoted_path)
        .expect("directory at promoted check-run path should create");

    let output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "query",
            root.to_str().unwrap(),
            "resolve-debug",
        ])
        .expect("CLI should parse"),
    )
    .expect("project query resolve-debug should succeed");
    assert_unknown_generated_shard(
        &root,
        &output,
        format!(".datum/check_runs/{check_run_id}.json"),
        "CheckRun",
        "CheckRun",
    );

    let model = ProjectResolver::new(&root)
        .resolve()
        .expect("project should still resolve with unreadable promoted check run");
    assert!(
        model.check_runs.contains_key(&check_run_id),
        "journal-recovered CheckRun should still populate resolved evidence"
    );

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_query_resolve_debug_replays_unknown_generated_evidence_across_later_transaction() {
    let root = unique_project_root("datum-eda-cli-project-resolve-debug-unknown-evidence-later");
    create_native_project(
        &root,
        Some("Resolve Debug Unknown Evidence Later Demo".to_string()),
    )
    .expect("initial scaffold should succeed");
    let artifact_id = commit_artifact_metadata(&root);
    commit_project_name_after_artifact_metadata(&root);
    let promoted_path = root.join(format!(".datum/artifacts/{artifact_id}.json"));
    std::fs::remove_file(&promoted_path).expect("promoted artifact metadata should remove");
    std::fs::create_dir(&promoted_path).expect("directory at promoted shard path should create");

    let output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "query",
            root.to_str().unwrap(),
            "resolve-debug",
        ])
        .expect("CLI should parse"),
    )
    .expect("project query resolve-debug should succeed after later transaction");
    assert_unknown_artifact_metadata(&root, artifact_id, &output);

    let _ = std::fs::remove_dir_all(&root);
}
