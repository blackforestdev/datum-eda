use super::*;

fn unique_project_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{}-{}", label, Uuid::new_v4()))
}

#[test]
fn project_inspect_pnp_reports_rows_from_csv() {
    let root = unique_project_root("datum-eda-cli-project-pnp-inspect");
    std::fs::create_dir_all(&root).expect("root should exist");

    let pnp_path = root.join("pnp.csv");
    std::fs::write(
        &pnp_path,
        "reference,x_nm,y_nm,rotation_deg,layer,side,package_uuid,part_uuid,value,locked\n\
         U1,1000,2000,90,1,top,aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa,11111111-1111-1111-1111-111111111111,MCU,false\n\
         J1,3000,4000,180,31,bottom,bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb,22222222-2222-2222-2222-222222222222,Conn,true\n",
    )
    .expect("pnp should write");

    let output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "inspect-pnp",
            pnp_path.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("inspect should succeed");
    let report: serde_json::Value = serde_json::from_str(&output).expect("report JSON");
    assert_eq!(report["action"], "inspect_pnp");
    assert_eq!(report["row_count"], 2);
    assert_eq!(report["rows"][0]["reference"], "U1");
    assert_eq!(report["rows"][0]["side"], "top");
    assert_eq!(report["rows"][1]["reference"], "J1");
    assert_eq!(report["rows"][1]["side"], "bottom");
    assert_eq!(report["rows"][1]["locked"], true);

    let _ = std::fs::remove_dir_all(&root);
}
