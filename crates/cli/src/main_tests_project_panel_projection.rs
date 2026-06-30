use super::*;
use eda_engine::ir::serialization::to_json_deterministic;
use eda_engine::substrate::ProjectResolver;

fn unique_project_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{}-{}", label, Uuid::new_v4()))
}

fn write_variant_fixture(root: &Path, variant_id: Uuid, name: &str) {
    let model = ProjectResolver::new(root)
        .resolve()
        .expect("project should resolve before variant fixture");
    let variant_path = root.join(".datum/variants/panel-test.json");
    std::fs::create_dir_all(variant_path.parent().unwrap()).expect("variant dir should create");
    std::fs::write(
        &variant_path,
        format!(
            "{}\n",
            to_json_deterministic(&serde_json::json!({
                "schema_version": 1,
                "variants": [{
                    "id": variant_id,
                    "name": name,
                    "base_model_revision": model.model_revision.0,
                    "variant_revision": 0,
                    "fitted": {},
                    "relationship_overrides": {},
                    "property_overrides": {}
                }]
            }))
            .expect("variant fixture should serialize")
        ),
    )
    .expect("variant fixture should write");
}

#[test]
fn project_panel_projection_round_trips_through_manufacturing_plan() {
    let root = unique_project_root("datum-eda-cli-project-panel-projection");
    create_native_project(&root, Some("Panel Projection Demo".to_string()))
        .expect("initial scaffold should succeed");

    let create_panel_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "create-panel-projection",
        root.to_str().unwrap(),
        "--key",
        "release-a-panel",
        "--name",
        "Release A panel",
    ])
    .expect("CLI should parse");
    let create_panel_output =
        execute(create_panel_cli).expect("panel projection create should succeed");
    let create_panel_report: serde_json::Value =
        serde_json::from_str(&create_panel_output).expect("panel projection create JSON");
    assert_eq!(
        create_panel_report["contract"],
        "panel_projection_mutation_v1"
    );
    assert_eq!(create_panel_report["action"], "create_panel_projection");
    assert_eq!(create_panel_report["created"], true);
    assert_eq!(
        create_panel_report["panel_projection"]["name"],
        "Release A panel"
    );
    assert_eq!(
        create_panel_report["panel_projection"]["board_instances"][0]["x_nm"],
        0
    );
    assert_eq!(
        create_panel_report["panel_projection"]["board_instances"][0]["rotation_deg"],
        0
    );
    let panel_projection = create_panel_report["panel_projection"]["id"]
        .as_str()
        .expect("panel projection id should serialize")
        .to_string();

    let create_plan_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "create-manufacturing-plan",
        root.to_str().unwrap(),
        "--prefix",
        "release-a",
        "--panel-projection",
        panel_projection.as_str(),
    ])
    .expect("CLI should parse");
    let create_plan_output =
        execute(create_plan_cli).expect("manufacturing plan create should succeed");
    let create_plan_report: serde_json::Value =
        serde_json::from_str(&create_plan_output).expect("manufacturing plan create JSON");
    assert_eq!(
        create_plan_report["manufacturing_plan"]["board_or_panel"],
        serde_json::json!(panel_projection)
    );
    let manufacturing_plan = create_plan_report["manufacturing_plan"]["id"]
        .as_str()
        .expect("manufacturing plan id should serialize")
        .to_string();

    let panel_query_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "query",
        root.to_str().unwrap(),
        "panel-projections",
    ])
    .expect("CLI should parse");
    let panel_query_output =
        execute(panel_query_cli).expect("panel-projections query should succeed");
    let panel_query_report: serde_json::Value =
        serde_json::from_str(&panel_query_output).expect("panel-projections JSON");
    assert_eq!(panel_query_report["contract"], "panel_projection_list_v1");
    assert_eq!(panel_query_report["panel_projection_count"], 1);
    assert_eq!(
        panel_query_report["panel_projections"][0]["id"],
        serde_json::json!(panel_projection)
    );

    let update_panel_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "update-panel-projection",
        root.to_str().unwrap(),
        "--panel-projection",
        panel_projection.as_str(),
        "--name",
        "Release A panel updated",
        "--x-nm",
        "1000",
        "--y-nm",
        "2000",
        "--rotation-deg",
        "90",
    ])
    .expect("CLI should parse");
    let update_panel_output = execute(update_panel_cli).expect("panel update should succeed");
    let update_panel_report: serde_json::Value =
        serde_json::from_str(&update_panel_output).expect("panel update JSON");
    assert_eq!(update_panel_report["action"], "update_panel_projection");
    assert_eq!(
        update_panel_report["panel_projection"]["name"],
        "Release A panel updated"
    );
    assert_eq!(
        update_panel_report["panel_projection"]["board_instances"][0]["x_nm"],
        1000
    );
    assert_eq!(
        update_panel_report["panel_projection"]["object_revision"],
        serde_json::json!(1)
    );

    let undo_panel_update_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "undo",
        root.to_str().unwrap(),
    ])
    .expect("CLI should parse");
    execute(undo_panel_update_cli).expect("panel update undo should succeed");
    let panel_after_undo: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(
            root.join(".datum/panel_projections")
                .join(format!("{panel_projection}.json")),
        )
        .expect("panel shard should read after undo"),
    )
    .expect("panel shard JSON after undo");
    assert_eq!(panel_after_undo["name"], "Release A panel");

    let redo_panel_update_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "redo",
        root.to_str().unwrap(),
    ])
    .expect("CLI should parse");
    execute(redo_panel_update_cli).expect("panel update redo should succeed");
    let panel_after_redo: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(
            root.join(".datum/panel_projections")
                .join(format!("{panel_projection}.json")),
        )
        .expect("panel shard should read after redo"),
    )
    .expect("panel shard JSON after redo");
    assert_eq!(panel_after_redo["name"], "Release A panel updated");
    assert_eq!(panel_after_redo["board_instances"][0]["rotation_deg"], 90);

    let variant_id = Uuid::new_v4();
    write_variant_fixture(&root, variant_id, "Release A variant");
    let variant = variant_id.to_string();
    let update_plan_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "update-manufacturing-plan",
        root.to_str().unwrap(),
        "--manufacturing-plan",
        manufacturing_plan.as_str(),
        "--name",
        "Release A fab updated",
        "--prefix",
        "release-a-updated",
        "--variant",
        variant.as_str(),
    ])
    .expect("CLI should parse");
    let update_plan_output =
        execute(update_plan_cli).expect("manufacturing plan update should succeed");
    let update_plan_report: serde_json::Value =
        serde_json::from_str(&update_plan_output).expect("manufacturing plan update JSON");
    assert_eq!(update_plan_report["action"], "update_manufacturing_plan");
    assert_eq!(
        update_plan_report["manufacturing_plan"]["prefix"],
        "release-a-updated"
    );
    assert_eq!(
        update_plan_report["manufacturing_plan"]["variant"],
        serde_json::json!(variant)
    );
    assert_eq!(
        update_plan_report["manufacturing_plan"]["object_revision"],
        serde_json::json!(1)
    );

    let undo_plan_update_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "undo",
        root.to_str().unwrap(),
    ])
    .expect("CLI should parse");
    execute(undo_plan_update_cli).expect("manufacturing plan update undo should succeed");
    let plan_after_undo: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(
            root.join(".datum/manufacturing_plans")
                .join(format!("{manufacturing_plan}.json")),
        )
        .expect("manufacturing plan shard should read after undo"),
    )
    .expect("manufacturing plan shard JSON after undo");
    assert_eq!(plan_after_undo["prefix"], "release-a");
    assert_eq!(plan_after_undo["variant"], serde_json::Value::Null);

    let redo_plan_update_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "redo",
        root.to_str().unwrap(),
    ])
    .expect("CLI should parse");
    execute(redo_plan_update_cli).expect("manufacturing plan update redo should succeed");

    let blocked_delete_panel_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "delete-panel-projection",
        root.to_str().unwrap(),
        "--panel-projection",
        panel_projection.as_str(),
    ])
    .expect("CLI should parse");
    let blocked_delete_panel = execute(blocked_delete_panel_cli)
        .expect_err("panel delete should reject a referenced panel projection");
    assert!(
        blocked_delete_panel
            .to_string()
            .contains("manufacturing plan")
    );

    let delete_plan_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "delete-manufacturing-plan",
        root.to_str().unwrap(),
        "--manufacturing-plan",
        manufacturing_plan.as_str(),
    ])
    .expect("CLI should parse");
    let delete_plan_output =
        execute(delete_plan_cli).expect("manufacturing plan delete should succeed");
    let delete_plan_report: serde_json::Value =
        serde_json::from_str(&delete_plan_output).expect("manufacturing plan delete JSON");
    assert_eq!(delete_plan_report["action"], "delete_manufacturing_plan");
    assert_eq!(delete_plan_report["created"], false);
    assert!(
        !root
            .join(".datum/manufacturing_plans")
            .join(format!("{manufacturing_plan}.json"))
            .exists()
    );

    let undo_plan_delete_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "undo",
        root.to_str().unwrap(),
    ])
    .expect("CLI should parse");
    execute(undo_plan_delete_cli).expect("manufacturing plan delete undo should succeed");
    assert!(
        root.join(".datum/manufacturing_plans")
            .join(format!("{manufacturing_plan}.json"))
            .exists()
    );

    let redo_plan_delete_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "redo",
        root.to_str().unwrap(),
    ])
    .expect("CLI should parse");
    execute(redo_plan_delete_cli).expect("manufacturing plan delete redo should succeed");
    assert!(
        !root
            .join(".datum/manufacturing_plans")
            .join(format!("{manufacturing_plan}.json"))
            .exists()
    );

    let delete_panel_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "delete-panel-projection",
        root.to_str().unwrap(),
        "--panel-projection",
        panel_projection.as_str(),
    ])
    .expect("CLI should parse");
    let delete_panel_output = execute(delete_panel_cli).expect("panel delete should succeed");
    let delete_panel_report: serde_json::Value =
        serde_json::from_str(&delete_panel_output).expect("panel delete JSON");
    assert_eq!(delete_panel_report["action"], "delete_panel_projection");
    assert_eq!(delete_panel_report["created"], false);
    assert!(
        !root
            .join(".datum/panel_projections")
            .join(format!("{panel_projection}.json"))
            .exists()
    );

    let undo_panel_delete_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "undo",
        root.to_str().unwrap(),
    ])
    .expect("CLI should parse");
    execute(undo_panel_delete_cli).expect("panel delete undo should succeed");
    assert!(
        root.join(".datum/panel_projections")
            .join(format!("{panel_projection}.json"))
            .exists()
    );

    let _ = std::fs::remove_dir_all(&root);
}
