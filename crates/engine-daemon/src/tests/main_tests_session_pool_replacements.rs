use super::*;

#[test]
fn get_package_change_candidates_dispatch_returns_unique_candidate_report() {
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
    let assign = dispatch_request(
        &mut engine,
        JsonRpcRequest {
            jsonrpc: "2.0".into(),
            id: json!(4),
            method: "assign_part".into(),
            params: json!({
                "uuid": "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa",
                "part_uuid": search.result.as_ref().unwrap()[0]["uuid"],
            }),
        },
    );
    assert!(assign.error.is_none(), "{assign:?}");

    let response = dispatch_request(
        &mut engine,
        JsonRpcRequest {
            jsonrpc: "2.0".into(),
            id: json!(5),
            method: "get_package_change_candidates".into(),
            params: json!({ "uuid": "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa" }),
        },
    );
    assert!(response.error.is_none(), "{response:?}");
    let report = response.result.expect("result should exist");
    assert_eq!(report["status"], "candidates_available");
    assert_eq!(report["candidates"].as_array().unwrap().len(), 1);
    assert_eq!(report["candidates"][0]["package_name"], "ALT-3");
}

#[test]
fn get_part_change_candidates_dispatch_returns_compatible_part_report() {
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
    let assign = dispatch_request(
        &mut engine,
        JsonRpcRequest {
            jsonrpc: "2.0".into(),
            id: json!(4),
            method: "assign_part".into(),
            params: json!({
                "uuid": "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa",
                "part_uuid": lmv321_part_uuid,
            }),
        },
    );
    assert!(assign.error.is_none(), "{assign:?}");

    let response = dispatch_request(
        &mut engine,
        JsonRpcRequest {
            jsonrpc: "2.0".into(),
            id: json!(5),
            method: "get_part_change_candidates".into(),
            params: json!({"uuid": "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa"}),
        },
    );
    assert!(response.error.is_none(), "{response:?}");
    let report = response.result.expect("response should contain result");
    assert_eq!(report["status"], "candidates_available");
    assert_eq!(report["current_part_uuid"], lmv321_part_uuid);
    assert!(
        report["candidates"]
            .as_array()
            .unwrap()
            .iter()
            .any(|candidate| candidate["package_name"] == "ALT-3" && candidate["value"] == "ALTAMP")
    );
}

#[test]
fn get_component_replacement_plan_dispatch_returns_combined_report() {
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
    let assign = dispatch_request(
        &mut engine,
        JsonRpcRequest {
            jsonrpc: "2.0".into(),
            id: json!(4),
            method: "assign_part".into(),
            params: json!({
                "uuid": "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa",
                "part_uuid": lmv321_part_uuid,
            }),
        },
    );
    assert!(assign.error.is_none(), "{assign:?}");

    let response = dispatch_request(
        &mut engine,
        JsonRpcRequest {
            jsonrpc: "2.0".into(),
            id: json!(5),
            method: "get_component_replacement_plan".into(),
            params: json!({"uuid": "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa"}),
        },
    );
    assert!(response.error.is_none(), "{response:?}");
    let report = response.result.expect("response should contain result");
    assert_eq!(report["current_reference"], "R1");
    assert_eq!(report["current_part_uuid"], lmv321_part_uuid);
    assert_eq!(report["package_change"]["status"], "candidates_available");
    assert_eq!(report["part_change"]["status"], "candidates_available");
}

#[test]
fn get_scoped_component_replacement_plan_dispatch_returns_resolved_preview() {
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

    let response = dispatch_request(
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
    assert!(response.error.is_none(), "{response:?}");
    let report = response.result.expect("response should contain result");
    assert_eq!(report["policy"], "best_compatible_package");
    assert_eq!(report["replacements"].as_array().unwrap().len(), 2);
    assert_eq!(report["replacements"][0]["current_reference"], "R1");
    assert_eq!(report["replacements"][0]["target_package_name"], "ALT-3");
    assert_eq!(report["replacements"][0]["target_value"], "ALTAMP");
}

#[test]
fn edit_scoped_component_replacement_plan_dispatch_applies_exclusions_and_overrides() {
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
    let lmv321 = dispatch_request(
        &mut engine,
        JsonRpcRequest {
            jsonrpc: "2.0".into(),
            id: json!(3),
            method: "search_pool".into(),
            params: json!({"query": "LMV321"}),
        },
    );
    let lmv321_part_uuid = lmv321.result.as_ref().unwrap()[0]["uuid"].clone();
    let altamp = dispatch_request(
        &mut engine,
        JsonRpcRequest {
            jsonrpc: "2.0".into(),
            id: json!(4),
            method: "search_pool".into(),
            params: json!({"query": "ALTAMP"}),
        },
    );
    let altamp_part_uuid = altamp.result.as_ref().unwrap()[0]["uuid"].clone();
    let altamp_package_uuid = altamp.result.as_ref().unwrap()[0]["package_uuid"].clone();
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

    let preview = dispatch_request(
        &mut engine,
        JsonRpcRequest {
            jsonrpc: "2.0".into(),
            id: json!(7),
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

    let edited = dispatch_request(
        &mut engine,
        JsonRpcRequest {
            jsonrpc: "2.0".into(),
            id: json!(8),
            method: "edit_scoped_component_replacement_plan".into(),
            params: json!({
                "plan": preview.result.unwrap(),
                "exclude_component_uuids": ["bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb"],
                "overrides": [{
                    "component_uuid": "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa",
                    "target_package_uuid": altamp_package_uuid,
                    "target_part_uuid": altamp_part_uuid,
                }],
            }),
        },
    );
    assert!(edited.error.is_none(), "{edited:?}");
    let report = edited.result.expect("response should contain result");
    assert_eq!(report["replacements"].as_array().unwrap().len(), 1);
    assert_eq!(
        report["replacements"][0]["component_uuid"],
        "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa"
    );
    assert_eq!(report["replacements"][0]["target_package_name"], "ALT-3");
}
