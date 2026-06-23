use super::*;
use eda_engine::ir::serialization::to_json_deterministic;

fn unique_project_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{label}-{}", Uuid::new_v4()))
}

#[test]
fn project_compare_gerber_set_reports_missing_mismatched_and_extra_files() {
    let root = unique_project_root("datum-eda-cli-project-gerber-set-compare");
    create_native_project(&root, Some("Gerber Set Compare Demo".to_string()))
        .expect("initial scaffold should succeed");
    let component_uuid = Uuid::new_v4();

    let board_json = root.join("board/board.json");
    std::fs::write(
        &board_json,
        format!(
            "{}\n",
            to_json_deterministic(&serde_json::json!({
                "schema_version": 1,
                "uuid": Uuid::new_v4(),
                "name": "Gerber Set Compare Demo Board",
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
                "packages": {
                    component_uuid.to_string(): {
                        "uuid": component_uuid,
                        "part": Uuid::new_v4(),
                        "package": Uuid::new_v4(),
                        "reference": "U1",
                        "value": "DRV",
                        "position": { "x": 3000000, "y": 4000000 },
                        "rotation": 90,
                        "layer": 3,
                        "locked": false
                    }
                },
                "component_silkscreen": {
                    component_uuid.to_string(): [{
                        "from": { "x": 0, "y": 0 },
                        "to": { "x": 1000000, "y": 0 },
                        "width_nm": 150000,
                        "layer": 3
                    }]
                },
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
            .expect("canonical serialization should succeed")
        ),
    )
    .expect("board file should write");

    let output_dir = root.join("gerbers");
    let export_cli = Cli::try_parse_from([
        "eda",
        "project",
        "export-gerber-set",
        root.to_str().unwrap(),
        "--output-dir",
        output_dir.to_str().unwrap(),
        "--prefix",
        "Release A",
    ])
    .expect("CLI should parse");
    execute(export_cli).expect("gerber set export should succeed");

    std::fs::remove_file(output_dir.join("release-a-l4-top-paste-paste.gbr"))
        .expect("paste gerber should remove");
    std::fs::write(
        output_dir.join("release-a-l3-top-silk-silk.gbr"),
        "G04 drifted*\nM02*\n",
    )
    .expect("silk gerber should rewrite");
    std::fs::write(output_dir.join("extra.gbr"), "G04 extra*\nM02*\n").expect("extra should write");

    let compare_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "compare-gerber-set",
        root.to_str().unwrap(),
        "--output-dir",
        output_dir.to_str().unwrap(),
        "--prefix",
        "Release A",
    ])
    .expect("CLI should parse");
    let (output, exit_code) = execute_with_exit_code(compare_cli).expect("comparison should run");
    let report: serde_json::Value = serde_json::from_str(&output).expect("report JSON");
    assert_eq!(exit_code, 1);
    assert_eq!(report["action"], "compare_gerber_set");
    assert_eq!(report["missing_count"], 1);
    assert_eq!(report["mismatched_count"], 1);
    assert_eq!(report["extra_count"], 1);
    assert_eq!(report["matched_count"], 4);
}
