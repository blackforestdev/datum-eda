use super::*;
use eda_engine::ir::serialization::to_json_deterministic;

fn unique_project_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{}-{}", label, Uuid::new_v4()))
}

#[test]
fn project_run_output_job_uses_manufacturing_plan_panel_for_pnp() {
    let root = unique_project_root("datum-eda-cli-project-panel-pnp-output-job");
    create_native_project(&root, Some("Panel PnP Output Job Demo".to_string()))
        .expect("initial scaffold should succeed");

    let package_id = Uuid::new_v4();
    let package_footprint = Uuid::new_v4();
    let part_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    let pad_id = Uuid::new_v4();
    let padstack_id = Uuid::new_v4();
    let via_id = Uuid::new_v4();
    let net_id = Uuid::new_v4();
    let net_class_id = Uuid::new_v4();
    std::fs::write(
        root.join("board/board.json"),
        format!(
            "{}\n",
            to_json_deterministic(&serde_json::json!({
                "schema_version": 1,
                "uuid": board_id,
                "name": "Panel PnP Board",
                "stackup": {
                    "layers": [
                        { "id": 1, "name": "Top Copper", "layer_type": "Copper", "thickness_nm": 35000 },
                        { "id": 2, "name": "Top Mask", "layer_type": "SolderMask", "thickness_nm": 10000 },
                        { "id": 3, "name": "Top Silk", "layer_type": "Silkscreen", "thickness_nm": 10000 },
                        { "id": 4, "name": "Top Paste", "layer_type": "Paste", "thickness_nm": 10000 },
                        { "id": 31, "name": "Bottom Copper", "layer_type": "Copper", "thickness_nm": 35000 },
                        { "id": 41, "name": "Mechanical 41", "layer_type": "Mechanical", "thickness_nm": 0 }
                    ]
                },
                "outline": {
                    "vertices": [
                        { "x": 0, "y": 0 },
                        { "x": 5000000, "y": 0 },
                        { "x": 5000000, "y": 3000000 },
                        { "x": 0, "y": 3000000 }
                    ],
                    "closed": true
                },
                "packages": {
                    package_id.to_string(): {
                        "uuid": package_id,
                        "reference": "U1",
                        "value": "MCU",
                        "package": package_footprint,
                        "part": part_id,
                        "position": { "x": 1000000, "y": 2000000 },
                        "rotation": 90,
                        "layer": 1,
                        "locked": false
                    }
                },
                "component_pads": {
                    package_id.to_string(): [{
                        "uuid": pad_id,
                        "name": "P1",
                        "position": { "x": 1000000, "y": 1500000 },
                        "padstack": padstack_id,
                        "layer": 1,
                        "drill_nm": 300000,
                        "shape": "circle",
                        "diameter_nm": 600000,
                        "width_nm": 0,
                        "height_nm": 0
                    }]
                },
                "component_silkscreen": {}, "component_silkscreen_texts": {},
                "component_silkscreen_arcs": {}, "component_silkscreen_circles": {},
                "component_silkscreen_polygons": {}, "component_silkscreen_polylines": {},
                "component_mechanical_lines": {}, "component_mechanical_texts": {},
                "component_mechanical_polygons": {}, "component_mechanical_polylines": {},
                "component_mechanical_circles": {}, "component_mechanical_arcs": {},
                "component_models_3d": {}, "pads": {}, "tracks": {},
                "vias": {
                    via_id.to_string(): {
                        "uuid": via_id,
                        "net": net_id,
                        "position": { "x": 500000, "y": 600000 },
                        "diameter": 700000,
                        "drill": 300000,
                        "from_layer": 1,
                        "to_layer": 31
                    }
                },
                "zones": {},
                "nets": {
                    net_id.to_string(): { "uuid": net_id, "name": "N$1", "class": net_class_id }
                },
                "net_classes": {
                    net_class_id.to_string(): {
                        "uuid": net_class_id,
                        "name": "Default",
                        "clearance": 150000,
                        "track_width": 200000,
                        "via_drill": 300000,
                        "via_diameter": 700000,
                        "diffpair_width": 0,
                        "diffpair_gap": 0
                    }
                },
                "rules": [],
                "keepouts": [],
                "dimensions": [],
                "texts": []
            }))
            .expect("canonical serialization should succeed")
        ),
    )
    .expect("board file should write");

    let panel_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "create-panel-projection",
            root.to_str().unwrap(),
            "--key",
            "assembly-panel",
            "--name",
            "Assembly panel",
            "--x-nm",
            "7000000",
            "--y-nm",
            "11000000",
        ])
        .expect("CLI should parse"),
    )
    .expect("panel projection create should succeed");
    let panel_report: serde_json::Value =
        serde_json::from_str(&panel_output).expect("panel projection JSON");
    let panel_projection = panel_report["panel_projection"]["id"].as_str().unwrap();

    let plan_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "create-manufacturing-plan",
            root.to_str().unwrap(),
            "--prefix",
            "Panel Release",
            "--panel-projection",
            panel_projection,
        ])
        .expect("CLI should parse"),
    )
    .expect("manufacturing plan create should succeed");
    let plan_report: serde_json::Value =
        serde_json::from_str(&plan_output).expect("manufacturing plan JSON");
    let manufacturing_plan = plan_report["manufacturing_plan"]["id"].as_str().unwrap();
    let output_dir = root.join("panel-output-job");

    let output_job_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "create-output-job",
            root.to_str().unwrap(),
            "--prefix",
            "Panel Release",
            "--include",
            "manufacturing-set",
            "--output-dir",
            output_dir.to_str().unwrap(),
            "--manufacturing-plan",
            manufacturing_plan,
        ])
        .expect("CLI should parse"),
    )
    .expect("output job create should succeed");
    let output_job_report: serde_json::Value =
        serde_json::from_str(&output_job_output).expect("output job JSON");
    let output_job = output_job_report["output_job"]["id"].as_str().unwrap();

    let run_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "run-output-job",
            root.to_str().unwrap(),
            "--output-job",
            output_job,
        ])
        .expect("CLI should parse"),
    )
    .expect("output job run should succeed");
    let run_report: serde_json::Value =
        serde_json::from_str(&run_output).expect("output job run JSON");
    assert_eq!(run_report["status"], "succeeded");
    let manufacturing_report = &run_report["artifact_report"]["generated"][0]["report"];
    assert_eq!(manufacturing_report["pnp_row_count"], 1);
    assert_eq!(manufacturing_report["drill_csv_row_count"], 1);
    assert_eq!(manufacturing_report["excellon_hit_count"], 2);
    let production_projections =
        manufacturing_report["artifact_metadata"]["production_projections"]
            .as_array()
            .unwrap();
    assert!(
        production_projections
            .iter()
            .any(|projection| projection["projection_kind"] == "panel_pnp"
                && projection["projection_contract"] == "datum.production_projection.panel_pnp.v1")
    );
    assert!(
        production_projections
            .iter()
            .any(
                |projection| projection["projection_kind"] == "panel_drill_csv"
                    && projection["projection_contract"]
                        == "datum.production_projection.panel_drill_csv.v1"
            )
    );
    assert!(
        production_projections
            .iter()
            .any(
                |projection| projection["projection_kind"] == "panel_excellon_drill"
                    && projection["projection_contract"]
                        == "datum.production_projection.panel_excellon_drill.v1"
            )
    );
    assert!(
        production_projections
            .iter()
            .any(
                |projection| projection["projection_kind"] == "panel_gerber_copper_layer"
                    && projection["projection_contract"]
                        == "datum.production_projection.panel_gerber_copper_layer.v1"
            )
    );

    let pnp = std::fs::read_to_string(output_dir.join("panel-release-pnp.csv"))
        .expect("panel PnP should read");
    assert!(pnp.contains(&format!(
        ",U1,8000000,13000000,90,1,top,{package_footprint},{part_id},MCU,false"
    )));
    let drill = std::fs::read_to_string(output_dir.join("panel-release-drill.csv"))
        .expect("panel drill CSV should read");
    assert!(drill.contains(&format!(
        "{via_id},{net_id},7500000,11600000,300000,700000,1,31"
    )));
    let excellon = std::fs::read_to_string(output_dir.join("panel-release-drill.drl"))
        .expect("panel Excellon should read");
    assert!(excellon.contains("X7.500000Y11.600000"));
    assert!(excellon.contains("X8.000000Y12.500000"));
    let outline = std::fs::read_to_string(output_dir.join("panel-release-outline.gbr"))
        .expect("panel outline Gerber should read");
    assert!(outline.contains("X7000000Y11000000D02*"));
    let copper = std::fs::read_to_string(output_dir.join("panel-release-l1-top-copper-copper.gbr"))
        .expect("panel copper Gerber should read");
    assert!(copper.contains("X7500000Y11600000D03*"));
    assert!(copper.contains("X8000000Y12500000D03*"));

    let validate_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "validate-manufacturing-set",
            root.to_str().unwrap(),
            "--output-dir",
            output_dir.to_str().unwrap(),
            "--output-job",
            output_job,
        ])
        .expect("CLI should parse"),
    )
    .expect("panel manufacturing validation should succeed");
    let validate_report: serde_json::Value =
        serde_json::from_str(&validate_output).expect("validation JSON");
    assert_eq!(validate_report["mismatched_count"], 0);
    assert_eq!(validate_report["artifact_validation_state"], "valid");

    let compare_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "compare-manufacturing-set",
            root.to_str().unwrap(),
            "--output-dir",
            output_dir.to_str().unwrap(),
            "--output-job",
            output_job,
        ])
        .expect("CLI should parse"),
    )
    .expect("panel manufacturing comparison should succeed");
    let compare_report: serde_json::Value =
        serde_json::from_str(&compare_output).expect("comparison JSON");
    assert_eq!(compare_report["mismatched_count"], 0);

    execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "update-panel-projection",
            root.to_str().unwrap(),
            "--panel-projection",
            panel_projection,
            "--rotation-deg",
            "90",
        ])
        .expect("CLI should parse"),
    )
    .expect("panel rotation update should succeed");
    let (rotated_run_output, rotated_exit_code) = execute_with_exit_code(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "run-output-job",
            root.to_str().unwrap(),
            "--output-job",
            output_job,
        ])
        .expect("CLI should parse"),
    )
    .expect("rotated panel output job should return failed JSON");
    assert_eq!(rotated_exit_code, 1);
    let rotated_report: serde_json::Value =
        serde_json::from_str(&rotated_run_output).expect("rotated run JSON");
    assert_eq!(rotated_report["status"], "failed");
    assert!(
        rotated_report["error"]
            .as_str()
            .unwrap()
            .contains("translation-only")
    );

    let _ = std::fs::remove_dir_all(&root);
}
