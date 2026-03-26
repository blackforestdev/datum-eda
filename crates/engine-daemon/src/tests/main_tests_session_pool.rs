use super::*;

#[test]
fn open_project_dispatch_returns_import_report() {
    let mut engine = Engine::new().expect("engine should initialize");
    let request = JsonRpcRequest {
        jsonrpc: "2.0".into(),
        id: json!(1),
        method: "open_project".into(),
        params: serde_json::to_value(OpenProjectParams {
            path: kicad_fixture_path("simple-demo.kicad_pcb"),
        })
        .unwrap(),
    };

    let response = dispatch_request(&mut engine, request);
    assert!(response.error.is_none(), "{response:?}");
    let result = response.result.expect("result should exist");
    assert_eq!(result["kind"], "kicad_board");
    assert_eq!(result["metadata"]["footprint_count"], "1");
}

#[test]
fn get_check_report_dispatch_returns_board_report_shape() {
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
        method: "get_check_report".into(),
        params: json!({}),
    };

    let response = dispatch_request(&mut engine, request);
    assert!(response.error.is_none(), "{response:?}");
    let result = response.result.expect("result should exist");
    assert_eq!(result["domain"], "board");
    assert_eq!(result["summary"]["status"], "info");
    assert_eq!(result["summary"]["by_code"][0]["code"], "net_without_copper");
}

#[test]
fn get_check_report_dispatch_returns_partial_route_board_report_shape() {
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
        method: "get_check_report".into(),
        params: json!({}),
    };

    let response = dispatch_request(&mut engine, request);
    assert!(response.error.is_none(), "{response:?}");
    let result = response.result.expect("result should exist");
    assert_eq!(result["domain"], "board");
    assert_eq!(result["summary"]["status"], "warning");
    assert!(
        result["summary"]["by_code"]
            .as_array()
            .unwrap()
            .iter()
            .any(|entry| entry["code"] == "partially_routed_net")
    );
    assert!(
        result["diagnostics"]
            .as_array()
            .unwrap()
            .iter()
            .any(|diagnostic| diagnostic["kind"] == "partially_routed_net")
    );
}

#[test]
fn get_board_summary_dispatch_returns_board_summary() {
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
        method: "get_board_summary".into(),
        params: json!({}),
    };

    let response = dispatch_request(&mut engine, request);
    assert!(response.error.is_none(), "{response:?}");
    let result = response.result.expect("result should exist");
    assert_eq!(result["name"], "simple-demo");
    assert_eq!(result["component_count"], 1);
    assert_eq!(result["net_count"], 2);
}

#[test]
fn get_components_dispatch_returns_board_components() {
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
        method: "get_components".into(),
        params: json!({}),
    };

    let response = dispatch_request(&mut engine, request);
    assert!(response.error.is_none(), "{response:?}");
    let result = response.result.expect("result should exist");
    let components = result
        .as_array()
        .expect("components result should be an array");
    assert_eq!(components.len(), 1);
    assert_eq!(components[0]["reference"], "R1");
}

#[test]
fn close_project_dispatch_clears_open_session() {
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

    let close = JsonRpcRequest {
        jsonrpc: "2.0".into(),
        id: json!(2),
        method: "close_project".into(),
        params: json!({}),
    };
    let close_response = dispatch_request(&mut engine, close);
    assert!(close_response.error.is_none(), "{close_response:?}");
    assert_eq!(close_response.result.unwrap()["closed"], true);

    let summary = JsonRpcRequest {
        jsonrpc: "2.0".into(),
        id: json!(3),
        method: "get_board_summary".into(),
        params: json!({}),
    };
    let summary_response = dispatch_request(&mut engine, summary);
    assert!(summary_response.error.is_some());
}

#[test]
fn search_pool_dispatch_returns_index_matches() {
    let mut engine = Engine::new().expect("engine should initialize");
    let open = JsonRpcRequest {
        jsonrpc: "2.0".into(),
        id: json!(1),
        method: "open_project".into(),
        params: serde_json::to_value(OpenProjectParams {
            path: eagle_fixture_path("bjt-sot23.lbr"),
        })
        .unwrap(),
    };
    let _ = dispatch_request(&mut engine, open);

    let search = JsonRpcRequest {
        jsonrpc: "2.0".into(),
        id: json!(2),
        method: "search_pool".into(),
        params: serde_json::to_value(SearchPoolParams {
            query: "sot23".into(),
        })
        .unwrap(),
    };
    let response = dispatch_request(&mut engine, search);
    assert!(response.error.is_none(), "{response:?}");
    let results = response
        .result
        .unwrap()
        .as_array()
        .cloned()
        .expect("search result should be an array");
    assert!(!results.is_empty());
}

#[test]
fn get_part_dispatch_returns_part_details() {
    let mut engine = Engine::new().expect("engine should initialize");
    let open = JsonRpcRequest {
        jsonrpc: "2.0".into(),
        id: json!(1),
        method: "open_project".into(),
        params: serde_json::to_value(OpenProjectParams {
            path: eagle_fixture_path("bjt-sot23.lbr"),
        })
        .unwrap(),
    };
    let _ = dispatch_request(&mut engine, open);

    let search = dispatch_request(
        &mut engine,
        JsonRpcRequest {
            jsonrpc: "2.0".into(),
            id: json!(2),
            method: "search_pool".into(),
            params: serde_json::to_value(SearchPoolParams {
                query: "sot23".into(),
            })
            .unwrap(),
        },
    );
    assert!(search.error.is_none(), "{search:?}");
    let results = search.result.unwrap();
    let list = results
        .as_array()
        .expect("search results should be an array");
    assert!(!list.is_empty(), "search should return at least one part");
    let part_uuid = list[0]["uuid"]
        .as_str()
        .expect("part uuid should be string")
        .to_string();

    let request = JsonRpcRequest {
        jsonrpc: "2.0".into(),
        id: json!(3),
        method: "get_part".into(),
        params: json!({ "uuid": part_uuid }),
    };
    let response = dispatch_request(&mut engine, request);
    assert!(response.error.is_none(), "{response:?}");
    let result = response.result.expect("result should exist");
    assert!(result["mpn"].is_string());
    assert!(result["package"]["name"].is_string());
    assert!(result["package"]["pads"].is_number());
}

#[test]
fn get_package_dispatch_returns_not_found_for_missing_uuid() {
    let mut engine = Engine::new().expect("engine should initialize");
    let open = JsonRpcRequest {
        jsonrpc: "2.0".into(),
        id: json!(1),
        method: "open_project".into(),
        params: serde_json::to_value(OpenProjectParams {
            path: eagle_fixture_path("bjt-sot23.lbr"),
        })
        .unwrap(),
    };
    let _ = dispatch_request(&mut engine, open);

    let request = JsonRpcRequest {
        jsonrpc: "2.0".into(),
        id: json!(2),
        method: "get_package".into(),
        params: json!({ "uuid": uuid::Uuid::new_v4().to_string() }),
    };
    let response = dispatch_request(&mut engine, request);
    assert!(response.error.is_some(), "expected not found error");
}
