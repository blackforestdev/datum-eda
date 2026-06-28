use super::main_tests_project_board_zone::{
    board_zones_query_cli, create_board_net_fixture, place_rectangular_zone_fixture,
    place_zone_fixture_with_thermal, unique_project_root, zone_fills_query,
};
use super::*;
use eda_engine::board::Zone;

#[test]
fn project_fill_zones_allows_same_net_copper_without_fake_clearance_subtraction() {
    let root = unique_project_root("datum-eda-cli-project-fill-zones-same-net");
    create_native_project(&root, Some("Same Net Fill Zones Demo".to_string()))
        .expect("initial scaffold should succeed");
    let zone_uuid = place_zone_fixture_with_thermal(&root, false);
    let zone_id = zone_uuid.to_string();
    let fills = zone_fills_query(&root);
    let zone_net = fills["zone_fills"][0]["zone_id"]
        .as_str()
        .expect("zone id should exist");
    assert_eq!(zone_net, zone_id);
    let zones_output =
        execute(board_zones_query_cli(&root)).expect("board zones query should succeed");
    let zones: Vec<Zone> = serde_json::from_str(&zones_output).expect("zones should parse");
    let net_uuid = zones[0].net.to_string();

    execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "draw-board-track",
            root.to_str().unwrap(),
            "--net",
            &net_uuid,
            "--from-x-nm",
            "100",
            "--from-y-nm",
            "100",
            "--to-x-nm",
            "900",
            "--to-y-nm",
            "100",
            "--width-nm",
            "100",
            "--layer",
            "1",
        ])
        .expect("CLI should parse"),
    )
    .expect("draw same-net track should succeed");
    execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "place-board-via",
            root.to_str().unwrap(),
            "--net",
            &net_uuid,
            "--x-nm",
            "500",
            "--y-nm",
            "500",
            "--drill-nm",
            "100",
            "--diameter-nm",
            "200",
            "--from-layer",
            "1",
            "--to-layer",
            "2",
        ])
        .expect("CLI should parse"),
    )
    .expect("place same-net via should succeed");
    execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "place-board-pad",
            root.to_str().unwrap(),
            "--package",
            &Uuid::new_v4().to_string(),
            "--name",
            "1",
            "--x-nm",
            "700",
            "--y-nm",
            "700",
            "--layer",
            "1",
            "--diameter-nm",
            "200",
            "--net",
            &net_uuid,
        ])
        .expect("CLI should parse"),
    )
    .expect("place same-net pad should succeed");

    let output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "fill-zones",
            root.to_str().unwrap(),
            "--zone",
            zone_id.as_str(),
        ])
        .expect("CLI should parse"),
    )
    .expect("fill-zones should succeed");
    let report: serde_json::Value = serde_json::from_str(&output).expect("fill-zones JSON");
    assert_eq!(report["zone_fills"][0]["state"], "filled");
    assert_eq!(
        report["zone_fills"][0]["provenance"],
        "datum-eda fill-zones: bounded same-net polygon island fill v1; no clearance subtraction required"
    );

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_fill_zones_cuts_out_single_foreign_orthogonal_track() {
    let root = unique_project_root("datum-eda-cli-project-fill-zones-foreign-track-cutout");
    create_native_project(&root, Some("Foreign Track Fill Zones Demo".to_string()))
        .expect("initial scaffold should succeed");
    let zone_uuid = place_rectangular_zone_fixture(&root);
    let foreign_net_uuid = create_board_net_fixture(&root, "VCC");

    execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "draw-board-track",
            root.to_str().unwrap(),
            "--net",
            &foreign_net_uuid,
            "--from-x-nm",
            "300000",
            "--from-y-nm",
            "500000",
            "--to-x-nm",
            "700000",
            "--to-y-nm",
            "500000",
            "--width-nm",
            "100000",
            "--layer",
            "1",
        ])
        .expect("CLI should parse"),
    )
    .expect("draw foreign-net track should succeed");

    let output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "fill-zones",
            root.to_str().unwrap(),
            "--zone",
            zone_uuid.as_str(),
        ])
        .expect("CLI should parse"),
    )
    .expect("fill-zones should succeed");
    let report: serde_json::Value = serde_json::from_str(&output).expect("fill-zones JSON");
    assert_eq!(report["zone_fills"][0]["state"], "filled");
    assert_eq!(
        report["zone_fills"][0]["islands"].as_array().unwrap().len(),
        4
    );
    assert_eq!(
        report["zone_fills"][0]["provenance"],
        "datum-eda fill-zones: bounded rectangular obstacle cutout fill v2; one foreign pad/via/orthogonal track inflated by netclass clearance"
    );

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_fill_zones_clips_edge_crossing_foreign_track_clearance_to_zone_bounds() {
    let root = unique_project_root("datum-eda-cli-project-fill-zones-edge-track-cutout");
    create_native_project(&root, Some("Edge Track Fill Zones Demo".to_string()))
        .expect("initial scaffold should succeed");
    let zone_uuid = place_rectangular_zone_fixture(&root);
    let foreign_net_uuid = create_board_net_fixture(&root, "VCC");

    execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "draw-board-track",
            root.to_str().unwrap(),
            "--net",
            &foreign_net_uuid,
            "--from-x-nm",
            "0",
            "--from-y-nm",
            "300000",
            "--to-x-nm",
            "0",
            "--to-y-nm",
            "700000",
            "--width-nm",
            "100000",
            "--layer",
            "1",
        ])
        .expect("CLI should parse"),
    )
    .expect("draw edge-crossing foreign-net track should succeed");

    let output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "fill-zones",
            root.to_str().unwrap(),
            "--zone",
            zone_uuid.as_str(),
        ])
        .expect("CLI should parse"),
    )
    .expect("fill-zones should succeed");
    let report: serde_json::Value = serde_json::from_str(&output).expect("fill-zones JSON");
    assert_eq!(report["zone_fills"][0]["state"], "filled");
    assert_eq!(
        report["zone_fills"][0]["islands"].as_array().unwrap().len(),
        3
    );
    assert_eq!(
        report["zone_fills"][0]["provenance"],
        "datum-eda fill-zones: bounded rectangular obstacle cutout fill v5; edge-crossing foreign pad/via/orthogonal-track clearances clipped to zone bounds before fill"
    );

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_fill_zones_cuts_out_board_keepout_bounds() {
    let root = unique_project_root("datum-eda-cli-project-fill-zones-keepout-cutout");
    create_native_project(&root, Some("Keepout Fill Zones Demo".to_string()))
        .expect("initial scaffold should succeed");
    let zone_uuid = place_rectangular_zone_fixture(&root);

    execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "place-board-keepout",
            root.to_str().unwrap(),
            "--kind",
            "copper",
            "--layer",
            "1",
            "--vertex",
            "400000:400000",
            "--vertex",
            "600000:400000",
            "--vertex",
            "600000:600000",
            "--vertex",
            "400000:600000",
        ])
        .expect("CLI should parse"),
    )
    .expect("place board keepout should succeed");

    let output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "fill-zones",
            root.to_str().unwrap(),
            "--zone",
            zone_uuid.as_str(),
        ])
        .expect("CLI should parse"),
    )
    .expect("fill-zones should succeed");
    let report: serde_json::Value = serde_json::from_str(&output).expect("fill-zones JSON");
    assert_eq!(report["zone_fills"][0]["state"], "filled");
    assert_eq!(
        report["zone_fills"][0]["islands"].as_array().unwrap().len(),
        4
    );
    assert_eq!(
        report["zone_fills"][0]["provenance"],
        "datum-eda fill-zones: bounded rectangular obstacle cutout fill v6; keepout bounds removed from copper before fill"
    );

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_fill_zones_cuts_out_multiple_non_overlapping_foreign_pads() {
    let root = unique_project_root("datum-eda-cli-project-fill-zones-multi-pad-cutout");
    create_native_project(&root, Some("Foreign Pad Fill Zones Demo".to_string()))
        .expect("initial scaffold should succeed");
    let zone_uuid = place_rectangular_zone_fixture(&root);
    let foreign_net_uuid = create_board_net_fixture(&root, "VCC");

    for (name, x_nm, y_nm) in [("1", "250000", "250000"), ("2", "750000", "750000")] {
        execute(
            Cli::try_parse_from([
                "eda",
                "--format",
                "json",
                "project",
                "place-board-pad",
                root.to_str().unwrap(),
                "--package",
                &Uuid::new_v4().to_string(),
                "--name",
                name,
                "--x-nm",
                x_nm,
                "--y-nm",
                y_nm,
                "--layer",
                "1",
                "--diameter-nm",
                "100000",
                "--net",
                &foreign_net_uuid,
            ])
            .expect("CLI should parse"),
        )
        .expect("place foreign-net pad should succeed");
    }

    let output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "fill-zones",
            root.to_str().unwrap(),
            "--zone",
            zone_uuid.as_str(),
        ])
        .expect("CLI should parse"),
    )
    .expect("fill-zones should succeed");
    let report: serde_json::Value = serde_json::from_str(&output).expect("fill-zones JSON");
    assert_eq!(report["zone_fills"][0]["state"], "filled");
    assert_eq!(
        report["zone_fills"][0]["islands"].as_array().unwrap().len(),
        23
    );
    assert_eq!(
        report["zone_fills"][0]["provenance"],
        "datum-eda fill-zones: bounded rectangular obstacle cutout fill v3; multiple non-overlapping foreign pads/vias/orthogonal tracks inflated by netclass clearance"
    );

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_fill_zones_conservatively_unions_overlapping_foreign_pad_clearances() {
    let root = unique_project_root("datum-eda-cli-project-fill-zones-overlap-pad-cutout");
    create_native_project(
        &root,
        Some("Overlapping Foreign Pad Fill Zones Demo".to_string()),
    )
    .expect("initial scaffold should succeed");
    let zone_uuid = place_rectangular_zone_fixture(&root);
    let foreign_net_uuid = create_board_net_fixture(&root, "VCC");

    for (name, x_nm, y_nm) in [("1", "450000", "500000"), ("2", "550000", "500000")] {
        execute(
            Cli::try_parse_from([
                "eda",
                "--format",
                "json",
                "project",
                "place-board-pad",
                root.to_str().unwrap(),
                "--package",
                &Uuid::new_v4().to_string(),
                "--name",
                name,
                "--x-nm",
                x_nm,
                "--y-nm",
                y_nm,
                "--layer",
                "1",
                "--diameter-nm",
                "100000",
                "--net",
                &foreign_net_uuid,
            ])
            .expect("CLI should parse"),
        )
        .expect("place foreign-net pad should succeed");
    }

    let output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "fill-zones",
            root.to_str().unwrap(),
            "--zone",
            zone_uuid.as_str(),
        ])
        .expect("CLI should parse"),
    )
    .expect("fill-zones should succeed");
    let report: serde_json::Value = serde_json::from_str(&output).expect("fill-zones JSON");
    assert_eq!(report["zone_fills"][0]["state"], "filled");
    assert_eq!(
        report["zone_fills"][0]["islands"].as_array().unwrap().len(),
        8
    );
    assert_eq!(
        report["zone_fills"][0]["provenance"],
        "datum-eda fill-zones: bounded rectangular obstacle cutout fill v4; overlapping foreign pad/via/orthogonal-track clearances conservatively unioned before fill"
    );

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_fill_zones_conservatively_cuts_out_non_orthogonal_track_bounds() {
    let root = unique_project_root("datum-eda-cli-project-fill-zones-foreign-net");
    create_native_project(&root, Some("Foreign Net Fill Zones Demo".to_string()))
        .expect("initial scaffold should succeed");
    let zone_uuid = place_rectangular_zone_fixture(&root);
    let foreign_net_uuid = create_board_net_fixture(&root, "VCC");

    execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "draw-board-track",
            root.to_str().unwrap(),
            "--net",
            &foreign_net_uuid,
            "--from-x-nm",
            "300000",
            "--from-y-nm",
            "300000",
            "--to-x-nm",
            "700000",
            "--to-y-nm",
            "700000",
            "--width-nm",
            "100000",
            "--layer",
            "1",
        ])
        .expect("CLI should parse"),
    )
    .expect("draw foreign-net track should succeed");

    let output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "fill-zones",
            root.to_str().unwrap(),
            "--zone",
            zone_uuid.as_str(),
        ])
        .expect("CLI should parse"),
    )
    .expect("fill-zones should succeed");
    let report: serde_json::Value = serde_json::from_str(&output).expect("fill-zones JSON");
    assert_eq!(report["zone_fills"][0]["state"], "filled");
    assert_eq!(
        report["zone_fills"][0]["islands"].as_array().unwrap().len(),
        4
    );
    assert_eq!(
        report["zone_fills"][0]["provenance"],
        "datum-eda fill-zones: bounded rectangular obstacle cutout fill v7; non-orthogonal foreign track bounds conservatively removed before fill"
    );

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_fill_zones_ignores_unresolved_component_pads_outside_zone() {
    let root = unique_project_root("datum-eda-cli-project-fill-zones-far-component-pad");
    create_native_project(&root, Some("Far Component Pad Fill Zones Demo".to_string()))
        .expect("initial scaffold should succeed");
    seed_unresolved_component_pad(&root, 2_000_000, 2_000_000);
    let zone_uuid = place_rectangular_zone_fixture(&root);

    let output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "fill-zones",
            root.to_str().unwrap(),
            "--zone",
            zone_uuid.as_str(),
        ])
        .expect("CLI should parse"),
    )
    .expect("fill-zones should succeed");
    let report: serde_json::Value = serde_json::from_str(&output).expect("fill-zones JSON");
    assert_eq!(report["zone_fills"][0]["state"], "filled");
    assert_eq!(
        report["zone_fills"][0]["provenance"],
        "datum-eda fill-zones: bounded same-net polygon island fill v1; no clearance subtraction required"
    );

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_fill_zones_rejects_unresolved_component_pad_intersecting_zone() {
    let root = unique_project_root("datum-eda-cli-project-fill-zones-near-component-pad");
    create_native_project(
        &root,
        Some("Near Component Pad Fill Zones Demo".to_string()),
    )
    .expect("initial scaffold should succeed");
    seed_unresolved_component_pad(&root, 500_000, 500_000);
    let zone_uuid = place_rectangular_zone_fixture(&root);

    let output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "fill-zones",
            root.to_str().unwrap(),
            "--zone",
            zone_uuid.as_str(),
        ])
        .expect("CLI should parse"),
    )
    .expect("fill-zones should succeed");
    let report: serde_json::Value = serde_json::from_str(&output).expect("fill-zones JSON");
    assert_eq!(report["zone_fills"][0]["state"], "unsupported");
    assert_eq!(
        report["zone_fills"][0]["provenance"],
        "datum-eda fill-zones: unsupported because an unresolved pad intersects the zone"
    );

    let _ = std::fs::remove_dir_all(&root);
}

fn seed_unresolved_component_pad(root: &std::path::Path, x_nm: i64, y_nm: i64) {
    let component_uuid = Uuid::parse_str("aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa").unwrap();
    let pad_uuid = Uuid::parse_str("bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb").unwrap();
    let padstack_uuid = Uuid::parse_str("cccccccc-cccc-4ccc-8ccc-cccccccccccc").unwrap();
    let board_json = root.join("board/board.json");
    let mut board: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(&board_json).expect("board file should read"),
    )
    .expect("board JSON should parse");
    board["component_pads"] = serde_json::json!({
        component_uuid.to_string(): [{
            "uuid": pad_uuid,
            "name": "1",
            "position": { "x": x_nm, "y": y_nm },
            "padstack": padstack_uuid,
            "layer": 1,
            "shape": "circle",
            "diameter_nm": 100000,
            "width_nm": 100000,
            "height_nm": 100000
        }]
    });
    std::fs::write(
        &board_json,
        format!(
            "{}\n",
            eda_engine::ir::serialization::to_json_deterministic(&board)
                .expect("canonical serialization should succeed")
        ),
    )
    .expect("board file should write");
}
