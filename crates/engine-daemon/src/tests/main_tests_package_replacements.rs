use super::*;

#[test]
fn set_package_dispatch_updates_component_package() {
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
    let package_uuid = search.result.as_ref().unwrap()[0]["package_uuid"].clone();

    let response = dispatch_request(
        &mut engine,
        JsonRpcRequest {
            jsonrpc: "2.0".into(),
            id: json!(4),
            method: "set_package".into(),
            params: json!({
                "uuid": "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa",
                "package_uuid": package_uuid,
            }),
        },
    );
    assert!(response.error.is_none(), "{response:?}");
    let components = engine.get_components().expect("components should query");
    let updated = components
        .iter()
        .find(|component| component.reference == "R1")
        .unwrap();
    assert_eq!(updated.package_uuid.to_string(), package_uuid.as_str().unwrap());
}

#[test]
fn set_package_dispatch_updates_followup_components_query() {
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
    let package_uuid = search.result.as_ref().unwrap()[0]["package_uuid"].clone();

    let baseline = dispatch_request(
        &mut engine,
        JsonRpcRequest {
            jsonrpc: "2.0".into(),
            id: json!(4),
            method: "get_components".into(),
            params: json!({}),
        },
    );
    assert_eq!(
        baseline.result.as_ref().unwrap()[0]["package_uuid"],
        "00000000-0000-0000-0000-000000000000"
    );

    let set_package = dispatch_request(
        &mut engine,
        JsonRpcRequest {
            jsonrpc: "2.0".into(),
            id: json!(5),
            method: "set_package".into(),
            params: json!({
                "uuid": "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa",
                "package_uuid": package_uuid,
            }),
        },
    );
    assert!(set_package.error.is_none(), "{set_package:?}");

    let after = dispatch_request(
        &mut engine,
        JsonRpcRequest {
            jsonrpc: "2.0".into(),
            id: json!(6),
            method: "get_components".into(),
            params: json!({}),
        },
    );
    assert_eq!(after.result.as_ref().unwrap()[0]["package_uuid"], package_uuid);
}

#[test]
fn set_package_dispatch_updates_followup_net_info_query() {
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
            params: json!({"query": "ALTAMP"}),
        },
    );
    let package_uuid = search.result.as_ref().unwrap()[0]["package_uuid"].clone();

    let baseline = dispatch_request(
        &mut engine,
        JsonRpcRequest {
            jsonrpc: "2.0".into(),
            id: json!(4),
            method: "get_net_info".into(),
            params: json!({}),
        },
    );
    let baseline_sig = baseline.result.as_ref().unwrap()
        .as_array().unwrap()
        .iter()
        .find(|net| net["name"] == "SIG")
        .expect("SIG net should exist");
    assert_eq!(baseline_sig["pins"].as_array().unwrap().len(), 2);

    let set_package = dispatch_request(
        &mut engine,
        JsonRpcRequest {
            jsonrpc: "2.0".into(),
            id: json!(5),
            method: "set_package".into(),
            params: json!({
                "uuid": "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa",
                "package_uuid": package_uuid,
            }),
        },
    );
    assert!(set_package.error.is_none(), "{set_package:?}");

    let after = dispatch_request(
        &mut engine,
        JsonRpcRequest {
            jsonrpc: "2.0".into(),
            id: json!(6),
            method: "get_net_info".into(),
            params: json!({}),
        },
    );
    let after_sig = after.result.as_ref().unwrap()
        .as_array().unwrap()
        .iter()
        .find(|net| net["name"] == "SIG")
        .expect("SIG net should exist");
    assert_eq!(after_sig["pins"].as_array().unwrap().len(), 1);
}

#[test]
fn set_package_dispatch_preserves_logical_nets_across_known_part_remap() {
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

    let set_package = dispatch_request(
        &mut engine,
        JsonRpcRequest {
            jsonrpc: "2.0".into(),
            id: json!(7),
            method: "set_package".into(),
            params: json!({
                "uuid": "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa",
                "package_uuid": altamp_package_uuid,
            }),
        },
    );
    assert!(set_package.error.is_none(), "{set_package:?}");

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
    assert_eq!(after_sig["pins"].as_array().unwrap().len(), intermediate_pin_count);
}
