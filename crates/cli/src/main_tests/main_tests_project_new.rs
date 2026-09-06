use super::*;
use eda_engine::board::{StackupLayer, StackupLayerType};
use eda_engine::ir::serialization::to_json_deterministic;

fn unique_project_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{}-{}", label, Uuid::new_v4()))
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

#[test]
fn project_new_creates_native_scaffold() {
    let root = unique_project_root("datum-eda-cli-project-new");

    let cli = Cli::try_parse_from([
        "eda",
        "project",
        "new",
        root.to_str().unwrap(),
        "--name",
        "Native Demo",
        "--units-source",
        "factory",
    ])
    .expect("CLI should parse");

    let output = execute(cli).expect("project new should succeed");
    assert!(output.contains("project_id:"));
    assert!(output.contains("units_source: Factory"));
    assert!(output.contains("units_seed_items: 8"));

    let project_json = root.join("project.json");
    let schematic_json = root.join("schematic/schematic.json");
    let board_json = root.join("board/board.json");
    let rules_json = root.join("rules/rules.json");
    assert!(project_json.exists());
    assert!(schematic_json.exists());
    assert!(board_json.exists());
    assert!(rules_json.exists());

    let project_value: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(&project_json).expect("project.json should read"),
    )
    .expect("project.json should parse");
    assert_eq!(project_value["schema_version"], 1);
    assert_eq!(project_value["name"], "Native Demo");
    assert_eq!(project_value["schematic"], "schematic/schematic.json");
    assert_eq!(project_value["board"], "board/board.json");
    assert_eq!(project_value["rules"], "rules/rules.json");
    assert_eq!(project_value["project_display_units"]["system"], "metric");
    assert_eq!(
        project_value["project_display_units"]["board_length"],
        "follow_system"
    );
    assert_eq!(
        project_value["project_units_seed_receipt"]["schema_name"],
        "datum.project.units_seed_receipt"
    );
    assert_eq!(
        project_value["project_units_seed_receipt"]["source"]["kind"],
        "factory"
    );
    assert_eq!(
        project_value["project_units_seed_receipt"]["source"]["profile_id"],
        "datum.units.factory.v1"
    );
    assert_eq!(
        project_value["project_units_seed_receipt"]["items"]
            .as_array()
            .unwrap()
            .len(),
        8
    );
    let rules_value: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(&rules_json).expect("rules.json should read"),
    )
    .expect("rules.json should parse");
    assert!(Uuid::parse_str(rules_value["uuid"].as_str().unwrap()).is_ok());
    assert_eq!(rules_value["object_revision"], 0);
    assert_eq!(rules_value["rules"].as_array().unwrap().len(), 0);

    let board_value: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(&board_json).expect("board.json should read"),
    )
    .expect("board.json should parse");
    let stackup: Vec<StackupLayer> =
        serde_json::from_value(board_value["stackup"]["layers"].clone())
            .expect("stackup should parse");
    assert_eq!(stackup.len(), 5);
    assert_eq!(stackup[0].id, 1);
    assert_eq!(stackup[0].layer_type, StackupLayerType::Copper);
    assert_eq!(stackup[1].id, 2);
    assert_eq!(stackup[1].layer_type, StackupLayerType::SolderMask);
    assert_eq!(stackup[2].id, 3);
    assert_eq!(stackup[2].layer_type, StackupLayerType::Silkscreen);
    assert_eq!(stackup[3].id, 4);
    assert_eq!(stackup[3].layer_type, StackupLayerType::Paste);
    assert_eq!(stackup[4].id, 41);
    assert_eq!(stackup[4].layer_type, StackupLayerType::Mechanical);

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_new_defaults_visibly_to_global_units_mode() {
    let cli = Cli::try_parse_from(["eda", "project", "new", "/tmp/datum-default-mode"])
        .expect("CLI should parse");
    let Commands::Project { action } = cli.command else {
        panic!("expected project command")
    };
    let ProjectCommands::New(args) = *action else {
        panic!("expected project new")
    };
    assert!(matches!(args.units_source, ProjectUnitsSourceArg::Global));
    assert!(args.expected_preferences.is_none());
}

#[test]
fn project_new_seeded_stackup_supports_topside_gerber_plan() {
    let root = unique_project_root("datum-eda-cli-project-new-gerber-plan");
    create_native_project(&root, Some("Seeded Plan".to_string()))
        .expect("initial scaffold should succeed");

    let cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "plan-gerber-export",
        root.to_str().unwrap(),
        "--prefix",
        "Seeded",
    ])
    .expect("CLI should parse");
    let output = execute(cli).expect("gerber plan should succeed");
    let report: serde_json::Value = serde_json::from_str(&output).expect("report JSON");

    assert_eq!(report["action"], "plan_gerber_export");
    assert_eq!(report["copper_layers"], 1);
    assert_eq!(report["soldermask_layers"], 1);
    assert_eq!(report["silkscreen_layers"], 1);
    assert_eq!(report["paste_layers"], 1);
    assert_eq!(report["mechanical_layers"], 1);

    let artifacts = report["artifacts"].as_array().expect("artifacts array");
    assert_eq!(artifacts.len(), 6);
    assert_eq!(artifacts[0]["filename"], "seeded-outline.gbr");
    assert_eq!(artifacts[1]["filename"], "seeded-l1-top-copper-copper.gbr");
    assert_eq!(artifacts[2]["filename"], "seeded-l2-top-mask-mask.gbr");
    assert_eq!(artifacts[3]["filename"], "seeded-l3-top-silk-silk.gbr");
    assert_eq!(artifacts[4]["filename"], "seeded-l4-top-paste-paste.gbr");
    assert_eq!(
        artifacts[5]["filename"],
        "seeded-l41-mechanical-41-mech.gbr"
    );

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_new_json_output_reports_created_ids() {
    let root = unique_project_root("datum-eda-cli-project-new-json");

    let cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "new",
        root.to_str().unwrap(),
        "--units-source",
        "factory",
    ])
    .expect("CLI should parse");

    let output = execute(cli).expect("project new should succeed");
    let report: serde_json::Value =
        serde_json::from_str(&output).expect("project new JSON should parse");
    assert_eq!(report["ok"], true);
    assert_eq!(report["schema"]["name"], "datum.project.new");
    assert_eq!(
        report["result"]["project_root_identity"],
        root.display().to_string()
    );
    assert!(report["result"]["project_id"].as_str().is_some());
    assert!(report["result"]["request_id"].as_str().is_some());
    assert!(
        report["result"]["genesis_request_digest"]
            .as_str()
            .unwrap()
            .starts_with("sha256:")
    );
    assert_eq!(
        report["result"]["units_receipt"]["items"]
            .as_array()
            .unwrap()
            .len(),
        8
    );
    assert_eq!(
        report["result"]["published_manifest_digests"]
            .as_object()
            .unwrap()
            .len(),
        4
    );

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_new_exact_request_replays_and_conflicting_retry_exits_two() {
    let root = unique_project_root("datum-eda-cli-project-new-retry");
    let request_id = Uuid::new_v4().to_string();
    let project_id = Uuid::new_v4().to_string();
    let args = || {
        Cli::try_parse_from([
            "eda",
            "project",
            "new",
            root.to_str().unwrap(),
            "--name",
            "Replay Project",
            "--request-id",
            &request_id,
            "--project-id",
            &project_id,
            "--units-source",
            "factory",
            "--json",
        ])
        .unwrap()
    };
    let (first, first_code) = execute_with_exit_code(args()).unwrap();
    let (replay, replay_code) = execute_with_exit_code(args()).unwrap();
    assert_eq!(first_code, 0);
    assert_eq!(replay_code, 0);
    assert_eq!(first, replay);

    let (refusal, code) = execute_with_exit_code(
        Cli::try_parse_from([
            "eda",
            "project",
            "new",
            root.to_str().unwrap(),
            "--name",
            "Changed Project",
            "--request-id",
            &request_id,
            "--project-id",
            &project_id,
            "--units-source",
            "factory",
            "--json",
        ])
        .unwrap(),
    )
    .unwrap();
    assert_eq!(code, 2);
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&refusal).unwrap()["error"]["code"],
        "idempotency_conflict"
    );
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn project_new_refuses_global_expectation_in_factory_mode_without_publication() {
    let root = unique_project_root("datum-eda-cli-project-new-invalid-source");
    let (refusal, code) = execute_with_exit_code(
        Cli::try_parse_from([
            "eda",
            "project",
            "new",
            root.to_str().unwrap(),
            "--units-source",
            "factory",
            "--expected-preferences",
            "{}",
            "--json",
        ])
        .unwrap(),
    )
    .unwrap();
    assert_eq!(code, 2);
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&refusal).unwrap()["error"]["code"],
        "invalid_request"
    );
    assert!(!root.exists());
}

#[test]
fn project_set_project_name_round_trips_through_journal_and_resolver_summary() {
    let root = unique_project_root("datum-eda-cli-project-name");
    create_native_project(&root, Some("Project Name Demo".to_string()))
        .expect("initial scaffold should succeed");
    let project_json = root.join("project.json");
    let stale_project = std::fs::read_to_string(&project_json).expect("project file should read");

    let output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "set-project-name",
            root.to_str().unwrap(),
            "--name",
            "Amplifier Project A",
        ])
        .expect("CLI should parse"),
    )
    .expect("set project name should succeed");
    let report: serde_json::Value = serde_json::from_str(&output).expect("output should parse");
    assert_eq!(report["action"], "set_project_name");
    assert_eq!(report["name"], "Amplifier Project A");

    let journal = journal_list(&root);
    assert_eq!(journal["count"], 1);
    assert_eq!(journal["transactions"][0]["reason"], "set project name");
    assert_eq!(journal["transactions"][0]["created"], 0);
    assert_eq!(journal["transactions"][0]["modified"], 1);
    assert_eq!(journal["transactions"][0]["deleted"], 0);
    assert_eq!(journal["transactions"][0]["operations"], 1);

    std::fs::write(&project_json, stale_project).expect("stale project file should restore");
    let summary_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "query",
            root.to_str().unwrap(),
            "summary",
        ])
        .expect("CLI should parse"),
    )
    .expect("summary query should succeed");
    let summary: serde_json::Value = serde_json::from_str(&summary_output).expect("summary JSON");
    assert_eq!(summary["project_name"], "Amplifier Project A");

    let _undo_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "undo",
            root.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("project undo should succeed");
    let summary_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "query",
            root.to_str().unwrap(),
            "summary",
        ])
        .expect("CLI should parse"),
    )
    .expect("summary query should succeed");
    let summary: serde_json::Value = serde_json::from_str(&summary_output).expect("summary JSON");
    assert_eq!(summary["project_name"], "Project Name Demo");

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_new_is_idempotent_for_existing_scaffold() {
    let root = unique_project_root("datum-eda-cli-project-new-idempotent");

    let first = create_native_project(&root, Some("Repeatable".to_string()))
        .expect("initial scaffold should succeed");
    let project_json = root.join("project.json");
    let schematic_json = root.join("schematic/schematic.json");
    let board_json = root.join("board/board.json");
    let rules_json = root.join("rules/rules.json");
    let before = [
        std::fs::read(&project_json).expect("project.json should read"),
        std::fs::read(&schematic_json).expect("schematic.json should read"),
        std::fs::read(&board_json).expect("board.json should read"),
        std::fs::read(&rules_json).expect("rules.json should read"),
    ];

    let second = create_native_project(&root, Some("Repeatable".to_string()))
        .expect("repeat scaffold should succeed");
    let after = [
        std::fs::read(&project_json).expect("project.json should read"),
        std::fs::read(&schematic_json).expect("schematic.json should read"),
        std::fs::read(&board_json).expect("board.json should read"),
        std::fs::read(&rules_json).expect("rules.json should read"),
    ];

    assert_eq!(before, after);
    assert_eq!(first.project_uuid, second.project_uuid);
    assert_eq!(first.schematic_uuid, second.schematic_uuid);
    assert_eq!(first.board_uuid, second.board_uuid);

    let project_value: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(&project_json).expect("project.json should read"),
    )
    .expect("project.json should parse");
    let canonical =
        to_json_deterministic(&project_value).expect("canonical serialization should succeed");
    assert_eq!(
        std::fs::read_to_string(&project_json).expect("project.json should read"),
        format!("{canonical}\n")
    );

    let _ = std::fs::remove_dir_all(&root);
}
