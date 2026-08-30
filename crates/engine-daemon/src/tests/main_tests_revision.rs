use super::*;

use eda_engine::api::native_write::genesis::{GenesisSpec, bootstrap_native_project};

fn request(id: u64, method: &str, params: Value) -> JsonRpcRequest {
    JsonRpcRequest {
        jsonrpc: "2.0".into(),
        id: json!(id),
        method: method.into(),
        params,
    }
}

#[test]
fn daemon_uses_engine_catalog_and_query_payload() {
    let root = std::env::temp_dir().join(format!("datum_daemon_revision_{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&root).unwrap();
    bootstrap_native_project(
        &root,
        GenesisSpec {
            project_name: "Revision daemon fixture".into(),
            existing_ids: None,
        },
    )
    .unwrap();
    let mut engine = Engine::new().unwrap();

    let catalog = dispatch_request(&mut engine, request(1, "revision.catalog", json!({})))
        .result
        .unwrap();
    assert_eq!(
        catalog,
        serde_json::to_value(eda_engine::api::native_write::revision::revision_public_catalog(),)
            .unwrap()
    );

    let response = dispatch_request(
        &mut engine,
        request(
            2,
            "revision.query",
            json!({
                "project_root": root,
                "query": "configuration_current",
                "as_of_sequence": null,
                "expected_model_revision": null
            }),
        ),
    )
    .result
    .unwrap();
    assert_eq!(response["result"]["authority_state"], "unmanaged");
    assert!(response["refusal"].is_null());
    let _ = std::fs::remove_dir_all(root);
}
