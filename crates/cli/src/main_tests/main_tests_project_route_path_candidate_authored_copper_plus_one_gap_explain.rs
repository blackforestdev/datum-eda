use super::*;
use eda_engine::ir::serialization::to_json_deterministic;

fn unique_project_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{}-{}", label, Uuid::new_v4()))
}

fn route_path_candidate_authored_copper_plus_one_gap_explain_query_cli(
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
        "route-path-candidate-authored-copper-plus-one-gap-explain",
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
fn project_query_route_path_candidate_authored_copper_plus_one_gap_explain_reports_selected_path() {
    let root = unique_project_root(
        "datum-eda-cli-project-route-path-candidate-authored-copper-plus-one-gap-explain",
    );
    create_native_project(
        &root,
        Some("Route Path Candidate Authored Copper Plus One Gap Explain Demo".to_string()),
    )
    .expect("initial scaffold should succeed");

    let target_net_uuid = Uuid::from_u128(0xa300);
    let class_uuid = Uuid::from_u128(0xa301);
    let package_a_uuid = Uuid::from_u128(0xa302);
    let package_b_uuid = Uuid::from_u128(0xa303);
    let anchor_a_uuid = Uuid::from_u128(0xa304);
    let anchor_b_uuid = Uuid::from_u128(0xa305);
    let track_a_uuid = Uuid::from_u128(0xa306);
    let track_b_uuid = Uuid::from_u128(0xa307);
    let board_json = root.join("board/board.json");

    std::fs::write(
        &board_json,
        format!(
            "{}\n",
            to_json_deterministic(&serde_json::json!({
                "schema_version": 1,
                "uuid": Uuid::from_u128(0xa308),
                "name": "Route Path Candidate Authored Copper Plus One Gap Explain Demo Board",
                "stackup": {
                    "layers": [
                        { "id": 1, "name": "Top Copper", "layer_type": "Copper", "thickness_nm": 35000 }
                    ]
                },
                "outline": {
                    "vertices": [
                        { "x": 0, "y": 0 },
                        { "x": 2000000, "y": 0 },
                        { "x": 2000000, "y": 1000000 },
                        { "x": 0, "y": 1000000 }
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
                        "position": { "x": 100000, "y": 500000 },
                        "layer": 1,
                        "shape": "circle",
                        "diameter": 300000,
                        "width": 0,
                        "height": 0
                    },
                    anchor_b_uuid.to_string(): {
                        "uuid": anchor_b_uuid,
                        "package": package_b_uuid,
                        "name": "1",
                        "net": target_net_uuid,
                        "position": { "x": 1900000, "y": 500000 },
                        "layer": 1,
                        "shape": "circle",
                        "diameter": 300000,
                        "width": 0,
                        "height": 0
                    }
                },
                "tracks": {
                    track_a_uuid.to_string(): {
                        "uuid": track_a_uuid,
                        "net": target_net_uuid,
                        "from": { "x": 100000, "y": 500000 },
                        "to": { "x": 700000, "y": 500000 },
                        "width": 150000,
                        "layer": 1
                    },
                    track_b_uuid.to_string(): {
                        "uuid": track_b_uuid,
                        "net": target_net_uuid,
                        "from": { "x": 1300000, "y": 500000 },
                        "to": { "x": 1900000, "y": 500000 },
                        "width": 150000,
                        "layer": 1
                    }
                },
                "vias": {},
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
                        "track_width": 150000,
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

    let output = execute(
        route_path_candidate_authored_copper_plus_one_gap_explain_query_cli(
            &root,
            target_net_uuid,
            anchor_a_uuid,
            anchor_b_uuid,
        ),
    )
    .expect("query should succeed");
    let report: serde_json::Value = serde_json::from_str(&output).expect("report should parse");

    assert_eq!(
        report["contract"],
        "m5_route_path_candidate_authored_copper_plus_one_gap_explain_v1"
    );
    assert_eq!(report["persisted_native_board_state_only"], true);
    assert_eq!(report["status"], "deterministic_path_found");
    assert_eq!(report["explanation_kind"], "deterministic_path_found");
    assert_eq!(report["summary"]["candidate_track_count"], 2);
    assert_eq!(report["summary"]["candidate_gap_count"], 1);
    assert_eq!(report["summary"]["path_gap_step_count"], 1);
    assert_eq!(
        report["selected_path"]["path"]["steps"]
            .as_array()
            .unwrap()
            .len(),
        3
    );
    assert_eq!(
        report["selected_path"]["path"]["steps"][0]["object_uuid"],
        track_a_uuid.to_string()
    );
    assert_eq!(
        report["selected_path"]["path"]["steps"][1]["object_uuid"],
        serde_json::Value::Null
    );
    assert_eq!(
        report["selected_path"]["path"]["steps"][2]["object_uuid"],
        track_b_uuid.to_string()
    );

    let repeated = execute(
        route_path_candidate_authored_copper_plus_one_gap_explain_query_cli(
            &root,
            target_net_uuid,
            anchor_a_uuid,
            anchor_b_uuid,
        ),
    )
    .expect("repeat should succeed");
    assert_eq!(output, repeated);

    let text_output = execute(
        Cli::try_parse_from([
            "eda",
            "project",
            "query",
            root.to_str().unwrap(),
            "route-path-candidate-authored-copper-plus-one-gap-explain",
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
    assert!(
        text_output
            .contains("contract: m5_route_path_candidate_authored_copper_plus_one_gap_explain_v1")
    );
    assert!(text_output.contains("status: deterministic_path_found"));
    assert!(text_output.contains("explanation_kind: deterministic_path_found"));

    let _ = std::fs::remove_dir_all(&root);
}
