use super::*;
use std::io::Write;
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::thread;

#[test]
fn set_net_class_dispatch_updates_net_class() {
    let mut engine = Engine::new().expect("engine should initialize");
    let _ = dispatch_request(
        &mut engine,
        JsonRpcRequest {
            jsonrpc: "2.0".into(),
            id: json!(1),
            method: "open_project".into(),
            params: serde_json::to_value(OpenProjectParams {
                path: kicad_fixture_path("simple-demo.kicad_pcb"),
            })
            .unwrap(),
        },
    );
    let net_uuid = engine
        .get_net_info()
        .expect("net info should query")
        .into_iter()
        .find(|net| net.name == "GND")
        .expect("GND net should exist")
        .uuid;
    let response = dispatch_request(
        &mut engine,
        JsonRpcRequest {
            jsonrpc: "2.0".into(),
            id: json!(2),
            method: "set_net_class".into(),
            params: json!({
                "net_uuid": net_uuid,
                "class_name": "power",
                "clearance": 125000,
                "track_width": 250000,
                "via_drill": 300000,
                "via_diameter": 600000,
                "diffpair_width": 0,
                "diffpair_gap": 0,
            }),
        },
    );
    assert!(response.error.is_none(), "{response:?}");
    let updated = engine
        .get_net_info()
        .expect("net info should query")
        .into_iter()
        .find(|net| net.uuid == net_uuid)
        .expect("updated net should exist");
    assert_eq!(updated.class, "power");
}

#[test]
fn set_net_class_dispatch_updates_followup_net_info_query() {
    let mut engine = Engine::new().expect("engine should initialize");
    let _ = dispatch_request(
        &mut engine,
        JsonRpcRequest {
            jsonrpc: "2.0".into(),
            id: json!(1),
            method: "open_project".into(),
            params: serde_json::to_value(OpenProjectParams {
                path: kicad_fixture_path("simple-demo.kicad_pcb"),
            })
            .unwrap(),
        },
    );
    let baseline = dispatch_request(
        &mut engine,
        JsonRpcRequest {
            jsonrpc: "2.0".into(),
            id: json!(2),
            method: "get_net_info".into(),
            params: json!({}),
        },
    );
    let baseline_gnd = baseline
        .result
        .as_ref()
        .unwrap()
        .as_array()
        .unwrap()
        .iter()
        .find(|net| net["name"] == "GND")
        .expect("baseline GND should exist");
    let net_uuid = uuid::Uuid::parse_str(baseline_gnd["uuid"].as_str().unwrap()).unwrap();
    assert_eq!(baseline_gnd["class"], "Default");

    let set = dispatch_request(
        &mut engine,
        JsonRpcRequest {
            jsonrpc: "2.0".into(),
            id: json!(3),
            method: "set_net_class".into(),
            params: json!({
                "net_uuid": net_uuid,
                "class_name": "power",
                "clearance": 125000,
                "track_width": 250000,
                "via_drill": 300000,
                "via_diameter": 600000,
                "diffpair_width": 0,
                "diffpair_gap": 0,
            }),
        },
    );
    assert!(set.error.is_none(), "{set:?}");

    let after = dispatch_request(
        &mut engine,
        JsonRpcRequest {
            jsonrpc: "2.0".into(),
            id: json!(4),
            method: "get_net_info".into(),
            params: json!({}),
        },
    );
    let after_gnd = after
        .result
        .as_ref()
        .unwrap()
        .as_array()
        .unwrap()
        .iter()
        .find(|net| net["uuid"] == net_uuid.to_string())
        .expect("updated GND should exist");
    assert_eq!(after_gnd["class"], "power");
}

#[test]
fn unknown_method_returns_json_rpc_error() {
    let mut engine = Engine::new().expect("engine should initialize");
    let request = JsonRpcRequest {
        jsonrpc: "2.0".into(),
        id: json!(1),
        method: "bogus".into(),
        params: json!({}),
    };

    let response = dispatch_request(&mut engine, request);
    assert!(response.result.is_none());
    let error = response.error.expect("error should exist");
    assert_eq!(error.code, -32601);
}

#[test]
#[ignore = "sandboxed test environment denies socket-pair writes"]
fn handle_client_round_trips_open_project_and_get_check_report() {
    let (client, server) = UnixStream::pair().expect("socket pair should open");
    let handle = thread::spawn(move || {
        let mut engine = Engine::new().expect("engine should initialize");
        handle_client(&mut engine, server).expect("client handling should succeed");
    });

    let open = JsonRpcRequest {
        jsonrpc: "2.0".into(),
        id: json!(1),
        method: "open_project".into(),
        params: serde_json::to_value(OpenProjectParams {
            path: kicad_fixture_path("simple-demo.kicad_sch"),
        })
        .unwrap(),
    };

    let mut writer = client.try_clone().expect("client clone should succeed");
    writer
        .write_all(format!("{}\n", open.to_json().unwrap()).as_bytes())
        .expect("open_project should write");
    let open_response = read_json_line(client.try_clone().expect("clone should succeed"));
    assert!(open_response.error.is_none(), "{open_response:?}");

    let report = JsonRpcRequest {
        jsonrpc: "2.0".into(),
        id: json!(2),
        method: "get_check_report".into(),
        params: json!({}),
    };
    writer
        .write_all(format!("{}\n", report.to_json().unwrap()).as_bytes())
        .expect("get_check_report should write");
    let report_response = read_json_line(client);
    assert!(report_response.error.is_none(), "{report_response:?}");
    let result = report_response.result.expect("report result should exist");
    assert_eq!(result["domain"], "schematic");
    assert_eq!(result["summary"]["status"], "warning");

    drop(writer);
    handle.join().expect("daemon thread should join");
}

#[test]
fn parse_socket_arg_extracts_socket_path() {
    let _unused = temp_socket_path("eda-engine-daemon-test");
    let parsed = {
        let args = ["eda-engine-daemon", "--socket", "/tmp/eda.sock"];
        let mut iter = args.into_iter().skip(1);
        let mut found = None;
        while let Some(arg) = iter.next() {
            if arg == "--socket" {
                found = iter.next().map(PathBuf::from);
                break;
            }
        }
        found
    };
    assert_eq!(parsed, Some(PathBuf::from("/tmp/eda.sock")));
}
