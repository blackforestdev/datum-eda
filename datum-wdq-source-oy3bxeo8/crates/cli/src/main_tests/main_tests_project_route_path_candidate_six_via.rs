use super::*;
use eda_engine::ir::serialization::to_json_deterministic;

fn unique_project_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{}-{}", label, Uuid::new_v4()))
}

fn route_path_candidate_six_via_query_cli(
    root: &Path,
    net_uuid: Uuid,
    from_anchor: Uuid,
    to_anchor: Uuid,
) -> Cli {
    Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "query",
        root.to_str().unwrap(),
        "route-path-candidate-six-via",
        "--net",
        &net_uuid.to_string(),
        "--from-anchor",
        &from_anchor.to_string(),
        "--to-anchor",
        &to_anchor.to_string(),
    ])
    .expect("CLI should parse")
}

#[test]
fn project_query_route_path_candidate_six_via_reports_deterministic_authored_six_via_path() {
    let root = unique_project_root("datum-eda-cli-project-route-path-candidate-six-via");
    create_native_project(&root, Some("Route Path Candidate Six Via Demo".to_string()))
        .expect("initial scaffold should succeed");

    let target_net_uuid = Uuid::from_u128(0x1050);
    let class_uuid = Uuid::from_u128(0x1051);
    let package_a_uuid = Uuid::from_u128(0x1052);
    let package_b_uuid = Uuid::from_u128(0x1053);
    let anchor_a_uuid = Uuid::from_u128(0x1054);
    let anchor_b_uuid = Uuid::from_u128(0x1055);
    let via_a_uuid = Uuid::from_u128(0x1056);
    let via_b_uuid = Uuid::from_u128(0x1057);
    let via_c_uuid = Uuid::from_u128(0x1058);
    let via_d_uuid = Uuid::from_u128(0x1059);
    let via_e_uuid = Uuid::from_u128(0x105a);
    let via_f_uuid = Uuid::from_u128(0x105b);
    let board_json = root.join("board/board.json");

    std::fs::write(
        &board_json,
        format!(
            "{}\n",
            to_json_deterministic(&serde_json::json!({
                "schema_version": 1,
                "uuid": Uuid::from_u128(0x105c),
                "name": "Route Path Candidate Six Via Demo Board",
                "stackup": {
                    "layers": [
                        { "id": 1, "name": "Top Copper", "layer_type": "Copper", "thickness_nm": 35000 },
                        { "id": 2, "name": "Core A", "layer_type": "Dielectric", "thickness_nm": 1600000 },
                        { "id": 3, "name": "Inner 1", "layer_type": "Copper", "thickness_nm": 35000 },
                        { "id": 4, "name": "Core B", "layer_type": "Dielectric", "thickness_nm": 1600000 },
                        { "id": 5, "name": "Inner 2", "layer_type": "Copper", "thickness_nm": 35000 },
                        { "id": 6, "name": "Core C", "layer_type": "Dielectric", "thickness_nm": 1600000 },
                        { "id": 7, "name": "Inner 3", "layer_type": "Copper", "thickness_nm": 35000 },
                        { "id": 8, "name": "Core D", "layer_type": "Dielectric", "thickness_nm": 1600000 },
                        { "id": 9, "name": "Inner 4", "layer_type": "Copper", "thickness_nm": 35000 },
                        { "id": 10, "name": "Core E", "layer_type": "Dielectric", "thickness_nm": 1600000 },
                        { "id": 11, "name": "Inner 5", "layer_type": "Copper", "thickness_nm": 35000 },
                        { "id": 12, "name": "Core F", "layer_type": "Dielectric", "thickness_nm": 1600000 },
                        { "id": 13, "name": "Bottom Copper", "layer_type": "Copper", "thickness_nm": 35000 }
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
                "packages": {},
                "pads": {
                    anchor_a_uuid.to_string(): {
                        "uuid": anchor_a_uuid,
                        "package": package_a_uuid,
                        "name": "1",
                        "net": target_net_uuid,
                        "position": { "x": 500000, "y": 600000 },
                        "layer": 1,
                        "shape": "circle",
                        "diameter": 450000,
                        "width": 0,
                        "height": 0
                    },
                    anchor_b_uuid.to_string(): {
                        "uuid": anchor_b_uuid,
                        "package": package_b_uuid,
                        "name": "1",
                        "net": target_net_uuid,
                        "position": { "x": 4500000, "y": 2400000 },
                        "layer": 13,
                        "shape": "circle",
                        "diameter": 450000,
                        "width": 0,
                        "height": 0
                    }
                },
                "tracks": {},
                "vias": {
                    via_a_uuid.to_string(): { "uuid": via_a_uuid, "net": target_net_uuid, "position": { "x": 900000, "y": 760000 }, "drill": 300000, "diameter": 600000, "from_layer": 1, "to_layer": 3 },
                    via_b_uuid.to_string(): { "uuid": via_b_uuid, "net": target_net_uuid, "position": { "x": 1450000, "y": 980000 }, "drill": 300000, "diameter": 600000, "from_layer": 3, "to_layer": 5 },
                    via_c_uuid.to_string(): { "uuid": via_c_uuid, "net": target_net_uuid, "position": { "x": 2050000, "y": 1230000 }, "drill": 300000, "diameter": 600000, "from_layer": 5, "to_layer": 7 },
                    via_d_uuid.to_string(): { "uuid": via_d_uuid, "net": target_net_uuid, "position": { "x": 2700000, "y": 1550000 }, "drill": 300000, "diameter": 600000, "from_layer": 7, "to_layer": 9 },
                    via_e_uuid.to_string(): { "uuid": via_e_uuid, "net": target_net_uuid, "position": { "x": 3350000, "y": 1880000 }, "drill": 300000, "diameter": 600000, "from_layer": 9, "to_layer": 11 },
                    via_f_uuid.to_string(): { "uuid": via_f_uuid, "net": target_net_uuid, "position": { "x": 3950000, "y": 2140000 }, "drill": 300000, "diameter": 600000, "from_layer": 11, "to_layer": 13 }
                },
                "zones": {},
                "nets": {
                    target_net_uuid.to_string(): {
                        "uuid": target_net_uuid,
                        "name": "SIG",
                        "class": class_uuid
                    }
                },
                "net_classes": {
                    class_uuid.to_string(): {
                        "uuid": class_uuid,
                        "name": "Default",
                        "clearance": 150000,
                        "track_width": 200000,
                        "via_drill": 300000,
                        "via_diameter": 600000,
                        "diffpair_width": 0,
                        "diffpair_gap": 0
                    }
                },
                "keepouts": [],
                "dimensions": [],
                "texts": []
            }))
            .expect("canonical serialization should succeed")
        ),
    )
    .expect("board file should write");

    let output = execute(route_path_candidate_six_via_query_cli(
        &root,
        target_net_uuid,
        anchor_a_uuid,
        anchor_b_uuid,
    ))
    .expect("query should succeed");
    let report: serde_json::Value = serde_json::from_str(&output).expect("report should parse");

    assert_eq!(report["contract"], "m5_route_path_candidate_six_via_v1");
    assert_eq!(report["persisted_native_board_state_only"], true);
    assert_eq!(
        report["selection_rule"],
        "select the first unblocked matching authored via sextuple in ascending (via_a_uuid, via_b_uuid, via_c_uuid, via_d_uuid, via_e_uuid, via_f_uuid) order whose layer sequence connects the requested anchor layers through five intermediate copper layers"
    );
    assert_eq!(report["status"], "deterministic_path_found");
    assert_eq!(report["summary"]["candidate_via_count"], 6);
    assert_eq!(report["summary"]["matching_via_sextuple_count"], 1);
    assert_eq!(report["summary"]["available_via_sextuple_count"], 1);
    assert_eq!(report["path"]["via_a_uuid"], via_a_uuid.to_string());
    assert_eq!(report["path"]["via_b_uuid"], via_b_uuid.to_string());
    assert_eq!(report["path"]["via_c_uuid"], via_c_uuid.to_string());
    assert_eq!(report["path"]["via_d_uuid"], via_d_uuid.to_string());
    assert_eq!(report["path"]["via_e_uuid"], via_e_uuid.to_string());
    assert_eq!(report["path"]["via_f_uuid"], via_f_uuid.to_string());
    assert_eq!(report["path"]["first_intermediate_layer"], 3);
    assert_eq!(report["path"]["second_intermediate_layer"], 5);
    assert_eq!(report["path"]["third_intermediate_layer"], 7);
    assert_eq!(report["path"]["fourth_intermediate_layer"], 9);
    assert_eq!(report["path"]["fifth_intermediate_layer"], 11);
    assert_eq!(report["path"]["segments"].as_array().unwrap().len(), 7);

    let repeated = execute(route_path_candidate_six_via_query_cli(
        &root,
        target_net_uuid,
        anchor_a_uuid,
        anchor_b_uuid,
    ))
    .expect("repeat should succeed");
    assert_eq!(output, repeated);

    let text_output = execute(
        Cli::try_parse_from([
            "eda",
            "project",
            "query",
            root.to_str().unwrap(),
            "route-path-candidate-six-via",
            "--net",
            &target_net_uuid.to_string(),
            "--from-anchor",
            &anchor_a_uuid.to_string(),
            "--to-anchor",
            &anchor_b_uuid.to_string(),
        ])
        .expect("CLI should parse"),
    )
    .expect("text query should succeed");
    assert!(text_output.contains("contract: m5_route_path_candidate_six_via_v1"));
    assert!(text_output.contains("status: deterministic_path_found"));
    assert!(text_output.contains("path_via_a_uuid:"));
    assert!(text_output.contains("path_via_b_uuid:"));
    assert!(text_output.contains("path_via_c_uuid:"));
    assert!(text_output.contains("path_via_d_uuid:"));
    assert!(text_output.contains("path_via_e_uuid:"));
    assert!(text_output.contains("path_via_f_uuid:"));
    assert!(text_output.contains("path_segments: 7"));

    let _ = std::fs::remove_dir_all(&root);
}
