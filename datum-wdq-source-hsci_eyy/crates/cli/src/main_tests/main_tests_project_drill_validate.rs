use super::*;
use eda_engine::ir::serialization::to_json_deterministic;

fn unique_project_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{}-{}", label, Uuid::new_v4()))
}

fn write_drill_validation_board(root: &Path) {
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
                "name": "Drill Validate Demo Board",
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
                    via_b_uuid.to_string(): {
                        "uuid": via_b_uuid,
                        "net": net_uuid,
                        "position": { "x": 2000, "y": 3000 },
                        "drill": 350000,
                        "diameter": 700000,
                        "from_layer": 31,
                        "to_layer": 1
                    },
                    via_a_uuid.to_string(): {
                        "uuid": via_a_uuid,
                        "net": net_uuid,
                        "position": { "x": 1000, "y": 1500 },
                        "drill": 300000,
                        "diameter": 600000,
                        "from_layer": 1,
                        "to_layer": 31
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
}

#[test]
fn project_validate_drill_reports_match_and_mismatch() {
    let root = unique_project_root("datum-eda-cli-project-drill-validate");
    create_native_project(&root, Some("Drill Validate Demo".to_string()))
        .expect("initial scaffold should succeed");
    write_drill_validation_board(&root);

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

    let (output, exit_code) = execute_with_exit_code(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "validate-drill",
            root.to_str().unwrap(),
            "--drill",
            drill_path.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("validation should run");
    let report: serde_json::Value = serde_json::from_str(&output).expect("report JSON");
    assert_eq!(exit_code, 0);
    assert_eq!(report["action"], "validate_drill");
    assert_eq!(report["matches_expected"], true);
    assert_eq!(report["rows"], 2);

    std::fs::write(
        &drill_path,
        "via_uuid,net_uuid,x_nm,y_nm,drill_nm,diameter_nm,from_layer,to_layer\nbad\n",
    )
    .expect("drill file should rewrite");

    let (output, exit_code) = execute_with_exit_code(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "validate-drill",
            root.to_str().unwrap(),
            "--drill",
            drill_path.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("validation should run");
    let report: serde_json::Value = serde_json::from_str(&output).expect("report JSON");
    assert_eq!(exit_code, 1);
    assert_eq!(report["matches_expected"], false);

    let _ = std::fs::remove_dir_all(&root);
}
