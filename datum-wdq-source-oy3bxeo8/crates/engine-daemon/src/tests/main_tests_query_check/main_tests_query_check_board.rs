use super::*;

#[test]
fn get_net_info_dispatch_returns_board_nets() {
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
        method: "get_net_info".into(),
        params: json!({}),
    };

    let response = dispatch_request(&mut engine, request);
    assert!(response.error.is_none(), "{response:?}");
    let result = response.result.expect("result should exist");
    let nets = result.as_array().expect("nets result should be an array");
    assert_eq!(nets.len(), 2);
    assert!(nets.iter().any(|net| net["name"] == "GND"));
    assert!(nets.iter().any(|net| net["name"] == ""));
}

#[test]
fn get_netlist_dispatch_returns_board_nets() {
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
        method: "get_netlist".into(),
        params: json!({}),
    };

    let response = dispatch_request(&mut engine, request);
    assert!(response.error.is_none(), "{response:?}");
    let result = response.result.expect("result should exist");
    let nets = result.as_array().expect("nets result should be an array");
    assert_eq!(nets.len(), 2);
    assert!(nets.iter().any(|net| net["name"] == "GND"));
    assert!(nets.iter().all(|net| net["routed_pct"].is_number()));
}

#[test]
fn get_unrouted_dispatch_returns_board_airwires() {
    let mut engine = Engine::new().expect("engine should initialize");
    let open = JsonRpcRequest {
        jsonrpc: "2.0".into(),
        id: json!(1),
        method: "open_project".into(),
        params: serde_json::to_value(OpenProjectParams {
            path: kicad_fixture_path("airwire-demo.kicad_pcb"),
        })
        .unwrap(),
    };
    let _ = dispatch_request(&mut engine, open);

    let request = JsonRpcRequest {
        jsonrpc: "2.0".into(),
        id: json!(2),
        method: "get_unrouted".into(),
        params: json!({}),
    };

    let response = dispatch_request(&mut engine, request);
    assert!(response.error.is_none(), "{response:?}");
    let result = response.result.expect("result should exist");
    let airwires = result
        .as_array()
        .expect("airwire result should be an array");
    assert_eq!(airwires.len(), 1);
    assert_eq!(airwires[0]["net_name"], "SIG");
    assert_eq!(airwires[0]["from"]["component"], "R1");
    assert_eq!(airwires[0]["to"]["component"], "R2");
}
