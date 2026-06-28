use super::*;

#[test]
fn run_erc_dispatch_returns_check_run_view_with_raw_schematic_findings() {
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
    assert_eq!(result["contract"], "check_run_v1");
    assert_eq!(result["persisted"], false);
    assert_eq!(result["profile_id"], "erc");
    let findings = result["findings"]
        .as_array()
        .expect("normalized ERC findings should be an array");
    assert_eq!(findings.len(), 2);
    assert!(findings.iter().any(|finding| {
        finding["code"] == "unconnected_component_pin"
            && finding["primary_target"]["kind"] == "object_uuid"
            && finding["fingerprint"]
                .as_str()
                .unwrap()
                .starts_with("sha256:")
    }));
    assert!(
        findings
            .iter()
            .any(|finding| finding["code"] == "undriven_power_net")
    );
    let raw = result["raw_report"]["erc"]
        .as_array()
        .expect("raw ERC compatibility findings should be nested");
    assert_eq!(raw.len(), findings.len());
}

#[test]
fn run_erc_dispatch_reports_input_without_explicit_driver_finding_in_check_run() {
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
    assert_eq!(result["contract"], "check_run_v1");
    assert_eq!(result["profile_id"], "erc");
    let findings = result["findings"]
        .as_array()
        .expect("normalized ERC findings should be an array");
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0]["code"], "input_without_explicit_driver");
    assert_eq!(findings[0]["severity"], "info");
    assert_eq!(
        result["raw_report"]["erc"][0]["code"],
        "input_without_explicit_driver"
    );
}

#[test]
fn run_drc_dispatch_returns_check_run_view_with_raw_board_report() {
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
    assert_eq!(result["contract"], "check_run_v1");
    assert_eq!(result["persisted"], false);
    assert_eq!(result["profile_id"], "drc");
    assert!(result["raw_report"]["drc"]["passed"].is_boolean());
    assert!(result["summary"]["errors"].is_u64());
    assert!(result["summary"]["warnings"].is_u64());
    assert!(result["findings"].is_array());
    assert!(result["raw_report"]["drc"]["violations"].is_array());
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
    assert_eq!(result["raw_report"]["drc"]["passed"], json!(false));
    let violations = result["findings"]
        .as_array()
        .expect("normalized DRC findings should be an array");
    assert!(
        violations
            .iter()
            .any(|violation| violation["code"] == "connectivity_unrouted_net"
                && violation["primary_target"]["kind"] == "object_uuid")
    );
}

#[test]
fn run_drc_dispatch_honors_selected_rule_params() {
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
        params: json!({"rules": ["TrackWidth"]}),
    };

    let response = dispatch_request(&mut engine, request);
    assert!(response.error.is_none(), "{response:?}");
    let result = response.result.expect("result should exist");
    let violations = result["findings"]
        .as_array()
        .expect("normalized DRC findings should be an array");
    assert!(
        violations
            .iter()
            .all(|violation| violation["code"] != "connectivity_unrouted_net")
    );
}

#[test]
fn explain_violation_dispatch_accepts_drc_finding_fingerprint() {
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

    let check_response = dispatch_request(
        &mut engine,
        JsonRpcRequest {
            jsonrpc: "2.0".into(),
            id: json!(2),
            method: "run_drc".into(),
            params: json!({}),
        },
    );
    assert!(check_response.error.is_none(), "{check_response:?}");
    let check_run = check_response.result.expect("check run should exist");
    let fingerprint = check_run["findings"]
        .as_array()
        .unwrap()
        .iter()
        .find(|finding| finding["code"] == "connectivity_unrouted_net")
        .and_then(|finding| finding["fingerprint"].as_str())
        .expect("connectivity finding should carry fingerprint")
        .to_string();

    let response = dispatch_request(
        &mut engine,
        JsonRpcRequest {
            jsonrpc: "2.0".into(),
            id: json!(3),
            method: "explain_violation".into(),
            params: json!({ "domain": "drc", "fingerprint": fingerprint }),
        },
    );
    assert!(response.error.is_none(), "{response:?}");
    let result = response.result.expect("explanation should exist");
    assert_eq!(result["fingerprint"], fingerprint);
    assert_eq!(
        result["rule_detail"],
        "drc connectivity_unrouted_net (error)"
    );
    assert!(result["explanation"].as_str().unwrap().contains("unrouted"));
    assert!(result["objects_involved"].is_array());
}
