use super::*;

#[test]
fn apply_component_replacement_plan_dispatch_resolves_package_and_part_selectors() {
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
    let altamp_search = dispatch_request(
        &mut engine,
        JsonRpcRequest {
            jsonrpc: "2.0".into(),
            id: json!(3),
            method: "search_pool".into(),
            params: json!({"query": "LMV321"}),
        },
    );
    let lmv321_part_uuid = altamp_search.result.as_ref().unwrap()[0]["uuid"].clone();
    let altamp_search = dispatch_request(
        &mut engine,
        JsonRpcRequest {
            jsonrpc: "2.0".into(),
            id: json!(4),
            method: "search_pool".into(),
            params: json!({"query": "ALTAMP"}),
        },
    );
    let altamp_package_uuid = altamp_search.result.as_ref().unwrap()[0]["package_uuid"].clone();
    let altamp_part_uuid = altamp_search.result.as_ref().unwrap()[0]["uuid"].clone();

    for (id, uuid) in [
        (5, "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa"),
        (6, "bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb"),
    ] {
        let assign = dispatch_request(
            &mut engine,
            JsonRpcRequest {
                jsonrpc: "2.0".into(),
                id: json!(id),
                method: "assign_part".into(),
                params: json!({
                    "uuid": uuid,
                    "part_uuid": lmv321_part_uuid,
                }),
            },
        );
        assert!(assign.error.is_none(), "{assign:?}");
    }

    let apply = dispatch_request(
        &mut engine,
        JsonRpcRequest {
            jsonrpc: "2.0".into(),
            id: json!(7),
            method: "apply_component_replacement_plan".into(),
            params: json!({
                "replacements": [
                    {
                        "uuid": "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa",
                        "package_uuid": altamp_package_uuid,
                        "part_uuid": null,
                    },
                    {
                        "uuid": "bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb",
                        "package_uuid": null,
                        "part_uuid": altamp_part_uuid,
                    }
                ]
            }),
        },
    );
    assert!(apply.error.is_none(), "{apply:?}");
    assert_eq!(apply.result.as_ref().unwrap()["description"], "replace_components 2");

    let components = dispatch_request(
        &mut engine,
        JsonRpcRequest {
            jsonrpc: "2.0".into(),
            id: json!(8),
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
    for (id, uuid) in [
        (4, "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa"),
        (5, "bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb"),
    ] {
        let assign = dispatch_request(
            &mut engine,
            JsonRpcRequest {
                jsonrpc: "2.0".into(),
                id: json!(id),
                method: "assign_part".into(),
                params: json!({
                    "uuid": uuid,
                    "part_uuid": lmv321_part_uuid,
                }),
            },
        );
        assert!(assign.error.is_none(), "{assign:?}");
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
    assert_eq!(apply.result.as_ref().unwrap()["description"], "replace_components 2");

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
    for (id, uuid) in [
        (4, "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa"),
        (5, "bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb"),
    ] {
        let assign = dispatch_request(
            &mut engine,
            JsonRpcRequest {
                jsonrpc: "2.0".into(),
                id: json!(id),
                method: "assign_part".into(),
                params: json!({
                    "uuid": uuid,
                    "part_uuid": lmv321_part_uuid,
                }),
            },
        );
        assert!(assign.error.is_none(), "{assign:?}");
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
    assert_eq!(apply.result.as_ref().unwrap()["description"], "replace_components 2");

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
    for (id, uuid) in [
        (4, "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa"),
        (5, "bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb"),
    ] {
        let assign = dispatch_request(
            &mut engine,
            JsonRpcRequest {
                jsonrpc: "2.0".into(),
                id: json!(id),
                method: "assign_part".into(),
                params: json!({
                    "uuid": uuid,
                    "part_uuid": lmv321_part_uuid,
                }),
            },
        );
        assert!(assign.error.is_none(), "{assign:?}");
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
    assert_eq!(apply.result.as_ref().unwrap()["description"], "replace_components 2");
}
