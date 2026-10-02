use super::*;
use eda_engine::substrate::ZONE_FILL_SCHEMA_VERSION;

#[test]
fn project_query_zone_fills_reports_resolver_derived_unfilled_state() {
    let root = unique_project_root("datum-eda-cli-project-zone-fills");
    create_native_project(&root, Some("Zone Fill Query Demo".to_string()))
        .expect("initial scaffold should succeed");
    let zone_uuid = place_zone_fixture(&root);

    let fills = zone_fills_query(&root);
    assert_eq!(fills["contract"], "zone_fills_query_v1");
    assert_eq!(fills["zone_fill_count"], 1);
    assert!(fills["model_revision"].as_str().is_some());
    let fills = fills["zone_fills"]
        .as_array()
        .expect("zone-fills should contain an array");
    assert_eq!(fills.len(), 1);
    assert_eq!(fills[0]["schema_version"], ZONE_FILL_SCHEMA_VERSION);
    assert_eq!(fills[0]["zone_id"], zone_uuid);
    assert_eq!(fills[0]["state"], "unfilled");
    assert_eq!(fills[0]["source_zone_revision"], 0);
    assert!(fills[0]["islands"].as_array().unwrap().is_empty());
    assert_eq!(fills[0]["provenance"], serde_json::Value::Null);

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_fill_zones_persists_filled_generated_evidence_for_safe_simple_zone() {
    let root = unique_project_root("datum-eda-cli-project-fill-zones-simple");
    create_native_project(&root, Some("Simple Fill Zones Demo".to_string()))
        .expect("initial scaffold should succeed");
    let zone_uuid = place_zone_fixture_with_thermal(&root, false);
    let zone_id = zone_uuid.to_string();

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
    assert_eq!(report["contract"], "zone_fill_generate_v1");
    assert_eq!(report["action"], "fill_zones");
    assert_eq!(report["zone_fill_count"], 1);
    assert_eq!(
        report["zone_fills"][0]["schema_version"],
        ZONE_FILL_SCHEMA_VERSION
    );
    assert_eq!(report["zone_fills"][0]["zone_id"], zone_uuid);
    assert_eq!(report["zone_fills"][0]["state"], "filled");
    assert_eq!(
        report["zone_fills"][0]["islands"].as_array().unwrap().len(),
        1
    );
    assert!(
        report["zone_fill_paths"][0]
            .as_str()
            .unwrap()
            .ends_with(&format!(".datum/zone_fills/{zone_uuid}.json"))
    );

    let fills = zone_fills_query(&root);
    assert_eq!(
        fills["zone_fills"][0]["schema_version"],
        ZONE_FILL_SCHEMA_VERSION
    );
    assert_eq!(fills["zone_fills"][0]["zone_id"], zone_uuid);
    assert_eq!(fills["zone_fills"][0]["state"], "filled");
    assert_eq!(
        fills["zone_fills"][0]["provenance"],
        "datum-eda fill-zones: bounded same-net polygon island fill v1; no clearance subtraction required"
    );
    let journal = journal_list(&root);
    assert_eq!(
        journal["transactions"].as_array().unwrap().last().unwrap()["reason"],
        "fill zones"
    );
    let transaction_id =
        journal["transactions"].as_array().unwrap().last().unwrap()["transaction_id"]
            .as_str()
            .unwrap();
    let show_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "journal",
            "show",
            root.to_str().unwrap(),
            "--transaction",
            transaction_id,
        ])
        .expect("CLI should parse"),
    )
    .expect("journal show should succeed");
    let shown: serde_json::Value =
        serde_json::from_str(&show_output).expect("journal show JSON should parse");
    assert_eq!(
        shown["transaction"]["operations"][0]["kind"],
        "set_zone_fill"
    );

    let check = check_run_query(&root);
    assert!(
        !check["findings"]
            .as_array()
            .unwrap()
            .iter()
            .any(|finding| finding["code"]
                .as_str()
                .unwrap_or("")
                .starts_with("zone_fill_"))
    );

    execute(
        Cli::try_parse_from(["eda", "project", "undo", root.to_str().unwrap()])
            .expect("CLI should parse"),
    )
    .expect("undo fill zones should succeed");
    let fills = zone_fills_query(&root);
    assert_eq!(fills["zone_fills"][0]["zone_id"], zone_uuid);
    assert_eq!(fills["zone_fills"][0]["state"], "unfilled");

    execute(
        Cli::try_parse_from(["eda", "project", "redo", root.to_str().unwrap()])
            .expect("CLI should parse"),
    )
    .expect("redo fill zones should succeed");
    let fills = zone_fills_query(&root);
    assert_eq!(fills["zone_fills"][0]["zone_id"], zone_uuid);
    assert_eq!(fills["zone_fills"][0]["state"], "filled");

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_fill_zones_undo_restores_prior_fill_when_promoted_shard_is_missing() {
    let root = unique_project_root("datum-eda-cli-project-fill-zones-missing-promoted");
    create_native_project(&root, Some("Missing Promoted Fill Demo".to_string()))
        .expect("initial scaffold should succeed");
    let zone_uuid = place_zone_fixture_with_thermal(&root, false);
    let zone_fill_path = root.join(format!(".datum/zone_fills/{zone_uuid}.json"));

    execute(
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
    .expect("initial fill-zones should succeed");
    assert!(zone_fill_path.exists());
    std::fs::remove_file(&zone_fill_path).expect("promoted zone fill should be removable");

    execute(
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
    .expect("refill should succeed from journal-materialized prior fill");

    execute(
        Cli::try_parse_from(["eda", "project", "undo", root.to_str().unwrap()])
            .expect("CLI should parse"),
    )
    .expect("undo refill should succeed");
    let fills = zone_fills_query(&root);
    assert_eq!(fills["zone_fills"][0]["zone_id"], zone_uuid);
    assert_eq!(fills["zone_fills"][0]["state"], "filled");

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_fill_zones_undo_restores_stale_prior_generated_evidence() {
    let root = unique_project_root("datum-eda-cli-project-fill-zones-stale-prior");
    create_native_project(&root, Some("Stale Prior Fill Demo".to_string()))
        .expect("initial scaffold should succeed");
    let zone_uuid = place_zone_fixture_with_thermal(&root, false);

    execute(
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
    .expect("initial fill-zones should succeed");

    execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "edit-board-zone",
            root.to_str().unwrap(),
            "--zone",
            zone_uuid.as_str(),
            "--priority",
            "7",
        ])
        .expect("CLI should parse"),
    )
    .expect("zone edit should make existing fill stale");
    let fills = zone_fills_query(&root);
    assert_eq!(fills["zone_fills"][0]["state"], "stale");

    execute(
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
    .expect("refill should succeed");
    let fills = zone_fills_query(&root);
    assert_eq!(fills["zone_fills"][0]["state"], "filled");

    execute(
        Cli::try_parse_from(["eda", "project", "undo", root.to_str().unwrap()])
            .expect("CLI should parse"),
    )
    .expect("undo refill should succeed");
    let fills = zone_fills_query(&root);
    assert_eq!(fills["zone_fills"][0]["zone_id"], zone_uuid);
    assert_eq!(fills["zone_fills"][0]["state"], "stale");

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_fill_zones_allows_thermal_relief_zone_without_same_net_anchors() {
    let root = unique_project_root("datum-eda-cli-project-fill-zones-thermal");
    create_native_project(&root, Some("Thermal Fill Zones Demo".to_string()))
        .expect("initial scaffold should succeed");
    let zone_uuid = place_zone_fixture(&root);
    let zone_id = zone_uuid.to_string();

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
    assert!(
        !report["zone_fills"][0]["islands"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    assert_eq!(
        report["zone_fills"][0]["provenance"],
        "datum-eda fill-zones: bounded same-net polygon island fill v1; no clearance subtraction required; thermal relief requested but no same-net pad/via anchors intersected the bounded fill"
    );

    let check = check_run_query(&root);
    let has_zone_fill_finding =
        check["findings"].as_array().unwrap().iter().any(|entry| {
            entry["source"] == "zone_fill" && entry["payload"]["zone_id"] == zone_uuid
        });
    assert!(!has_zone_fill_finding);

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_fill_zones_rejects_thermal_relief_zone_with_same_net_pad_anchor() {
    let root = unique_project_root("datum-eda-cli-project-fill-zones-thermal-anchor");
    create_native_project(&root, Some("Thermal Anchor Fill Zones Demo".to_string()))
        .expect("initial scaffold should succeed");
    let zone_uuid = place_zone_fixture(&root);
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
            "place-board-pad",
            root.to_str().unwrap(),
            "--package",
            &Uuid::new_v4().to_string(),
            "--name",
            "1",
            "--x-nm",
            "500",
            "--y-nm",
            "500",
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
            zone_uuid.as_str(),
        ])
        .expect("CLI should parse"),
    )
    .expect("fill-zones should succeed");
    let report: serde_json::Value = serde_json::from_str(&output).expect("fill-zones JSON");
    assert_eq!(report["zone_fills"][0]["state"], "unsupported");
    assert!(
        report["zone_fills"][0]["islands"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    assert_eq!(
        report["zone_fills"][0]["provenance"],
        "datum-eda fill-zones: unsupported because thermal relief generation for same-net pad/via anchors is not implemented"
    );

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn check_fill_zones_alias_persists_unsupported_generated_evidence() {
    let root = unique_project_root("datum-eda-cli-check-fill-zones");
    create_native_project(&root, Some("Check Fill Zones Demo".to_string()))
        .expect("initial scaffold should succeed");
    let zone_uuid = place_zone_fixture(&root);
    let zone_id = zone_uuid.to_string();
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
            "place-board-pad",
            root.to_str().unwrap(),
            "--package",
            &Uuid::new_v4().to_string(),
            "--name",
            "1",
            "--x-nm",
            "500",
            "--y-nm",
            "500",
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
            "check",
            "fill-zones",
            root.to_str().unwrap(),
            "--zone",
            zone_id.as_str(),
        ])
        .expect("CLI should parse"),
    )
    .expect("check fill-zones should succeed");
    let report: serde_json::Value = serde_json::from_str(&output).expect("fill-zones JSON");
    assert_eq!(report["contract"], "zone_fill_generate_v1");
    assert_eq!(
        report["zone_fills"][0]["schema_version"],
        ZONE_FILL_SCHEMA_VERSION
    );
    assert_eq!(report["zone_fills"][0]["zone_id"], zone_uuid);
    assert_eq!(report["zone_fills"][0]["state"], "unsupported");

    let fills = zone_fills_query(&root);
    assert_eq!(
        fills["zone_fills"][0]["schema_version"],
        ZONE_FILL_SCHEMA_VERSION
    );
    assert_eq!(fills["zone_fills"][0]["state"], "unsupported");

    let _ = std::fs::remove_dir_all(&root);
}
