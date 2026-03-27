use super::*;

#[test]
fn run_erc_dispatch_returns_raw_schematic_findings() {
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
        method: "run_erc".into(),
        params: json!({}),
    };

    let response = dispatch_request(&mut engine, request);
    assert!(response.error.is_none(), "{response:?}");
    let result = response.result.expect("result should exist");
    let findings = result
        .as_array()
        .expect("run_erc result should be an array");
    assert_eq!(findings.len(), 2);
    assert!(
        findings
            .iter()
            .any(|finding| finding["code"] == "unconnected_component_pin")
    );
    assert!(
        findings
            .iter()
            .any(|finding| finding["code"] == "undriven_power_net")
    );
}

#[test]
fn run_erc_dispatch_returns_input_without_explicit_driver_finding() {
    let mut engine = Engine::new().expect("engine should initialize");
    let open = JsonRpcRequest {
        jsonrpc: "2.0".into(),
        id: json!(1),
        method: "open_project".into(),
        params: serde_json::to_value(OpenProjectParams {
            path: kicad_fixture_path("analog-input-demo.kicad_sch"),
        })
        .unwrap(),
    };
    let _ = dispatch_request(&mut engine, open);

    let request = JsonRpcRequest {
        jsonrpc: "2.0".into(),
        id: json!(2),
        method: "run_erc".into(),
        params: json!({}),
    };

    let response = dispatch_request(&mut engine, request);
    assert!(response.error.is_none(), "{response:?}");
    let result = response.result.expect("result should exist");
    let findings = result
        .as_array()
        .expect("run_erc result should be an array");
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0]["code"], "input_without_explicit_driver");
    assert_eq!(findings[0]["severity"], "Info");
}

#[test]
fn run_drc_dispatch_returns_board_report_shape() {
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
        method: "run_drc".into(),
        params: json!({}),
    };

    let response = dispatch_request(&mut engine, request);
    assert!(response.error.is_none(), "{response:?}");
    let result = response.result.expect("result should exist");
    assert!(result["passed"].is_boolean());
    assert!(result["summary"]["errors"].is_u64());
    assert!(result["summary"]["warnings"].is_u64());
    assert!(result["violations"].is_array());
}

#[test]
fn run_drc_dispatch_reports_connectivity_violation_on_partial_route_board() {
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
        method: "run_drc".into(),
        params: json!({}),
    };

    let response = dispatch_request(&mut engine, request);
    assert!(response.error.is_none(), "{response:?}");
    let result = response.result.expect("result should exist");
    assert_eq!(result["passed"], json!(false));
    let violations = result["violations"]
        .as_array()
        .expect("violations should be an array");
    assert!(
        violations
            .iter()
            .any(|violation| violation["code"] == "connectivity_unrouted_net")
    );
}
