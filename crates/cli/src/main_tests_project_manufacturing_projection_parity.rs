use super::*;

fn unique_project_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{}-{}", label, Uuid::new_v4()))
}

#[test]
fn manufacturing_export_files_match_manifest_projection() {
    let root = unique_project_root("datum-eda-cli-project-manufacturing-projection-parity");
    create_native_project(&root, Some("Manufacturing Projection Parity".to_string()))
        .expect("initial scaffold should succeed");

    let output_dir = root.join("manufacturing");
    let export = execute(
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
            "Parity A",
            "--include",
            "bom,pnp",
        ])
        .expect("CLI should parse"),
    )
    .expect("manufacturing export should succeed");
    let export_report: serde_json::Value = serde_json::from_str(&export).expect("export JSON");

    let manifest = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "manifest-manufacturing-set",
            root.to_str().unwrap(),
            "--output-dir",
            output_dir.to_str().unwrap(),
            "--prefix",
            "Parity A",
            "--include",
            "bom,pnp",
        ])
        .expect("CLI should parse"),
    )
    .expect("manifest should succeed");
    let manifest_report: serde_json::Value =
        serde_json::from_str(&manifest).expect("manifest JSON");

    let manifest_files = manifest_report["entries"]
        .as_array()
        .expect("manifest entries should be an array")
        .iter()
        .map(|entry| entry["filename"].as_str().unwrap().to_string())
        .collect::<Vec<_>>();
    let metadata_files = export_report["artifact_metadata"]["files"]
        .as_array()
        .expect("metadata files should be an array")
        .iter()
        .map(|entry| entry["path"].as_str().unwrap().to_string())
        .collect::<Vec<_>>();
    let artifact_files = export_report["artifacts"]
        .as_array()
        .expect("artifact views should be an array")
        .iter()
        .map(|entry| {
            std::path::Path::new(entry["output_path"].as_str().unwrap())
                .file_name()
                .unwrap()
                .to_string_lossy()
                .into_owned()
        })
        .collect::<Vec<_>>();

    assert_eq!(metadata_files, manifest_files);
    assert_eq!(artifact_files, manifest_files);

    let _ = std::fs::remove_dir_all(&root);
}
