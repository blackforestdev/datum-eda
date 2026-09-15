use super::*;

fn unique_project_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{}-{}", label, Uuid::new_v4()))
}

#[test]
fn project_inspect_bom_reports_rows_from_csv() {
    let root = unique_project_root("datum-eda-cli-project-bom-inspect");
    std::fs::create_dir_all(&root).expect("root should exist");

    let bom_path = root.join("bom.csv");
    std::fs::write(
        &bom_path,
        "reference,value,part_uuid,package_uuid,layer,x_nm,y_nm,rotation_deg,locked\n\
         R1,10k,11111111-1111-1111-1111-111111111111,aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa,1,1000,2000,90,false\n\
         C1,\"1uF, X7R\",22222222-2222-2222-2222-222222222222,bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb,31,3000,4000,180,true\n",
    )
    .expect("bom should write");

    let output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "inspect-bom",
            bom_path.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("inspect should succeed");
    let report: serde_json::Value = serde_json::from_str(&output).expect("report JSON");
    assert_eq!(report["action"], "inspect_bom");
    assert_eq!(report["row_count"], 2);
    assert_eq!(report["rows"][0]["reference"], "R1");
    assert_eq!(report["rows"][0]["value"], "10k");
    assert_eq!(report["rows"][1]["reference"], "C1");
    assert_eq!(report["rows"][1]["value"], "1uF, X7R");
    assert_eq!(report["rows"][1]["locked"], true);

    let _ = std::fs::remove_dir_all(&root);
}
