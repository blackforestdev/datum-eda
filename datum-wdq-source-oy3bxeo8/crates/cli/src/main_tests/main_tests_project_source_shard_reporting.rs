use super::*;
use eda_engine::ir::serialization::to_json_deterministic;
use eda_engine::substrate::{
    ARTIFACT_METADATA_SCHEMA_VERSION, ArtifactFile, ArtifactKind, ArtifactMetadata,
    ArtifactProductionProjection, ArtifactValidationState, CommitProvenance, CommitSource,
    ManufacturingPlan, ObjectRevision, Operation, OperationBatch, OutputJob,
    PRODUCTION_RECORD_SCHEMA_VERSION, PanelBoardInstance, PanelProjection, ProjectResolver,
};

fn unique_project_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{}-{}", label, Uuid::new_v4()))
}

#[test]
fn project_query_resolve_debug_reports_dirty_materialized_source_shard() {
    let root = unique_project_root("datum-eda-cli-project-resolve-debug-dirty-shard");
    create_native_project(&root, Some("Resolve Debug Dirty Shard Demo".to_string()))
        .expect("initial scaffold should succeed");
    let board_path = root.join("board/board.json");
    let original_board_bytes = std::fs::read(&board_path).expect("board should read");
    let board: serde_json::Value =
        serde_json::from_slice(&original_board_bytes).expect("board should parse");
    let board_id =
        Uuid::parse_str(board["uuid"].as_str().expect("board uuid should exist")).unwrap();
    let model = ProjectResolver::new(&root)
        .resolve()
        .expect("project resolves before board name update");
    let batch_path = root.join("commit-batch-dirty-shard.json");
    let batch = OperationBatch {
        batch_id: Uuid::new_v4(),
        expected_model_revision: Some(model.model_revision),
        provenance: CommitProvenance {
            actor: "cli-test".to_string(),
            source: CommitSource::Cli,
            reason: "create stale promoted board shard".to_string(),
        },
        operations: vec![Operation::SetBoardName {
            board_id,
            name: "Journaled Board Name".to_string(),
        }],
    };
    std::fs::write(
        &batch_path,
        to_json_deterministic(&batch).expect("batch should serialize"),
    )
    .expect("batch should write");

    execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "query",
            root.to_str().unwrap(),
            "resolve-debug",
            "--commit-batch",
            batch_path.to_str().unwrap(),
            "--apply",
        ])
        .expect("CLI should parse"),
    )
    .expect("project query resolve-debug --apply should succeed");
    std::fs::write(&board_path, original_board_bytes).expect("stale promoted board should write");

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
    let report: serde_json::Value =
        serde_json::from_str(&output).expect("resolve-debug JSON should parse");
    assert!(
        report["source_shards"]
            .as_array()
            .unwrap()
            .iter()
            .any(|shard| {
                shard["path"] == "board/board.json"
                    && shard["kind"] == "BoardRoot"
                    && shard["dirty_state"] == "Dirty"
            }),
        "resolve-debug should expose stale promoted board shard as Dirty"
    );

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_query_resolve_debug_reports_missing_generated_evidence_shard() {
    let root = unique_project_root("datum-eda-cli-project-resolve-debug-missing-shard");
    create_native_project(&root, Some("Resolve Debug Missing Shard Demo".to_string()))
        .expect("initial scaffold should succeed");
    let mut model = ProjectResolver::new(&root)
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
            &root,
            OperationBatch {
                batch_id: Uuid::new_v4(),
                expected_model_revision: Some(model.model_revision.clone()),
                provenance: CommitProvenance {
                    actor: "cli-test".to_string(),
                    source: CommitSource::Cli,
                    reason: "record missing generated evidence shard".to_string(),
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
    std::fs::remove_file(root.join(format!(".datum/artifacts/{artifact_id}.json")))
        .expect("promoted artifact metadata should remove");

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
    let report: serde_json::Value =
        serde_json::from_str(&output).expect("resolve-debug JSON should parse");
    assert!(
        report["source_shards"]
            .as_array()
            .unwrap()
            .iter()
            .any(|shard| {
                shard["path"] == format!(".datum/artifacts/{artifact_id}.json")
                    && shard["kind"] == "ArtifactMetadata"
                    && shard["taxon"] == "ArtifactMetadata"
                    && shard["authority"] == "GeneratedEvidence"
                    && shard["dirty_state"] == "Missing"
            }),
        "resolve-debug should expose journal-recovered generated evidence taxonomy as Missing"
    );

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_query_resolve_debug_reports_missing_forward_annotation_review_sidecar() {
    let root = unique_project_root("datum-eda-cli-project-resolve-debug-missing-fa-review");
    create_native_project(&root, Some("Resolve Debug Missing Review Demo".to_string()))
        .expect("initial scaffold should succeed");
    let mut model = ProjectResolver::new(&root)
        .resolve()
        .expect("project resolves before review sidecar");
    let review = serde_json::json!({
        "schema_version": 1,
        "reviews": {
            "action-1": {
                "action_id": "action-1",
                "status": "deferred"
            }
        }
    });
    model
        .commit_journaled(
            &root,
            OperationBatch {
                batch_id: Uuid::new_v4(),
                expected_model_revision: Some(model.model_revision.clone()),
                provenance: CommitProvenance {
                    actor: "cli-test".to_string(),
                    source: CommitSource::Cli,
                    reason: "record missing forward-annotation review shard".to_string(),
                },
                operations: vec![Operation::SetForwardAnnotationReview {
                    relative_path: ".datum/forward_annotation_review/review.json".to_string(),
                    previous_review: None,
                    review,
                }],
            },
        )
        .expect("forward-annotation review should commit");
    std::fs::remove_file(root.join(".datum/forward_annotation_review/review.json"))
        .expect("promoted review sidecar should remove");

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
    let report: serde_json::Value =
        serde_json::from_str(&output).expect("resolve-debug JSON should parse");
    assert!(
        report["source_shards"]
            .as_array()
            .unwrap()
            .iter()
            .any(|shard| {
                shard["path"] == ".datum/forward_annotation_review/review.json"
                    && shard["kind"] == "ForwardAnnotationReview"
                    && shard["taxon"] == "ForwardAnnotationReview"
                    && shard["authority"] == "SidecarMetadata"
                    && shard["dirty_state"] == "Missing"
            }),
        "resolve-debug should expose journal-recovered review sidecar taxonomy as Missing"
    );

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_query_resolve_debug_reports_missing_pool_taxon() {
    let root = unique_project_root("datum-eda-cli-project-resolve-debug-missing-pool-taxon");
    create_native_project(
        &root,
        Some("Resolve Debug Missing Pool Taxon Demo".to_string()),
    )
    .expect("initial scaffold should succeed");
    let mut model = ProjectResolver::new(&root)
        .resolve()
        .expect("project resolves before pool symbol");
    let symbol_id = Uuid::new_v4();
    let relative_path = format!("pool/symbols/{symbol_id}.json");
    model
        .commit_journaled(
            &root,
            OperationBatch {
                batch_id: Uuid::new_v4(),
                expected_model_revision: Some(model.model_revision.clone()),
                provenance: CommitProvenance {
                    actor: "cli-test".to_string(),
                    source: CommitSource::Cli,
                    reason: "record missing pool symbol shard".to_string(),
                },
                operations: vec![Operation::CreatePoolLibraryObject {
                    object_id: symbol_id,
                    relative_path: relative_path.clone(),
                    object_kind: "symbols".to_string(),
                    object: serde_json::json!({
                        "schema_version": 1,
                        "uuid": symbol_id,
                        "name": "ResolveDebugSymbol",
                        "unit": Uuid::new_v4()
                    }),
                }],
            },
        )
        .expect("pool symbol should commit");
    std::fs::remove_file(root.join(&relative_path)).expect("promoted pool symbol should remove");

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
    let report: serde_json::Value =
        serde_json::from_str(&output).expect("resolve-debug JSON should parse");
    assert!(
        report["source_shards"]
            .as_array()
            .unwrap()
            .iter()
            .any(|shard| {
                shard["path"] == relative_path
                    && shard["kind"] == "Pool"
                    && shard["taxon"] == "PoolSymbol"
                    && shard["authority"] == "AuthoredDesign"
                    && shard["dirty_state"] == "Missing"
            }),
        "resolve-debug should expose missing journal-recovered pool symbol taxonomy"
    );

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_query_resolve_debug_reports_missing_production_taxons() {
    let root = unique_project_root("datum-eda-cli-project-resolve-debug-missing-production-taxons");
    create_native_project(
        &root,
        Some("Resolve Debug Missing Production Taxons Demo".to_string()),
    )
    .expect("initial scaffold should succeed");
    let board_path = root.join("board/board.json");
    let board: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&board_path).expect("board should read"))
            .expect("board should parse");
    let board_id =
        Uuid::parse_str(board["uuid"].as_str().expect("board uuid should exist")).unwrap();
    let mut model = ProjectResolver::new(&root)
        .resolve()
        .expect("project resolves before production records");
    let plan_id = Uuid::new_v4();
    let panel_id = Uuid::new_v4();
    let job_id = Uuid::new_v4();
    let panel = PanelProjection {
        schema_version: PRODUCTION_RECORD_SCHEMA_VERSION,
        id: panel_id,
        name: "Resolve Debug Panel".to_string(),
        board_instances: vec![PanelBoardInstance {
            board: board_id,
            x_nm: 0,
            y_nm: 0,
            rotation_deg: 0,
        }],
        object_revision: ObjectRevision(0),
    };
    let plan = ManufacturingPlan {
        schema_version: PRODUCTION_RECORD_SCHEMA_VERSION,
        id: plan_id,
        name: "Resolve Debug Plan".to_string(),
        board_or_panel: panel_id,
        variant: None,
        prefix: "resolve-debug".to_string(),
        object_revision: ObjectRevision(0),
    };
    let job = OutputJob {
        schema_version: PRODUCTION_RECORD_SCHEMA_VERSION,
        id: job_id,
        name: "Resolve Debug Gerbers".to_string(),
        include: vec![ArtifactKind::GerberSet],
        prefix: "resolve-debug".to_string(),
        output_dir: None,
        board_or_panel: panel_id,
        variant: None,
        manufacturing_plan: Some(plan_id),
        object_revision: ObjectRevision(0),
    };
    model
        .commit_journaled(
            &root,
            OperationBatch {
                batch_id: Uuid::new_v4(),
                expected_model_revision: Some(model.model_revision.clone()),
                provenance: CommitProvenance {
                    actor: "cli-test".to_string(),
                    source: CommitSource::Cli,
                    reason: "record missing production taxon shards".to_string(),
                },
                operations: vec![
                    Operation::CreatePanelProjection {
                        panel_projection_id: panel_id,
                        panel_projection: serde_json::to_value(&panel)
                            .expect("panel should serialize"),
                    },
                    Operation::CreateManufacturingPlan {
                        manufacturing_plan_id: plan_id,
                        manufacturing_plan: serde_json::to_value(&plan)
                            .expect("plan should serialize"),
                    },
                    Operation::CreateOutputJob {
                        output_job_id: job_id,
                        output_job: serde_json::to_value(&job).expect("job should serialize"),
                    },
                ],
            },
        )
        .expect("production records should commit");
    std::fs::remove_file(root.join(format!(".datum/manufacturing_plans/{plan_id}.json")))
        .expect("promoted manufacturing plan should remove");
    std::fs::remove_file(root.join(format!(".datum/panel_projections/{panel_id}.json")))
        .expect("promoted panel projection should remove");
    std::fs::remove_file(root.join(format!(".datum/output_jobs/{job_id}.json")))
        .expect("promoted output job should remove");

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
    let report: serde_json::Value =
        serde_json::from_str(&output).expect("resolve-debug JSON should parse");
    for (path, kind, taxon) in [
        (
            format!(".datum/manufacturing_plans/{plan_id}.json"),
            "ManufacturingPlan",
            "ManufacturingPlan",
        ),
        (
            format!(".datum/panel_projections/{panel_id}.json"),
            "PanelProjection",
            "PanelProjection",
        ),
        (
            format!(".datum/output_jobs/{job_id}.json"),
            "OutputJob",
            "OutputJob",
        ),
    ] {
        assert!(
            report["source_shards"]
                .as_array()
                .unwrap()
                .iter()
                .any(|shard| {
                    shard["path"] == path
                        && shard["kind"] == kind
                        && shard["taxon"] == taxon
                        && shard["authority"] == "AuthoredDesign"
                        && shard["dirty_state"] == "Missing"
                }),
            "resolve-debug should expose missing journal-recovered {kind} taxonomy"
        );
    }

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_query_resolve_debug_reports_missing_identity_relationship_taxons() {
    let root = unique_project_root("datum-eda-cli-project-resolve-debug-missing-identity-taxons");
    create_native_project(
        &root,
        Some("Resolve Debug Missing Identity Taxons Demo".to_string()),
    )
    .expect("initial scaffold should succeed");
    let project: serde_json::Value =
        serde_json::from_slice(&std::fs::read(root.join("project.json")).unwrap()).unwrap();
    let project_id = Uuid::parse_str(project["uuid"].as_str().unwrap()).unwrap();
    let mut schematic: serde_json::Value =
        serde_json::from_slice(&std::fs::read(root.join("schematic/schematic.json")).unwrap())
            .unwrap();
    let sheet_id = Uuid::new_v4();
    let sheet_path = format!("sheets/{sheet_id}.json");
    schematic["sheets"][sheet_id.to_string()] = serde_json::Value::String(sheet_path.clone());
    std::fs::write(
        root.join("schematic/schematic.json"),
        format!("{}\n", to_json_deterministic(&schematic).unwrap()),
    )
    .unwrap();
    let sheet_file = root.join("schematic").join(&sheet_path);
    std::fs::create_dir_all(sheet_file.parent().unwrap()).unwrap();
    let symbol_id = Uuid::new_v4();
    let part_id = Uuid::new_v4();
    let sheet = serde_json::json!({
        "schema_version": 1,
        "uuid": sheet_id,
        "name": "Main",
        "symbols": {
            symbol_id.to_string(): {
                "uuid": symbol_id,
                "part": part_id,
                "entity": Uuid::new_v5(&project_id, b"entity"),
                "gate": Uuid::new_v5(&project_id, b"gate"),
                "lib_id": "test:R",
                "reference": "U1",
                "value": "OLD",
                "fields": [],
                "pins": [],
                "position": { "x": 0, "y": 0 },
                "rotation": 0,
                "mirrored": false,
                "unit_selection": null,
                "display_mode": "LibraryDefault",
                "pin_overrides": [],
                "hidden_power_behavior": "SourceDefinedImplicit"
            }
        },
        "wires": {},
        "junctions": {},
        "labels": {},
        "buses": {},
        "bus_entries": {},
        "ports": {},
        "noconnects": {},
        "texts": {},
        "drawings": {}
    });
    std::fs::write(
        &sheet_file,
        format!("{}\n", to_json_deterministic(&sheet).unwrap()),
    )
    .unwrap();
    let package_id = Uuid::new_v4();
    let mut board: serde_json::Value =
        serde_json::from_slice(&std::fs::read(root.join("board/board.json")).unwrap()).unwrap();
    board["packages"][package_id.to_string()] = serde_json::json!({
        "uuid": package_id,
        "part": part_id,
        "package": Uuid::new_v5(&project_id, b"package"),
        "reference": "U1",
        "value": "OLD",
        "position": { "x": 0, "y": 0 },
        "rotation": 0,
        "layer": 0,
        "locked": false
    });
    std::fs::write(
        root.join("board/board.json"),
        format!("{}\n", to_json_deterministic(&board).unwrap()),
    )
    .unwrap();

    let mut model = ProjectResolver::new(&root)
        .resolve()
        .expect("project resolves before identity records");
    let component_instance_id = Uuid::new_v4();
    let relationship_id = Uuid::new_v4();
    let variant_id = Uuid::new_v4();
    model
        .commit_journaled(
            &root,
            OperationBatch {
                batch_id: Uuid::new_v4(),
                expected_model_revision: Some(model.model_revision.clone()),
                provenance: CommitProvenance {
                    actor: "cli-test".to_string(),
                    source: CommitSource::Cli,
                    reason: "record missing identity/relationship taxon shards".to_string(),
                },
                operations: vec![
                    Operation::CreateComponentInstance {
                        component_instance_id,
                        component_instance: serde_json::json!({
                            "uuid": component_instance_id,
                            "object_revision": 0,
                            "placed_symbol_refs": [{
                                "object_id": symbol_id,
                                "object_revision": 0
                            }],
                            "placed_package_refs": [{
                                "object_id": package_id,
                                "object_revision": 0
                            }]
                        }),
                    },
                    Operation::CreateRelationship {
                        relationship_id,
                        relationship: serde_json::json!({
                            "id": relationship_id,
                            "kind": "implemented_by",
                            "from": [{
                                "object_id": symbol_id,
                                "object_revision": 0
                            }],
                            "to": [{
                                "object_id": package_id,
                                "object_revision": 0
                            }],
                            "authored_intent": [],
                            "object_revision": 0
                        }),
                    },
                    Operation::CreateVariantOverlay {
                        variant_id,
                        variant: serde_json::json!({
                            "id": variant_id,
                            "name": "No U1",
                            "base_model_revision": model.model_revision,
                            "variant_revision": 0,
                            "fitted": {
                                package_id.to_string(): "unfitted"
                            },
                            "relationship_overrides": {},
                            "property_overrides": {}
                        }),
                    },
                ],
            },
        )
        .expect("identity/relationship records should commit");
    std::fs::remove_file(root.join(format!(
        ".datum/component_instances/{component_instance_id}.json"
    )))
    .expect("promoted component instance should remove");
    std::fs::remove_file(root.join(format!(".datum/relationships/{relationship_id}.json")))
        .expect("promoted relationship should remove");
    std::fs::remove_file(root.join(format!(".datum/variants/{variant_id}.json")))
        .expect("promoted variant should remove");

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
    let report: serde_json::Value =
        serde_json::from_str(&output).expect("resolve-debug JSON should parse");
    for (path, kind, taxon) in [
        (
            format!(".datum/component_instances/{component_instance_id}.json"),
            "ComponentInstance",
            "ComponentInstance",
        ),
        (
            format!(".datum/relationships/{relationship_id}.json"),
            "Relationship",
            "Relationship",
        ),
        (
            format!(".datum/variants/{variant_id}.json"),
            "VariantOverlay",
            "VariantOverlay",
        ),
    ] {
        assert!(
            report["source_shards"]
                .as_array()
                .unwrap()
                .iter()
                .any(|shard| {
                    shard["path"] == path
                        && shard["kind"] == kind
                        && shard["taxon"] == taxon
                        && shard["authority"] == "AuthoredDesign"
                        && shard["dirty_state"] == "Missing"
                }),
            "resolve-debug should expose missing journal-recovered {kind} taxonomy"
        );
    }

    let _ = std::fs::remove_dir_all(&root);
}
