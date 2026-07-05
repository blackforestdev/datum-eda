use super::*;
use eda_engine::ir::serialization::to_json_deterministic;
use eda_engine::substrate::{ProjectResolver, SourceShardAuthority, SourceShardKind};

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
                "name": "Output Job Replay Board",
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

fn assert_output_job_run_replays_without_authored_revision_mutation(
    root: &Path,
    authored_revision: eda_engine::substrate::ModelRevision,
    run_report: &serde_json::Value,
) {
    assert_eq!(run_report["status"], "succeeded");
    assert_eq!(run_report["model_revision"], authored_revision.0);
    assert_eq!(run_report["output_job_run"]["schema_version"], 1);
    let run_id = run_report["output_job_run"]["run_id"]
        .as_str()
        .expect("run id should serialize")
        .to_string();
    let output_job_run_path = root.join(format!(".datum/output_job_runs/{run_id}.json"));
    assert!(output_job_run_path.is_file());
    std::fs::remove_file(&output_job_run_path).expect("promoted output job run should remove");

    let replayed = ProjectResolver::new(root)
        .resolve()
        .expect("project should recover CLI output job run from journal");
    assert_eq!(
        replayed.model_revision, authored_revision,
        "generated output job runs must not mutate authored model revision"
    );
    assert!(
        replayed.output_job_runs[&Uuid::parse_str(&run_id).unwrap()].schema_version == 1,
        "CLI-generated output job run should replay after promoted shard deletion"
    );
    assert!(replayed.source_shards.iter().any(|shard| {
        shard.kind == SourceShardKind::OutputJobRun
            && shard.authority == SourceShardAuthority::GeneratedEvidence
            && shard.relative_path == format!(".datum/output_job_runs/{run_id}.json")
            && shard.schema_version == Some(1)
    }));
}

#[test]
fn project_run_output_job_generated_run_replays_without_authored_revision_mutation() {
    let root = unique_project_root("datum-eda-cli-project-run-output-job-replay");
    create_native_project(&root, Some("Output Job Replay Demo".to_string()))
        .expect("initial scaffold should succeed");
    write_minimal_gerber_board(&root);

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
    let authored_revision = ProjectResolver::new(&root)
        .resolve()
        .expect("project should resolve before output job run")
        .model_revision;

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
    assert_output_job_run_replays_without_authored_revision_mutation(
        &root,
        authored_revision,
        &run_report,
    );

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_run_output_job_multi_scope_run_replays_without_authored_revision_mutation() {
    let root = unique_project_root("datum-eda-cli-project-run-output-job-multi-scope-replay");
    create_native_project(
        &root,
        Some("Output Job Multi Scope Replay Demo".to_string()),
    )
    .expect("initial scaffold should succeed");
    write_minimal_gerber_board(&root);

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
            "--include",
            "bom,pnp",
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
    let authored_revision = ProjectResolver::new(&root)
        .resolve()
        .expect("project should resolve before output job run")
        .model_revision;

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
    assert_eq!(
        run_report["artifact_report"]["include"],
        serde_json::json!(["bom", "pnp"])
    );
    assert_eq!(run_report["artifact_report"]["generated_count"], 2);
    assert_output_job_run_replays_without_authored_revision_mutation(
        &root,
        authored_revision,
        &run_report,
    );

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_run_output_job_drill_run_replays_without_authored_revision_mutation() {
    let root = unique_project_root("datum-eda-cli-project-run-output-job-drill-replay");
    create_native_project(&root, Some("Output Job Drill Replay Demo".to_string()))
        .expect("initial scaffold should succeed");
    write_minimal_gerber_board(&root);

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
            "--include",
            "drill",
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
    let authored_revision = ProjectResolver::new(&root)
        .resolve()
        .expect("project should resolve before output job run")
        .model_revision;

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
    assert_eq!(
        run_report["artifact_report"]["include"],
        serde_json::json!(["drill"])
    );
    assert_eq!(run_report["artifact_report"]["generated_count"], 1);
    assert_output_job_run_replays_without_authored_revision_mutation(
        &root,
        authored_revision,
        &run_report,
    );

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_run_output_job_manufacturing_set_run_replays_without_authored_revision_mutation() {
    let root = unique_project_root("datum-eda-cli-project-run-output-job-manufacturing-replay");
    create_native_project(
        &root,
        Some("Output Job Manufacturing Replay Demo".to_string()),
    )
    .expect("initial scaffold should succeed");
    write_minimal_gerber_board(&root);

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
            "--include",
            "manufacturing-set",
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
    let authored_revision = ProjectResolver::new(&root)
        .resolve()
        .expect("project should resolve before output job run")
        .model_revision;

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
    assert_eq!(
        run_report["artifact_report"]["include"],
        serde_json::json!(["manufacturing-set"])
    );
    assert_eq!(run_report["artifact_report"]["generated_count"], 1);
    assert_output_job_run_replays_without_authored_revision_mutation(
        &root,
        authored_revision,
        &run_report,
    );

    let _ = std::fs::remove_dir_all(&root);
}
