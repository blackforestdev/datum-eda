use super::*;

#[test]
fn explain_violation_dispatch_returns_erc_explanation() {
    let mut engine = Engine::new().expect("engine should initialize");
    let open = JsonRpcRequest {
        jsonrpc: "2.0".into(),
        id: json!(1),
        method: "open_project".into(),
        params: serde_json::to_value(OpenProjectParams {
            path: kicad_fixture_path("simple-demo.kicad_sch"),
        })
        .unwrap(),
    };
    let _ = dispatch_request(&mut engine, open);

    let request = JsonRpcRequest {
        jsonrpc: "2.0".into(),
        id: json!(2),
        method: "explain_violation".into(),
        params: json!({ "domain": "erc", "index": 0 }),
    };
    let response = dispatch_request(&mut engine, request);
    assert!(response.error.is_none(), "{response:?}");
    let result = response.result.expect("result should exist");
    assert!(result["explanation"].is_string());
    assert!(result["rule_detail"].is_string());
    assert!(result["objects_involved"].is_array());
    assert!(result["suggestion"].is_string());
}

#[test]
fn get_connectivity_diagnostics_dispatch_returns_board_diagnostics() {
    let mut engine = Engine::new().expect("engine should initialize");
    let open = JsonRpcRequest {
        jsonrpc: "2.0".into(),
        id: json!(1),
        method: "open_project".into(),
        params: serde_json::to_value(OpenProjectParams {
            path: kicad_fixture_path("simple-demo.kicad_pcb"),
        })
        .unwrap(),
    };
    let _ = dispatch_request(&mut engine, open);

    let request = JsonRpcRequest {
        jsonrpc: "2.0".into(),
        id: json!(2),
        method: "get_connectivity_diagnostics".into(),
        params: json!({}),
    };

    let response = dispatch_request(&mut engine, request);
    assert!(response.error.is_none(), "{response:?}");
    let result = response.result.expect("result should exist");
    let diagnostics = result
        .as_array()
        .expect("diagnostics result should be an array");
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0]["kind"], "net_without_copper");
}

#[test]
fn get_connectivity_diagnostics_dispatch_returns_partial_route_board_diagnostics() {
    let mut engine = Engine::new().expect("engine should initialize");
    let open = JsonRpcRequest {
        jsonrpc: "2.0".into(),
        id: json!(1),
        method: "open_project".into(),
        params: serde_json::to_value(OpenProjectParams {
            path: kicad_fixture_path("partial-route-demo.kicad_pcb"),
        })
        .unwrap(),
    };
    let _ = dispatch_request(&mut engine, open);

    let request = JsonRpcRequest {
        jsonrpc: "2.0".into(),
        id: json!(2),
        method: "get_connectivity_diagnostics".into(),
        params: json!({}),
    };

    let response = dispatch_request(&mut engine, request);
    assert!(response.error.is_none(), "{response:?}");
    let result = response.result.expect("result should exist");
    let diagnostics = result
        .as_array()
        .expect("diagnostics result should be an array");
    assert_eq!(diagnostics.len(), 2);
    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic["kind"] == "partially_routed_net")
    );
}

#[test]
fn get_connectivity_diagnostics_dispatch_returns_schematic_diagnostics() {
    let mut engine = Engine::new().expect("engine should initialize");
    let open = JsonRpcRequest {
        jsonrpc: "2.0".into(),
        id: json!(1),
        method: "open_project".into(),
        params: serde_json::to_value(OpenProjectParams {
            path: kicad_fixture_path("simple-demo.kicad_sch"),
        })
        .unwrap(),
    };
    let _ = dispatch_request(&mut engine, open);

    let request = JsonRpcRequest {
        jsonrpc: "2.0".into(),
        id: json!(2),
        method: "get_connectivity_diagnostics".into(),
        params: json!({}),
    };

    let response = dispatch_request(&mut engine, request);
    assert!(response.error.is_none(), "{response:?}");
    let result = response.result.expect("result should exist");
    let diagnostics = result
        .as_array()
        .expect("diagnostics result should be an array");
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0]["kind"], "dangling_component_pin");
}

#[test]
fn get_design_rules_dispatch_returns_board_rules_array() {
    let mut engine = Engine::new().expect("engine should initialize");
    let open = JsonRpcRequest {
        jsonrpc: "2.0".into(),
        id: json!(1),
        method: "open_project".into(),
        params: serde_json::to_value(OpenProjectParams {
            path: kicad_fixture_path("simple-demo.kicad_pcb"),
        })
        .unwrap(),
    };
    let _ = dispatch_request(&mut engine, open);

    let request = JsonRpcRequest {
        jsonrpc: "2.0".into(),
        id: json!(2),
        method: "get_design_rules".into(),
        params: json!({}),
    };
    let response = dispatch_request(&mut engine, request);
    assert!(response.error.is_none(), "{response:?}");
    let result = response.result.expect("result should exist");
    let rules = result.as_array().expect("rules result should be an array");
    assert!(rules.is_empty());
}
