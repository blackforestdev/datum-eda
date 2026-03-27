use super::*;

#[test]
fn set_package_with_part_dispatch_preserves_logical_nets_for_explicit_candidate() {
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

    let first_assign = dispatch_request(
        &mut engine,
        JsonRpcRequest {
            jsonrpc: "2.0".into(),
            id: json!(5),
            method: "assign_part".into(),
            params: json!({
                "uuid": "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa",
                "part_uuid": lmv321_part_uuid,
            }),
        },
    );
    assert!(first_assign.error.is_none(), "{first_assign:?}");

    let intermediate = dispatch_request(
        &mut engine,
        JsonRpcRequest {
            jsonrpc: "2.0".into(),
            id: json!(6),
            method: "get_net_info".into(),
            params: json!({}),
        },
    );
    let intermediate_sig = intermediate
        .result
        .as_ref()
        .unwrap()
        .as_array()
        .unwrap()
        .iter()
        .find(|net| net["name"] == "SIG")
        .expect("SIG net should exist");
    let intermediate_pin_count = intermediate_sig["pins"].as_array().unwrap().len();

    let set_package_with_part = dispatch_request(
        &mut engine,
        JsonRpcRequest {
            jsonrpc: "2.0".into(),
            id: json!(7),
            method: "set_package_with_part".into(),
            params: json!({
                "uuid": "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa",
                "package_uuid": altamp_package_uuid,
                "part_uuid": altamp_part_uuid,
            }),
        },
    );
    assert!(
        set_package_with_part.error.is_none(),
        "{set_package_with_part:?}"
    );

    let after = dispatch_request(
        &mut engine,
        JsonRpcRequest {
            jsonrpc: "2.0".into(),
            id: json!(8),
            method: "get_net_info".into(),
            params: json!({}),
        },
    );
    let after_sig = after
        .result
        .as_ref()
        .unwrap()
        .as_array()
        .unwrap()
        .iter()
        .find(|net| net["name"] == "SIG")
        .expect("SIG net should exist");
    assert_eq!(
        after_sig["pins"].as_array().unwrap().len(),
        intermediate_pin_count
    );
}

#[test]
fn replace_component_dispatch_preserves_logical_nets_for_explicit_candidate() {
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

    let first_assign = dispatch_request(
        &mut engine,
        JsonRpcRequest {
            jsonrpc: "2.0".into(),
            id: json!(5),
            method: "assign_part".into(),
            params: json!({
                "uuid": "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa",
                "part_uuid": lmv321_part_uuid,
            }),
        },
    );
    assert!(first_assign.error.is_none(), "{first_assign:?}");

    let intermediate = dispatch_request(
        &mut engine,
        JsonRpcRequest {
            jsonrpc: "2.0".into(),
            id: json!(6),
            method: "get_net_info".into(),
            params: json!({}),
        },
    );
    let intermediate_sig = intermediate
        .result
        .as_ref()
        .unwrap()
        .as_array()
        .unwrap()
        .iter()
        .find(|net| net["name"] == "SIG")
        .expect("SIG net should exist");
    let intermediate_pin_count = intermediate_sig["pins"].as_array().unwrap().len();

    let replace_component = dispatch_request(
        &mut engine,
        JsonRpcRequest {
            jsonrpc: "2.0".into(),
            id: json!(7),
            method: "replace_component".into(),
            params: json!({
                "uuid": "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa",
                "package_uuid": altamp_package_uuid,
                "part_uuid": altamp_part_uuid,
            }),
        },
    );
    assert!(replace_component.error.is_none(), "{replace_component:?}");
    assert_eq!(
        replace_component.result.as_ref().unwrap()["description"],
        "replace_component aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa"
    );

    let after = dispatch_request(
        &mut engine,
        JsonRpcRequest {
            jsonrpc: "2.0".into(),
            id: json!(8),
            method: "get_net_info".into(),
            params: json!({}),
        },
    );
    let after_sig = after
        .result
        .as_ref()
        .unwrap()
        .as_array()
        .unwrap()
        .iter()
        .find(|net| net["name"] == "SIG")
        .expect("SIG net should exist");
    assert_eq!(
        after_sig["pins"].as_array().unwrap().len(),
        intermediate_pin_count
    );
}

#[test]
fn replace_components_dispatch_batches_multiple_replacements_into_one_undo_step() {
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
            params: json!({"query": "ALTAMP"}),
        },
    );
    let altamp_package_uuid = altamp_search.result.as_ref().unwrap()[0]["package_uuid"].clone();
    let altamp_part_uuid = altamp_search.result.as_ref().unwrap()[0]["uuid"].clone();

    let replace_components = dispatch_request(
        &mut engine,
        JsonRpcRequest {
            jsonrpc: "2.0".into(),
            id: json!(4),
            method: "replace_components".into(),
            params: json!({
                "replacements": [
                    {
                        "uuid": "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa",
                        "package_uuid": altamp_package_uuid,
                        "part_uuid": altamp_part_uuid,
                    },
                    {
                        "uuid": "bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb",
                        "package_uuid": altamp_package_uuid,
                        "part_uuid": altamp_part_uuid,
                    }
                ]
            }),
        },
    );
    assert!(replace_components.error.is_none(), "{replace_components:?}");
    assert_eq!(
        replace_components.result.as_ref().unwrap()["description"],
        "replace_components 2"
    );

    let undo = dispatch_request(
        &mut engine,
        JsonRpcRequest {
            jsonrpc: "2.0".into(),
            id: json!(5),
            method: "undo".into(),
            params: json!({}),
        },
    );
    assert!(undo.error.is_none(), "{undo:?}");
    assert_eq!(
        undo.result.as_ref().unwrap()["description"],
        "undo replace_components 2"
    );

    let components = dispatch_request(
        &mut engine,
        JsonRpcRequest {
            jsonrpc: "2.0".into(),
            id: json!(6),
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
    assert_eq!(values.iter().filter(|value| **value == "10k").count(), 2);
}
