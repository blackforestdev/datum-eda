//! Public native library reads and closed service-policy boundaries.
use super::*;

#[test]
fn optional_library_read_is_snapshot_qualified_and_does_not_adopt_legacy_data() {
    let mut engine = Engine::new().unwrap();
    let root = temp_native_project("library_inspection");
    let manifest = std::fs::read(root.join("project.json")).unwrap();
    let ordinary = describe(&mut engine, 1, &root);
    assert!(ordinary.get("library").is_none());
    let reply = dispatch_request(
        &mut engine,
        native_request(
            2,
            "native.describe",
            json!({"project_root":root,"include_library":true}),
        ),
    );
    assert!(reply.error.is_none(), "{reply:?}");
    let description = reply.result.unwrap();
    assert_eq!(description["model_revision"], ordinary["model_revision"]);
    assert_eq!(description["transaction_tip"], ordinary["transaction_tip"]);
    let library = &description["library"];
    assert_eq!(library["project_id"], description["project_id"]);
    assert_eq!(library["model_revision"], description["model_revision"]);
    assert_eq!(library["managed"], false);
    assert_eq!(library["common_record_status"], "not_adopted");
    assert!(library["catalog_manifest"].is_null());
    assert_eq!(library["assets"], json!([]));
    assert_eq!(std::fs::read(root.join("project.json")).unwrap(), manifest);
    assert!(!root.join("library").exists());
    assert_eq!(describe(&mut engine, 3, &root)["journal_len"], 0);
    let unsupported = dispatch_request(
        &mut engine,
        native_request(
            4,
            "native.describe",
            json!({"project_root":root,"library_geometry_selection":{
                "expected_catalog":null,"selected":null}}),
        ),
    );
    assert!(
        unsupported.error.is_some(),
        "Malformed geometry selection must refuse"
    );
    assert_eq!(std::fs::read(root.join("project.json")).unwrap(), manifest);
    assert!(!root.join("library").exists());
    assert_eq!(describe(&mut engine, 5, &root)["journal_len"], 0);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn native_library_action_cannot_enable_policy_or_raise_limits_from_the_wire() {
    let mut engine = Engine::new().unwrap();
    let root = temp_native_project("library_policy_refusal");
    let described = describe(&mut engine, 1, &root);
    let manifest = std::fs::read(root.join("project.json")).unwrap();
    let params = json!({"project_root":root,"verb":"datum.library.edit",
        "params":{"action":"adopt_profile"},"actor":"datum-gui","source":"manual",
        "reason":"Explicit format adoption control",
        "expected_model_revision":described["model_revision"],
        "expected_transaction_tip":described["transaction_tip"]});
    let response = dispatch_request(
        &mut engine,
        native_request(2, "native.write", params.clone()),
    );
    let error = response.error.expect("unconfigured service must refuse");
    assert_eq!(error.code, -32060);
    assert!(error.message.contains("unconfigured"), "{error:?}");
    let mut override_attempt = params.clone();
    override_attempt["params"]["limits"] = json!({"files":1_000_000,"total_bytes":u64::MAX});
    let response = dispatch_request(
        &mut engine,
        native_request(3, "native.write", override_attempt),
    );
    let error = response
        .error
        .expect("request must not configure service policy");
    assert_eq!(error.code, -32060);
    assert!(
        error.message.contains("serialization error") && error.message.contains("limits"),
        "{error:?}"
    );
    let unchanged_policy = dispatch_request(&mut engine, native_request(4, "native.write", params));
    let error = unchanged_policy
        .error
        .expect("rejected input must leave service unconfigured");
    assert_eq!(error.code, -32060);
    assert!(error.message.contains("unconfigured"), "{error:?}");
    assert_eq!(std::fs::read(root.join("project.json")).unwrap(), manifest);
    assert!(!root.join("library").exists());
    assert_eq!(describe(&mut engine, 5, &root)["journal_len"], 0);
    std::fs::remove_dir_all(root).unwrap();
}
