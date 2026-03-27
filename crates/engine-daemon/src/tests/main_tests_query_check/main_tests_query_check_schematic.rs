use super::*;

#[test]
fn get_check_report_dispatch_returns_schematic_report_shape() {
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
        method: "get_check_report".into(),
        params: json!({}),
    };

    let response = dispatch_request(&mut engine, request);
    assert!(response.error.is_none(), "{response:?}");
    let result = response.result.expect("result should exist");
    assert_eq!(result["domain"], "schematic");
    assert_eq!(result["summary"]["status"], "warning");
    assert_eq!(result["summary"]["by_code"].as_array().unwrap().len(), 3);
    assert_eq!(result["erc"].as_array().unwrap().len(), 2);
}

#[test]
fn get_check_report_dispatch_includes_input_without_explicit_driver_code() {
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
        method: "get_check_report".into(),
        params: json!({}),
    };

    let response = dispatch_request(&mut engine, request);
    assert!(response.error.is_none(), "{response:?}");
    let result = response.result.expect("result should exist");
    assert_eq!(result["domain"], "schematic");
    assert_eq!(result["summary"]["status"], "info");
    assert!(
        result["summary"]["by_code"]
            .as_array()
            .unwrap()
            .iter()
            .any(|entry| entry["code"] == "input_without_explicit_driver" && entry["count"] == 1)
    );
    assert!(
        result["erc"]
            .as_array()
            .unwrap()
            .iter()
            .any(|finding| finding["code"] == "input_without_explicit_driver")
    );
}

#[test]
fn get_schematic_summary_dispatch_returns_schematic_summary() {
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
        method: "get_schematic_summary".into(),
        params: json!({}),
    };

    let response = dispatch_request(&mut engine, request);
    assert!(response.error.is_none(), "{response:?}");
    let result = response.result.expect("result should exist");
    assert_eq!(result["sheet_count"], 1);
    assert_eq!(result["symbol_count"], 1);
    assert_eq!(result["net_label_count"], 3);
}

#[test]
fn get_sheets_dispatch_returns_schematic_sheets() {
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
        method: "get_sheets".into(),
        params: json!({}),
    };

    let response = dispatch_request(&mut engine, request);
    assert!(response.error.is_none(), "{response:?}");
    let result = response.result.expect("result should exist");
    let sheets = result.as_array().expect("sheets result should be an array");
    assert_eq!(sheets.len(), 1);
    assert_eq!(sheets[0]["name"], "Root");
}

#[test]
fn get_labels_dispatch_returns_schematic_labels() {
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
        method: "get_labels".into(),
        params: json!({}),
    };

    let response = dispatch_request(&mut engine, request);
    assert!(response.error.is_none(), "{response:?}");
    let result = response.result.expect("result should exist");
    let labels = result.as_array().expect("labels result should be an array");
    assert_eq!(labels.len(), 3);
    assert!(labels.iter().any(|label| label["name"] == "SCL"));
    assert!(labels.iter().any(|label| label["name"] == "VCC"));
    assert!(labels.iter().any(|label| label["name"] == "SUB_IN"));
}

#[test]
fn get_ports_dispatch_returns_schematic_ports() {
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
        method: "get_ports".into(),
        params: json!({}),
    };

    let response = dispatch_request(&mut engine, request);
    assert!(response.error.is_none(), "{response:?}");
    let result = response.result.expect("result should exist");
    let ports = result.as_array().expect("ports result should be an array");
    assert_eq!(ports.len(), 1);
    assert_eq!(ports[0]["name"], "SUB_IN");
}

#[test]
fn get_symbols_dispatch_returns_schematic_symbols() {
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
        method: "get_symbols".into(),
        params: json!({}),
    };

    let response = dispatch_request(&mut engine, request);
    assert!(response.error.is_none(), "{response:?}");
    let result = response.result.expect("result should exist");
    let symbols = result
        .as_array()
        .expect("symbols result should be an array");
    assert_eq!(symbols.len(), 1);
    assert_eq!(symbols[0]["reference"], "R1");
    assert_eq!(symbols[0]["value"], "10k");
}

#[test]
fn get_symbol_fields_dispatch_returns_symbol_fields() {
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

    let symbols = dispatch_request(
        &mut engine,
        JsonRpcRequest {
            jsonrpc: "2.0".into(),
            id: json!(2),
            method: "get_symbols".into(),
            params: json!({}),
        },
    );
    let symbol_uuid = symbols.result.unwrap()[0]["uuid"]
        .as_str()
        .expect("uuid should be string")
        .to_string();

    let request = JsonRpcRequest {
        jsonrpc: "2.0".into(),
        id: json!(3),
        method: "get_symbol_fields".into(),
        params: json!({ "symbol_uuid": symbol_uuid }),
    };

    let response = dispatch_request(&mut engine, request);
    assert!(response.error.is_none(), "{response:?}");
    let result = response.result.expect("result should exist");
    let fields = result.as_array().expect("fields result should be an array");
    assert!(!fields.is_empty());
    assert!(fields.iter().any(|field| field["key"] == "Reference"));
}

#[test]
fn get_buses_dispatch_returns_schematic_buses() {
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
        method: "get_buses".into(),
        params: json!({}),
    };

    let response = dispatch_request(&mut engine, request);
    assert!(response.error.is_none(), "{response:?}");
    let result = response.result.expect("result should exist");
    let buses = result.as_array().expect("buses result should be an array");
    assert_eq!(buses.len(), 1);
    assert_eq!(buses[0]["name"], "BUS_77777777-7777-7777-7777-777777777777");
}

#[test]
fn get_bus_entries_dispatch_returns_schematic_bus_entries() {
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
        method: "get_bus_entries".into(),
        params: json!({}),
    };

    let response = dispatch_request(&mut engine, request);
    assert!(response.error.is_none(), "{response:?}");
    let result = response.result.expect("result should exist");
    assert!(result.is_array(), "bus entries result should be an array");
}

#[test]
fn get_noconnects_dispatch_returns_schematic_noconnects() {
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
        method: "get_noconnects".into(),
        params: json!({}),
    };

    let response = dispatch_request(&mut engine, request);
    assert!(response.error.is_none(), "{response:?}");
    let result = response.result.expect("result should exist");
    let noconnects = result
        .as_array()
        .expect("noconnects result should be an array");
    assert_eq!(noconnects.len(), 1);
}

#[test]
fn get_hierarchy_dispatch_returns_schematic_hierarchy() {
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
        method: "get_hierarchy".into(),
        params: json!({}),
    };

    let response = dispatch_request(&mut engine, request);
    assert!(response.error.is_none(), "{response:?}");
    let result = response.result.expect("result should exist");
    assert_eq!(result["instances"].as_array().unwrap().len(), 1);
    assert_eq!(result["links"].as_array().unwrap().len(), 0);
    assert_eq!(result["instances"][0]["name"], "Sub");
}

#[test]
fn get_schematic_net_info_dispatch_returns_schematic_nets() {
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
        method: "get_schematic_net_info".into(),
        params: json!({}),
    };

    let response = dispatch_request(&mut engine, request);
    assert!(response.error.is_none(), "{response:?}");
    let result = response.result.expect("result should exist");
    let nets = result.as_array().expect("nets result should be an array");
    assert_eq!(nets.len(), 4);
    assert!(nets.iter().any(|net| net["name"] == "SCL"));
    assert!(nets.iter().any(|net| net["name"] == "VCC"));
}

#[test]
fn get_netlist_dispatch_returns_schematic_nets() {
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
        method: "get_netlist".into(),
        params: json!({}),
    };

    let response = dispatch_request(&mut engine, request);
    assert!(response.error.is_none(), "{response:?}");
    let result = response.result.expect("result should exist");
    let nets = result.as_array().expect("nets result should be an array");
    assert_eq!(nets.len(), 4);
    assert!(nets.iter().any(|net| net["name"] == "VCC"));
    assert!(
        nets.iter()
            .all(|net| net["routed_pct"].is_null() && net["labels"].is_number())
    );
}
