use super::*;
use eda_engine::ir::serialization::to_json_deterministic;

fn unique_project_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{}-{}", label, Uuid::new_v4()))
}

fn write_drill_compare_board(root: &Path) -> (Uuid, Uuid, Uuid) {
    let board_json = root.join("board/board.json");
    let net_uuid = Uuid::new_v4();
    let via_a_uuid = Uuid::new_v4();
    let via_b_uuid = Uuid::new_v4();
    std::fs::write(
        &board_json,
        format!(
            "{}\n",
            to_json_deterministic(&serde_json::json!({
                "schema_version": 1,
                "uuid": Uuid::new_v4(),
                "name": "Drill Compare Demo Board",
                "stackup": { "layers": [] },
                "outline": { "vertices": [], "closed": true },
                "packages": {},
                "component_pads": {},
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
                "component_models_3d": {},
                "pads": {},
                "tracks": {},
                "vias": {
                    via_a_uuid.to_string(): {
                        "uuid": via_a_uuid,
                        "net": net_uuid,
                        "position": { "x": 1000, "y": 1500 },
                        "drill": 300000,
                        "diameter": 600000,
                        "from_layer": 1,
                        "to_layer": 31
                    },
                    via_b_uuid.to_string(): {
                        "uuid": via_b_uuid,
                        "net": net_uuid,
                        "position": { "x": 2000, "y": 3000 },
                        "drill": 350000,
                        "diameter": 700000,
                        "from_layer": 31,
                        "to_layer": 1
                    }
                },
                "zones": {},
                "nets": {
                    net_uuid.to_string(): {
                        "uuid": net_uuid,
                        "name": "N$1",
                        "class": null
                    }
                },
                "net_classes": {},
                "keepouts": [],
                "dimensions": [],
                "texts": []
            }))
            .expect("canonical serialization should succeed")
        ),
    )
    .expect("board file should write");
    (net_uuid, via_a_uuid, via_b_uuid)
}

#[test]
fn project_compare_drill_reports_missing_extra_and_drift() {
    let root = unique_project_root("datum-eda-cli-project-drill-compare");
    create_native_project(&root, Some("Drill Compare Demo".to_string()))
        .expect("initial scaffold should succeed");
    let (net_uuid, via_a_uuid, via_b_uuid) = write_drill_compare_board(&root);

    let drill_path = root.join("drill.csv");
    execute(
        Cli::try_parse_from([
            "eda",
            "project",
            "export-drill",
            root.to_str().unwrap(),
            "--out",
            drill_path.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("drill export should succeed");

    let output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "compare-drill",
            root.to_str().unwrap(),
            "--drill",
            drill_path.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("comparison should succeed");
    let report: serde_json::Value = serde_json::from_str(&output).expect("report JSON");
    assert_eq!(report["action"], "compare_drill");
    assert_eq!(report["matched_count"], 2);
    assert_eq!(report["missing_count"], 0);
    assert_eq!(report["extra_count"], 0);
    assert_eq!(report["drift_count"], 0);

    let extra_via = Uuid::new_v4();
    std::fs::write(
        &drill_path,
        format!(
            "via_uuid,net_uuid,x_nm,y_nm,drill_nm,diameter_nm,from_layer,to_layer\n{via_a_uuid},{net_uuid},1000,1500,300000,999999,1,31\n{extra_via},{net_uuid},5000,6000,300000,600000,1,31\n"
        ),
    )
    .expect("drill file should rewrite");

    let output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "compare-drill",
            root.to_str().unwrap(),
            "--drill",
            drill_path.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("comparison should succeed");
    let report: serde_json::Value = serde_json::from_str(&output).expect("report JSON");
    assert_eq!(report["matched_count"], 0);
    assert_eq!(report["missing_count"], 1);
    assert_eq!(report["extra_count"], 1);
    assert_eq!(report["drift_count"], 1);
    assert!(
        report["missing"]
            .as_array()
            .unwrap()
            .iter()
            .any(|v| v == &serde_json::Value::String(via_b_uuid.to_string()))
    );
    assert!(
        report["extra"]
            .as_array()
            .unwrap()
            .iter()
            .any(|v| v == &serde_json::Value::String(extra_via.to_string()))
    );
    assert!(
        report["drift"]
            .as_array()
            .unwrap()
            .iter()
            .any(|v| v == &serde_json::Value::String(via_a_uuid.to_string()))
    );

    let _ = std::fs::remove_dir_all(&root);
}
