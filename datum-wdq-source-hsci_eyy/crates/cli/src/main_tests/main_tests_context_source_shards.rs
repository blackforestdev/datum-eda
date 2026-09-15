use super::*;
use eda_engine::substrate::{
    ARTIFACT_METADATA_SCHEMA_VERSION, ArtifactFile, ArtifactKind, ArtifactMetadata,
    ArtifactProductionProjection, ArtifactValidationState, CommitProvenance, CommitSource,
    Operation, OperationBatch, ProjectResolver,
};

fn unique_context_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{}-{}", label, Uuid::new_v4()))
}

fn commit_context_artifact_metadata(root: &Path) -> Uuid {
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
                    reason: "record unknown context evidence shard".to_string(),
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

#[test]
fn context_refresh_exposes_missing_identity_relationship_source_shards() {
    let root = unique_context_root("datum-eda-cli-context-source-shard-identity");
    create_native_project(&root, Some("Context Source Shard Demo".to_string()))
        .expect("native project should be created");
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
        serde_json::to_string_pretty(&schematic).unwrap(),
    )
    .unwrap();
    let sheet_file = root.join("schematic").join(&sheet_path);
    std::fs::create_dir_all(sheet_file.parent().unwrap()).unwrap();
    let symbol_id = Uuid::new_v4();
    let part_id = Uuid::new_v4();
    std::fs::write(
        &sheet_file,
        serde_json::to_string_pretty(&serde_json::json!({
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
        }))
        .unwrap(),
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
        serde_json::to_string_pretty(&board).unwrap(),
    )
    .unwrap();

    let mut model = ProjectResolver::new(&root)
        .resolve()
        .expect("project should resolve before sidecar commit");
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
                    reason: "record missing context identity sidecars".to_string(),
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
        .expect("identity sidecars should commit");
    std::fs::remove_file(root.join(format!(
        ".datum/component_instances/{component_instance_id}.json"
    )))
    .expect("promoted component instance should remove");
    std::fs::remove_file(root.join(format!(".datum/relationships/{relationship_id}.json")))
        .expect("promoted relationship should remove");
    std::fs::remove_file(root.join(format!(".datum/variants/{variant_id}.json")))
        .expect("promoted variant should remove");
    std::fs::create_dir_all(root.join(".datum")).expect("datum dir should exist");
    std::fs::write(
        root.join(".datum/gui-terminal-context.json"),
        r#"{
  "contract": "datum_terminal_context_v1",
  "session_id": "session-source-shards",
  "context_id": "context-source-shards",
  "datum_cli": "datum-eda"
}"#,
    )
    .expect("context envelope should be written");

    let output = execute(Cli {
        format: OutputFormat::Json,
        command: Commands::Context {
            action: ContextCommands::Refresh(ContextGetArgs {
                session: Some("session-source-shards".to_string()),
                path: None,
                project_root: Some(root.clone()),
            }),
        },
    })
    .expect("context refresh should succeed");
    let value: serde_json::Value =
        serde_json::from_str(&output).expect("context refresh output should be JSON");
    assert_eq!(value["source_shard_status"]["missing"], 3);
    for (relative_path, kind, taxon) in [
        (
            format!(".datum/component_instances/{component_instance_id}.json"),
            "component_instance",
            "component_instance",
        ),
        (
            format!(".datum/relationships/{relationship_id}.json"),
            "relationship",
            "relationship",
        ),
        (
            format!(".datum/variants/{variant_id}.json"),
            "variant_overlay",
            "variant_overlay",
        ),
    ] {
        assert!(
            value["source_shard_status"]["attention"]
                .as_array()
                .unwrap()
                .iter()
                .any(|item| {
                    item["relative_path"] == relative_path
                        && item["kind"] == kind
                        && item["authority"] == "authored_design"
                        && item["taxon"] == taxon
                        && item["dirty_state"] == "missing"
                }),
            "context refresh should expose missing {kind} source-shard attention"
        );
    }

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn context_refresh_exposes_unknown_generated_evidence_source_shard() {
    let root = unique_context_root("datum-eda-cli-context-source-shard-unknown");
    create_native_project(&root, Some("Context Unknown Source Shard Demo".to_string()))
        .expect("native project should be created");
    let artifact_id = commit_context_artifact_metadata(&root);
    let promoted_path = root.join(format!(".datum/artifacts/{artifact_id}.json"));
    std::fs::remove_file(&promoted_path).expect("promoted artifact metadata should remove");
    std::fs::create_dir(&promoted_path).expect("directory at promoted shard path should create");
    std::fs::create_dir_all(root.join(".datum")).expect("datum dir should exist");
    std::fs::write(
        root.join(".datum/gui-terminal-context.json"),
        r#"{
  "contract": "datum_terminal_context_v1",
  "session_id": "session-source-shard-unknown",
  "context_id": "context-source-shard-unknown",
  "datum_cli": "datum-eda"
}"#,
    )
    .expect("context envelope should be written");

    let output = execute(Cli {
        format: OutputFormat::Json,
        command: Commands::Context {
            action: ContextCommands::Refresh(ContextGetArgs {
                session: Some("session-source-shard-unknown".to_string()),
                path: None,
                project_root: Some(root.clone()),
            }),
        },
    })
    .expect("context refresh should succeed");
    let value: serde_json::Value =
        serde_json::from_str(&output).expect("context refresh output should be JSON");
    assert_eq!(value["source_shard_status"]["unknown"], 1);
    assert!(
        value["source_shard_status"]["attention"]
            .as_array()
            .unwrap()
            .iter()
            .any(|item| {
                item["relative_path"] == format!(".datum/artifacts/{artifact_id}.json")
                    && item["kind"] == "artifact_metadata"
                    && item["authority"] == "generated_evidence"
                    && item["taxon"] == "artifact_metadata"
                    && item["dirty_state"] == "unknown"
            }),
        "context refresh should expose unknown generated evidence source-shard attention"
    );

    let _ = std::fs::remove_dir_all(&root);
}
