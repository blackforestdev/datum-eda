use super::*;
use eda_engine::ir::serialization::to_json_deterministic;

fn unique_project_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{label}-{}", Uuid::new_v4()))
}

#[test]
fn project_export_gerber_set_writes_planned_artifact_set() {
    let root = unique_project_root("datum-eda-cli-project-gerber-set");
    create_native_project(&root, Some("Gerber Set Demo".to_string()))
        .expect("initial scaffold should succeed");

    let board_json = root.join("board/board.json");
    std::fs::write(
        &board_json,
        format!(
            "{}\n",
            to_json_deterministic(&serde_json::json!({
                "schema_version": 1,
                "uuid": Uuid::new_v4(),
                "name": "Gerber Set Demo Board",
                "stackup": {
                    "layers": [
                        {"id": 1, "name": "Top Copper", "layer_type": "Copper", "thickness_nm": 35000},
                        {"id": 2, "name": "Top Mask", "layer_type": "SolderMask", "thickness_nm": 10000},
                        {"id": 3, "name": "Top Silk", "layer_type": "Silkscreen", "thickness_nm": 10000},
                        {"id": 4, "name": "Top Paste", "layer_type": "Paste", "thickness_nm": 10000},
                        {"id": 41, "name": "Mechanical 41", "layer_type": "Mechanical", "thickness_nm": 0}
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
                "packages": {},
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
                "pads": {},
                "tracks": {},
                "vias": {},
                "zones": {},
                "nets": {},
                "net_classes": {},
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

    let create_plan_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "create-manufacturing-plan",
        root.to_str().unwrap(),
        "--prefix",
        "release-a",
        "--name",
        "Release A manufacturing",
    ])
    .expect("CLI should parse");
    let create_plan_output =
        execute(create_plan_cli).expect("manufacturing plan create should succeed");
    let create_plan_report: serde_json::Value =
        serde_json::from_str(&create_plan_output).expect("manufacturing plan create JSON");
    assert_eq!(
        create_plan_report["contract"],
        "manufacturing_plan_mutation_v1"
    );
    assert_eq!(create_plan_report["action"], "create_manufacturing_plan");
    assert_eq!(create_plan_report["created"], true);
    assert_eq!(
        create_plan_report["manufacturing_plan"]["prefix"],
        "release-a"
    );
    let manufacturing_plan = create_plan_report["manufacturing_plan"]["id"]
        .as_str()
        .expect("manufacturing plan id should serialize")
        .to_string();

    let create_job_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "create-gerber-output-job",
        root.to_str().unwrap(),
        "--prefix",
        "release-a",
        "--name",
        "Release A fabrication",
        "--manufacturing-plan",
        manufacturing_plan.as_str(),
    ])
    .expect("CLI should parse");
    let create_job_output = execute(create_job_cli).expect("output job create should succeed");
    let create_job_report: serde_json::Value =
        serde_json::from_str(&create_job_output).expect("output job create JSON");
    assert_eq!(create_job_report["contract"], "output_job_mutation_v1");
    assert_eq!(create_job_report["action"], "create_gerber_set_output_job");
    assert_eq!(create_job_report["created"], true);
    assert_eq!(
        create_job_report["output_job"]["name"],
        "Release A fabrication"
    );
    assert_eq!(
        create_job_report["output_job"]["manufacturing_plan"],
        serde_json::json!(manufacturing_plan)
    );
    let output_job = create_job_report["output_job"]["id"]
        .as_str()
        .expect("output job id should serialize")
        .to_string();

    let output_dir = root.join("gerbers");
    let export_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "export-gerber-set",
        root.to_str().unwrap(),
        "--output-dir",
        output_dir.to_str().unwrap(),
        "--prefix",
        "Release A",
    ])
    .expect("CLI should parse");
    let export_output = execute(export_cli).expect("gerber set export should succeed");
    let report: serde_json::Value = serde_json::from_str(&export_output).expect("report JSON");
    assert_eq!(report["action"], "export_gerber_set");
    assert_eq!(report["written_count"], 6);
    assert!(std::path::Path::new(report["artifact_manifest_path"].as_str().unwrap()).is_file());
    assert_eq!(report["artifact_metadata"]["kind"], "gerber_set");
    assert_eq!(report["output_job_run"]["status"], "succeeded");
    assert!(std::path::Path::new(report["output_job_run_path"].as_str().unwrap()).is_file());
    assert_eq!(
        report["output_job_run"]["output_job"],
        serde_json::json!(output_job)
    );
    assert_eq!(report["output_job_run"]["exit_code"], 0);
    assert_eq!(
        report["output_job_run"]["artifact_id"],
        report["artifact_metadata"]["artifact_id"]
    );
    assert_eq!(report["output_job_run"]["log"][0]["sequence"], 1);
    assert_eq!(report["output_job_run"]["log"][0]["level"], "info");
    assert_eq!(
        report["artifact_metadata"]["validation_state"],
        "not_validated"
    );
    assert_eq!(
        report["artifact_metadata"]["output_dir"],
        output_dir.display().to_string()
    );
    assert_eq!(
        report["artifact_metadata"]["output_job"],
        serde_json::json!(output_job)
    );
    assert!(output_job.parse::<Uuid>().is_ok());
    assert_eq!(
        report["artifact_metadata"]["variant"],
        serde_json::Value::Null
    );
    assert_eq!(
        report["artifact_metadata"]["files"]
            .as_array()
            .unwrap()
            .len(),
        6
    );
    assert!(
        report["artifact_metadata"]["artifact_id"]
            .as_str()
            .unwrap()
            .parse::<Uuid>()
            .is_ok()
    );
    assert!(
        report["artifact_metadata"]["project_id"]
            .as_str()
            .unwrap()
            .parse::<Uuid>()
            .is_ok()
    );
    assert!(
        !report["artifact_metadata"]["generator_version"]
            .as_str()
            .unwrap()
            .is_empty()
    );
    assert!(
        report["artifact_metadata"]["model_revision"]
            .as_str()
            .unwrap()
            .len()
            >= 64
    );
    let production_projections = report["artifact_metadata"]["production_projections"]
        .as_array()
        .expect("Gerber set artifact should expose production projection proofs");
    assert_eq!(production_projections.len(), 1);
    assert_eq!(
        production_projections[0]["projection_kind"],
        "gerber_copper_layer"
    );
    assert_eq!(
        production_projections[0]["projection_contract"],
        "datum.production_projection.gerber_copper_layer.v1"
    );
    assert_eq!(
        production_projections[0]["model_revision"],
        report["artifact_metadata"]["model_revision"]
    );
    assert!(
        production_projections[0]["byte_count"]
            .as_u64()
            .expect("projection byte count should serialize")
            > 0
    );
    assert!(
        production_projections[0]["sha256"]
            .as_str()
            .unwrap()
            .starts_with("sha256:")
    );
    assert!(
        report["output_job_run"]["log"]
            .as_array()
            .unwrap()
            .iter()
            .any(|entry| entry["message"]
                .as_str()
                .unwrap()
                .contains("production projection gerber_copper_layer"))
    );
    for file in report["artifact_metadata"]["files"].as_array().unwrap() {
        assert!(file["path"].as_str().unwrap().ends_with(".gbr"));
        assert!(file["sha256"].as_str().unwrap().starts_with("sha256:"));
    }
    for artifact in report["artifacts"].as_array().unwrap() {
        assert!(artifact["sha256"].as_str().unwrap().starts_with("sha256:"));
    }
    let preview_file = report["artifact_metadata"]["files"][0]["path"]
        .as_str()
        .expect("artifact file path should serialize")
        .to_string();
    let preview_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "artifact",
        "preview",
        root.to_str().unwrap(),
        "--artifact",
        report["artifact_metadata"]["artifact_id"].as_str().unwrap(),
        "--file",
        preview_file.as_str(),
    ])
    .expect("CLI should parse");
    let preview_output = execute(preview_cli).expect("artifact preview should succeed");
    let preview: serde_json::Value =
        serde_json::from_str(&preview_output).expect("artifact preview JSON");
    assert_eq!(preview["contract"], "artifact_file_preview_v1");
    assert_eq!(
        preview["artifact_id"],
        report["artifact_metadata"]["artifact_id"]
    );
    assert_eq!(preview["file"], serde_json::json!(preview_file));
    assert_eq!(preview["preview_kind"], "gerber_rs274x");
    assert_eq!(preview["output_dir"], output_dir.display().to_string());
    assert_eq!(preview["preview_available"], true);
    assert_eq!(preview["hash_matches_metadata"], true);
    assert!(
        preview["primitive_count"]
            .as_u64()
            .expect("preview should expose primitive count")
            > 0
    );
    assert!(
        preview["primitives"]
            .as_array()
            .expect("preview should expose primitives")
            .iter()
            .any(|primitive| primitive["kind"] == "stroke" || primitive["kind"] == "flash")
    );
    assert!(
        preview["inspection"]["geometry_count"]
            .as_u64()
            .expect("preview should expose real Gerber geometry")
            > 0
    );

    let repeat_export_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "export-gerber-set",
        root.to_str().unwrap(),
        "--output-dir",
        output_dir.to_str().unwrap(),
        "--prefix",
        "Release A",
    ])
    .expect("CLI should parse");
    let repeat_export_output =
        execute(repeat_export_cli).expect("repeat gerber set export should succeed");
    let repeat_report: serde_json::Value =
        serde_json::from_str(&repeat_export_output).expect("repeat report JSON");
    assert_eq!(
        repeat_report["artifact_metadata"]["artifact_id"],
        report["artifact_metadata"]["artifact_id"]
    );
    assert_eq!(
        repeat_report["artifact_metadata"]["files"],
        report["artifact_metadata"]["files"]
    );
    assert_eq!(
        repeat_report["artifact_metadata"]["output_job"],
        serde_json::json!(output_job)
    );

    let resolve_debug_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "query",
        root.to_str().unwrap(),
        "resolve-debug",
    ])
    .expect("CLI should parse");
    let resolve_debug_output =
        execute(resolve_debug_cli).expect("resolve-debug query should succeed");
    let resolve_debug: serde_json::Value =
        serde_json::from_str(&resolve_debug_output).expect("resolve-debug JSON");
    assert_eq!(resolve_debug["output_job_count"], 1);

    let output_jobs_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "query",
        root.to_str().unwrap(),
        "output-jobs",
    ])
    .expect("CLI should parse");
    let output_jobs_output = execute(output_jobs_cli).expect("output-jobs query should succeed");
    let output_jobs_report: serde_json::Value =
        serde_json::from_str(&output_jobs_output).expect("output-jobs JSON");
    assert_eq!(output_jobs_report["contract"], "output_job_list_v1");
    assert_eq!(output_jobs_report["output_job_count"], 1);
    assert_eq!(
        output_jobs_report["output_jobs"][0]["manufacturing_plan"],
        serde_json::json!(manufacturing_plan)
    );
    assert_eq!(
        output_jobs_report["output_jobs"][0]["id"],
        serde_json::json!(output_job)
    );
    assert_eq!(output_jobs_report["output_jobs"][0]["status"], "succeeded");
    assert_eq!(output_jobs_report["output_jobs"][0]["execution_count"], 2);
    assert_eq!(
        output_jobs_report["output_jobs"][0]["latest_run"]["run_id"],
        repeat_report["output_job_run"]["run_id"]
    );
    assert_eq!(
        output_jobs_report["output_jobs"][0]["artifacts"][0]["artifact_id"],
        report["artifact_metadata"]["artifact_id"]
    );
    assert_eq!(
        output_jobs_report["output_jobs"][0]["name"],
        "Release A fabrication"
    );
    assert_eq!(
        output_jobs_report["output_jobs"][0]["include"],
        serde_json::json!(["gerber_set"])
    );

    let manufacturing_plans_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "query",
        root.to_str().unwrap(),
        "manufacturing-plans",
    ])
    .expect("CLI should parse");
    let manufacturing_plans_output =
        execute(manufacturing_plans_cli).expect("manufacturing-plans query should succeed");
    let manufacturing_plans_report: serde_json::Value =
        serde_json::from_str(&manufacturing_plans_output).expect("manufacturing-plans JSON");
    assert_eq!(
        manufacturing_plans_report["contract"],
        "manufacturing_plan_list_v1"
    );
    assert_eq!(manufacturing_plans_report["manufacturing_plan_count"], 1);
    assert_eq!(
        manufacturing_plans_report["manufacturing_plans"][0]["id"],
        serde_json::json!(manufacturing_plan)
    );

    let artifacts_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "query",
        root.to_str().unwrap(),
        "artifacts",
    ])
    .expect("CLI should parse");
    let artifacts_output = execute(artifacts_cli).expect("artifact query should succeed");
    let artifacts_report: serde_json::Value =
        serde_json::from_str(&artifacts_output).expect("artifact query JSON");
    assert_eq!(artifacts_report["contract"], "artifact_metadata_list_v1");
    assert_eq!(artifacts_report["artifact_count"], 1);
    assert_eq!(
        artifacts_report["artifacts"][0]["artifact_id"],
        report["artifact_metadata"]["artifact_id"]
    );
    assert_eq!(
        artifacts_report["artifacts"][0]["files"],
        report["artifact_metadata"]["files"]
    );
    assert_eq!(
        artifacts_report["artifacts"][0]["production_projections"],
        report["artifact_metadata"]["production_projections"]
    );
    assert_eq!(
        artifacts_report["artifacts"][0]["output_job"],
        serde_json::json!(output_job)
    );

    let validate_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "validate-gerber-set",
        root.to_str().unwrap(),
        "--output-dir",
        output_dir.to_str().unwrap(),
        "--prefix",
        "Release A",
    ])
    .expect("CLI should parse");
    let validate_output = execute(validate_cli).expect("gerber set validation should succeed");
    let validate_report: serde_json::Value =
        serde_json::from_str(&validate_output).expect("validate report JSON");
    assert_eq!(validate_report["artifact_validation_state"], "valid");
    assert_eq!(validate_report["artifact_file_hash_mismatch_count"], 0);
    assert_eq!(
        validate_report["artifact_id"],
        report["artifact_metadata"]["artifact_id"]
    );

    let artifacts_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "query",
        root.to_str().unwrap(),
        "artifacts",
    ])
    .expect("CLI should parse");
    let artifacts_output = execute(artifacts_cli).expect("artifact query should succeed");
    let artifacts_report: serde_json::Value =
        serde_json::from_str(&artifacts_output).expect("artifact query JSON");
    assert_eq!(
        artifacts_report["artifacts"][0]["validation_state"],
        "valid"
    );
    assert_eq!(
        artifacts_report["artifacts"][0]["production_projections"],
        report["artifact_metadata"]["production_projections"]
    );

    let compare_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "compare-gerber-export-plan",
        root.to_str().unwrap(),
        "--output-dir",
        output_dir.to_str().unwrap(),
        "--prefix",
        "Release A",
    ])
    .expect("CLI should parse");
    let compare_output = execute(compare_cli).expect("gerber plan compare should succeed");
    let compare_report: serde_json::Value =
        serde_json::from_str(&compare_output).expect("compare report JSON");
    assert_eq!(compare_report["missing_count"], 0);
    assert_eq!(compare_report["extra_count"], 0);
    assert_eq!(compare_report["expected_count"], 6);
    assert_eq!(
        output_dir.join("release-a-l2-top-mask-mask.gbr").exists(),
        true
    );
    assert_eq!(
        output_dir.join("release-a-l4-top-paste-paste.gbr").exists(),
        true
    );
}

#[test]
fn project_validate_gerber_set_reports_missing_mismatched_and_extra_files() {
    let root = unique_project_root("datum-eda-cli-project-gerber-set-validate");
    create_native_project(&root, Some("Gerber Set Validate Demo".to_string()))
        .expect("initial scaffold should succeed");

    let board_json = root.join("board/board.json");
    std::fs::write(
        &board_json,
        format!(
            "{}\n",
            to_json_deterministic(&serde_json::json!({
                "schema_version": 1,
                "uuid": Uuid::new_v4(),
                "name": "Gerber Set Validate Demo Board",
                "stackup": {
                    "layers": [
                        {"id": 1, "name": "Top Copper", "layer_type": "Copper", "thickness_nm": 35000},
                        {"id": 2, "name": "Top Mask", "layer_type": "SolderMask", "thickness_nm": 10000},
                        {"id": 3, "name": "Top Silk", "layer_type": "Silkscreen", "thickness_nm": 10000},
                        {"id": 4, "name": "Top Paste", "layer_type": "Paste", "thickness_nm": 10000},
                        {"id": 41, "name": "Mechanical 41", "layer_type": "Mechanical", "thickness_nm": 0}
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
                "packages": {},
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
                "pads": {},
                "tracks": {},
                "vias": {},
                "zones": {},
                "nets": {},
                "net_classes": {},
                "keepouts": [],
                "dimensions": [],
                "texts": []
            }))
            .expect("canonical serialization should succeed")
        ),
    )
    .expect("board file should write");

    let output_dir = root.join("gerbers");
    let export_cli = Cli::try_parse_from([
        "eda",
        "project",
        "export-gerber-set",
        root.to_str().unwrap(),
        "--output-dir",
        output_dir.to_str().unwrap(),
        "--prefix",
        "Release A",
    ])
    .expect("CLI should parse");
    execute(export_cli).expect("gerber set export should succeed");

    std::fs::remove_file(output_dir.join("release-a-l4-top-paste-paste.gbr"))
        .expect("paste gerber should remove");
    std::fs::write(
        output_dir.join("release-a-l3-top-silk-silk.gbr"),
        "G04 drifted silk*\nM02*\n",
    )
    .expect("silkscreen gerber should rewrite");
    std::fs::write(output_dir.join("extra.gbr"), "G04 extra*\nM02*\n").expect("extra should write");

    let validate_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "validate-gerber-set",
        root.to_str().unwrap(),
        "--output-dir",
        output_dir.to_str().unwrap(),
        "--prefix",
        "Release A",
    ])
    .expect("CLI should parse");
    let (output, exit_code) = execute_with_exit_code(validate_cli).expect("validation should run");
    let report: serde_json::Value = serde_json::from_str(&output).expect("report JSON");
    assert_eq!(exit_code, 1);
    assert_eq!(report["action"], "validate_gerber_set");
    assert_eq!(report["missing_count"], 1);
    assert_eq!(report["mismatched_count"], 1);
    assert_eq!(report["extra_count"], 1);
    assert_eq!(report["matched_count"], 4);
    assert_eq!(report["artifact_validation_state"], "invalid");
    assert!(
        report["artifact_file_hash_mismatch_count"]
            .as_u64()
            .unwrap()
            > 0
    );

    let check_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "query",
        root.to_str().unwrap(),
        "check-run",
    ])
    .expect("CLI should parse");
    let check_output = execute(check_cli).expect("check run should succeed");
    let check_report: serde_json::Value =
        serde_json::from_str(&check_output).expect("check run JSON");
    assert_eq!(check_report["status"], "error");
    assert!(check_report["summary"]["errors"].as_u64().unwrap() > 0);
    assert!(
        check_report["findings"]
            .as_array()
            .unwrap()
            .iter()
            .any(|entry| {
                entry["source"] == "artifact"
                    && entry["code"] == "artifact_validation_invalid"
                    && entry["payload"]["validation_state"] == "invalid"
                    && entry["payload"]["production_projections"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .any(|projection| {
                            projection["projection_kind"] == "gerber_copper_layer"
                                && projection["projection_contract"]
                                    == "datum.production_projection.gerber_copper_layer.v1"
                        })
            })
    );
}
