use super::*;

fn repo_native_validation_manifest_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../crates/test-harness/testdata/quality/native_project_validation_manifest_v1.json",
    )
}

#[test]
fn project_validate_repo_native_fixtures_manifest_entries() {
    let manifest_path = repo_native_validation_manifest_path();
    let manifest: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(&manifest_path).expect("manifest should read"),
    )
    .expect("manifest should parse");
    assert_eq!(manifest["kind"], "native_project_validation_manifest");
    assert_eq!(manifest["version"], 1);

    let fixtures = manifest["fixtures"]
        .as_array()
        .expect("fixtures should be an array");
    assert!(!fixtures.is_empty(), "fixture manifest should not be empty");

    for fixture in fixtures {
        let fixture_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join(
                fixture["project_root"]
                    .as_str()
                    .expect("project_root should be a string"),
            );
        let expected_valid = fixture["expected_valid"]
            .as_bool()
            .expect("expected_valid should be bool");

        let cli = Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "validate",
            fixture_root.to_str().unwrap(),
        ])
        .expect("CLI should parse");
        let (output, exit_code) =
            execute_with_exit_code(cli).expect("project validate should execute");
        let report: serde_json::Value =
            serde_json::from_str(&output).expect("validation output should parse");

        assert_eq!(report["action"], "validate_project");
        assert_eq!(report["project_root"], fixture_root.display().to_string());
        assert_eq!(report["valid"], expected_valid);
        assert_eq!(exit_code, if expected_valid { 0 } else { 1 });
    }
}
