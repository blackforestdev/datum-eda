use super::*;
use eda_engine::board::{StackupLayer, StackupLayerType};
use eda_engine::ir::serialization::to_json_deterministic;

fn unique_project_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{}-{}", label, Uuid::new_v4()))
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
    ])
    .expect("CLI should parse");

    let output = execute(cli).expect("project new should succeed");
    assert!(output.contains("project_name: Native Demo"));

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
    ])
    .expect("CLI should parse");

    let output = execute(cli).expect("project new should succeed");
    let report: serde_json::Value =
        serde_json::from_str(&output).expect("project new JSON should parse");
    assert_eq!(report["project_root"], root.display().to_string());
    assert_eq!(
        report["project_name"],
        root.file_name().unwrap().to_string_lossy().to_string()
    );
    assert!(report["project_uuid"].as_str().is_some());
    assert!(report["schematic_uuid"].as_str().is_some());
    assert!(report["board_uuid"].as_str().is_some());
    assert_eq!(report["files_written"].as_array().unwrap().len(), 4);

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
