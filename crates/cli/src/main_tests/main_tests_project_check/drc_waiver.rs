use super::*;

#[test]
fn project_query_check_includes_waived_native_drc_results() {
    let root = unique_project_root("datum-eda-cli-project-query-check-drc");
    create_native_project(&root, Some("Check Demo".to_string()))
        .expect("initial scaffold should succeed");
    let _ = build_native_check_fixture(&root);
    let net_uuid = seed_board_drc_fixture(&root);

    write_native_waivers(
        &root,
        &[serde_json::to_value(serde_json::json!({
            "uuid": Uuid::new_v4(),
            "domain": CheckDomain::DRC,
            "target": WaiverTarget::Object(net_uuid),
            "rationale": "Intentional unrouted fixture net",
            "created_by": "cli-test"
        }))
        .expect("waiver should serialize")],
    );

    let cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "query",
        root.to_str().unwrap(),
        "check",
    ])
    .expect("CLI should parse");

    let output = execute(cli).expect("project query check should succeed");
    let report: serde_json::Value = serde_json::from_str(&output).expect("query JSON should parse");
    assert_eq!(report["domain"], "combined");
    assert_eq!(report["summary"]["status"], "warning");
    assert_eq!(report["summary"]["errors"], 0);
    assert_eq!(report["summary"]["warnings"], 3);
    assert_eq!(report["summary"]["waived"], 2);
    assert!(
        report["drc"]
            .as_array()
            .unwrap()
            .iter()
            .all(|entry| entry["waived"] == true)
    );

    let _ = std::fs::remove_dir_all(&root);
}
