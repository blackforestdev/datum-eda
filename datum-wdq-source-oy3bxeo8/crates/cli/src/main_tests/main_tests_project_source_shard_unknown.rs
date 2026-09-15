use super::*;
use eda_engine::ir::serialization::to_json_deterministic;
use eda_engine::substrate::{
    ArtifactKind, CommitProvenance, CommitSource, ManufacturingPlan, ObjectRevision, Operation,
    OperationBatch, OutputJob, PRODUCTION_RECORD_SCHEMA_VERSION, PanelBoardInstance,
    PanelProjection, ProjectResolver,
};

fn unique_project_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{}-{}", label, Uuid::new_v4()))
}

fn commit_production_records(root: &Path) -> (Uuid, Uuid, Uuid) {
    let mut model = ProjectResolver::new(root)
        .resolve()
        .expect("project resolves before production records");
    let board: serde_json::Value = serde_json::from_slice(
        &std::fs::read(root.join("board/board.json")).expect("board root should read"),
    )
    .expect("board root should parse");
    let board_id =
        Uuid::parse_str(board["uuid"].as_str().expect("board uuid should exist")).unwrap();
    let panel_id = Uuid::new_v4();
    let plan_id = Uuid::new_v4();
    let job_id = Uuid::new_v4();
    let panel = PanelProjection {
        schema_version: PRODUCTION_RECORD_SCHEMA_VERSION,
        id: panel_id,
        name: "Unknown panel".to_string(),
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
        name: "Unknown manufacturing plan".to_string(),
        board_or_panel: panel_id,
        variant: None,
        prefix: "unknown".to_string(),
        object_revision: ObjectRevision(0),
    };
    let job = OutputJob {
        schema_version: PRODUCTION_RECORD_SCHEMA_VERSION,
        id: job_id,
        name: "Unknown output job".to_string(),
        include: vec![ArtifactKind::GerberSet],
        prefix: "unknown".to_string(),
        output_dir: None,
        board_or_panel: panel_id,
        variant: None,
        manufacturing_plan: Some(plan_id),
        object_revision: ObjectRevision(0),
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
                    reason: "record unreadable authored production shards".to_string(),
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
    (plan_id, panel_id, job_id)
}

fn commit_identity_relationship_records(root: &Path) -> (Uuid, Uuid, Uuid) {
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
    let symbol_id = Uuid::new_v4();
    let package_id = Uuid::new_v4();
    let part_id = Uuid::new_v4();
    let sheet_file = root.join("schematic").join(&sheet_path);
    std::fs::create_dir_all(sheet_file.parent().unwrap()).unwrap();
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

    let mut model = ProjectResolver::new(root)
        .resolve()
        .expect("project resolves before identity records");
    let component_instance_id = Uuid::new_v4();
    let relationship_id = Uuid::new_v4();
    let variant_id = Uuid::new_v4();
    model
        .commit_journaled(
            root,
            OperationBatch {
                batch_id: Uuid::new_v4(),
                expected_model_revision: Some(model.model_revision.clone()),
                provenance: CommitProvenance {
                    actor: "cli-test".to_string(),
                    source: CommitSource::Cli,
                    reason: "record unreadable identity/relationship shards".to_string(),
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
                ],
            },
        )
        .expect("identity/relationship records should commit");
    let mut model = ProjectResolver::new(root)
        .resolve()
        .expect("project resolves before variant record");
    model
        .commit_journaled(
            root,
            OperationBatch {
                batch_id: Uuid::new_v4(),
                expected_model_revision: Some(model.model_revision.clone()),
                provenance: CommitProvenance {
                    actor: "cli-test".to_string(),
                    source: CommitSource::Cli,
                    reason: "record unreadable variant shard".to_string(),
                },
                operations: vec![Operation::CreateVariantOverlay {
                    variant_id,
                    variant: serde_json::json!({
                        "id": variant_id,
                        "name": "No U1",
                        "base_model_revision": model.model_revision,
                        "variant_revision": 0,
                        "fitted": {
                            component_instance_id.to_string(): "unfitted"
                        },
                        "relationship_overrides": {},
                        "property_overrides": {}
                    }),
                }],
            },
        )
        .expect("variant record should commit");
    (component_instance_id, relationship_id, variant_id)
}

fn commit_import_map_sidecar(root: &Path) -> String {
    let board: serde_json::Value =
        serde_json::from_slice(&std::fs::read(root.join("board/board.json")).unwrap()).unwrap();
    let board_id = Uuid::parse_str(board["uuid"].as_str().unwrap()).unwrap();
    let mut model = ProjectResolver::new(root)
        .resolve()
        .expect("project resolves before import map");
    let board_shard = model
        .source_shards
        .iter()
        .find(|shard| shard.relative_path == "board/board.json")
        .expect("board shard should exist");
    let relative_path = ".datum/import_map/kicad.json".to_string();
    model
        .commit_journaled(
            root,
            OperationBatch {
                batch_id: Uuid::new_v4(),
                expected_model_revision: Some(model.model_revision.clone()),
                provenance: CommitProvenance {
                    actor: "cli-test".to_string(),
                    source: CommitSource::Cli,
                    reason: "record unreadable import map sidecar".to_string(),
                },
                operations: vec![Operation::CreateImportMapShard {
                    relative_path: relative_path.clone(),
                    shard: serde_json::json!({
                        "schema_version": 1,
                        "entries": [{
                            "import_key": "kicad:board:root",
                            "object_id": board_id,
                            "source_shard_id": board_shard.shard_id,
                            "source_hash": board_shard.content_hash
                        }]
                    }),
                }],
            },
        )
        .expect("import map should commit");
    relative_path
}

fn commit_pool_symbol(root: &Path) -> String {
    let mut model = ProjectResolver::new(root)
        .resolve()
        .expect("project resolves before pool symbol");
    let symbol_id = Uuid::new_v4();
    let relative_path = format!("pool/symbols/{symbol_id}.json");
    model
        .commit_journaled(
            root,
            OperationBatch {
                batch_id: Uuid::new_v4(),
                expected_model_revision: Some(model.model_revision.clone()),
                provenance: CommitProvenance {
                    actor: "cli-test".to_string(),
                    source: CommitSource::Cli,
                    reason: "record unreadable pool symbol shard".to_string(),
                },
                operations: vec![Operation::CreatePoolLibraryObject {
                    object_id: symbol_id,
                    relative_path: relative_path.clone(),
                    object_kind: "symbols".to_string(),
                    object: serde_json::json!({
                        "schema_version": 1,
                        "uuid": symbol_id,
                        "name": "UnknownPoolSymbol",
                        "unit": Uuid::new_v4()
                    }),
                }],
            },
        )
        .expect("pool symbol should commit");
    relative_path
}

#[test]
fn project_query_resolve_debug_reports_unknown_pool_symbol_shard() {
    let root = unique_project_root("datum-eda-cli-project-resolve-debug-unknown-pool");
    create_native_project(
        &root,
        Some("Resolve Debug Unknown Pool Shard Demo".to_string()),
    )
    .expect("initial scaffold should succeed");
    let relative_path = commit_pool_symbol(&root);
    let promoted_path = root.join(&relative_path);
    std::fs::remove_file(&promoted_path).expect("promoted pool symbol should remove");
    std::fs::create_dir(&promoted_path)
        .expect("directory at promoted pool symbol path should create");

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
                    && shard["dirty_state"] == "Unknown"
            }),
        "resolve-debug should expose unreadable journal-recovered pool symbol as Unknown"
    );

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_query_resolve_debug_reports_unknown_import_map_sidecar() {
    let root = unique_project_root("datum-eda-cli-project-resolve-debug-unknown-import-map");
    create_native_project(
        &root,
        Some("Resolve Debug Unknown ImportMap Sidecar Demo".to_string()),
    )
    .expect("initial scaffold should succeed");
    let relative_path = commit_import_map_sidecar(&root);
    let promoted_path = root.join(&relative_path);
    std::fs::remove_file(&promoted_path).expect("promoted import map sidecar should remove");
    std::fs::create_dir(&promoted_path)
        .expect("directory at promoted import map sidecar path should create");

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
                    && shard["kind"] == "ImportMap"
                    && shard["taxon"] == "ImportMap"
                    && shard["authority"] == "SidecarMetadata"
                    && shard["dirty_state"] == "Unknown"
            }),
        "resolve-debug should expose unreadable journal-recovered ImportMap sidecar as Unknown"
    );

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_query_resolve_debug_reports_unknown_authored_production_shards() {
    let root = unique_project_root("datum-eda-cli-project-resolve-debug-unknown-production");
    create_native_project(
        &root,
        Some("Resolve Debug Unknown Production Shard Demo".to_string()),
    )
    .expect("initial scaffold should succeed");
    let (plan_id, panel_id, job_id) = commit_production_records(&root);
    for relative_path in [
        format!(".datum/manufacturing_plans/{plan_id}.json"),
        format!(".datum/panel_projections/{panel_id}.json"),
        format!(".datum/output_jobs/{job_id}.json"),
    ] {
        let promoted_path = root.join(&relative_path);
        std::fs::remove_file(&promoted_path).expect("promoted production shard should remove");
        std::fs::create_dir(&promoted_path)
            .expect("directory at promoted production shard path should create");
    }

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
    let source_shards = report["source_shards"].as_array().unwrap();
    for (relative_path, kind, taxon) in [
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
            source_shards.iter().any(|shard| {
                shard["path"] == relative_path
                    && shard["kind"] == kind
                    && shard["taxon"] == taxon
                    && shard["authority"] == "AuthoredDesign"
                    && shard["dirty_state"] == "Unknown"
            }),
            "resolve-debug should expose unreadable journal-recovered authored shard as Unknown: {relative_path}"
        );
    }

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_query_resolve_debug_reports_unknown_identity_relationship_shards() {
    let root = unique_project_root("datum-eda-cli-project-resolve-debug-unknown-identity");
    create_native_project(
        &root,
        Some("Resolve Debug Unknown Identity Shard Demo".to_string()),
    )
    .expect("initial scaffold should succeed");
    let (component_instance_id, relationship_id, variant_id) =
        commit_identity_relationship_records(&root);
    for relative_path in [
        format!(".datum/component_instances/{component_instance_id}.json"),
        format!(".datum/relationships/{relationship_id}.json"),
        format!(".datum/variants/{variant_id}.json"),
    ] {
        let promoted_path = root.join(&relative_path);
        std::fs::remove_file(&promoted_path).expect("promoted identity shard should remove");
        std::fs::create_dir(&promoted_path)
            .expect("directory at promoted identity shard path should create");
    }

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
    let source_shards = report["source_shards"].as_array().unwrap();
    for (relative_path, kind, taxon) in [
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
            source_shards.iter().any(|shard| {
                shard["path"] == relative_path
                    && shard["kind"] == kind
                    && shard["taxon"] == taxon
                    && shard["authority"] == "AuthoredDesign"
                    && shard["dirty_state"] == "Unknown"
            }),
            "resolve-debug should expose unreadable journal-recovered identity shard as Unknown: {relative_path}"
        );
    }

    let _ = std::fs::remove_dir_all(&root);
}
