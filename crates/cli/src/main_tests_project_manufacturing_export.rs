use super::*;
use eda_engine::ir::serialization::to_json_deterministic;

fn unique_project_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{}-{}", label, Uuid::new_v4()))
}

#[test]
fn project_export_manufacturing_set_writes_supported_artifacts() {
    let root = unique_project_root("datum-eda-cli-project-manufacturing-export");
    create_native_project(&root, Some("Manufacturing Export Demo".to_string()))
        .expect("initial scaffold should succeed");

    let board_json = root.join("board/board.json");
    let component_uuid = Uuid::new_v4();
    let package_uuid = Uuid::new_v4();
    let part_uuid = Uuid::new_v4();
    let via_uuid = Uuid::new_v4();
    let net_uuid = Uuid::new_v4();
    let net_class_uuid = Uuid::new_v4();
    std::fs::write(
        &board_json,
        format!(
            "{}\n",
            to_json_deterministic(&serde_json::json!({
                "schema_version": 1,
                "uuid": Uuid::new_v4(),
                "name": "Manufacturing Export Demo Board",
                "stackup": {
                    "layers": [
                        { "id": 1, "name": "Top Copper", "layer_type": "Copper", "thickness_nm": 35000 },
                        { "id": 2, "name": "Top Mask", "layer_type": "SolderMask", "thickness_nm": 10000 },
                        { "id": 3, "name": "Top Silk", "layer_type": "Silkscreen", "thickness_nm": 10000 },
                        { "id": 4, "name": "Top Paste", "layer_type": "Paste", "thickness_nm": 10000 },
                        { "id": 31, "name": "Bottom Copper", "layer_type": "Copper", "thickness_nm": 35000 },
                        { "id": 41, "name": "Mechanical 41", "layer_type": "Mechanical", "thickness_nm": 0 }
                    ]
                },
                "outline": {
                    "vertices": [
                        { "x": 0, "y": 0 },
                        { "x": 1000, "y": 0 },
                        { "x": 1000, "y": 500 },
                        { "x": 0, "y": 500 }
                    ],
                    "closed": true
                },
                "packages": {
                    component_uuid.to_string(): {
                        "uuid": component_uuid,
                        "reference": "U1",
                        "value": "MCU",
                        "package": package_uuid,
                        "part": part_uuid,
                        "position": { "x": 1000000, "y": 2000000 },
                        "rotation": 0,
                        "layer": 1,
                        "locked": false
                    }
                },
                "component_pads": {
                    component_uuid.to_string(): [
                        {
                            "uuid": Uuid::new_v4(),
                            "name": "P1",
                            "position": { "x": 1000000, "y": 1500000 },
                            "padstack": Uuid::new_v4(),
                            "layer": 1,
                            "drill_nm": 300000,
                            "shape": "circle",
                            "diameter_nm": 600000,
                            "width_nm": 0,
                            "height_nm": 0
                        }
                    ]
                },
                "component_silkscreen": {},
                "component_silkscreen_texts": {},
                "component_silkscreen_arcs": {},
                "component_silkscreen_circles": {},
                "component_silkscreen_polygons": {},
                "component_silkscreen_polylines": {},
                "component_mechanical_lines": {},
                "component_mechanical_texts": {},
                "component_mechanical_polygons": {},
                "component_mechanical_polylines": {},
                "component_mechanical_circles": {},
                "component_mechanical_arcs": {},
                "component_models_3d": {},
                "pads": {},
                "tracks": {},
                "vias": {
                    via_uuid.to_string(): {
                        "uuid": via_uuid,
                        "net": net_uuid,
                        "position": { "x": 500000, "y": 500000 },
                        "diameter": 700000, "drill": 300000,
                        "from_layer": 1,
                        "to_layer": 31
                    }
                },
                "zones": {},
                "nets": {
                    net_uuid.to_string(): {
                        "uuid": net_uuid, "name": "N$1", "class": net_class_uuid
                    }
                },
                "net_classes": {
                    net_class_uuid.to_string(): {
                        "uuid": net_class_uuid, "name": "Default",
                        "clearance": 150000, "track_width": 200000,
                        "via_drill": 300000, "via_diameter": 600000,
                        "diffpair_width": 0, "diffpair_gap": 0
                    }
                },
                "keepouts": [],
                "dimensions": [],
                "texts": [{
                    "uuid": Uuid::new_v4(),
                    "text": "TOP",
                    "position": { "x": 1000000, "y": 2000000 },
                    "rotation": 0,
                    "height_nm": 1000000,
                    "stroke_width_nm": 120000,
                    "layer": 3
                }]
            }))
            .expect("canonical serialization should succeed")
        ),
    )
    .expect("board file should write");

    let output_dir = root.join("manufacturing");
    let cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "artifact",
        "export-manufacturing-set",
        root.to_str().unwrap(),
        "--output-dir",
        output_dir.to_str().unwrap(),
        "--prefix",
        "Release A",
    ])
    .expect("CLI should parse");
    let output = execute(cli).expect("manufacturing export should succeed");
    let report: serde_json::Value = serde_json::from_str(&output).expect("report JSON");
    assert_eq!(report["action"], "export_manufacturing_set");
    assert_eq!(report["prefix"], "release-a");
    assert_eq!(report["bom_row_count"], 1);
    assert_eq!(report["pnp_row_count"], 1);
    assert_eq!(report["drill_csv_row_count"], 1);
    assert_eq!(report["excellon_hit_count"], 2);
    assert_eq!(report["gerber_artifact_count"], 7);
    assert_eq!(report["written_count"], 11);
    assert_eq!(report["artifact_metadata"]["kind"], "manufacturing_set");
    assert_eq!(
        report["artifact_metadata"]["output_dir"],
        output_dir.display().to_string()
    );
    assert_eq!(
        report["artifact_metadata"]["validation_state"],
        "not_validated"
    );
    assert_eq!(
        report["artifact_metadata"]["files"]
            .as_array()
            .unwrap()
            .len(),
        11
    );
    let production_projections = report["artifact_metadata"]["production_projections"]
        .as_array()
        .expect("manufacturing set artifact should expose production projection proofs");
    assert_eq!(production_projections.len(), 3);
    assert_eq!(
        production_projections
            .iter()
            .filter(|entry| entry["projection_kind"] == "gerber_copper_layer")
            .count(),
        2
    );
    assert!(
        production_projections
            .iter()
            .any(|entry| entry["projection_kind"] == "excellon_drill"
                && entry["projection_contract"] == "datum.production_projection.excellon_drill.v1")
    );
    assert!(production_projections.iter().all(|entry| {
        entry["model_revision"] == report["artifact_metadata"]["model_revision"]
            && entry["byte_count"].as_u64().unwrap_or_default() > 0
            && entry["sha256"]
                .as_str()
                .unwrap_or_default()
                .starts_with("sha256:")
    }));
    assert_eq!(
        report["output_job_run"]["artifact_id"],
        report["artifact_metadata"]["artifact_id"]
    );
    assert!(std::path::Path::new(report["output_job_run_path"].as_str().unwrap()).is_file());
    assert_eq!(report["output_job_run"]["status"], "succeeded");
    assert_eq!(report["output_job_run"]["exit_code"], 0);
    assert_eq!(
        report["output_job_run"]["log"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|entry| entry["message"]
                .as_str()
                .unwrap()
                .contains("production projection"))
            .count(),
        3
    );
    assert!(std::path::Path::new(report["artifact_manifest_path"].as_str().unwrap()).is_file());
    assert!(
        report["artifact_metadata"]["artifact_id"]
            .as_str()
            .unwrap()
            .parse::<Uuid>()
            .is_ok()
    );
    assert!(
        report["artifacts"]
            .as_array()
            .unwrap()
            .iter()
            .all(|entry| entry["sha256"].as_str().unwrap().starts_with("sha256:"))
    );
    let preview = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "artifact",
            "preview",
            root.to_str().unwrap(),
            "--artifact",
            report["artifact_metadata"]["artifact_id"].as_str().unwrap(),
            "--file",
            "release-a-drill.drl",
        ])
        .expect("CLI should parse"),
    )
    .expect("artifact preview should succeed");
    let preview_report: serde_json::Value = serde_json::from_str(&preview).expect("preview JSON");
    assert_eq!(preview_report["contract"], "artifact_file_preview_v1");
    assert_eq!(preview_report["preview_kind"], "excellon_drill");
    assert_eq!(
        preview_report["output_dir"],
        output_dir.display().to_string()
    );
    assert_eq!(preview_report["hash_matches_metadata"], true);
    assert_eq!(preview_report["primitive_count"], 2);
    assert_eq!(preview_report["primitives"][0]["kind"], "drill_hit");
    assert_eq!(preview_report["primitives"][0]["points"][0]["x_nm"], 500000);
    assert_eq!(preview_report["inspection"]["hit_count"], 2);
    let bom_preview = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "artifact",
            "preview",
            root.to_str().unwrap(),
            "--artifact",
            report["artifact_metadata"]["artifact_id"].as_str().unwrap(),
            "--file",
            "release-a-bom.csv",
        ])
        .expect("CLI should parse"),
    )
    .expect("BOM artifact preview should succeed");
    let bom_preview: serde_json::Value =
        serde_json::from_str(&bom_preview).expect("BOM preview JSON");
    assert_eq!(bom_preview["preview_kind"], "bom_csv");
    assert_eq!(bom_preview["inspection"]["row_count"], 1);
    assert_eq!(
        bom_preview["inspection"]["columns"][0],
        "component_instance_uuid"
    );
    assert_eq!(bom_preview["inspection"]["columns"][1], "reference");
    assert_eq!(bom_preview["inspection"]["rows"][0][1], "U1");

    let artifacts = execute(
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
    let artifacts_report: serde_json::Value =
        serde_json::from_str(&artifacts).expect("artifacts JSON");
    assert_eq!(artifacts_report["contract"], "artifact_metadata_list_v1");
    assert!(
        artifacts_report["artifacts"]
            .as_array()
            .unwrap()
            .iter()
            .any(|entry| entry["artifact_id"] == report["artifact_metadata"]["artifact_id"])
    );
    let shown_artifact = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "artifact",
            "show",
            root.to_str().unwrap(),
            "--artifact",
            report["artifact_metadata"]["artifact_id"].as_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("artifact show should succeed");
    let shown_artifact_report: serde_json::Value =
        serde_json::from_str(&shown_artifact).expect("shown artifact JSON");
    assert_eq!(shown_artifact_report["contract"], "artifact_metadata_v1");
    assert_eq!(
        shown_artifact_report["artifact"]["artifact_id"],
        report["artifact_metadata"]["artifact_id"]
    );
    let other_artifact_id = artifacts_report["artifacts"]
        .as_array()
        .unwrap()
        .iter()
        .find_map(|entry| {
            let artifact_id = entry["artifact_id"].as_str()?;
            (artifact_id != report["artifact_metadata"]["artifact_id"].as_str().unwrap())
                .then_some(artifact_id)
        })
        .expect("export should leave another generated artifact metadata record");
    let artifact_comparison = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "artifact",
            "compare",
            root.to_str().unwrap(),
            "--before",
            other_artifact_id,
            "--after",
            report["artifact_metadata"]["artifact_id"].as_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("artifact compare should succeed");
    let artifact_comparison_report: serde_json::Value =
        serde_json::from_str(&artifact_comparison).expect("artifact comparison JSON");
    assert_eq!(
        artifact_comparison_report["contract"],
        "artifact_metadata_compare_v1"
    );
    assert_eq!(artifact_comparison_report["equivalent"], false);
    assert_eq!(artifact_comparison_report["kind_equal"], false);
    assert_eq!(artifact_comparison_report["files_equal"], false);
    assert_eq!(
        artifact_comparison_report["after_artifact_id"],
        report["artifact_metadata"]["artifact_id"]
    );
    let artifact_validation = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "artifact",
            "validate",
            root.to_str().unwrap(),
            "--artifact",
            report["artifact_metadata"]["artifact_id"].as_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("artifact validate should succeed");
    let artifact_validation_report: serde_json::Value =
        serde_json::from_str(&artifact_validation).expect("artifact validation JSON");
    assert_eq!(
        artifact_validation_report["contract"],
        "artifact_metadata_validation_v1"
    );
    assert_eq!(artifact_validation_report["valid"], true);
    assert_eq!(
        artifact_validation_report["artifact_id"],
        report["artifact_metadata"]["artifact_id"]
    );
    assert_eq!(
        artifact_validation_report["unsafe_file_paths"],
        serde_json::json!([])
    );
    assert_eq!(
        artifact_validation_report["invalid_file_hashes"],
        serde_json::json!([])
    );

    let generated_dir = root.join("generated-gerbers");
    let generated = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "artifact",
            "generate",
            root.to_str().unwrap(),
            "--output-dir",
            generated_dir.to_str().unwrap(),
            "--include",
            "gerber-set",
            "--prefix",
            "Generated A",
        ])
        .expect("CLI should parse"),
    )
    .expect("artifact generate should succeed");
    let generated_report: serde_json::Value =
        serde_json::from_str(&generated).expect("artifact generate JSON");
    assert_eq!(generated_report["contract"], "artifact_generate_v1");
    assert_eq!(generated_report["action"], "generate_artifacts");
    assert_eq!(
        generated_report["include"],
        serde_json::json!(["gerber-set"])
    );
    assert_eq!(generated_report["generated_count"], 1);
    assert_eq!(generated_report["generated"][0]["kind"], "gerber_set");
    assert_eq!(
        generated_report["generated"][0]["report"]["action"],
        "export_gerber_set"
    );
    assert!(
        std::path::Path::new(
            generated_report["generated"][0]["artifact_manifest_path"]
                .as_str()
                .unwrap()
        )
        .is_file()
    );
    let generated_artifact_id = generated_report["generated"][0]["artifact_id"]
        .as_str()
        .expect("generated artifact id");
    let artifact_files = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "artifact",
            "files",
            root.to_str().unwrap(),
            "--artifact",
            generated_artifact_id,
        ])
        .expect("CLI should parse"),
    )
    .expect("artifact files should succeed");
    let artifact_files_report: serde_json::Value =
        serde_json::from_str(&artifact_files).expect("artifact files JSON");
    assert_eq!(artifact_files_report["contract"], "artifact_files_v1");
    assert_eq!(artifact_files_report["artifact_id"], generated_artifact_id);
    assert_eq!(artifact_files_report["kind"], "gerber_set");
    assert_eq!(
        artifact_files_report["output_dir"],
        generated_dir.display().to_string()
    );
    assert!(artifact_files_report["file_count"].as_u64().unwrap() > 0);
    assert!(
        artifact_files_report["production_projection_count"]
            .as_u64()
            .unwrap()
            > 0
    );
    let scoped_dir = root.join("generated-scoped-artifacts");
    let scoped_drill_job = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "create-output-job",
            root.to_str().unwrap(),
            "--prefix",
            "Scoped A",
            "--output-dir",
            scoped_dir.to_str().unwrap(),
            "--include",
            "drill",
            "--name",
            "Scoped Drill",
        ])
        .expect("CLI should parse"),
    )
    .expect("scoped drill output job should be authored");
    let scoped_drill_job: serde_json::Value =
        serde_json::from_str(&scoped_drill_job).expect("scoped drill output job JSON");
    assert_eq!(scoped_drill_job["action"], "create_output_job");
    assert_eq!(
        scoped_drill_job["output_job"]["include"],
        serde_json::json!(["drill"])
    );
    assert_eq!(scoped_drill_job["output_job"]["prefix"], "scoped-a");
    assert_eq!(
        scoped_drill_job["output_job"]["output_dir"],
        scoped_dir.display().to_string()
    );
    let scoped_drill_job_id = scoped_drill_job["output_job"]["id"].as_str().unwrap();
    let scoped_generated = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "artifact",
            "generate",
            root.to_str().unwrap(),
            "--output-dir",
            scoped_dir.to_str().unwrap(),
            "--include",
            "bom,pnp,drill",
            "--prefix",
            "Scoped A",
        ])
        .expect("CLI should parse"),
    )
    .expect("scoped artifact generate should succeed");
    let scoped_report: serde_json::Value =
        serde_json::from_str(&scoped_generated).expect("scoped artifact generate JSON");
    assert_eq!(
        scoped_report["include"],
        serde_json::json!(["bom", "pnp", "drill"])
    );
    assert_eq!(scoped_report["generated_count"], 3);
    let scoped_generated = scoped_report["generated"].as_array().unwrap();
    assert!(scoped_generated.iter().any(|entry| {
        entry["include"] == "bom" && entry["kind"] == "bom" && entry["file_count"] == 1
    }));
    assert!(scoped_generated.iter().any(|entry| {
        entry["include"] == "pnp" && entry["kind"] == "pnp" && entry["file_count"] == 1
    }));
    let drill_entry = scoped_generated
        .iter()
        .find(|entry| entry["include"] == "drill")
        .expect("drill artifact should be generated");
    let drill_artifact_id = drill_entry["artifact_id"].as_str().unwrap();
    assert_eq!(drill_entry["artifact_run"], serde_json::Value::Null);
    assert_eq!(
        drill_entry["output_job_run"]["artifact_id"],
        drill_artifact_id
    );
    assert_eq!(
        drill_entry["output_job_run"]["output_job"],
        scoped_drill_job_id
    );
    assert!(std::path::Path::new(drill_entry["output_job_run_path"].as_str().unwrap()).is_file());
    assert_eq!(drill_entry["file_count"], 2);
    assert_eq!(
        drill_entry["report"]["projection"]["entries"][0]["filename"],
        "scoped-a-drill.csv"
    );
    assert_eq!(
        drill_entry["report"]["projection"]["entries"][1]["filename"],
        "scoped-a-drill.drl"
    );
    assert!(scoped_dir.join("scoped-a-bom.csv").is_file());
    assert!(scoped_dir.join("scoped-a-pnp.csv").is_file());
    assert!(scoped_dir.join("scoped-a-drill.csv").is_file());
    assert!(scoped_dir.join("scoped-a-drill.drl").is_file());
    let scoped_drill_preview = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "artifact",
            "preview",
            root.to_str().unwrap(),
            "--artifact",
            drill_artifact_id,
            "--file",
            "scoped-a-drill.drl",
        ])
        .expect("CLI should parse"),
    )
    .expect("scoped drill artifact preview should succeed");
    let scoped_drill_preview: serde_json::Value =
        serde_json::from_str(&scoped_drill_preview).expect("scoped drill preview JSON");
    assert_eq!(scoped_drill_preview["kind"], "drill");
    assert_eq!(scoped_drill_preview["preview_kind"], "excellon_drill");
    let scoped_drill_validation = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "artifact",
            "validate",
            root.to_str().unwrap(),
            "--artifact",
            drill_artifact_id,
        ])
        .expect("CLI should parse"),
    )
    .expect("scoped drill artifact validation should succeed");
    let scoped_drill_validation: serde_json::Value =
        serde_json::from_str(&scoped_drill_validation).expect("scoped drill validation JSON");
    assert_eq!(scoped_drill_validation["valid"], true);
    assert_eq!(scoped_drill_validation["validation_state"], "valid");
    assert_eq!(
        scoped_drill_validation["semantic_mismatches"],
        serde_json::json!([])
    );
    assert!(
        std::path::Path::new(
            scoped_drill_validation["artifact_manifest_path"]
                .as_str()
                .unwrap()
        )
        .is_file()
    );
    let scoped_output_jobs = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "query",
            root.to_str().unwrap(),
            "output-jobs",
        ])
        .expect("CLI should parse"),
    )
    .expect("output jobs query should include scoped artifact");
    let scoped_output_jobs: serde_json::Value =
        serde_json::from_str(&scoped_output_jobs).expect("scoped output jobs JSON");
    assert!(
        scoped_output_jobs["output_jobs"]
            .as_array()
            .unwrap()
            .iter()
            .any(|entry| {
                entry["id"] == scoped_drill_job_id
                    && entry["include"] == serde_json::json!(["drill"])
                    && entry["status"] == "succeeded"
                    && entry["execution_count"] == 1
                    && entry["latest_run"]["artifact_id"] == drill_artifact_id
                    && entry["artifacts"][0]["artifact_id"] == drill_artifact_id
                    && entry["artifacts"][0]["output_job"] == scoped_drill_job_id
                    && entry["artifacts"][0]["validation_state"] == "valid"
            })
    );
    let scoped_drill_artifact = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "artifact",
            "show",
            root.to_str().unwrap(),
            "--artifact",
            drill_artifact_id,
        ])
        .expect("CLI should parse"),
    )
    .expect("artifact show should expose linked output-job run");
    let scoped_drill_artifact: serde_json::Value =
        serde_json::from_str(&scoped_drill_artifact).expect("scoped drill artifact JSON");
    assert_eq!(
        scoped_drill_artifact["artifact"]["artifact_id"],
        drill_artifact_id
    );
    assert_eq!(scoped_drill_artifact["run_count"], 0);
    assert_eq!(scoped_drill_artifact["output_job_run_count"], 1);
    assert_eq!(
        scoped_drill_artifact["latest_output_job_run"]["artifact_id"],
        drill_artifact_id
    );
    assert_eq!(
        scoped_drill_artifact["latest_output_job_run"]["output_job"],
        scoped_drill_job_id
    );

    let artifact_list = execute(
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
    .expect("artifact list should expose linked output-job runs");
    let artifact_list: serde_json::Value =
        serde_json::from_str(&artifact_list).expect("artifact list JSON");
    assert!(artifact_list["output_job_run_count"].as_u64().unwrap() >= 1);
    assert!(
        artifact_list["output_job_runs"]
            .as_array()
            .unwrap()
            .iter()
            .any(|run| run["artifact_id"] == drill_artifact_id
                && run["output_job"] == scoped_drill_job_id)
    );

    assert!(output_dir.join("release-a-bom.csv").is_file());
    assert!(output_dir.join("release-a-pnp.csv").is_file());
    assert!(output_dir.join("release-a-drill.csv").is_file());
    assert!(output_dir.join("release-a-drill.drl").is_file());
    assert!(output_dir.join("release-a-outline.gbr").is_file());
    assert!(
        output_dir
            .join("release-a-l1-top-copper-copper.gbr")
            .is_file()
    );
    assert!(output_dir.join("release-a-l2-top-mask-mask.gbr").is_file());
    assert!(output_dir.join("release-a-l3-top-silk-silk.gbr").is_file());
    assert!(
        output_dir
            .join("release-a-l4-top-paste-paste.gbr")
            .is_file()
    );
    assert!(
        output_dir
            .join("release-a-l31-bottom-copper-copper.gbr")
            .is_file()
    );
    assert!(
        output_dir
            .join("release-a-l41-mechanical-41-mech.gbr")
            .is_file()
    );

    let output_jobs = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "query",
            root.to_str().unwrap(),
            "output-jobs",
        ])
        .expect("CLI should parse"),
    )
    .expect("output jobs query should succeed");
    let output_jobs_report: serde_json::Value =
        serde_json::from_str(&output_jobs).expect("output jobs JSON");
    assert!(
        output_jobs_report["output_jobs"]
            .as_array()
            .unwrap()
            .iter()
            .any(|entry| {
                entry["include"] == serde_json::json!(["manufacturing_set"])
                    && entry["status"] == "succeeded"
                    && entry["artifacts"][0]["artifact_id"]
                        == report["artifact_metadata"]["artifact_id"]
                    && entry["artifacts"][0]["production_projections"]
                        == report["artifact_metadata"]["production_projections"]
            })
    );

    let _ = std::fs::remove_dir_all(&root);
}
