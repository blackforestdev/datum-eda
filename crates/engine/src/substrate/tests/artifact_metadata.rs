use super::*;

const VALID_ARTIFACT_SHA256: &str =
    "sha256:5f9fd18a00b8234c45d8e981d96bf609c0c8b3d32a4f58b8c34f277ed111cb9b";
const VALID_PROJECTION_SHA256: &str =
    "sha256:28b3adfae87a0db63bb3e0f8bb9ea8f7c6f1f9955b5f7f4188c5bb47a0f5f3f6";

#[test]
fn artifact_metadata_serializes_with_substrate_contract_names() {
    let project_id = Uuid::new_v4();
    let artifact = ArtifactMetadata {
        artifact_id: Uuid::new_v5(&project_id, b"gerber-set"),
        kind: ArtifactKind::GerberSet,
        project_id,
        model_revision: ModelRevision("revision-a".to_string()),
        output_job: None,
        variant: None,
        generator_version: "test-generator".to_string(),
        output_dir: None,
        files: vec![ArtifactFile {
            path: PathBuf::from("board-F_Cu.gbr"),
            sha256: VALID_ARTIFACT_SHA256.to_string(),
        }],
        production_projections: vec![ArtifactProductionProjection {
            projection_kind: "gerber_copper_layer".to_string(),
            projection_contract: "datum.production_projection.gerber_copper_layer.v1".to_string(),
            model_revision: ModelRevision("revision-a".to_string()),
            byte_count: 128,
            sha256: VALID_PROJECTION_SHA256.to_string(),
        }],
        validation_state: ArtifactValidationState::NotValidated,
    };

    let json = serde_json::to_value(&artifact).expect("artifact metadata should serialize");
    assert_eq!(json["kind"], "gerber_set");
    assert_eq!(json["model_revision"], "revision-a");
    assert_eq!(json["validation_state"], "not_validated");
    assert_eq!(json["output_job"], serde_json::Value::Null);
    assert_eq!(json["variant"], serde_json::Value::Null);
    assert_eq!(json["files"][0]["path"], "board-F_Cu.gbr");
    assert_eq!(json["files"][0]["sha256"], VALID_ARTIFACT_SHA256);
    assert_eq!(
        json["production_projections"][0]["projection_kind"],
        "gerber_copper_layer"
    );
    assert_eq!(
        json["production_projections"][0]["projection_contract"],
        "datum.production_projection.gerber_copper_layer.v1"
    );
    assert_eq!(
        json["production_projections"][0]["model_revision"],
        "revision-a"
    );
    assert_eq!(json["production_projections"][0]["byte_count"], 128);
    assert_eq!(
        json["production_projections"][0]["sha256"],
        VALID_PROJECTION_SHA256
    );
}

#[test]
fn output_job_run_serializes_with_substrate_contract_names() {
    let project_id = Uuid::new_v4();
    let output_job = Uuid::new_v5(&project_id, b"gerber-set-job");
    let run = OutputJobRun {
        run_id: Uuid::new_v5(&project_id, b"gerber-set-run"),
        output_job,
        run_sequence: 7,
        project_id,
        model_revision: ModelRevision("revision-a".to_string()),
        status: OutputJobRunStatus::Succeeded,
        artifact_id: Some(Uuid::new_v5(&project_id, b"gerber-set-artifact")),
        exit_code: Some(0),
        provenance: Some(OutputJobRunProvenance {
            launcher: OutputJobRunLauncher::GuiTerminal,
            terminal_session_id: Some("terminal-test".to_string()),
            terminal_context_path: Some(PathBuf::from(".datum/gui-terminal-context.json")),
            project_root: Some(PathBuf::from("/tmp/datum/project")),
            source_revision: Some("source-revision-test".to_string()),
        }),
        log: vec![OutputJobLogEntry {
            sequence: 1,
            level: OutputJobLogLevel::Info,
            message: "generated Gerber set".to_string(),
        }],
    };

    let json = serde_json::to_value(&run).expect("output job run should serialize");
    assert_eq!(json["status"], "succeeded");
    assert_eq!(json["log"][0]["level"], "info");
    assert_eq!(json["output_job"], serde_json::json!(output_job));
    assert_eq!(json["run_sequence"], 7);
    assert_eq!(json["exit_code"], 0);
    assert_eq!(json["provenance"]["launcher"], "gui_terminal");
    assert_eq!(json["provenance"]["terminal_session_id"], "terminal-test");
    assert_eq!(
        json["provenance"]["terminal_context_path"],
        ".datum/gui-terminal-context.json"
    );
    assert_eq!(json["provenance"]["project_root"], "/tmp/datum/project");
    assert_eq!(
        json["provenance"]["source_revision"],
        "source-revision-test"
    );
}

#[test]
fn artifact_run_serializes_with_substrate_contract_names() {
    let project_id = Uuid::new_v4();
    let artifact_id = Uuid::new_v5(&project_id, b"ad-hoc-bom-artifact");
    let run = ArtifactRun {
        run_id: Uuid::new_v5(&project_id, b"ad-hoc-bom-run"),
        artifact_id,
        run_sequence: 3,
        project_id,
        model_revision: ModelRevision("revision-a".to_string()),
        status: OutputJobRunStatus::Succeeded,
        exit_code: Some(0),
        provenance: None,
        log: vec![OutputJobLogEntry {
            sequence: 1,
            level: OutputJobLogLevel::Info,
            message: "generated unlinked BOM artifact".to_string(),
        }],
    };

    let json = serde_json::to_value(&run).expect("artifact run should serialize");
    assert_eq!(json["artifact_id"], serde_json::json!(artifact_id));
    assert_eq!(json["run_sequence"], 3);
    assert_eq!(json["status"], "succeeded");
    assert_eq!(json["exit_code"], 0);
    assert_eq!(json["log"][0]["level"], "info");
}

#[test]
fn generated_evidence_persistence_helpers_round_trip_without_model_revision_changes() {
    let root = temp_project_root("generated_evidence_persistence");
    let project_id = Uuid::new_v4();
    write_minimal_project(&root, project_id, Uuid::new_v4());
    let initial = ProjectResolver::new(&root)
        .resolve()
        .expect("minimal project should resolve");
    let output_job = OutputJob {
        id: Uuid::new_v5(&project_id, b"output-job"),
        name: "Generated Evidence Job".to_string(),
        include: vec![ArtifactKind::GerberSet],
        prefix: "generated-evidence".to_string(),
        output_dir: None,
        board_or_panel: initial.project.project_id,
        variant: None,
        manufacturing_plan: None,
        object_revision: ObjectRevision(0),
    };
    let output_job_dir = root.join(".datum/output_jobs");
    std::fs::create_dir_all(&output_job_dir).expect("output job dir should create");
    std::fs::write(
        output_job_dir.join(format!("{}.json", output_job.id)),
        format!(
            "{}\n",
            to_json_deterministic(&output_job).expect("output job should serialize")
        ),
    )
    .expect("output job should write");
    let before = ProjectResolver::new(&root)
        .resolve()
        .expect("project with output job should resolve");
    let artifact = ArtifactMetadata {
        artifact_id: Uuid::new_v5(&project_id, b"persisted-artifact"),
        kind: ArtifactKind::GerberSet,
        project_id,
        model_revision: before.model_revision.clone(),
        output_job: None,
        variant: None,
        generator_version: "test-generator".to_string(),
        output_dir: None,
        files: vec![ArtifactFile {
            path: PathBuf::from("board-F_Cu.gbr"),
            sha256: VALID_ARTIFACT_SHA256.to_string(),
        }],
        production_projections: vec![ArtifactProductionProjection {
            projection_kind: "gerber_copper_layer".to_string(),
            projection_contract: "datum.production_projection.gerber_copper_layer.v1".to_string(),
            model_revision: before.model_revision.clone(),
            byte_count: 128,
            sha256: VALID_PROJECTION_SHA256.to_string(),
        }],
        validation_state: ArtifactValidationState::NotValidated,
    };
    let run = OutputJobRun {
        run_id: Uuid::new_v5(&project_id, b"persisted-run"),
        output_job: output_job.id,
        run_sequence: 1,
        project_id,
        model_revision: before.model_revision.clone(),
        status: OutputJobRunStatus::Succeeded,
        artifact_id: Some(artifact.artifact_id),
        exit_code: Some(0),
        provenance: None,
        log: vec![OutputJobLogEntry {
            sequence: 1,
            level: OutputJobLogLevel::Info,
            message: "generated Gerber set".to_string(),
        }],
    };
    let artifact_run = ArtifactRun {
        run_id: Uuid::new_v5(&project_id, b"persisted-artifact-run"),
        artifact_id: artifact.artifact_id,
        run_sequence: 1,
        project_id,
        model_revision: before.model_revision.clone(),
        status: OutputJobRunStatus::Succeeded,
        exit_code: Some(0),
        provenance: None,
        log: vec![OutputJobLogEntry {
            sequence: 1,
            level: OutputJobLogLevel::Info,
            message: "generated ad hoc artifact".to_string(),
        }],
    };
    let check_run = CheckRun {
        check_run_id: Uuid::new_v5(&project_id, b"persisted-check-run"),
        project_id,
        model_revision: before.model_revision.clone(),
        profile_id: "native-combined".to_string(),
        status: "warning".to_string(),
        summary: serde_json::json!({
            "status": "warning",
            "warnings": 1,
            "errors": 0
        }),
        finding_count: 1,
        findings: vec![CheckFinding {
            finding_id: Uuid::new_v5(&project_id, b"persisted-check-finding"),
            index: 0,
            source: "drc".to_string(),
            code: "process_aperture".to_string(),
            severity: "warning".to_string(),
            fingerprint:
                "sha256:5f9fd18a00b8234c45d8e981d96bf609c0c8b3d32a4f58b8c34f277ed111cb9b"
                    .to_string(),
            domain: "drc".to_string(),
            rule_id: "process_aperture".to_string(),
            standards_basis: None,
            rule_revision: None,
            import_key: None,
            status: "active".to_string(),
            primary_target: serde_json::Value::Null,
            related_targets: Vec::new(),
            message: "mask expansion below profile".to_string(),
            explanation:
                "mask expansion below profile Rule process_aperture produced this finding from the recorded evidence."
                    .to_string(),
            suggested_next_action: Some(
                "Run datum-eda check repair-standards to create reviewed repair proposals."
                    .to_string(),
            ),
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

    let artifact_path =
        persist_artifact_metadata(&root, &artifact).expect("artifact metadata should persist");
    let run_path = persist_output_job_run(&root, &run).expect("output job run should persist");
    let artifact_run_path =
        persist_artifact_run(&root, &artifact_run).expect("artifact run should persist");
    let check_run_path = persist_check_run(&root, &check_run).expect("check run should persist");
    assert_eq!(
        artifact_path,
        root.join(format!(".datum/artifacts/{}.json", artifact.artifact_id))
    );
    assert_eq!(
        run_path,
        root.join(format!(".datum/output_job_runs/{}.json", run.run_id))
    );
    assert_eq!(
        artifact_run_path,
        root.join(format!(".datum/artifact_runs/{}.json", artifact_run.run_id))
    );
    assert_eq!(
        check_run_path,
        root.join(format!(".datum/check_runs/{}.json", check_run.check_run_id))
    );

    let after = ProjectResolver::new(&root)
        .resolve()
        .expect("project with generated evidence should resolve");
    assert_eq!(after.model_revision, before.model_revision);
    assert_eq!(after.output_jobs[&output_job.id], output_job);
    assert_eq!(after.artifact_metadata[&artifact.artifact_id], artifact);
    assert_eq!(after.output_job_runs[&run.run_id], run);
    assert_eq!(after.artifact_runs[&artifact_run.run_id], artifact_run);
    assert_eq!(after.check_runs[&check_run.check_run_id], check_run);
    assert!(after.source_shards.iter().any(|shard| {
        shard.kind == SourceShardKind::ArtifactMetadata
            && shard.authority == SourceShardAuthority::GeneratedEvidence
    }));
    assert!(after.source_shards.iter().any(|shard| {
        shard.kind == SourceShardKind::OutputJobRun
            && shard.authority == SourceShardAuthority::GeneratedEvidence
    }));
    assert!(after.source_shards.iter().any(|shard| {
        shard.kind == SourceShardKind::ArtifactRun
            && shard.authority == SourceShardAuthority::GeneratedEvidence
    }));
    assert!(after.source_shards.iter().any(|shard| {
        shard.kind == SourceShardKind::CheckRun
            && shard.authority == SourceShardAuthority::GeneratedEvidence
    }));
}

#[test]
fn resolver_rejects_semantically_invalid_check_run_evidence() {
    let root = temp_project_root("invalid_check_run_evidence");
    let project_id = Uuid::new_v4();
    write_minimal_project(&root, project_id, Uuid::new_v4());
    let before = ProjectResolver::new(&root)
        .resolve()
        .expect("minimal project should resolve");
    let check_run_id = Uuid::new_v5(&project_id, b"invalid-check-run");
    let check_run = CheckRun {
        check_run_id,
        project_id,
        model_revision: before.model_revision,
        profile_id: "native-combined".to_string(),
        status: "warning".to_string(),
        summary: serde_json::json!({ "status": "warning" }),
        finding_count: 1,
        findings: vec![CheckFinding {
            finding_id: Uuid::new_v5(&project_id, b"invalid-check-finding"),
            index: 0,
            source: "drc".to_string(),
            code: "track_width_below_min".to_string(),
            severity: "warning".to_string(),
            fingerprint: "sha256:track-width".to_string(),
            domain: "drc".to_string(),
            rule_id: "track_width_below_min".to_string(),
            standards_basis: None,
            rule_revision: None,
            import_key: None,
            status: "active".to_string(),
            primary_target: serde_json::Value::Null,
            related_targets: Vec::new(),
            message: "track width below minimum".to_string(),
            explanation: "track width below minimum".to_string(),
            suggested_next_action: None,
            evidence: Vec::new(),
            payload: serde_json::json!({ "objects": [] }),
            proposal_refs: Vec::new(),
            proposal_links: Vec::new(),
            waiver_refs: Vec::new(),
            deviation_refs: Vec::new(),
        }],
        proposal_refs: Vec::new(),
        proposal_links: Vec::new(),
        profile_basis: Default::default(),
        coverage: Vec::new(),
        raw_report: serde_json::json!({ "domain": "drc" }),
    };
    persist_check_run(&root, &check_run).expect("invalid generated evidence can be written");

    let resolved = ProjectResolver::new(&root)
        .resolve()
        .expect("resolver should reject invalid check run without failing project resolution");

    assert!(!resolved.check_runs.contains_key(&check_run_id));
    assert!(resolved.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == "invalid_check_run"
            && diagnostic
                .message
                .contains("fingerprint must be a sha256:<64 lowercase hex> value")
    }));
}

#[test]
fn generated_evidence_reader_rejects_filename_id_mismatch() {
    let root = temp_project_root("generated_evidence_filename_guard");
    let project_id = Uuid::new_v4();
    write_minimal_project(&root, project_id, Uuid::new_v4());
    let before = ProjectResolver::new(&root)
        .resolve()
        .expect("minimal project should resolve");
    let artifact = ArtifactMetadata {
        artifact_id: Uuid::new_v5(&project_id, b"real-artifact-id"),
        kind: ArtifactKind::GerberSet,
        project_id,
        model_revision: before.model_revision.clone(),
        output_job: None,
        variant: None,
        generator_version: "test-generator".to_string(),
        output_dir: None,
        files: vec![ArtifactFile {
            path: PathBuf::from("board-F_Cu.gbr"),
            sha256: VALID_ARTIFACT_SHA256.to_string(),
        }],
        production_projections: Vec::new(),
        validation_state: ArtifactValidationState::NotValidated,
    };
    let artifact_dir = root.join(".datum/artifacts");
    std::fs::create_dir_all(&artifact_dir).expect("artifact dir should create");
    std::fs::write(
        artifact_dir.join(format!("{}.json", Uuid::new_v4())),
        format!(
            "{}\n",
            to_json_deterministic(&artifact).expect("artifact should serialize")
        ),
    )
    .expect("mismatched artifact metadata should write");

    let run = OutputJobRun {
        run_id: Uuid::new_v5(&project_id, b"real-run-id"),
        output_job: Uuid::new_v5(&project_id, b"output-job"),
        run_sequence: 1,
        project_id,
        model_revision: before.model_revision.clone(),
        status: OutputJobRunStatus::Succeeded,
        artifact_id: Some(artifact.artifact_id),
        exit_code: Some(0),
        provenance: None,
        log: Vec::new(),
    };
    let run_dir = root.join(".datum/output_job_runs");
    std::fs::create_dir_all(&run_dir).expect("run dir should create");
    std::fs::write(
        run_dir.join(format!("{}.json", Uuid::new_v4())),
        format!(
            "{}\n",
            to_json_deterministic(&run).expect("run should serialize")
        ),
    )
    .expect("mismatched output job run should write");

    let resolved = ProjectResolver::new(&root)
        .resolve()
        .expect("project should resolve with diagnostics");
    assert!(
        !resolved
            .artifact_metadata
            .contains_key(&artifact.artifact_id)
    );
    assert!(!resolved.output_job_runs.contains_key(&run.run_id));
    assert!(
        resolved
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "invalid_artifact_metadata"
                && diagnostic.message.contains("filename does not match"))
    );
    assert!(
        resolved
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "invalid_output_job_run"
                && diagnostic.message.contains("filename does not match"))
    );
}

#[test]
fn resolver_exposes_output_job_and_artifact_metadata_state() {
    let root = temp_project_root("artifact_metadata");
    write_minimal_project(&root, Uuid::new_v4(), Uuid::new_v4());

    let before = ProjectResolver::new(&root)
        .resolve()
        .expect("minimal project should resolve");
    assert!(before.manufacturing_plans.is_empty());
    assert!(before.panel_projections.is_empty());
    assert!(before.output_jobs.is_empty());
    assert!(before.artifact_metadata.is_empty());

    let panel_projection = PanelProjection {
        id: Uuid::new_v5(&before.project.project_id, b"panel-projection-release-a"),
        name: "Release A panel".to_string(),
        board_instances: vec![PanelBoardInstance {
            board: before.project.project_id,
            x_nm: 0,
            y_nm: 0,
            rotation_deg: 0,
        }],
        object_revision: ObjectRevision(0),
    };
    let panel_projection_dir = root.join(".datum/panel_projections");
    std::fs::create_dir_all(&panel_projection_dir).expect("panel projection dir should create");
    std::fs::write(
        panel_projection_dir.join(format!("{}.json", panel_projection.id)),
        format!(
            "{}\n",
            to_json_deterministic(&panel_projection).expect("panel projection should serialize")
        ),
    )
    .expect("panel projection should write");

    let with_panel = ProjectResolver::new(&root)
        .resolve()
        .expect("project with panel projection should resolve");
    assert_ne!(with_panel.model_revision, before.model_revision);
    assert_eq!(
        with_panel.panel_projections[&panel_projection.id],
        panel_projection
    );
    assert_eq!(
        with_panel.objects[&panel_projection.id].kind,
        "panel_projection"
    );
    assert!(
        with_panel
            .source_shards
            .iter()
            .any(|shard| shard.kind == SourceShardKind::PanelProjection)
    );

    let manufacturing_plan = ManufacturingPlan {
        id: Uuid::new_v5(
            &with_panel.project.project_id,
            b"manufacturing-plan-release-a",
        ),
        name: "Release A".to_string(),
        board_or_panel: panel_projection.id,
        variant: None,
        prefix: "release-a".to_string(),
        object_revision: ObjectRevision(0),
    };
    let manufacturing_plan_dir = root.join(".datum/manufacturing_plans");
    std::fs::create_dir_all(&manufacturing_plan_dir).expect("manufacturing plan dir should create");
    std::fs::write(
        manufacturing_plan_dir.join(format!("{}.json", manufacturing_plan.id)),
        format!(
            "{}\n",
            to_json_deterministic(&manufacturing_plan)
                .expect("manufacturing plan should serialize")
        ),
    )
    .expect("manufacturing plan should write");

    let with_plan = ProjectResolver::new(&root)
        .resolve()
        .expect("project with manufacturing plan should resolve");
    assert_ne!(with_plan.model_revision, with_panel.model_revision);
    assert_eq!(
        with_plan.manufacturing_plans[&manufacturing_plan.id],
        manufacturing_plan
    );
    assert!(
        with_plan
            .source_shards
            .iter()
            .any(|shard| shard.kind == SourceShardKind::ManufacturingPlan)
    );

    let output_job = OutputJob {
        id: Uuid::new_v5(&with_plan.project.project_id, b"gerber-set-job"),
        name: "Gerber set".to_string(),
        include: vec![ArtifactKind::GerberSet],
        prefix: "release-a".to_string(),
        output_dir: None,
        board_or_panel: with_plan
            .objects
            .values()
            .find(|object| object.kind == "uuid")
            .map(|object| object.object_id)
            .unwrap_or(with_plan.project.project_id),
        variant: None,
        manufacturing_plan: Some(manufacturing_plan.id),
        object_revision: ObjectRevision(0),
    };
    let output_job_dir = root.join(".datum/output_jobs");
    std::fs::create_dir_all(&output_job_dir).expect("output job dir should create");
    std::fs::write(
        output_job_dir.join(format!("{}.json", output_job.id)),
        format!(
            "{}\n",
            to_json_deterministic(&output_job).expect("output job should serialize")
        ),
    )
    .expect("output job should write");

    let with_job = ProjectResolver::new(&root)
        .resolve()
        .expect("project with output job should resolve");
    assert_ne!(with_job.model_revision, with_plan.model_revision);
    assert_eq!(with_job.output_jobs[&output_job.id], output_job);
    assert!(
        with_job
            .source_shards
            .iter()
            .any(|shard| shard.kind == SourceShardKind::OutputJob)
    );

    let artifact = ArtifactMetadata {
        artifact_id: Uuid::new_v5(&with_job.project.project_id, b"gerber-set"),
        kind: ArtifactKind::GerberSet,
        project_id: with_job.project.project_id,
        model_revision: with_job.model_revision.clone(),
        output_job: Some(output_job.id),
        variant: None,
        generator_version: "test-generator".to_string(),
        output_dir: None,
        files: vec![ArtifactFile {
            path: PathBuf::from("board-F_Cu.gbr"),
            sha256: VALID_ARTIFACT_SHA256.to_string(),
        }],
        production_projections: Vec::new(),
        validation_state: ArtifactValidationState::NotValidated,
    };
    let artifact_dir = root.join(".datum/artifacts");
    std::fs::create_dir_all(&artifact_dir).expect("artifact dir should create");
    std::fs::write(
        artifact_dir.join(format!("{}.json", artifact.artifact_id)),
        format!(
            "{}\n",
            to_json_deterministic(&artifact).expect("artifact should serialize")
        ),
    )
    .expect("artifact metadata should write");

    let after = ProjectResolver::new(&root)
        .resolve()
        .expect("project with artifact metadata should resolve");
    assert_eq!(after.model_revision, with_job.model_revision);
    assert_eq!(after.output_jobs[&output_job.id], output_job);
    assert_eq!(after.artifact_metadata[&artifact.artifact_id], artifact);
    assert!(
        after
            .source_shards
            .iter()
            .any(|shard| shard.kind == SourceShardKind::ArtifactMetadata)
    );

    let run = OutputJobRun {
        run_id: Uuid::new_v5(&with_job.project.project_id, b"gerber-set-run"),
        output_job: output_job.id,
        run_sequence: 1,
        project_id: with_job.project.project_id,
        model_revision: with_job.model_revision.clone(),
        status: OutputJobRunStatus::Succeeded,
        artifact_id: Some(artifact.artifact_id),
        exit_code: Some(0),
        provenance: None,
        log: vec![OutputJobLogEntry {
            sequence: 1,
            level: OutputJobLogLevel::Info,
            message: "generated Gerber set".to_string(),
        }],
    };
    let run_dir = root.join(".datum/output_job_runs");
    std::fs::create_dir_all(&run_dir).expect("output job run dir should create");
    std::fs::write(
        run_dir.join(format!("{}.json", run.run_id)),
        format!(
            "{}\n",
            to_json_deterministic(&run).expect("output job run should serialize")
        ),
    )
    .expect("output job run should write");

    let after_run = ProjectResolver::new(&root)
        .resolve()
        .expect("project with output job run should resolve");
    assert_eq!(after_run.model_revision, after.model_revision);
    assert_eq!(after_run.output_job_runs[&run.run_id], run);
    assert!(
        after_run
            .source_shards
            .iter()
            .any(|shard| shard.kind == SourceShardKind::OutputJobRun)
    );

    let _ = std::fs::remove_dir_all(&root);
}
