// Dispatch tests for the terminally frozen imported-session replacement arms
// (`apply_component_replacement_policy`, `apply_scoped_component_replacement_policy`,
// `apply_scoped_component_replacement_plan`). These arms are imported-session
// compatibility only (see the fence in dispatch.rs); test setup uses the engine
// API directly because the retired `assign_part` dispatch arm no longer exists.

use eda_engine::api::AssignPartInput;

use super::*;

fn assign_part_via_engine(
    engine: &mut Engine,
    component_uuid: &str,
    part_uuid: &serde_json::Value,
) {
    engine
        .assign_part(AssignPartInput {
            uuid: uuid::Uuid::parse_str(component_uuid).expect("component uuid should parse"),
            part_uuid: uuid::Uuid::parse_str(part_uuid.as_str().expect("part uuid should be str"))
                .expect("part uuid should parse"),
        })
        .expect("assign_part should succeed");
}

#[test]
fn apply_component_replacement_policy_dispatch_resolves_best_candidates() {
    let mut engine = Engine::new().expect("engine should initialize");
    let _ = dispatch_request(
        &mut engine,
        JsonRpcRequest {
            jsonrpc: "2.0".into(),
            id: json!(1),
            method: "open_project".into(),
            params: serde_json::to_value(OpenProjectParams {
                path: eagle_fixture_path("simple-opamp.lbr"),
            })
            .unwrap(),
        },
    );
    let _ = dispatch_request(
        &mut engine,
        JsonRpcRequest {
            jsonrpc: "2.0".into(),
            id: json!(2),
            method: "open_project".into(),
            params: serde_json::to_value(OpenProjectParams {
                path: kicad_fixture_path("partial-route-demo.kicad_pcb"),
            })
            .unwrap(),
        },
    );
    let lmv321_search = dispatch_request(
        &mut engine,
        JsonRpcRequest {
            jsonrpc: "2.0".into(),
            id: json!(3),
            method: "search_pool".into(),
            params: json!({"query": "LMV321"}),
        },
    );
    let lmv321_part_uuid = lmv321_search.result.as_ref().unwrap()[0]["uuid"].clone();
    for uuid in [
        "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa",
        "bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb",
    ] {
        assign_part_via_engine(&mut engine, uuid, &lmv321_part_uuid);
    }

    let apply = dispatch_request(
        &mut engine,
        JsonRpcRequest {
            jsonrpc: "2.0".into(),
            id: json!(6),
            method: "apply_component_replacement_policy".into(),
            params: json!({
                "replacements": [
                    {
                        "uuid": "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa",
                        "policy": "best_compatible_package",
                    },
                    {
                        "uuid": "bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb",
                        "policy": "best_compatible_part",
                    }
                ]
            }),
        },
    );
    assert!(apply.error.is_none(), "{apply:?}");
    assert_eq!(
        apply.result.as_ref().unwrap()["description"],
        "replace_components 2"
    );

    let components = dispatch_request(
        &mut engine,
        JsonRpcRequest {
            jsonrpc: "2.0".into(),
            id: json!(7),
            method: "get_components".into(),
            params: json!({}),
        },
    );
    let values: Vec<_> = components
        .result
        .as_ref()
        .unwrap()
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|component| component["value"].as_str())
        .collect();
    assert_eq!(values.iter().filter(|value| **value == "ALTAMP").count(), 2);
}

#[test]
fn apply_scoped_component_replacement_policy_dispatch_targets_filtered_components() {
    let mut engine = Engine::new().expect("engine should initialize");
    let _ = dispatch_request(
        &mut engine,
        JsonRpcRequest {
            jsonrpc: "2.0".into(),
            id: json!(1),
            method: "open_project".into(),
            params: serde_json::to_value(OpenProjectParams {
                path: eagle_fixture_path("simple-opamp.lbr"),
            })
            .unwrap(),
        },
    );
    let _ = dispatch_request(
        &mut engine,
        JsonRpcRequest {
            jsonrpc: "2.0".into(),
            id: json!(2),
            method: "open_project".into(),
            params: serde_json::to_value(OpenProjectParams {
                path: kicad_fixture_path("partial-route-demo.kicad_pcb"),
            })
            .unwrap(),
        },
    );
    let lmv321_search = dispatch_request(
        &mut engine,
        JsonRpcRequest {
            jsonrpc: "2.0".into(),
            id: json!(3),
            method: "search_pool".into(),
            params: json!({"query": "LMV321"}),
        },
    );
    let lmv321_part_uuid = lmv321_search.result.as_ref().unwrap()[0]["uuid"].clone();
    for uuid in [
        "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa",
        "bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb",
    ] {
        assign_part_via_engine(&mut engine, uuid, &lmv321_part_uuid);
    }

    let apply = dispatch_request(
        &mut engine,
        JsonRpcRequest {
            jsonrpc: "2.0".into(),
            id: json!(6),
            method: "apply_scoped_component_replacement_policy".into(),
            params: json!({
                "scope": {
                    "reference_prefix": "R",
                    "value_equals": "LMV321",
                },
                "policy": "best_compatible_package",
            }),
        },
    );
    assert!(apply.error.is_none(), "{apply:?}");
    assert_eq!(
        apply.result.as_ref().unwrap()["description"],
        "replace_components 2"
    );

    let components = dispatch_request(
        &mut engine,
        JsonRpcRequest {
            jsonrpc: "2.0".into(),
            id: json!(7),
            method: "get_components".into(),
            params: json!({}),
        },
    );
    let values: Vec<_> = components
        .result
        .as_ref()
        .unwrap()
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|component| component["value"].as_str())
        .collect();
    assert_eq!(values.iter().filter(|value| **value == "ALTAMP").count(), 2);

    // The kept frozen arms still push to the imported-session undo stack, so
    // the daemon `undo` arm must stay while these arms live.
    let undo = dispatch_request(
        &mut engine,
        JsonRpcRequest {
            jsonrpc: "2.0".into(),
            id: json!(8),
            method: "undo".into(),
            params: json!({}),
        },
    );
    assert!(undo.error.is_none(), "{undo:?}");
    assert_eq!(
        undo.result.as_ref().unwrap()["description"],
        "undo replace_components 2"
    );
}

#[test]
fn apply_scoped_component_replacement_plan_dispatch_applies_preview_without_reresolving() {
    let mut engine = Engine::new().expect("engine should initialize");
    let _ = dispatch_request(
        &mut engine,
        JsonRpcRequest {
            jsonrpc: "2.0".into(),
            id: json!(1),
            method: "open_project".into(),
            params: serde_json::to_value(OpenProjectParams {
                path: eagle_fixture_path("simple-opamp.lbr"),
            })
            .unwrap(),
        },
    );
    let _ = dispatch_request(
        &mut engine,
        JsonRpcRequest {
            jsonrpc: "2.0".into(),
            id: json!(2),
            method: "open_project".into(),
            params: serde_json::to_value(OpenProjectParams {
                path: kicad_fixture_path("partial-route-demo.kicad_pcb"),
            })
            .unwrap(),
        },
    );
    let search = dispatch_request(
        &mut engine,
        JsonRpcRequest {
            jsonrpc: "2.0".into(),
            id: json!(3),
            method: "search_pool".into(),
            params: json!({"query": "LMV321"}),
        },
    );
    let lmv321_part_uuid = search.result.as_ref().unwrap()[0]["uuid"].clone();
    for uuid in [
        "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa",
        "bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb",
    ] {
        assign_part_via_engine(&mut engine, uuid, &lmv321_part_uuid);
    }
    let preview = dispatch_request(
        &mut engine,
        JsonRpcRequest {
            jsonrpc: "2.0".into(),
            id: json!(6),
            method: "get_scoped_component_replacement_plan".into(),
            params: json!({
                "scope": {
                    "reference_prefix": "R",
                    "value_equals": "LMV321",
                },
                "policy": "best_compatible_package",
            }),
        },
    );
    assert!(preview.error.is_none(), "{preview:?}");

    let apply = dispatch_request(
        &mut engine,
        JsonRpcRequest {
            jsonrpc: "2.0".into(),
            id: json!(7),
            method: "apply_scoped_component_replacement_plan".into(),
            params: json!({
                "plan": preview.result.unwrap(),
            }),
        },
    );
    assert!(apply.error.is_none(), "{apply:?}");
    assert_eq!(
        apply.result.as_ref().unwrap()["description"],
        "replace_components 2"
    );
}

#[test]
fn retired_component_write_arms_return_method_not_found() {
    let mut engine = Engine::new().expect("engine should initialize");
    let _ = dispatch_request(
        &mut engine,
        JsonRpcRequest {
            jsonrpc: "2.0".into(),
            id: json!(1),
            method: "open_project".into(),
            params: serde_json::to_value(OpenProjectParams {
                path: kicad_fixture_path("partial-route-demo.kicad_pcb"),
            })
            .unwrap(),
        },
    );
    for (id, method, params) in [
        (
            2,
            "move_component",
            json!({"uuid": "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa", "x_mm": 1.0, "y_mm": 2.0}),
        ),
        (
            3,
            "set_value",
            json!({"uuid": "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa", "value": "22k"}),
        ),
        (4, "replace_components", json!({"replacements": []})),
        (
            5,
            "apply_component_replacement_plan",
            json!({"replacements": []}),
        ),
    ] {
        let response = dispatch_request(
            &mut engine,
            JsonRpcRequest {
                jsonrpc: "2.0".into(),
                id: json!(id),
                method: method.into(),
                params,
            },
        );
        assert!(response.result.is_none(), "{method} should be retired");
        assert_eq!(
            response.error.expect("retired arm should error").code,
            -32601,
            "{method} should return method-not-found"
        );
    }
}
