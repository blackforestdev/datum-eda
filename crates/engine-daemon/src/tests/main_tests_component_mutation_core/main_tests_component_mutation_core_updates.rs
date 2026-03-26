use super::*;

#[test]
fn move_component_dispatch_updates_component_position() {
    let mut engine = Engine::new().expect("engine should initialize");
    let fixture = kicad_fixture_path("partial-route-demo.kicad_pcb");
    let open = JsonRpcRequest {
        jsonrpc: "2.0".into(),
        id: json!(1),
        method: "open_project".into(),
        params: serde_json::to_value(OpenProjectParams { path: fixture }).unwrap(),
    };
    let open_response = dispatch_request(&mut engine, open);
    assert!(open_response.error.is_none(), "{open_response:?}");

    let move_component = JsonRpcRequest {
        jsonrpc: "2.0".into(),
        id: json!(2),
        method: "move_component".into(),
        params: json!({
            "uuid": "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa",
            "x_mm": 15.0,
            "y_mm": 12.0,
            "rotation_deg": 90.0
        }),
    };
    let response = dispatch_request(&mut engine, move_component);
    assert!(response.error.is_none(), "{response:?}");
    assert_eq!(
        response.result.expect("result should exist")["diff"]["modified"][0]["object_type"],
        "component"
    );
    let components = engine.get_components().expect("components should query");
    let moved = components
        .iter()
        .find(|component| component.reference == "R1")
        .unwrap();
    assert_eq!(moved.position.x, 15_000_000);
    assert_eq!(moved.position.y, 12_000_000);
    assert_eq!(moved.rotation, 90);
}

#[test]
fn rotate_component_dispatch_updates_component_rotation() {
    let mut engine = Engine::new().expect("engine should initialize");
    let fixture = kicad_fixture_path("partial-route-demo.kicad_pcb");
    let open = JsonRpcRequest {
        jsonrpc: "2.0".into(),
        id: json!(1),
        method: "open_project".into(),
        params: serde_json::to_value(OpenProjectParams { path: fixture }).unwrap(),
    };
    let open_response = dispatch_request(&mut engine, open);
    assert!(open_response.error.is_none(), "{open_response:?}");

    let rotate_component = JsonRpcRequest {
        jsonrpc: "2.0".into(),
        id: json!(2),
        method: "rotate_component".into(),
        params: json!({
            "uuid": "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa",
            "x_mm": 10.0,
            "y_mm": 10.0,
            "rotation_deg": 180.0
        }),
    };
    let response = dispatch_request(&mut engine, rotate_component);
    assert!(response.error.is_none(), "{response:?}");
    let components = engine.get_components().expect("components should query");
    let rotated = components
        .iter()
        .find(|component| component.reference == "R1")
        .unwrap();
    assert_eq!(rotated.rotation, 180);
}

#[test]
fn set_value_dispatch_updates_component_value() {
    let mut engine = Engine::new().expect("engine should initialize");
    let fixture = kicad_fixture_path("partial-route-demo.kicad_pcb");
    let open = JsonRpcRequest {
        jsonrpc: "2.0".into(),
        id: json!(1),
        method: "open_project".into(),
        params: serde_json::to_value(OpenProjectParams { path: fixture }).unwrap(),
    };
    let open_response = dispatch_request(&mut engine, open);
    assert!(open_response.error.is_none(), "{open_response:?}");

    let set_value = JsonRpcRequest {
        jsonrpc: "2.0".into(),
        id: json!(2),
        method: "set_value".into(),
        params: json!({
            "uuid": "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa",
            "value": "22k"
        }),
    };
    let response = dispatch_request(&mut engine, set_value);
    assert!(response.error.is_none(), "{response:?}");
    assert_eq!(
        response.result.expect("result should exist")["diff"]["modified"][0]["object_type"],
        "component"
    );
    let components = engine.get_components().expect("components should query");
    let updated = components
        .iter()
        .find(|component| component.reference == "R1")
        .unwrap();
    assert_eq!(updated.value, "22k");
}

#[test]
fn set_value_dispatch_updates_followup_components_query() {
    let mut engine = Engine::new().expect("engine should initialize");
    let fixture = kicad_fixture_path("partial-route-demo.kicad_pcb");
    let open = JsonRpcRequest {
        jsonrpc: "2.0".into(),
        id: json!(1),
        method: "open_project".into(),
        params: serde_json::to_value(OpenProjectParams { path: fixture }).unwrap(),
    };
    let open_response = dispatch_request(&mut engine, open);
    assert!(open_response.error.is_none(), "{open_response:?}");

    let baseline = dispatch_request(
        &mut engine,
        JsonRpcRequest {
            jsonrpc: "2.0".into(),
            id: json!(2),
            method: "get_components".into(),
            params: json!({}),
        },
    );
    assert!(baseline.error.is_none(), "{baseline:?}");
    let baseline_components = baseline.result.expect("result should exist");
    let baseline_r1 = baseline_components
        .as_array()
        .unwrap()
        .iter()
        .find(|component| component["reference"] == "R1")
        .expect("R1 should exist");
    assert_eq!(baseline_r1["value"], "10k");

    let set_value = dispatch_request(
        &mut engine,
        JsonRpcRequest {
            jsonrpc: "2.0".into(),
            id: json!(3),
            method: "set_value".into(),
            params: json!({
                "uuid": "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa",
                "value": "22k"
            }),
        },
    );
    assert!(set_value.error.is_none(), "{set_value:?}");

    let after = dispatch_request(
        &mut engine,
        JsonRpcRequest {
            jsonrpc: "2.0".into(),
            id: json!(4),
            method: "get_components".into(),
            params: json!({}),
        },
    );
    assert!(after.error.is_none(), "{after:?}");
    let after_components = after.result.expect("result should exist");
    let after_r1 = after_components
        .as_array()
        .unwrap()
        .iter()
        .find(|component| component["reference"] == "R1")
        .expect("R1 should exist");
    assert_eq!(after_r1["value"], "22k");
}

#[test]
fn set_reference_dispatch_updates_component_reference() {
    let mut engine = Engine::new().expect("engine should initialize");
    let fixture = kicad_fixture_path("partial-route-demo.kicad_pcb");
    let open = JsonRpcRequest {
        jsonrpc: "2.0".into(),
        id: json!(1),
        method: "open_project".into(),
        params: serde_json::to_value(OpenProjectParams { path: fixture }).unwrap(),
    };
    let open_response = dispatch_request(&mut engine, open);
    assert!(open_response.error.is_none(), "{open_response:?}");

    let set_reference = JsonRpcRequest {
        jsonrpc: "2.0".into(),
        id: json!(2),
        method: "set_reference".into(),
        params: json!({
            "uuid": "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa",
            "reference": "R10"
        }),
    };
    let response = dispatch_request(&mut engine, set_reference);
    assert!(response.error.is_none(), "{response:?}");
    assert_eq!(
        response.result.expect("result should exist")["diff"]["modified"][0]["object_type"],
        "component"
    );
    let components = engine.get_components().expect("components should query");
    let updated = components
        .iter()
        .find(|component| {
            component.uuid == uuid::Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap()
        })
        .unwrap();
    assert_eq!(updated.reference, "R10");
}

#[test]
fn set_reference_dispatch_updates_followup_components_query() {
    let mut engine = Engine::new().expect("engine should initialize");
    let fixture = kicad_fixture_path("partial-route-demo.kicad_pcb");
    let open = JsonRpcRequest {
        jsonrpc: "2.0".into(),
        id: json!(1),
        method: "open_project".into(),
        params: serde_json::to_value(OpenProjectParams { path: fixture }).unwrap(),
    };
    let open_response = dispatch_request(&mut engine, open);
    assert!(open_response.error.is_none(), "{open_response:?}");

    let baseline = dispatch_request(
        &mut engine,
        JsonRpcRequest {
            jsonrpc: "2.0".into(),
            id: json!(2),
            method: "get_components".into(),
            params: json!({}),
        },
    );
    assert!(baseline.error.is_none(), "{baseline:?}");
    let baseline_components = baseline.result.expect("result should exist");
    let baseline_r1 = baseline_components
        .as_array()
        .unwrap()
        .iter()
        .find(|component| component["reference"] == "R1")
        .expect("R1 should exist");
    assert_eq!(baseline_r1["uuid"], "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa");

    let set_reference = dispatch_request(
        &mut engine,
        JsonRpcRequest {
            jsonrpc: "2.0".into(),
            id: json!(3),
            method: "set_reference".into(),
            params: json!({
                "uuid": "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa",
                "reference": "R10"
            }),
        },
    );
    assert!(set_reference.error.is_none(), "{set_reference:?}");

    let after = dispatch_request(
        &mut engine,
        JsonRpcRequest {
            jsonrpc: "2.0".into(),
            id: json!(4),
            method: "get_components".into(),
            params: json!({}),
        },
    );
    assert!(after.error.is_none(), "{after:?}");
    let after_components = after.result.expect("result should exist");
    let after_r1 = after_components
        .as_array()
        .unwrap()
        .iter()
        .find(|component| component["uuid"] == "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa")
        .expect("component should exist");
    assert_eq!(after_r1["reference"], "R10");
}
