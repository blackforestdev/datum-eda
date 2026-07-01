use super::main_tests_project_pool_pin_pad_map::*;
use super::*;

#[test]
fn proposal_create_pool_pin_pad_map_defers_until_accept_apply() {
    let root = unique_project_root("datum-eda-cli-proposal-pool-pin-pad-map");
    create_native_project(&root, Some("Pool PinPadMap Proposal".to_string()))
        .expect("initial scaffold should succeed");
    let fixture = create_fixture(&root, &["IN+"], &["1"]);
    let map_id = Uuid::new_v4();
    let proposal_id = Uuid::new_v4();
    let map_path = root.join(format!("pool/pin_pad_maps/{map_id}.json"));

    let output = run_project_command(&[
        "eda",
        "--format",
        "json",
        "proposal",
        "create-pool-pin-pad-map",
        root.to_str().unwrap(),
        "--map",
        &map_id.to_string(),
        "--part",
        &fixture.part_id.to_string(),
        "--footprint",
        &fixture.footprint_id.to_string(),
        "--entry",
        &format!("{}:{}", fixture.pin_ids[0], fixture.pad_ids[0]),
        "--set-default",
        "--proposal",
        &proposal_id.to_string(),
        "--rationale",
        "review pin pad map",
    ])
    .expect("PinPadMap proposal create should succeed");
    let report: serde_json::Value =
        serde_json::from_str(&output).expect("proposal report JSON should parse");
    assert_eq!(report["contract"], "proposal_create_v1");
    assert_eq!(report["action"], "create_pool_pin_pad_map_proposal");
    assert_eq!(report["proposal_id"], proposal_id.to_string());
    assert!(
        !map_path.exists(),
        "proposal creation must not write the PinPadMap shard"
    );
    let part_payload = query_pool_object_payload(&root, "parts", fixture.part_id);
    assert!(
        part_payload
            .get("default_pin_pad_map")
            .is_none_or(serde_json::Value::is_null)
    );

    run_project_command(&[
        "eda",
        "--format",
        "json",
        "proposal",
        "accept-apply",
        root.to_str().unwrap(),
        "--proposal",
        &proposal_id.to_string(),
    ])
    .expect("PinPadMap proposal accept-apply should succeed");

    assert!(map_path.exists());
    let map_payload = query_pool_object_payload(&root, "pin_pad_maps", map_id);
    assert_eq!(map_payload["part"], fixture.part_id.to_string());
    assert_eq!(map_payload["footprint"], fixture.footprint_id.to_string());
    assert_eq!(
        map_payload["mappings"][fixture.pad_ids[0].to_string()]["pin"],
        fixture.pin_ids[0].to_string()
    );
    let part_payload = query_pool_object_payload(&root, "parts", fixture.part_id);
    assert_eq!(part_payload["default_pin_pad_map"], map_id.to_string());
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn proposal_set_pool_pin_pad_map_defers_until_accept_apply() {
    let root = unique_project_root("datum-eda-cli-proposal-set-pool-pin-pad-map");
    create_native_project(&root, Some("Set Pool PinPadMap Proposal".to_string()))
        .expect("initial scaffold should succeed");
    let fixture = create_fixture(&root, &["A", "B"], &["1", "2"]);
    let map_id =
        create_default_pin_pad_map(&root, &fixture, &[(fixture.pin_ids[0], fixture.pad_ids[0])]);
    let proposal_id = Uuid::new_v4();

    let output = run_project_command(&[
        "eda",
        "--format",
        "json",
        "proposal",
        "set-pool-pin-pad-map",
        root.to_str().unwrap(),
        "--map",
        &map_id.to_string(),
        "--mode",
        "replace",
        "--entry",
        &format!("{}:{}", fixture.pin_ids[1], fixture.pad_ids[1]),
        "--proposal",
        &proposal_id.to_string(),
        "--rationale",
        "review pin pad map update",
    ])
    .expect("PinPadMap set proposal should succeed");
    let report: serde_json::Value =
        serde_json::from_str(&output).expect("proposal report JSON should parse");
    assert_eq!(report["action"], "set_pool_pin_pad_map_proposal");

    let map_payload = query_pool_object_payload(&root, "pin_pad_maps", map_id);
    assert_eq!(map_payload["mappings"].as_object().unwrap().len(), 1);
    assert!(
        map_payload["mappings"]
            .as_object()
            .unwrap()
            .contains_key(&fixture.pad_ids[0].to_string()),
        "proposal creation must not mutate mappings"
    );

    run_project_command(&[
        "eda",
        "--format",
        "json",
        "proposal",
        "accept-apply",
        root.to_str().unwrap(),
        "--proposal",
        &proposal_id.to_string(),
    ])
    .expect("PinPadMap set proposal accept-apply should succeed");

    let map_payload = query_pool_object_payload(&root, "pin_pad_maps", map_id);
    assert_eq!(map_payload["mappings"].as_object().unwrap().len(), 1);
    assert!(
        map_payload["mappings"]
            .as_object()
            .unwrap()
            .contains_key(&fixture.pad_ids[1].to_string())
    );
    assert_eq!(
        map_payload["mappings"][fixture.pad_ids[1].to_string()]["pin"],
        fixture.pin_ids[1].to_string()
    );
    let _ = std::fs::remove_dir_all(&root);
}
