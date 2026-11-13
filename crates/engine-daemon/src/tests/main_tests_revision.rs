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
    assert!(response["result"].is_null());
    assert_eq!(response["refusal"]["code"], "unsupported_algorithm");
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn daemon_current_and_historical_refusals_match_engine_packet() {
    let root = std::env::temp_dir().join(format!(
        "datum_daemon_revision_refusal_{}",
        uuid::Uuid::new_v4()
    ));
    std::fs::create_dir_all(&root).unwrap();
    let report = bootstrap_native_project(
        &root,
        GenesisSpec {
            project_name: "Revision refusal daemon".into(),
            existing_ids: None,
        },
    )
    .unwrap();
    let mut engine = Engine::new().unwrap();
    for as_of_sequence in [None, Some(0), Some(99)] {
        let typed = eda_engine::api::native_write::revision::RevisionQueryRequest {
            project_id: report.project_uuid,
            query: "release_readiness".into(),
            as_of_sequence,
            expected_model_revision: None,
        };
        let expected =
            eda_engine::api::native_write::revision::query_revision_authority(&root, &typed)
                .unwrap();
        let response = dispatch_request(
            &mut engine,
            request(
                3,
                "revision.query",
                json!({
                    "project_root": root, "query": typed.query, "as_of_sequence": as_of_sequence,
                    "expected_model_revision": null,
                }),
            ),
        );
        assert!(response.error.is_none());
        assert_eq!(
            response.result.unwrap(),
            serde_json::to_value(expected).unwrap()
        );
    }
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn daemon_rejects_unsupported_selectors_before_resolving_project() {
    let mut engine = Engine::new().unwrap();
    for selector in ["subject", "subject_id", "baseline_id", "release_id"] {
        let mut params =
            json!({"project_root": "/nonexistent/revision-selector", "query": "release_readiness"});
        params[selector] = json!("unsupported");
        let response = dispatch_request(&mut engine, request(4, "revision.query", params));
        assert!(response.result.is_none());
        let error = response.error.unwrap();
        assert_eq!(error.code, -32602);
        assert!(error.message.contains(selector));
    }
}
