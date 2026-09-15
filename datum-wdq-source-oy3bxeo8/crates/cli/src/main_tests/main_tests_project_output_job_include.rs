use super::*;
use eda_engine::ir::serialization::to_json_deterministic;

fn unique_project_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{label}-{}", Uuid::new_v4()))
}

fn write_minimal_board(root: &Path) {
    let board_json = root.join("board/board.json");
    std::fs::write(
        &board_json,
        format!(
            "{}\n",
            to_json_deterministic(&serde_json::json!({
                "schema_version": 1,
                "uuid": Uuid::new_v4(),
                "name": "Output Job Include Board",
                "stackup": {
                    "layers": [
                        {"id": 1, "name": "Top Copper", "layer_type": "Copper", "thickness_nm": 35000}
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

#[test]
fn project_output_job_runs_stored_multi_include_scope() {
    let root = unique_project_root("datum-eda-cli-project-output-job-multi-include");
    create_native_project(&root, Some("Output Job Include Demo".to_string()))
        .expect("initial scaffold should succeed");
    write_minimal_board(&root);
    let output_dir = root.join("manufacturing-bundle");

    let create_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "create-output-job",
            root.to_str().unwrap(),
            "--prefix",
            "release-a",
            "--output-dir",
            output_dir.to_str().unwrap(),
            "--include",
            "bom,pnp",
        ])
        .expect("CLI should parse"),
    )
    .expect("output job create should succeed");
    let create_report: serde_json::Value =
        serde_json::from_str(&create_output).expect("create output job JSON");
    let output_job = create_report["output_job"]["id"].as_str().unwrap();
    assert_eq!(
        create_report["output_job"]["include"],
        serde_json::json!(["bom", "pnp"])
    );

    let run_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "run-output-job",
            root.to_str().unwrap(),
            "--output-job",
            output_job,
        ])
        .expect("CLI should parse"),
    )
    .expect("output job run should succeed");
    let run_report: serde_json::Value =
        serde_json::from_str(&run_output).expect("run output job JSON");

    assert_eq!(run_report["status"], "succeeded");
    assert_eq!(
        run_report["artifact_report"]["include"],
        serde_json::json!(["bom", "pnp"])
    );
    assert_eq!(run_report["artifact_report"]["generated_count"], 2);
    assert_eq!(run_report["output_job_run"]["output_job"], output_job);
    assert_eq!(run_report["output_job_run"]["run_sequence"], 1);
    assert_eq!(
        run_report["output_job_run"]["artifact_id"],
        serde_json::Value::Null
    );
    assert_eq!(
        run_report["output_job_run"]["log"]
            .as_array()
            .unwrap()
            .len(),
        3
    );
    for generated in run_report["artifact_report"]["generated"]
        .as_array()
        .unwrap()
    {
        assert_eq!(
            generated["report"]["output_job_run"],
            serde_json::Value::Null
        );
        assert_eq!(
            generated["report"]["output_job_run_path"],
            serde_json::Value::Null
        );
    }
    assert!(output_dir.join("release-a-bom.csv").is_file());
    assert!(output_dir.join("release-a-pnp.csv").is_file());

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
        serde_json::from_str(&query_output).expect("query output jobs JSON");
    assert_eq!(query_report["output_jobs"][0]["execution_count"], 1);
    assert_eq!(
        query_report["output_jobs"][0]["latest_run"]["run_sequence"],
        1
    );
    assert_eq!(
        query_report["output_jobs"][0]["latest_run"]["artifact_id"],
        serde_json::Value::Null
    );
}

#[test]
fn project_export_manufacturing_set_uses_stored_output_job_include_scope() {
    let root = unique_project_root("datum-eda-cli-project-output-job-manufacturing-include");
    create_native_project(
        &root,
        Some("Output Job Manufacturing Include Demo".to_string()),
    )
    .expect("initial scaffold should succeed");
    write_minimal_board(&root);
    let output_dir = root.join("selected-manufacturing");

    let create_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "create-output-job",
            root.to_str().unwrap(),
            "--prefix",
            "release-b",
            "--include",
            "bom,pnp",
        ])
        .expect("CLI should parse"),
    )
    .expect("output job create should succeed");
    let create_report: serde_json::Value =
        serde_json::from_str(&create_output).expect("create output job JSON");
    let output_job = create_report["output_job"]["id"].as_str().unwrap();

    let export_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "export-manufacturing-set",
            root.to_str().unwrap(),
            "--output-dir",
            output_dir.to_str().unwrap(),
            "--output-job",
            output_job,
        ])
        .expect("CLI should parse"),
    )
    .expect("manufacturing export should succeed");
    let export_report: serde_json::Value =
        serde_json::from_str(&export_output).expect("manufacturing export JSON");
    assert_eq!(export_report["prefix"], "release-b");
    assert_eq!(export_report["written_count"], 2);
    assert_eq!(export_report["artifact_metadata"]["output_job"], output_job);
    assert_eq!(
        export_report["artifacts"]
            .as_array()
            .unwrap()
            .iter()
            .map(|artifact| artifact["kind"].as_str().unwrap())
            .collect::<Vec<_>>(),
        vec!["bom", "pnp"]
    );
    assert!(output_dir.join("release-b-bom.csv").is_file());
    assert!(output_dir.join("release-b-pnp.csv").is_file());
    assert!(!output_dir.join("release-b-drill.csv").exists());

    let validate_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "validate-manufacturing-set",
            root.to_str().unwrap(),
            "--output-dir",
            output_dir.to_str().unwrap(),
            "--output-job",
            output_job,
        ])
        .expect("CLI should parse"),
    )
    .expect("manufacturing validation should succeed");
    let validate_report: serde_json::Value =
        serde_json::from_str(&validate_output).expect("manufacturing validate JSON");
    assert_eq!(validate_report["expected_count"], 2);
    assert_eq!(validate_report["matched_count"], 2);
    assert_eq!(validate_report["missing_count"], 0);
    assert_eq!(validate_report["mismatched_count"], 0);
}

#[test]
fn project_export_manufacturing_set_resolves_output_job_by_name() {
    let root = unique_project_root("datum-eda-cli-project-output-job-name");
    create_native_project(&root, Some("Output Job Name Demo".to_string()))
        .expect("initial scaffold should succeed");
    write_minimal_board(&root);
    let output_dir = root.join("named-manufacturing");

    execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "create-output-job",
            root.to_str().unwrap(),
            "--prefix",
            "release-c",
            "--include",
            "bom,pnp",
            "--name",
            "Assembly CSV",
        ])
        .expect("CLI should parse"),
    )
    .expect("named output job create should succeed");

    let export_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "export-manufacturing-set",
            root.to_str().unwrap(),
            "--output-dir",
            output_dir.to_str().unwrap(),
            "--job",
            "Assembly CSV",
        ])
        .expect("CLI should parse"),
    )
    .expect("manufacturing export by job name should succeed");
    let export_report: serde_json::Value =
        serde_json::from_str(&export_output).expect("manufacturing export JSON");
    assert_eq!(export_report["prefix"], "release-c");
    assert_eq!(export_report["written_count"], 2);
    assert!(output_dir.join("release-c-bom.csv").is_file());
    assert!(output_dir.join("release-c-pnp.csv").is_file());
    assert!(!output_dir.join("release-c-drill.csv").exists());
}

#[test]
fn project_export_manufacturing_set_rejects_ambiguous_output_job_name() {
    let root = unique_project_root("datum-eda-cli-project-output-job-ambiguous-name");
    create_native_project(&root, Some("Output Job Ambiguous Name Demo".to_string()))
        .expect("initial scaffold should succeed");
    write_minimal_board(&root);
    for prefix in ["release-d", "release-e"] {
        execute(
            Cli::try_parse_from([
                "eda",
                "--format",
                "json",
                "project",
                "create-output-job",
                root.to_str().unwrap(),
                "--prefix",
                prefix,
                "--include",
                "bom",
                "--name",
                "Duplicated Job",
            ])
            .expect("CLI should parse"),
        )
        .expect("named output job create should succeed");
    }

    let error = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "export-manufacturing-set",
            root.to_str().unwrap(),
            "--output-dir",
            root.join("ambiguous-manufacturing").to_str().unwrap(),
            "--job",
            "Duplicated Job",
        ])
        .expect("CLI should parse"),
    )
    .expect_err("ambiguous output job name should be rejected");
    assert!(
        format!("{error:#}").contains("ambiguous"),
        "error should explain ambiguous output job names: {error:#}"
    );
}
