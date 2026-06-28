use super::*;
use eda_engine::board::Zone;
use eda_engine::ir::serialization::to_json_deterministic;

pub(super) fn unique_project_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{}-{}", label, Uuid::new_v4()))
}

pub(super) fn board_zones_query_cli(root: &Path) -> Cli {
    Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "query",
        root.to_str().unwrap(),
        "board-zones",
    ])
    .expect("CLI should parse")
}

pub(super) fn zone_fills_query(root: &Path) -> serde_json::Value {
    let output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "query",
            root.to_str().unwrap(),
            "zone-fills",
        ])
        .expect("CLI should parse"),
    )
    .expect("zone-fills query should succeed");
    serde_json::from_str(&output).expect("zone-fills JSON should parse")
}

pub(super) fn check_run_query(root: &Path) -> serde_json::Value {
    let output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "query",
            root.to_str().unwrap(),
            "check-run",
        ])
        .expect("CLI should parse"),
    )
    .expect("check-run should succeed");
    serde_json::from_str(&output).expect("check-run JSON should parse")
}

fn journal_list(root: &Path) -> serde_json::Value {
    let output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "query",
            root.to_str().unwrap(),
            "journal-list",
        ])
        .expect("CLI should parse"),
    )
    .expect("journal-list should succeed");
    serde_json::from_str(&output).expect("journal-list JSON should parse")
}

pub(super) fn place_zone_fixture(root: &Path) -> String {
    place_zone_fixture_with_thermal(root, true)
}

pub(super) fn create_board_net_fixture(root: &Path, name: &str) -> String {
    let class_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "place-board-net-class",
            root.to_str().unwrap(),
            "--name",
            &format!("{name}Class"),
            "--clearance-nm",
            "150000",
            "--track-width-nm",
            "200000",
            "--via-drill-nm",
            "300000",
            "--via-diameter-nm",
            "600000",
        ])
        .expect("CLI should parse"),
    )
    .expect("place board net class should succeed");
    let class_report: serde_json::Value =
        serde_json::from_str(&class_output).expect("class output should parse");
    let net_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "place-board-net",
            root.to_str().unwrap(),
            "--name",
            name,
            "--class",
            class_report["net_class_uuid"].as_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("place board net should succeed");
    let net_report: serde_json::Value =
        serde_json::from_str(&net_output).expect("net output should parse");
    net_report["net_uuid"].as_str().unwrap().to_string()
}

pub(super) fn place_zone_fixture_with_thermal(root: &Path, thermal_relief: bool) -> String {
    let net_uuid = create_board_net_fixture(root, "GND");

    let place_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "place-board-zone",
            root.to_str().unwrap(),
            "--net",
            &net_uuid,
            "--vertex",
            "0:0",
            "--vertex",
            "1000:0",
            "--vertex",
            "1000:1000",
            "--layer",
            "1",
            "--priority",
            "2",
            "--thermal-relief",
            if thermal_relief { "true" } else { "false" },
            "--thermal-gap-nm",
            if thermal_relief { "250000" } else { "0" },
            "--thermal-spoke-width-nm",
            if thermal_relief { "200000" } else { "0" },
        ])
        .expect("CLI should parse"),
    )
    .expect("place board zone should succeed");
    let placed: serde_json::Value =
        serde_json::from_str(&place_output).expect("place output should parse");
    placed["zone_uuid"].as_str().unwrap().to_string()
}

pub(super) fn place_rectangular_zone_fixture(root: &Path) -> String {
    let net_uuid = create_board_net_fixture(root, "GND");

    let place_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "place-board-zone",
            root.to_str().unwrap(),
            "--net",
            &net_uuid,
            "--vertex",
            "0:0",
            "--vertex",
            "1000000:0",
            "--vertex",
            "1000000:1000000",
            "--vertex",
            "0:1000000",
            "--layer",
            "1",
            "--priority",
            "2",
            "--thermal-relief",
            "false",
            "--thermal-gap-nm",
            "0",
            "--thermal-spoke-width-nm",
            "0",
        ])
        .expect("CLI should parse"),
    )
    .expect("place rectangular board zone should succeed");
    let placed: serde_json::Value =
        serde_json::from_str(&place_output).expect("place output should parse");
    placed["zone_uuid"].as_str().unwrap().to_string()
}

#[test]
fn project_board_zone_mutations_round_trip_through_native_query() {
    let root = unique_project_root("datum-eda-cli-project-board-zone");
    create_native_project(&root, Some("Board Zone Demo".to_string()))
        .expect("initial scaffold should succeed");

    let class_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "place-board-net-class",
        root.to_str().unwrap(),
        "--name",
        "Default",
        "--clearance-nm",
        "150000",
        "--track-width-nm",
        "200000",
        "--via-drill-nm",
        "300000",
        "--via-diameter-nm",
        "600000",
    ])
    .expect("CLI should parse");
    let class_output = execute(class_cli).expect("place board net class should succeed");
    let class_report: serde_json::Value =
        serde_json::from_str(&class_output).expect("class output should parse");

    let net_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "place-board-net",
        root.to_str().unwrap(),
        "--name",
        "GND",
        "--class",
        class_report["net_class_uuid"].as_str().unwrap(),
    ])
    .expect("CLI should parse");
    let net_output = execute(net_cli).expect("place board net should succeed");
    let net_report: serde_json::Value =
        serde_json::from_str(&net_output).expect("net output should parse");
    let net_uuid = net_report["net_uuid"].as_str().unwrap().to_string();

    let place_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "place-board-zone",
        root.to_str().unwrap(),
        "--net",
        &net_uuid,
        "--vertex",
        "0:0",
        "--vertex",
        "1000:0",
        "--vertex",
        "1000:1000",
        "--layer",
        "1",
        "--priority",
        "2",
        "--thermal-relief",
        "true",
        "--thermal-gap-nm",
        "250000",
        "--thermal-spoke-width-nm",
        "200000",
    ])
    .expect("CLI should parse");
    let place_output = execute(place_cli).expect("place board zone should succeed");
    let placed: serde_json::Value =
        serde_json::from_str(&place_output).expect("place output should parse");
    let zone_uuid = placed["zone_uuid"].as_str().unwrap().to_string();

    let zones_output =
        execute(board_zones_query_cli(&root)).expect("board zones query should succeed");
    let zones: Vec<Zone> = serde_json::from_str(&zones_output).expect("query output should parse");
    assert_eq!(zones.len(), 1);
    assert_eq!(zones[0].uuid.to_string(), zone_uuid);
    assert_eq!(zones[0].net.to_string(), net_uuid);
    assert_eq!(zones[0].layer, 1);
    assert_eq!(zones[0].priority, 2);
    assert!(zones[0].thermal_relief);
    assert_eq!(zones[0].thermal_gap, 250000);
    assert_eq!(zones[0].thermal_spoke_width, 200000);
    assert_eq!(zones[0].polygon.vertices.len(), 3);
    let journal = journal_list(&root);
    assert_eq!(
        journal["transactions"].as_array().unwrap().last().unwrap()["reason"],
        "place board zone"
    );

    let edit_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "edit-board-zone",
        root.to_str().unwrap(),
        "--zone",
        &zone_uuid,
        "--vertex",
        "0:0",
        "--vertex",
        "2000:0",
        "--vertex",
        "2000:1000",
        "--vertex",
        "0:1000",
        "--layer",
        "2",
        "--priority",
        "5",
        "--thermal-relief",
        "false",
        "--thermal-gap-nm",
        "0",
        "--thermal-spoke-width-nm",
        "0",
    ])
    .expect("CLI should parse");
    let edit_output = execute(edit_cli).expect("edit board zone should succeed");
    let edited: serde_json::Value =
        serde_json::from_str(&edit_output).expect("edit output should parse");
    assert_eq!(edited["action"], "edit_board_zone");
    assert_eq!(edited["zone_uuid"], zone_uuid);

    let zones_output =
        execute(board_zones_query_cli(&root)).expect("board zones query should succeed");
    let zones: Vec<Zone> = serde_json::from_str(&zones_output).expect("query output should parse");
    assert_eq!(zones.len(), 1);
    assert_eq!(zones[0].uuid.to_string(), zone_uuid);
    assert_eq!(zones[0].net.to_string(), net_uuid);
    assert_eq!(zones[0].layer, 2);
    assert_eq!(zones[0].priority, 5);
    assert!(!zones[0].thermal_relief);
    assert_eq!(zones[0].thermal_gap, 0);
    assert_eq!(zones[0].thermal_spoke_width, 0);
    assert_eq!(zones[0].polygon.vertices.len(), 4);
    let journal = journal_list(&root);
    assert_eq!(
        journal["transactions"].as_array().unwrap().last().unwrap()["reason"],
        "edit board zone"
    );

    let delete_cli = Cli::try_parse_from([
        "eda",
        "project",
        "delete-board-zone",
        root.to_str().unwrap(),
        "--zone",
        &zone_uuid,
    ])
    .expect("CLI should parse");
    let delete_output = execute(delete_cli).expect("delete board zone should succeed");
    assert!(delete_output.contains("action: delete_board_zone"));

    let zones_output =
        execute(board_zones_query_cli(&root)).expect("board zones query should succeed");
    let zones: Vec<Zone> = serde_json::from_str(&zones_output).expect("query output should parse");
    assert!(zones.is_empty());
    let journal = journal_list(&root);
    assert_eq!(journal["count"], 5);
    assert_eq!(
        journal["transactions"].as_array().unwrap().last().unwrap()["reason"],
        "delete board zone"
    );

    let summary_cli =
        Cli::try_parse_from(["eda", "project", "query", root.to_str().unwrap(), "summary"])
            .expect("CLI should parse");
    let summary_output = execute(summary_cli).expect("summary query should succeed");
    assert!(summary_output.contains("board_zones: 0"));

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_query_check_run_reports_unfilled_zone_fill_findings() {
    let root = unique_project_root("datum-eda-cli-project-board-zone-check-run");
    create_native_project(&root, Some("Board Zone Check Demo".to_string()))
        .expect("initial scaffold should succeed");
    let zone_uuid = place_zone_fixture(&root);

    let report = check_run_query(&root);
    let finding = report["findings"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["code"] == "zone_fill_unfilled")
        .expect("zone fill finding should exist");
    assert_eq!(finding["source"], "zone_fill");
    assert_eq!(finding["severity"], "error");
    assert_eq!(finding["payload"]["zone_id"], zone_uuid);
    assert_eq!(report["status"], "error");

    assert_eq!(report, check_run_query(&root));

    let _ = std::fs::remove_dir_all(&root);
}

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
    assert_eq!(fills[0]["schema_version"], 1);
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
    assert_eq!(report["zone_fills"][0]["schema_version"], 1);
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
    assert_eq!(fills["zone_fills"][0]["schema_version"], 1);
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
fn project_query_check_run_reports_stale_zone_fill_findings() {
    let root = unique_project_root("datum-eda-cli-project-fill-zones-stale");
    create_native_project(&root, Some("Stale Fill Zones Demo".to_string()))
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
    .expect("fill-zones should succeed");

    let _second_zone_uuid = place_zone_fixture_with_thermal(&root, false);
    let check = check_run_query(&root);
    let finding = check["findings"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| {
            entry["code"] == "zone_fill_stale" && entry["payload"]["zone_id"] == zone_uuid
        })
        .expect("stale zone fill finding should exist");
    assert_eq!(finding["source"], "zone_fill");
    assert_eq!(finding["severity"], "error");
    assert!(
        finding["suggested_next_action"]
            .as_str()
            .unwrap()
            .contains("Regenerate zone fills")
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
    assert_eq!(report["zone_fills"][0]["schema_version"], 1);
    assert_eq!(report["zone_fills"][0]["zone_id"], zone_uuid);
    assert_eq!(report["zone_fills"][0]["state"], "unsupported");

    let fills = zone_fills_query(&root);
    assert_eq!(fills["zone_fills"][0]["schema_version"], 1);
    assert_eq!(fills["zone_fills"][0]["state"], "unsupported");

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_query_board_zones_reads_existing_native_board_file() {
    let root = unique_project_root("datum-eda-cli-project-board-zone-query");
    create_native_project(&root, Some("Board Zone Query Demo".to_string()))
        .expect("initial scaffold should succeed");

    let net_uuid = Uuid::new_v4();
    let class_uuid = Uuid::new_v4();
    let zone_uuid = Uuid::new_v4();
    let board_json = root.join("board/board.json");
    std::fs::write(
        &board_json,
        format!(
            "{}\n",
            to_json_deterministic(&serde_json::json!({
                "schema_version": 1,
                "uuid": Uuid::new_v4(),
                "name": "Board Zone Query Demo Board",
                "stackup": { "layers": [] },
                "outline": { "vertices": [], "closed": true },
                "packages": {},
                "pads": {},
                "tracks": {},
                "vias": {},
                "zones": {
                    zone_uuid.to_string(): {
                        "uuid": zone_uuid,
                        "net": net_uuid,
                        "polygon": {
                            "vertices": [
                                { "x": 0, "y": 0 },
                                { "x": 10, "y": 0 },
                                { "x": 10, "y": 10 }
                            ],
                            "closed": true
                        },
                        "layer": 1,
                        "priority": 3,
                        "thermal_relief": true,
                        "thermal_gap": 250000,
                        "thermal_spoke_width": 200000
                    }
                },
                "nets": {
                    net_uuid.to_string(): {
                        "uuid": net_uuid,
                        "name": "GND",
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
                "rules": [],
                "keepouts": [],
                "dimensions": [],
                "texts": []
            }))
            .expect("canonical serialization should succeed")
        ),
    )
    .expect("board file should write");

    let output = execute(board_zones_query_cli(&root)).expect("board zones query should succeed");
    let zones: Vec<Zone> = serde_json::from_str(&output).expect("query output should parse");
    assert_eq!(zones.len(), 1);
    assert_eq!(zones[0].uuid, zone_uuid);
    assert_eq!(zones[0].net, net_uuid);
    assert_eq!(zones[0].layer, 1);
    assert_eq!(zones[0].priority, 3);
    assert!(zones[0].thermal_relief);
    assert_eq!(zones[0].thermal_gap, 250000);
    assert_eq!(zones[0].thermal_spoke_width, 200000);
    assert_eq!(zones[0].polygon.vertices.len(), 3);

    let _ = std::fs::remove_dir_all(&root);
}
