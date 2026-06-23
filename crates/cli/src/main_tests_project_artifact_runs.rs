use super::*;

fn unique_project_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{label}-{}", Uuid::new_v4()))
}

fn place_unfilled_zone(root: &Path) {
    let class_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "place-board-net-class",
            root.to_str().unwrap(),
            "--name",
            "Default",
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
    execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "place-board-zone",
            root.to_str().unwrap(),
            "--net",
            net_report["net_uuid"].as_str().unwrap(),
            "--vertex",
            "0:0",
            "--vertex",
            "1000:0",
            "--vertex",
            "1000:1000",
            "--layer",
            "1",
            "--priority",
            "2",
            "--thermal-gap-nm",
            "0",
            "--thermal-spoke-width-nm",
            "0",
        ])
        .expect("CLI should parse"),
    )
    .expect("place board zone should succeed");
    execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "place-board-zone",
            root.to_str().unwrap(),
            "--net",
            net_report["net_uuid"].as_str().unwrap(),
            "--vertex",
            "2000:0",
            "--vertex",
            "3000:0",
            "--vertex",
            "3000:1000",
            "--layer",
            "1",
            "--priority",
            "1",
            "--thermal-gap-nm",
            "0",
            "--thermal-spoke-width-nm",
            "0",
        ])
        .expect("CLI should parse"),
    )
    .expect("place second board zone should succeed");
}

#[test]
fn unlinked_artifact_generate_records_artifact_run_evidence() {
    let root = unique_project_root("datum-eda-cli-artifact-run");
    create_native_project(&root, Some("Artifact Run Demo".to_string()))
        .expect("initial scaffold should succeed");
    let output_dir = root.join("generated-unlinked-artifacts");

    let generated = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "artifact",
            "generate",
            root.to_str().unwrap(),
            "--output-dir",
            output_dir.to_str().unwrap(),
            "--include",
            "bom",
            "--prefix",
            "Unlinked A",
        ])
        .expect("CLI should parse"),
    )
    .expect("unlinked artifact generate should succeed");
    let report: serde_json::Value =
        serde_json::from_str(&generated).expect("artifact generate JSON");
    let entry = &report["generated"][0];
    assert_eq!(entry["include"], "bom");
    assert_eq!(entry["output_job_run"], serde_json::Value::Null);
    assert_eq!(entry["artifact_run"]["status"], "succeeded");
    assert_eq!(entry["artifact_run"]["run_sequence"], 1);
    assert_eq!(entry["artifact_run"]["artifact_id"], entry["artifact_id"]);
    assert!(std::path::Path::new(entry["artifact_run_path"].as_str().unwrap()).is_file());
    assert_eq!(entry["report"]["output_job_run"], serde_json::Value::Null);
    assert_eq!(entry["report"]["artifact_run"]["status"], "succeeded");
    assert_eq!(entry["report"]["artifact_run"]["run_sequence"], 1);
    assert_eq!(
        entry["report"]["artifact_run"]["artifact_id"],
        entry["artifact_id"]
    );
    assert!(std::path::Path::new(entry["report"]["artifact_run_path"].as_str().unwrap()).is_file());

    let artifact_id = entry["artifact_id"].as_str().unwrap();
    let shown = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "artifact",
            "show",
            root.to_str().unwrap(),
            "--artifact",
            artifact_id,
        ])
        .expect("CLI should parse"),
    )
    .expect("artifact show should succeed");
    let shown: serde_json::Value = serde_json::from_str(&shown).expect("artifact show JSON");
    assert_eq!(shown["run_count"], 1);
    assert_eq!(shown["latest_run"]["artifact_id"], artifact_id);
    assert_eq!(shown["latest_run"]["run_sequence"], 1);

    let listed = execute(
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
    let listed: serde_json::Value = serde_json::from_str(&listed).expect("artifact list JSON");
    assert_eq!(listed["artifact_run_count"], 1);
    assert_eq!(listed["artifact_runs"][0]["artifact_id"], artifact_id);

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn artifact_generate_include_blocks_active_release_check_errors() {
    let root = unique_project_root("datum-eda-cli-artifact-run-check-gate");
    create_native_project(&root, Some("Artifact Run Check Gate Demo".to_string()))
        .expect("initial scaffold should succeed");
    place_unfilled_zone(&root);
    let output_dir = root.join("blocked-artifacts");

    let error = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "artifact",
            "generate",
            root.to_str().unwrap(),
            "--output-dir",
            output_dir.to_str().unwrap(),
            "--include",
            "bom",
            "--prefix",
            "Blocked",
        ])
        .expect("CLI should parse"),
    )
    .expect_err("artifact generate should be blocked by release check gate");
    let message = format!("{error:#}");
    assert!(message.contains("release check gate failed"));
    assert!(message.contains("2 active error code(s)"));
    assert!(message.contains("zone_fill_unfilled"));
    assert!(
        !output_dir.exists(),
        "release gate should block before artifact files are written"
    );

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_export_gerber_set_blocks_active_release_check_errors() {
    let root = unique_project_root("datum-eda-cli-gerber-export-check-gate");
    create_native_project(&root, Some("Gerber Export Check Gate Demo".to_string()))
        .expect("initial scaffold should succeed");
    place_unfilled_zone(&root);
    let output_dir = root.join("blocked-gerbers");

    let error = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "export-gerber-set",
            root.to_str().unwrap(),
            "--output-dir",
            output_dir.to_str().unwrap(),
            "--prefix",
            "Blocked",
        ])
        .expect("CLI should parse"),
    )
    .expect_err("gerber export should be blocked by release check gate");
    let message = format!("{error:#}");
    assert!(message.contains("release check gate failed"));
    assert!(message.contains("2 active error code(s)"));
    assert!(message.contains("zone_fill_unfilled"));
    assert!(!output_dir.exists());

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn artifact_export_manufacturing_set_blocks_active_release_check_errors() {
    let root = unique_project_root("datum-eda-cli-manufacturing-export-check-gate");
    create_native_project(
        &root,
        Some("Manufacturing Export Check Gate Demo".to_string()),
    )
    .expect("initial scaffold should succeed");
    place_unfilled_zone(&root);
    let output_dir = root.join("blocked-manufacturing");

    let error = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "artifact",
            "export-manufacturing-set",
            root.to_str().unwrap(),
            "--output-dir",
            output_dir.to_str().unwrap(),
            "--prefix",
            "Blocked",
        ])
        .expect("CLI should parse"),
    )
    .expect_err("manufacturing export should be blocked by release check gate");
    let message = format!("{error:#}");
    assert!(message.contains("release check gate failed"));
    assert!(message.contains("2 active error code(s)"));
    assert!(message.contains("zone_fill_unfilled"));
    assert!(!output_dir.exists());

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_export_manufacturing_set_blocks_active_release_check_errors() {
    let root = unique_project_root("datum-eda-cli-project-manufacturing-export-check-gate");
    create_native_project(
        &root,
        Some("Project Manufacturing Export Check Gate Demo".to_string()),
    )
    .expect("initial scaffold should succeed");
    place_unfilled_zone(&root);
    let output_dir = root.join("blocked-project-manufacturing");

    let error = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "export-manufacturing-set",
            root.to_str().unwrap(),
            "--output-dir",
            output_dir.to_str().unwrap(),
            "--prefix",
            "Blocked",
        ])
        .expect("CLI should parse"),
    )
    .expect_err("project manufacturing export should be blocked by release check gate");
    let message = format!("{error:#}");
    assert!(message.contains("release check gate failed"));
    assert!(message.contains("2 active error code(s)"));
    assert!(message.contains("zone_fill_unfilled"));
    assert!(!output_dir.exists());

    let _ = std::fs::remove_dir_all(&root);
}
