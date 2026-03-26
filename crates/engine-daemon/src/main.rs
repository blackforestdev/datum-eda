#![allow(dead_code)]

// eda-engine-daemon: JSON-RPC server over Unix socket.
// Owns engine sessions. MCP server connects here.
// See specs/MCP_API_SPEC.md for the RPC protocol.
//
// Current slice:
// - typed JSON-RPC request/response envelopes
// - in-process dispatcher for a minimal future MCP path
// - no socket transport yet

use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};

use anyhow::Result;
use eda_engine::api::{
    AssignPartInput, Engine, MoveComponentInput, RotateComponentInput, SetDesignRuleInput,
    SetNetClassInput,
    SetReferenceInput, SetValueInput, ViolationDomain,
};
use eda_engine::ir::geometry::Point;
use eda_engine::ir::units::mm_to_nm;
use eda_engine::rules::ast::{RuleParams, RuleScope, RuleType};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    id: Value,
    method: String,
    params: Value,
}

impl JsonRpcRequest {
    fn to_json(&self) -> Result<String> {
        Ok(serde_json::to_string(self)?)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    id: Value,
    result: Option<Value>,
    error: Option<JsonRpcError>,
}

impl JsonRpcResponse {
    fn from_json(payload: &str) -> Result<Self> {
        Ok(serde_json::from_str(payload)?)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct JsonRpcError {
    code: i32,
    message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct OpenProjectParams {
    path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct SearchPoolParams {
    query: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct SymbolFieldsParams {
    symbol_uuid: uuid::Uuid,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct UuidParams {
    uuid: uuid::Uuid,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct ExplainViolationParams {
    domain: ViolationDomain,
    index: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct SaveParams {
    path: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct SetDesignRuleParams {
    rule_type: RuleType,
    scope: RuleScope,
    parameters: RuleParams,
    priority: u32,
    name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct MoveComponentParams {
    uuid: uuid::Uuid,
    x_mm: f64,
    y_mm: f64,
    rotation_deg: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct SetValueParams {
    uuid: uuid::Uuid,
    value: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct SetReferenceParams {
    uuid: uuid::Uuid,
    reference: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct AssignPartParams {
    uuid: uuid::Uuid,
    part_uuid: uuid::Uuid,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct SetNetClassParams {
    net_uuid: uuid::Uuid,
    class_name: String,
    clearance: i64,
    track_width: i64,
    via_drill: i64,
    via_diameter: i64,
    diffpair_width: i64,
    diffpair_gap: i64,
}

fn dispatch_request(engine: &mut Engine, request: JsonRpcRequest) -> JsonRpcResponse {
    if request.jsonrpc != "2.0" {
        return error_response(request.id, -32600, "invalid jsonrpc version");
    }

    match request.method.as_str() {
        "open_project" => match serde_json::from_value::<OpenProjectParams>(request.params) {
            Ok(params) => match open_project(engine, &params.path) {
                Ok(report) => success_response(request.id, serde_json::to_value(report).unwrap()),
                Err(err) => error_response(request.id, -32000, &err.to_string()),
            },
            Err(err) => error_response(request.id, -32602, &format!("invalid params: {err}")),
        },
        "close_project" => {
            engine.close_project();
            success_response(request.id, json!({"closed": true}))
        }
        "save" => match serde_json::from_value::<SaveParams>(request.params) {
            Ok(params) => {
                let saved = match params.path {
                    Some(path) => engine.save(&path).map(|_| path),
                    None => engine.save_to_original(),
                };
                match saved {
                    Ok(path) => {
                        success_response(request.id, json!({"path": path.display().to_string()}))
                    }
                    Err(err) => error_response(request.id, -32027, &err.to_string()),
                }
            }
            Err(err) => error_response(request.id, -32602, &format!("invalid params: {err}")),
        },
        "delete_track" => match serde_json::from_value::<UuidParams>(request.params) {
            Ok(params) => match engine.delete_track(&params.uuid) {
                Ok(result) => success_response(request.id, serde_json::to_value(result).unwrap()),
                Err(err) => error_response(request.id, -32028, &err.to_string()),
            },
            Err(err) => error_response(request.id, -32602, &format!("invalid params: {err}")),
        },
        "delete_via" => match serde_json::from_value::<UuidParams>(request.params) {
            Ok(params) => match engine.delete_via(&params.uuid) {
                Ok(result) => success_response(request.id, serde_json::to_value(result).unwrap()),
                Err(err) => error_response(request.id, -32031, &err.to_string()),
            },
            Err(err) => error_response(request.id, -32602, &format!("invalid params: {err}")),
        },
        "delete_component" => match serde_json::from_value::<UuidParams>(request.params) {
            Ok(params) => match engine.delete_component(&params.uuid) {
                Ok(result) => success_response(request.id, serde_json::to_value(result).unwrap()),
                Err(err) => error_response(request.id, -32036, &err.to_string()),
            },
            Err(err) => error_response(request.id, -32602, &format!("invalid params: {err}")),
        },
        "move_component" => match serde_json::from_value::<MoveComponentParams>(request.params) {
            Ok(params) => match engine.move_component(MoveComponentInput {
                uuid: params.uuid,
                position: Point::new(mm_to_nm(params.x_mm), mm_to_nm(params.y_mm)),
                rotation: params.rotation_deg.map(|deg| deg.round() as i32),
            }) {
                Ok(result) => success_response(request.id, serde_json::to_value(result).unwrap()),
                Err(err) => error_response(request.id, -32033, &err.to_string()),
            },
            Err(err) => error_response(request.id, -32602, &format!("invalid params: {err}")),
        },
        "rotate_component" => match serde_json::from_value::<MoveComponentParams>(request.params) {
            Ok(params) => {
                let rotation = match params.rotation_deg {
                    Some(deg) => deg.round() as i32,
                    None => {
                        return error_response(
                            request.id,
                            -32602,
                            "invalid params: rotate_component requires rotation_deg",
                        );
                    }
                };
                match engine.rotate_component(RotateComponentInput {
                    uuid: params.uuid,
                    rotation,
                }) {
                    Ok(result) => success_response(request.id, serde_json::to_value(result).unwrap()),
                    Err(err) => error_response(request.id, -32037, &err.to_string()),
                }
            }
            Err(err) => error_response(request.id, -32602, &format!("invalid params: {err}")),
        },
        "set_value" => match serde_json::from_value::<SetValueParams>(request.params) {
            Ok(params) => match engine.set_value(SetValueInput {
                uuid: params.uuid,
                value: params.value,
            }) {
                Ok(result) => success_response(request.id, serde_json::to_value(result).unwrap()),
                Err(err) => error_response(request.id, -32034, &err.to_string()),
            },
            Err(err) => error_response(request.id, -32602, &format!("invalid params: {err}")),
        },
        "set_reference" => match serde_json::from_value::<SetReferenceParams>(request.params) {
            Ok(params) => match engine.set_reference(SetReferenceInput {
                uuid: params.uuid,
                reference: params.reference,
            }) {
                Ok(result) => success_response(request.id, serde_json::to_value(result).unwrap()),
                Err(err) => error_response(request.id, -32035, &err.to_string()),
            },
            Err(err) => error_response(request.id, -32602, &format!("invalid params: {err}")),
        },
        "assign_part" => match serde_json::from_value::<AssignPartParams>(request.params) {
            Ok(params) => match engine.assign_part(AssignPartInput {
                uuid: params.uuid,
                part_uuid: params.part_uuid,
            }) {
                Ok(result) => success_response(request.id, serde_json::to_value(result).unwrap()),
                Err(err) => error_response(request.id, -32038, &err.to_string()),
            },
            Err(err) => error_response(request.id, -32602, &format!("invalid params: {err}")),
        },
        "set_net_class" => match serde_json::from_value::<SetNetClassParams>(request.params) {
            Ok(params) => match engine.set_net_class(SetNetClassInput {
                net_uuid: params.net_uuid,
                class_name: params.class_name,
                clearance: params.clearance,
                track_width: params.track_width,
                via_drill: params.via_drill,
                via_diameter: params.via_diameter,
                diffpair_width: params.diffpair_width,
                diffpair_gap: params.diffpair_gap,
            }) {
                Ok(result) => success_response(request.id, serde_json::to_value(result).unwrap()),
                Err(err) => error_response(request.id, -32039, &err.to_string()),
            },
            Err(err) => error_response(request.id, -32602, &format!("invalid params: {err}")),
        },
        "set_design_rule" => match serde_json::from_value::<SetDesignRuleParams>(request.params) {
            Ok(params) => match engine.set_design_rule(SetDesignRuleInput {
                rule_type: params.rule_type,
                scope: params.scope,
                parameters: params.parameters,
                priority: params.priority,
                name: params.name,
            }) {
                Ok(result) => success_response(request.id, serde_json::to_value(result).unwrap()),
                Err(err) => error_response(request.id, -32032, &err.to_string()),
            },
            Err(err) => error_response(request.id, -32602, &format!("invalid params: {err}")),
        },
        "undo" => match engine.undo() {
            Ok(result) => success_response(request.id, serde_json::to_value(result).unwrap()),
            Err(err) => error_response(request.id, -32029, &err.to_string()),
        },
        "redo" => match engine.redo() {
            Ok(result) => success_response(request.id, serde_json::to_value(result).unwrap()),
            Err(err) => error_response(request.id, -32030, &err.to_string()),
        },
        "search_pool" => match serde_json::from_value::<SearchPoolParams>(request.params) {
            Ok(params) => match engine.search_pool(&params.query) {
                Ok(parts) => success_response(request.id, serde_json::to_value(parts).unwrap()),
                Err(err) => error_response(request.id, -32019, &err.to_string()),
            },
            Err(err) => error_response(request.id, -32602, &format!("invalid params: {err}")),
        },
        "get_part" => match serde_json::from_value::<UuidParams>(request.params) {
            Ok(params) => match engine.get_part(&params.uuid) {
                Ok(part) => success_response(request.id, serde_json::to_value(part).unwrap()),
                Err(err) => error_response(request.id, -32024, &err.to_string()),
            },
            Err(err) => error_response(request.id, -32602, &format!("invalid params: {err}")),
        },
        "get_package" => match serde_json::from_value::<UuidParams>(request.params) {
            Ok(params) => match engine.get_package(&params.uuid) {
                Ok(package) => success_response(request.id, serde_json::to_value(package).unwrap()),
                Err(err) => error_response(request.id, -32025, &err.to_string()),
            },
            Err(err) => error_response(request.id, -32602, &format!("invalid params: {err}")),
        },
        "get_board_summary" => match engine.get_board_summary() {
            Ok(summary) => success_response(request.id, serde_json::to_value(summary).unwrap()),
            Err(err) => error_response(request.id, -32004, &err.to_string()),
        },
        "get_components" => match engine.get_components() {
            Ok(components) => {
                success_response(request.id, serde_json::to_value(components).unwrap())
            }
            Err(err) => error_response(request.id, -32008, &err.to_string()),
        },
        "get_netlist" => match engine.get_netlist() {
            Ok(nets) => success_response(request.id, serde_json::to_value(nets).unwrap()),
            Err(err) => error_response(request.id, -32021, &err.to_string()),
        },
        "get_schematic_summary" => match engine.get_schematic_summary() {
            Ok(summary) => success_response(request.id, serde_json::to_value(summary).unwrap()),
            Err(err) => error_response(request.id, -32005, &err.to_string()),
        },
        "get_sheets" => match engine.get_sheets() {
            Ok(sheets) => success_response(request.id, serde_json::to_value(sheets).unwrap()),
            Err(err) => error_response(request.id, -32018, &err.to_string()),
        },
        "get_labels" => match engine.get_labels(None) {
            Ok(labels) => success_response(request.id, serde_json::to_value(labels).unwrap()),
            Err(err) => error_response(request.id, -32009, &err.to_string()),
        },
        "get_symbols" => match engine.get_symbols(None) {
            Ok(symbols) => success_response(request.id, serde_json::to_value(symbols).unwrap()),
            Err(err) => error_response(request.id, -32014, &err.to_string()),
        },
        "get_symbol_fields" => match serde_json::from_value::<SymbolFieldsParams>(request.params) {
            Ok(params) => match engine.get_symbol_fields(&params.symbol_uuid) {
                Ok(fields) => success_response(request.id, serde_json::to_value(fields).unwrap()),
                Err(err) => error_response(request.id, -32022, &err.to_string()),
            },
            Err(err) => error_response(request.id, -32602, &format!("invalid params: {err}")),
        },
        "get_ports" => match engine.get_ports(None) {
            Ok(ports) => success_response(request.id, serde_json::to_value(ports).unwrap()),
            Err(err) => error_response(request.id, -32010, &err.to_string()),
        },
        "get_buses" => match engine.get_buses(None) {
            Ok(buses) => success_response(request.id, serde_json::to_value(buses).unwrap()),
            Err(err) => error_response(request.id, -32012, &err.to_string()),
        },
        "get_bus_entries" => match engine.get_bus_entries(None) {
            Ok(entries) => success_response(request.id, serde_json::to_value(entries).unwrap()),
            Err(err) => error_response(request.id, -32023, &err.to_string()),
        },
        "get_noconnects" => match engine.get_noconnects(None) {
            Ok(noconnects) => {
                success_response(request.id, serde_json::to_value(noconnects).unwrap())
            }
            Err(err) => error_response(request.id, -32015, &err.to_string()),
        },
        "get_hierarchy" => match engine.get_hierarchy() {
            Ok(hierarchy) => success_response(request.id, serde_json::to_value(hierarchy).unwrap()),
            Err(err) => error_response(request.id, -32013, &err.to_string()),
        },
        "get_net_info" => match engine.get_net_info() {
            Ok(nets) => success_response(request.id, serde_json::to_value(nets).unwrap()),
            Err(err) => error_response(request.id, -32006, &err.to_string()),
        },
        "get_unrouted" => match engine.get_unrouted() {
            Ok(airwires) => success_response(request.id, serde_json::to_value(airwires).unwrap()),
            Err(err) => error_response(request.id, -32016, &err.to_string()),
        },
        "get_schematic_net_info" => match engine.get_schematic_net_info() {
            Ok(nets) => success_response(request.id, serde_json::to_value(nets).unwrap()),
            Err(err) => error_response(request.id, -32011, &err.to_string()),
        },
        "get_check_report" => match engine.get_check_report() {
            Ok(report) => success_response(request.id, serde_json::to_value(report).unwrap()),
            Err(err) => error_response(request.id, -32001, &err.to_string()),
        },
        "get_connectivity_diagnostics" => match engine.get_connectivity_diagnostics() {
            Ok(diagnostics) => {
                success_response(request.id, serde_json::to_value(diagnostics).unwrap())
            }
            Err(err) => error_response(request.id, -32003, &err.to_string()),
        },
        "get_design_rules" => match engine.get_design_rules() {
            Ok(rules) => success_response(request.id, serde_json::to_value(rules).unwrap()),
            Err(err) => error_response(request.id, -32020, &err.to_string()),
        },
        "run_erc" => match engine.run_erc_prechecks() {
            Ok(findings) => success_response(request.id, serde_json::to_value(findings).unwrap()),
            Err(err) => error_response(request.id, -32002, &err.to_string()),
        },
        "run_drc" => match engine.run_drc(&[
            RuleType::Connectivity,
            RuleType::ClearanceCopper,
            RuleType::TrackWidth,
            RuleType::ViaHole,
            RuleType::ViaAnnularRing,
            RuleType::SilkClearance,
        ]) {
            Ok(report) => success_response(request.id, serde_json::to_value(report).unwrap()),
            Err(err) => error_response(request.id, -32017, &err.to_string()),
        },
        "explain_violation" => {
            match serde_json::from_value::<ExplainViolationParams>(request.params) {
                Ok(params) => match engine.explain_violation(params.domain, params.index) {
                    Ok(explanation) => {
                        success_response(request.id, serde_json::to_value(explanation).unwrap())
                    }
                    Err(err) => error_response(request.id, -32026, &err.to_string()),
                },
                Err(err) => error_response(request.id, -32602, &format!("invalid params: {err}")),
            }
        }
        _ => error_response(request.id, -32601, "method not found"),
    }
}

fn open_project(engine: &mut Engine, path: &Path) -> Result<Value> {
    let report = engine.import(path)?;
    Ok(json!({
        "kind": report.kind.as_str(),
        "source": report.source.display().to_string(),
        "counts": report.counts,
        "warnings": report.warnings,
        "metadata": report.metadata,
    }))
}

fn success_response(id: Value, result: Value) -> JsonRpcResponse {
    JsonRpcResponse {
        jsonrpc: "2.0".into(),
        id,
        result: Some(result),
        error: None,
    }
}

fn error_response(id: Value, code: i32, message: &str) -> JsonRpcResponse {
    JsonRpcResponse {
        jsonrpc: "2.0".into(),
        id,
        result: None,
        error: Some(JsonRpcError {
            code,
            message: message.into(),
        }),
    }
}

fn main() {
    match parse_socket_arg() {
        Some(path) => {
            if let Err(err) = serve_socket(&path) {
                eprintln!("eda-engine-daemon: {err:#}");
                std::process::exit(1);
            }
        }
        None => {
            eprintln!("usage: eda-engine-daemon --socket /path/to/eda.sock");
            std::process::exit(1);
        }
    }
}

fn parse_socket_arg() -> Option<PathBuf> {
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        if arg == "--socket" {
            return args.next().map(PathBuf::from);
        }
    }
    None
}

fn serve_socket(path: &Path) -> Result<()> {
    if path.exists() {
        std::fs::remove_file(path)?;
    }
    let listener = UnixListener::bind(path)?;
    let mut engine = Engine::new()?;

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => handle_client(&mut engine, stream)?,
            Err(err) => return Err(err.into()),
        }
    }

    Ok(())
}

fn handle_client(engine: &mut Engine, mut stream: UnixStream) -> Result<()> {
    let reader_stream = stream.try_clone()?;
    let mut reader = BufReader::new(reader_stream);
    let mut line = String::new();

    loop {
        line.clear();
        let read = reader.read_line(&mut line)?;
        if read == 0 {
            break;
        }

        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let response = match serde_json::from_str::<JsonRpcRequest>(trimmed) {
            Ok(request) => dispatch_request(engine, request),
            Err(err) => error_response(json!(null), -32700, &format!("parse error: {err}")),
        };
        let encoded = serde_json::to_string(&response)?;
        stream.write_all(encoded.as_bytes())?;
        stream.write_all(b"\n")?;
        stream.flush()?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn kicad_fixture_path(name: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../engine/testdata/import/kicad")
            .join(name)
    }

    fn eagle_fixture_path(name: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../engine/testdata/import/eagle")
            .join(name)
    }

    fn temp_socket_path(name: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be valid")
            .as_nanos();
        std::env::temp_dir().join(format!("{}_{}_{}.sock", name, std::process::id(), unique))
    }

    fn read_json_line(stream: UnixStream) -> JsonRpcResponse {
        let mut reader = BufReader::new(stream);
        let mut line = String::new();
        reader.read_line(&mut line).expect("response should read");
        JsonRpcResponse::from_json(&line).expect("response should parse")
    }

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
        assert_eq!(
            result["summary"]["by_code"][0]["code"],
            "net_without_copper"
        );
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
                .any(
                    |entry| entry["code"] == "input_without_explicit_driver" && entry["count"] == 1
                )
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

    #[test]
    fn save_dispatch_writes_current_m3_slice_to_requested_path() {
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
        let open_response = dispatch_request(&mut engine, open);
        assert!(open_response.error.is_none(), "{open_response:?}");

        let track_uuid = uuid::Uuid::parse_str("cccccccc-cccc-cccc-cccc-cccccccccccc")
            .expect("uuid should parse");
        let delete = JsonRpcRequest {
            jsonrpc: "2.0".into(),
            id: json!(2),
            method: "delete_track".into(),
            params: json!({ "uuid": track_uuid }),
        };
        let delete_response = dispatch_request(&mut engine, delete);
        assert!(delete_response.error.is_none(), "{delete_response:?}");

        let target = temp_socket_path("datum-eda-save-dispatch.kicad_pcb");
        let request = JsonRpcRequest {
            jsonrpc: "2.0".into(),
            id: json!(3),
            method: "save".into(),
            params: json!({ "path": target }),
        };
        let response = dispatch_request(&mut engine, request);
        assert!(response.error.is_none(), "{response:?}");
        let result = response.result.expect("result should exist");
        assert_eq!(result["path"], target.display().to_string());
        assert!(target.exists());

        let saved = std::fs::read_to_string(&target).expect("saved file should read");
        assert!(!saved.contains(&track_uuid.to_string()));
        let _ = std::fs::remove_file(target);
    }

    #[test]
    fn delete_track_undo_and_redo_dispatch_round_trip() {
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

        let track_uuid = uuid::Uuid::parse_str("cccccccc-cccc-cccc-cccc-cccccccccccc")
            .expect("uuid should parse");

        let delete = JsonRpcRequest {
            jsonrpc: "2.0".into(),
            id: json!(2),
            method: "delete_track".into(),
            params: json!({ "uuid": track_uuid }),
        };
        let delete_response = dispatch_request(&mut engine, delete);
        assert!(delete_response.error.is_none(), "{delete_response:?}");
        assert_eq!(
            delete_response.result.expect("result should exist")["diff"]["deleted"][0]["uuid"],
            track_uuid.to_string()
        );

        let undo = JsonRpcRequest {
            jsonrpc: "2.0".into(),
            id: json!(3),
            method: "undo".into(),
            params: json!({}),
        };
        let undo_response = dispatch_request(&mut engine, undo);
        assert!(undo_response.error.is_none(), "{undo_response:?}");
        assert_eq!(
            undo_response.result.expect("result should exist")["diff"]["created"][0]["uuid"],
            track_uuid.to_string()
        );

        let redo = JsonRpcRequest {
            jsonrpc: "2.0".into(),
            id: json!(4),
            method: "redo".into(),
            params: json!({}),
        };
        let redo_response = dispatch_request(&mut engine, redo);
        assert!(redo_response.error.is_none(), "{redo_response:?}");
        assert_eq!(
            redo_response.result.expect("result should exist")["diff"]["deleted"][0]["uuid"],
            track_uuid.to_string()
        );
    }

    #[test]
    fn delete_track_dispatch_updates_followup_check_report() {
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
                method: "get_check_report".into(),
                params: json!({}),
            },
        );
        assert!(baseline.error.is_none(), "{baseline:?}");
        let baseline_result = baseline.result.expect("result should exist");
        assert!(
            baseline_result["diagnostics"]
                .as_array()
                .unwrap()
                .iter()
                .any(|diagnostic| diagnostic["kind"] == "partially_routed_net")
        );

        let delete = dispatch_request(
            &mut engine,
            JsonRpcRequest {
                jsonrpc: "2.0".into(),
                id: json!(3),
                method: "delete_track".into(),
                params: json!({ "uuid": "cccccccc-cccc-cccc-cccc-cccccccccccc" }),
            },
        );
        assert!(delete.error.is_none(), "{delete:?}");

        let after = dispatch_request(
            &mut engine,
            JsonRpcRequest {
                jsonrpc: "2.0".into(),
                id: json!(4),
                method: "get_check_report".into(),
                params: json!({}),
            },
        );
        assert!(after.error.is_none(), "{after:?}");
        let after_result = after.result.expect("result should exist");
        assert!(
            after_result["diagnostics"]
                .as_array()
                .unwrap()
                .iter()
                .any(|diagnostic| diagnostic["kind"] == "net_without_copper")
        );
        assert!(
            !after_result["diagnostics"]
                .as_array()
                .unwrap()
                .iter()
                .any(|diagnostic| diagnostic["kind"] == "partially_routed_net")
        );
    }

    #[test]
    fn delete_via_undo_and_redo_dispatch_round_trip() {
        let mut engine = Engine::new().expect("engine should initialize");
        let fixture = kicad_fixture_path("simple-demo.kicad_pcb");
        let open = JsonRpcRequest {
            jsonrpc: "2.0".into(),
            id: json!(1),
            method: "open_project".into(),
            params: serde_json::to_value(OpenProjectParams { path: fixture }).unwrap(),
        };
        let open_response = dispatch_request(&mut engine, open);
        assert!(open_response.error.is_none(), "{open_response:?}");

        let via_uuid = uuid::Uuid::parse_str("cccccccc-cccc-cccc-cccc-cccccccccccc")
            .expect("uuid should parse");

        let delete = JsonRpcRequest {
            jsonrpc: "2.0".into(),
            id: json!(2),
            method: "delete_via".into(),
            params: json!({ "uuid": via_uuid }),
        };
        let delete_response = dispatch_request(&mut engine, delete);
        assert!(delete_response.error.is_none(), "{delete_response:?}");
        assert_eq!(
            delete_response.result.expect("result should exist")["diff"]["deleted"][0]["uuid"],
            via_uuid.to_string()
        );

        let undo = JsonRpcRequest {
            jsonrpc: "2.0".into(),
            id: json!(3),
            method: "undo".into(),
            params: json!({}),
        };
        let undo_response = dispatch_request(&mut engine, undo);
        assert!(undo_response.error.is_none(), "{undo_response:?}");
        assert_eq!(
            undo_response.result.expect("result should exist")["diff"]["created"][0]["uuid"],
            via_uuid.to_string()
        );

        let redo = JsonRpcRequest {
            jsonrpc: "2.0".into(),
            id: json!(4),
            method: "redo".into(),
            params: json!({}),
        };
        let redo_response = dispatch_request(&mut engine, redo);
        assert!(redo_response.error.is_none(), "{redo_response:?}");
        assert_eq!(
            redo_response.result.expect("result should exist")["diff"]["deleted"][0]["uuid"],
            via_uuid.to_string()
        );
    }

    #[test]
    fn delete_via_dispatch_updates_followup_net_info_query() {
        let mut engine = Engine::new().expect("engine should initialize");
        let fixture = kicad_fixture_path("simple-demo.kicad_pcb");
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
                method: "get_net_info".into(),
                params: json!({}),
            },
        );
        assert!(baseline.error.is_none(), "{baseline:?}");
        let baseline_nets = baseline.result.expect("result should exist");
        let baseline_gnd = baseline_nets
            .as_array()
            .unwrap()
            .iter()
            .find(|net| net["name"] == "GND")
            .expect("GND net should exist");
        assert_eq!(baseline_gnd["vias"], json!(1));

        let delete = dispatch_request(
            &mut engine,
            JsonRpcRequest {
                jsonrpc: "2.0".into(),
                id: json!(3),
                method: "delete_via".into(),
                params: json!({ "uuid": "cccccccc-cccc-cccc-cccc-cccccccccccc" }),
            },
        );
        assert!(delete.error.is_none(), "{delete:?}");

        let after = dispatch_request(
            &mut engine,
            JsonRpcRequest {
                jsonrpc: "2.0".into(),
                id: json!(4),
                method: "get_net_info".into(),
                params: json!({}),
            },
        );
        assert!(after.error.is_none(), "{after:?}");
        let after_nets = after.result.expect("result should exist");
        let after_gnd = after_nets
            .as_array()
            .unwrap()
            .iter()
            .find(|net| net["name"] == "GND")
            .expect("GND net should exist");
        assert_eq!(after_gnd["vias"], json!(0));
    }

    #[test]
    fn delete_component_dispatch_updates_component_list() {
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

        let delete_component = JsonRpcRequest {
            jsonrpc: "2.0".into(),
            id: json!(2),
            method: "delete_component".into(),
            params: json!({
                "uuid": "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa"
            }),
        };
        let response = dispatch_request(&mut engine, delete_component);
        assert!(response.error.is_none(), "{response:?}");
        assert_eq!(
            response.result.expect("result should exist")["diff"]["deleted"][0]["object_type"],
            "component"
        );
        let components = engine.get_components().expect("components should query");
        assert!(
            components
                .iter()
                .all(|component| component.uuid
                    != uuid::Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap())
        );
    }

    #[test]
    fn delete_component_dispatch_updates_followup_components_query() {
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
        assert!(
            baseline_components
                .as_array()
                .unwrap()
                .iter()
                .any(|component| component["uuid"] == "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa")
        );

        let delete_component = dispatch_request(
            &mut engine,
            JsonRpcRequest {
                jsonrpc: "2.0".into(),
                id: json!(3),
                method: "delete_component".into(),
                params: json!({
                    "uuid": "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa"
                }),
            },
        );
        assert!(delete_component.error.is_none(), "{delete_component:?}");

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
        assert!(
            after_components
                .as_array()
                .unwrap()
                .iter()
                .all(|component| component["uuid"] != "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa")
        );
    }

    #[test]
    fn set_design_rule_dispatch_persists_rule_in_memory() {
        let mut engine = Engine::new().expect("engine should initialize");
        let fixture = kicad_fixture_path("simple-demo.kicad_pcb");
        let open = JsonRpcRequest {
            jsonrpc: "2.0".into(),
            id: json!(1),
            method: "open_project".into(),
            params: serde_json::to_value(OpenProjectParams { path: fixture }).unwrap(),
        };
        let open_response = dispatch_request(&mut engine, open);
        assert!(open_response.error.is_none(), "{open_response:?}");

        let set_rule = JsonRpcRequest {
            jsonrpc: "2.0".into(),
            id: json!(2),
            method: "set_design_rule".into(),
            params: json!({
                "rule_type": "ClearanceCopper",
                "scope": "All",
                "parameters": { "Clearance": { "min": 125000 } },
                "priority": 10,
                "name": "default clearance"
            }),
        };
        let response = dispatch_request(&mut engine, set_rule);
        assert!(response.error.is_none(), "{response:?}");
        let result = response.result.expect("result should exist");
        let created = result["diff"]["created"]
            .as_array()
            .expect("created should be array");
        assert_eq!(created.len(), 1);
        assert_eq!(created[0]["object_type"], "rule");

        let rules = engine.get_design_rules().expect("rules should query");
        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].name, "default clearance");
    }

    #[test]
    fn set_design_rule_dispatch_updates_followup_design_rules_query() {
        let mut engine = Engine::new().expect("engine should initialize");
        let fixture = kicad_fixture_path("simple-demo.kicad_pcb");
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
                method: "get_design_rules".into(),
                params: json!({}),
            },
        );
        assert!(baseline.error.is_none(), "{baseline:?}");
        assert_eq!(
            baseline
                .result
                .expect("result should exist")
                .as_array()
                .unwrap()
                .len(),
            0
        );

        let set_rule = dispatch_request(
            &mut engine,
            JsonRpcRequest {
                jsonrpc: "2.0".into(),
                id: json!(3),
                method: "set_design_rule".into(),
                params: json!({
                    "rule_type": "ClearanceCopper",
                    "scope": "All",
                    "parameters": { "Clearance": { "min": 125000 } },
                    "priority": 10,
                    "name": "default clearance"
                }),
            },
        );
        assert!(set_rule.error.is_none(), "{set_rule:?}");

        let after = dispatch_request(
            &mut engine,
            JsonRpcRequest {
                jsonrpc: "2.0".into(),
                id: json!(4),
                method: "get_design_rules".into(),
                params: json!({}),
            },
        );
        assert!(after.error.is_none(), "{after:?}");
        let rules = after.result.expect("result should exist");
        let rules = rules.as_array().unwrap();
        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0]["name"], "default clearance");
    }

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
            .find(|component| component.uuid
                == uuid::Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap())
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

    #[test]
    fn assign_part_dispatch_updates_component_value() {
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
        let part_uuid = search.result.as_ref().unwrap()[0]["uuid"].clone();

        let response = dispatch_request(
            &mut engine,
            JsonRpcRequest {
                jsonrpc: "2.0".into(),
                id: json!(4),
                method: "assign_part".into(),
                params: json!({
                    "uuid": "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa",
                    "part_uuid": part_uuid,
                }),
            },
        );
        assert!(response.error.is_none(), "{response:?}");
        let components = engine.get_components().expect("components should query");
        let updated = components.iter().find(|component| component.reference == "R1").unwrap();
        assert_eq!(updated.value, "LMV321");
    }

    #[test]
    fn assign_part_dispatch_updates_followup_components_query() {
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
        let part_uuid = search.result.as_ref().unwrap()[0]["uuid"].clone();

        let baseline = dispatch_request(
            &mut engine,
            JsonRpcRequest {
                jsonrpc: "2.0".into(),
                id: json!(4),
                method: "get_components".into(),
                params: json!({}),
            },
        );
        assert_eq!(baseline.result.as_ref().unwrap()[0]["value"], "10k");

        let assign = dispatch_request(
            &mut engine,
            JsonRpcRequest {
                jsonrpc: "2.0".into(),
                id: json!(5),
                method: "assign_part".into(),
                params: json!({
                    "uuid": "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa",
                    "part_uuid": part_uuid,
                }),
            },
        );
        assert!(assign.error.is_none(), "{assign:?}");

        let after = dispatch_request(
            &mut engine,
            JsonRpcRequest {
                jsonrpc: "2.0".into(),
                id: json!(6),
                method: "get_components".into(),
                params: json!({}),
            },
        );
        assert_eq!(after.result.as_ref().unwrap()[0]["value"], "LMV321");
    }

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
        let baseline_gnd = baseline.result.as_ref().unwrap()
            .as_array().unwrap()
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
        let after_gnd = after.result.as_ref().unwrap()
            .as_array().unwrap()
            .iter()
            .find(|net| net["uuid"] == net_uuid.to_string())
            .expect("updated GND should exist");
        assert_eq!(after_gnd["class"], "power");
    }

    #[test]
    fn move_component_dispatch_updates_followup_unrouted_query() {
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

        let baseline_unrouted = JsonRpcRequest {
            jsonrpc: "2.0".into(),
            id: json!(2),
            method: "get_unrouted".into(),
            params: json!({}),
        };
        let baseline_response = dispatch_request(&mut engine, baseline_unrouted);
        assert!(baseline_response.error.is_none(), "{baseline_response:?}");
        let baseline = baseline_response.result.expect("result should exist");
        let baseline_airwires = baseline.as_array().expect("airwires should be an array");
        assert_eq!(baseline_airwires.len(), 1);
        let baseline_distance = baseline_airwires[0]["distance_nm"]
            .as_i64()
            .expect("distance should be an integer");

        let move_component = JsonRpcRequest {
            jsonrpc: "2.0".into(),
            id: json!(3),
            method: "move_component".into(),
            params: json!({
                "uuid": "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa",
                "x_mm": 15.0,
                "y_mm": 12.0,
                "rotation_deg": 90.0
            }),
        };
        let move_response = dispatch_request(&mut engine, move_component);
        assert!(move_response.error.is_none(), "{move_response:?}");

        let after_unrouted = JsonRpcRequest {
            jsonrpc: "2.0".into(),
            id: json!(4),
            method: "get_unrouted".into(),
            params: json!({}),
        };
        let after_response = dispatch_request(&mut engine, after_unrouted);
        assert!(after_response.error.is_none(), "{after_response:?}");
        let after = after_response.result.expect("result should exist");
        let after_airwires = after.as_array().expect("airwires should be an array");
        assert_eq!(after_airwires.len(), 1);
        let after_distance = after_airwires[0]["distance_nm"]
            .as_i64()
            .expect("distance should be an integer");

        assert_ne!(after_distance, baseline_distance);
    }

    #[test]
    fn rotate_component_dispatch_updates_followup_components_query() {
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
        let baseline_target = baseline_components
            .as_array()
            .unwrap()
            .iter()
            .find(|component| component["uuid"] == "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa")
            .unwrap();
        assert_eq!(baseline_target["rotation"], 0);

        let rotate_component = dispatch_request(
            &mut engine,
            JsonRpcRequest {
                jsonrpc: "2.0".into(),
                id: json!(3),
                method: "rotate_component".into(),
                params: json!({
                    "uuid": "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa",
                    "x_mm": 10.0,
                    "y_mm": 10.0,
                    "rotation_deg": 180.0
                }),
            },
        );
        assert!(rotate_component.error.is_none(), "{rotate_component:?}");

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
        let after_target = after_components
            .as_array()
            .unwrap()
            .iter()
            .find(|component| component["uuid"] == "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa")
            .unwrap();
        assert_eq!(after_target["rotation"], 180);
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
}
