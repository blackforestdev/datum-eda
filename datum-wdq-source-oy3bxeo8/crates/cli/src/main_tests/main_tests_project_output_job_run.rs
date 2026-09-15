use super::*;
use eda_engine::ir::serialization::to_json_deterministic;

fn unique_project_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{label}-{}", Uuid::new_v4()))
}

fn write_minimal_gerber_board(root: &Path) {
    let board_json = root.join("board/board.json");
    std::fs::write(
        &board_json,
        format!(
            "{}\n",
            to_json_deterministic(&serde_json::json!({
                "schema_version": 1,
                "uuid": Uuid::new_v4(),
                "name": "Output Job Run Board",
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
            .expect("canonical board serialization should succeed")
        ),
    )
    .expect("board file should write");
}

fn write_minimal_gerber_board_with_unfilled_zone(root: &Path) {
    let board_json = root.join("board/board.json");
    let net_uuid = Uuid::new_v4();
    let class_uuid = Uuid::new_v4();
    let zone_uuid = Uuid::new_v4();
    std::fs::write(
        &board_json,
        format!(
            "{}\n",
            to_json_deterministic(&serde_json::json!({
                "schema_version": 1,
                "uuid": Uuid::new_v4(),
                "name": "Output Job Blocked Board",
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
                "zones": {
                    zone_uuid.to_string(): {
                        "uuid": zone_uuid,
                        "net": net_uuid,
                        "polygon": {
                            "vertices": [
                                { "x": 100, "y": 100 },
                                { "x": 900, "y": 100 },
                                { "x": 900, "y": 400 }
                            ],
                            "closed": true
                        },
                        "layer": 1,
                        "priority": 0,
                        "thermal_relief": false,
                        "thermal_gap": 0,
                        "thermal_spoke_width": 0
                    }
                },
                "nets": {
                    net_uuid.to_string(): {
                        "uuid": net_uuid,
                        "name": "GND",
                        "class": class_uuid
                    }
                },
                "net_classes": {
                    class_uuid.to_string(): {
                        "uuid": class_uuid,
                        "name": "Default",
                        "clearance": 150000,
                        "track_width": 200000,
                        "via_drill": 300000,
                        "via_diameter": 600000,
                        "diffpair_width": 0,
                        "diffpair_gap": 0
                    }
                },
                "keepouts": [],
                "dimensions": [],
                "texts": []
            }))
            .expect("canonical board serialization should succeed")
        ),
    )
    .expect("board file should write");
}

#[test]
fn project_run_output_job_uses_authored_output_dir_and_records_run() {
    let root = unique_project_root("datum-eda-cli-project-run-output-job");
    create_native_project(&root, Some("Output Job Run Demo".to_string()))
        .expect("initial scaffold should succeed");
    write_minimal_gerber_board(&root);
    let stored_output_dir = root.join("stored-fab");

    let create_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "create-gerber-output-job",
            root.to_str().unwrap(),
            "--prefix",
            "release-a",
            "--output-dir",
            stored_output_dir.to_str().unwrap(),
            "--name",
            "Release A Gerbers",
        ])
        .expect("CLI should parse"),
    )
    .expect("output job create should succeed");
    let create_report: serde_json::Value =
        serde_json::from_str(&create_output).expect("create output job JSON");
    let output_job = create_report["output_job"]["id"]
        .as_str()
        .expect("output job id should serialize")
        .to_string();

    let run_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "run-output-job",
            root.to_str().unwrap(),
            "--output-job",
            output_job.as_str(),
        ])
        .expect("CLI should parse"),
    )
    .expect("output job run should succeed");
    let run_report: serde_json::Value =
        serde_json::from_str(&run_output).expect("run output job JSON");

    assert_eq!(run_report["contract"], "output_job_run_v1");
    assert_eq!(run_report["action"], "run_output_job");
    assert_eq!(run_report["status"], "succeeded");
    assert_eq!(run_report["exit_code"], 0);
    assert_eq!(run_report["output_job"]["id"], output_job);
    assert_eq!(
        run_report["output_dir"],
        stored_output_dir.display().to_string()
    );
    assert_eq!(
        run_report["artifact_report"]["output_dir"],
        stored_output_dir.display().to_string()
    );
    assert_eq!(
        run_report["artifact_report"]["include"],
        serde_json::json!(["gerber-set"])
    );
    assert_eq!(
        run_report["artifact_report"]["generated"][0]["report"]["output_job_run"],
        serde_json::Value::Null
    );
    assert_eq!(
        run_report["output_job_run"]["artifact_id"],
        run_report["artifact_report"]["generated"][0]["artifact_id"]
    );
    assert_eq!(run_report["output_job_run"]["run_sequence"], 1);
    let first_file = run_report["artifact_report"]["generated"][0]["report"]["artifact_metadata"]
        ["files"][0]["path"]
        .as_str()
        .expect("artifact metadata should list generated files");
    assert!(
        stored_output_dir.join(first_file).is_file(),
        "run-output-job should write artifact files into the authored output_dir"
    );

    let query_output = execute(
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
    .expect("output job query should succeed");
    let query_report: serde_json::Value =
        serde_json::from_str(&query_output).expect("output job query JSON");
    assert_eq!(query_report["output_jobs"][0]["status"], "succeeded");
    assert_eq!(query_report["output_jobs"][0]["execution_count"], 1);
    assert_eq!(
        query_report["output_jobs"][0]["latest_run"]["output_job"],
        output_job
    );
    assert_eq!(
        query_report["output_jobs"][0]["latest_run"]["run_sequence"],
        1
    );
    assert_eq!(
        query_report["output_jobs"][0]["artifacts"]
            .as_array()
            .unwrap()
            .len(),
        1
    );

    let second_run_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "run-output-job",
            root.to_str().unwrap(),
            "--output-job",
            output_job.as_str(),
        ])
        .expect("CLI should parse"),
    )
    .expect("second output job run should succeed");
    let second_run_report: serde_json::Value =
        serde_json::from_str(&second_run_output).expect("second run output job JSON");
    assert_eq!(second_run_report["output_job_run"]["run_sequence"], 2);
    assert_ne!(
        second_run_report["output_job_run"]["run_id"],
        run_report["output_job_run"]["run_id"]
    );
    let second_query_output = execute(
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
    .expect("second output job query should succeed");
    let second_query_report: serde_json::Value =
        serde_json::from_str(&second_query_output).expect("second output job query JSON");
    assert_eq!(second_query_report["output_jobs"][0]["execution_count"], 2);
    assert_eq!(
        second_query_report["output_jobs"][0]["latest_run"]["run_id"],
        second_run_report["output_job_run"]["run_id"]
    );
    assert_eq!(
        second_query_report["output_jobs"][0]["latest_run"]["run_sequence"],
        2
    );
}

#[test]
fn project_run_output_job_blocks_active_release_check_errors() {
    let root = unique_project_root("datum-eda-cli-project-run-output-job-check-gate");
    create_native_project(&root, Some("Output Job Check Gate Demo".to_string()))
        .expect("initial scaffold should succeed");
    write_minimal_gerber_board_with_unfilled_zone(&root);

    let create_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "create-gerber-output-job",
            root.to_str().unwrap(),
            "--prefix",
            "release-blocked",
        ])
        .expect("CLI should parse"),
    )
    .expect("output job create should succeed");
    let create_report: serde_json::Value =
        serde_json::from_str(&create_output).expect("create output job JSON");
    let output_job = create_report["output_job"]["id"]
        .as_str()
        .expect("output job id should serialize")
        .to_string();

    let (run_output, exit_code) = execute_with_exit_code(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "run-output-job",
            root.to_str().unwrap(),
            "--output-job",
            output_job.as_str(),
        ])
        .expect("CLI should parse"),
    )
    .expect("blocked output job run should return JSON");
    assert_eq!(exit_code, 1);
    let run_report: serde_json::Value =
        serde_json::from_str(&run_output).expect("blocked run output job JSON");
    assert_eq!(run_report["contract"], "output_job_run_v1");
    assert_eq!(run_report["status"], "failed");
    assert_eq!(run_report["exit_code"], 1);
    assert_eq!(run_report["artifact_report"], serde_json::Value::Null);
    assert_eq!(run_report["output_job_run"]["status"], "failed");
    assert_eq!(
        run_report["output_job_run"]["artifact_id"],
        serde_json::Value::Null
    );
    assert_eq!(run_report["check_run"]["profile_id"], "release");
    assert_eq!(run_report["check_run"]["status"], "error");
    assert!(
        run_report["check_run"]["active_error_count"]
            .as_u64()
            .unwrap()
            > 0
    );
    assert!(
        run_report["check_run"]["active_error_codes"]
            .as_array()
            .unwrap()
            .iter()
            .any(|code| code == "zone_fill_unfilled")
    );
    assert!(
        run_report["error"]
            .as_str()
            .unwrap()
            .contains("zone_fill_unfilled")
    );
    assert!(
        std::path::Path::new(run_report["output_job_run_path"].as_str().unwrap()).is_file(),
        "blocked run should persist failed output-job evidence"
    );

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn artifact_generate_output_job_executes_authored_output_job() {
    let root = unique_project_root("datum-eda-cli-artifact-generate-output-job");
    create_native_project(&root, Some("Artifact Output Job Demo".to_string()))
        .expect("initial scaffold should succeed");
    write_minimal_gerber_board(&root);
    let stored_output_dir = root.join("artifact-job-fab");

    let create_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "create-gerber-output-job",
            root.to_str().unwrap(),
            "--prefix",
            "release-c",
            "--output-dir",
            stored_output_dir.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("output job create should succeed");
    let create_report: serde_json::Value =
        serde_json::from_str(&create_output).expect("create output job JSON");
    let output_job = create_report["output_job"]["id"]
        .as_str()
        .expect("output job id should serialize");

    let run_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "artifact",
            "generate",
            root.to_str().unwrap(),
            "--output-job",
            output_job,
        ])
        .expect("CLI should parse"),
    )
    .expect("artifact output job run should succeed");
    let run_report: serde_json::Value =
        serde_json::from_str(&run_output).expect("artifact output job run JSON");

    assert_eq!(run_report["contract"], "output_job_run_v1");
    assert_eq!(run_report["action"], "run_output_job");
    assert_eq!(run_report["output_job"]["id"], output_job);
    assert_eq!(
        run_report["output_dir"],
        stored_output_dir.display().to_string()
    );
    assert_eq!(run_report["output_job_run"]["run_sequence"], 1);

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_run_output_job_records_failed_run_for_generation_error() {
    let root = unique_project_root("datum-eda-cli-project-run-output-job-failed");
    create_native_project(&root, Some("Output Job Failed Run Demo".to_string()))
        .expect("initial scaffold should succeed");
    let blocked_output_dir = root.join("blocked-output");
    std::fs::write(&blocked_output_dir, "not a directory").expect("blocked path should write");

    let create_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "create-gerber-output-job",
            root.to_str().unwrap(),
            "--prefix",
            "release-a",
            "--output-dir",
            blocked_output_dir.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("output job create should succeed");
    let create_report: serde_json::Value =
        serde_json::from_str(&create_output).expect("create output job JSON");
    let output_job = create_report["output_job"]["id"]
        .as_str()
        .expect("output job id should serialize")
        .to_string();

    let (run_output, exit_code) = execute_with_exit_code(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "run-output-job",
            root.to_str().unwrap(),
            "--output-job",
            output_job.as_str(),
        ])
        .expect("CLI should parse"),
    )
    .expect("failed output job run should still return JSON");
    assert_eq!(exit_code, 1);
    let run_report: serde_json::Value =
        serde_json::from_str(&run_output).expect("failed run output job JSON");
    assert_eq!(run_report["contract"], "output_job_run_v1");
    assert_eq!(run_report["status"], "failed");
    assert_eq!(run_report["exit_code"], 1);
    assert_eq!(run_report["output_job"]["id"], output_job);
    assert_eq!(run_report["output_job_run"]["status"], "failed");
    assert_eq!(run_report["output_job_run"]["run_sequence"], 1);
    assert_eq!(
        run_report["output_job_run"]["artifact_id"],
        serde_json::Value::Null
    );
    assert!(
        std::path::Path::new(run_report["output_job_run_path"].as_str().unwrap()).is_file(),
        "failed run should persist generated-evidence shard"
    );
    assert!(
        run_report["error"]
            .as_str()
            .unwrap()
            .contains("failed to create")
    );

    let query_output = execute(
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
    .expect("output job query should succeed");
    let query_report: serde_json::Value =
        serde_json::from_str(&query_output).expect("output job query JSON");
    assert_eq!(query_report["output_jobs"][0]["status"], "failed");
    assert_eq!(query_report["output_jobs"][0]["execution_count"], 1);
    assert_eq!(
        query_report["output_jobs"][0]["latest_run"]["status"],
        "failed"
    );
}

#[test]
fn project_output_job_run_lifecycle_records_running_and_canceled_status() {
    let root = unique_project_root("datum-eda-cli-project-output-job-run-lifecycle");
    create_native_project(&root, Some("Output Job Lifecycle Demo".to_string()))
        .expect("initial scaffold should succeed");

    let create_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "create-gerber-output-job",
            root.to_str().unwrap(),
            "--prefix",
            "release-a",
        ])
        .expect("CLI should parse"),
    )
    .expect("output job create should succeed");
    let create_report: serde_json::Value =
        serde_json::from_str(&create_output).expect("create output job JSON");
    let output_job = create_report["output_job"]["id"]
        .as_str()
        .expect("output job id should serialize")
        .to_string();

    let start_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "artifact",
            "start-output-job-run",
            root.to_str().unwrap(),
            "--output-job",
            output_job.as_str(),
        ])
        .expect("CLI should parse"),
    )
    .expect("output job run start should succeed");
    let start_report: serde_json::Value =
        serde_json::from_str(&start_output).expect("start output job run JSON");
    assert_eq!(start_report["contract"], "output_job_run_lifecycle_v1");
    assert_eq!(start_report["action"], "start_output_job_run");
    assert_eq!(start_report["output_job_run"]["status"], "running");
    assert_eq!(start_report["output_job_run"]["run_sequence"], 1);
    let run_id = start_report["output_job_run"]["run_id"]
        .as_str()
        .expect("run id should serialize")
        .to_string();

    let running_query = execute(
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
    let running_report: serde_json::Value =
        serde_json::from_str(&running_query).expect("running output jobs JSON");
    assert_eq!(running_report["output_jobs"][0]["status"], "running");
    assert_eq!(
        running_report["output_jobs"][0]["latest_run"]["run_id"],
        run_id
    );

    let cancel_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "artifact",
            "cancel-output-job-run",
            root.to_str().unwrap(),
            "--run",
            run_id.as_str(),
        ])
        .expect("CLI should parse"),
    )
    .expect("output job run cancel should succeed");
    let cancel_report: serde_json::Value =
        serde_json::from_str(&cancel_output).expect("cancel output job run JSON");
    assert_eq!(cancel_report["action"], "cancel_output_job_run");
    assert_eq!(cancel_report["output_job_run"]["status"], "canceled");
    assert_eq!(cancel_report["output_job_run"]["run_sequence"], 1);
    assert_eq!(cancel_report["output_job_run"]["exit_code"], 130);

    let canceled_query = execute(
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
    let canceled_report: serde_json::Value =
        serde_json::from_str(&canceled_query).expect("canceled output jobs JSON");
    assert_eq!(canceled_report["output_jobs"][0]["status"], "canceled");
    assert_eq!(canceled_report["output_jobs"][0]["execution_count"], 1);
    assert_eq!(
        canceled_report["output_jobs"][0]["latest_run"]["run_id"],
        run_id
    );
}
