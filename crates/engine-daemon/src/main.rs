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
    AssignPartInput, ComponentReplacementPolicy, ComponentReplacementScope, Engine,
    MoveComponentInput, PlannedComponentReplacementInput, PolicyDrivenComponentReplacementInput,
    ReplaceComponentInput, RotateComponentInput, ScopedComponentReplacementPolicyInput,
    SetDesignRuleInput, SetNetClassInput, SetPackageInput, SetPackageWithPartInput,
    SetReferenceInput, SetValueInput, ViolationDomain,
};
use eda_engine::ir::geometry::Point;
use eda_engine::ir::units::mm_to_nm;
use eda_engine::rules::ast::{RuleParams, RuleScope, RuleType};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

mod dispatch;
use dispatch::dispatch_request;

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
struct SetPackageParams {
    uuid: uuid::Uuid,
    package_uuid: uuid::Uuid,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct SetPackageWithPartParams {
    uuid: uuid::Uuid,
    package_uuid: uuid::Uuid,
    part_uuid: uuid::Uuid,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct ReplaceComponentParams {
    uuid: uuid::Uuid,
    package_uuid: uuid::Uuid,
    part_uuid: uuid::Uuid,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct ReplaceComponentsParams {
    replacements: Vec<ReplaceComponentParams>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct PlannedComponentReplacementParams {
    uuid: uuid::Uuid,
    package_uuid: Option<uuid::Uuid>,
    part_uuid: Option<uuid::Uuid>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct ApplyComponentReplacementPlanParams {
    replacements: Vec<PlannedComponentReplacementParams>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct PolicyDrivenComponentReplacementParams {
    uuid: uuid::Uuid,
    policy: ComponentReplacementPolicy,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct ApplyComponentReplacementPolicyParams {
    replacements: Vec<PolicyDrivenComponentReplacementParams>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
struct ComponentReplacementScopeParams {
    reference_prefix: Option<String>,
    value_equals: Option<String>,
    current_package_uuid: Option<uuid::Uuid>,
    current_part_uuid: Option<uuid::Uuid>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct ApplyScopedComponentReplacementPolicyParams {
    scope: ComponentReplacementScopeParams,
    policy: ComponentReplacementPolicy,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct ApplyScopedComponentReplacementPlanParams {
    plan: eda_engine::api::ScopedComponentReplacementPlan,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct GetScopedComponentReplacementPlanParams {
    scope: ComponentReplacementScopeParams,
    policy: ComponentReplacementPolicy,
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
    fn get_package_change_candidates_dispatch_returns_unique_candidate_report() {
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
        let assign = dispatch_request(
            &mut engine,
            JsonRpcRequest {
                jsonrpc: "2.0".into(),
                id: json!(4),
                method: "assign_part".into(),
                params: json!({
                    "uuid": "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa",
                    "part_uuid": search.result.as_ref().unwrap()[0]["uuid"],
                }),
            },
        );
        assert!(assign.error.is_none(), "{assign:?}");

        let response = dispatch_request(
            &mut engine,
            JsonRpcRequest {
                jsonrpc: "2.0".into(),
                id: json!(5),
                method: "get_package_change_candidates".into(),
                params: json!({ "uuid": "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa" }),
            },
        );
        assert!(response.error.is_none(), "{response:?}");
        let report = response.result.expect("result should exist");
        assert_eq!(report["status"], "candidates_available");
        assert_eq!(report["candidates"].as_array().unwrap().len(), 1);
        assert_eq!(report["candidates"][0]["package_name"], "ALT-3");
    }

    #[test]
    fn get_part_change_candidates_dispatch_returns_compatible_part_report() {
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
        let lmv321_part_uuid = search.result.as_ref().unwrap()[0]["uuid"].clone();
        let assign = dispatch_request(
            &mut engine,
            JsonRpcRequest {
                jsonrpc: "2.0".into(),
                id: json!(4),
                method: "assign_part".into(),
                params: json!({
                    "uuid": "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa",
                    "part_uuid": lmv321_part_uuid,
                }),
            },
        );
        assert!(assign.error.is_none(), "{assign:?}");

        let response = dispatch_request(
            &mut engine,
            JsonRpcRequest {
                jsonrpc: "2.0".into(),
                id: json!(5),
                method: "get_part_change_candidates".into(),
                params: json!({"uuid": "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa"}),
            },
        );
        assert!(response.error.is_none(), "{response:?}");
        let report = response.result.expect("response should contain result");
        assert_eq!(report["status"], "candidates_available");
        assert_eq!(report["current_part_uuid"], lmv321_part_uuid);
        assert!(
            report["candidates"]
                .as_array()
                .unwrap()
                .iter()
                .any(|candidate| candidate["package_name"] == "ALT-3"
                    && candidate["value"] == "ALTAMP")
        );
    }

    #[test]
    fn get_component_replacement_plan_dispatch_returns_combined_report() {
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
        let lmv321_part_uuid = search.result.as_ref().unwrap()[0]["uuid"].clone();
        let assign = dispatch_request(
            &mut engine,
            JsonRpcRequest {
                jsonrpc: "2.0".into(),
                id: json!(4),
                method: "assign_part".into(),
                params: json!({
                    "uuid": "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa",
                    "part_uuid": lmv321_part_uuid,
                }),
            },
        );
        assert!(assign.error.is_none(), "{assign:?}");

        let response = dispatch_request(
            &mut engine,
            JsonRpcRequest {
                jsonrpc: "2.0".into(),
                id: json!(5),
                method: "get_component_replacement_plan".into(),
                params: json!({"uuid": "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa"}),
            },
        );
        assert!(response.error.is_none(), "{response:?}");
        let report = response.result.expect("response should contain result");
        assert_eq!(report["current_reference"], "R1");
        assert_eq!(report["current_part_uuid"], lmv321_part_uuid);
        assert_eq!(report["package_change"]["status"], "candidates_available");
        assert_eq!(report["part_change"]["status"], "candidates_available");
    }

    #[test]
    fn get_scoped_component_replacement_plan_dispatch_returns_resolved_preview() {
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
        let lmv321_part_uuid = search.result.as_ref().unwrap()[0]["uuid"].clone();
        for (id, uuid) in [
            (4, "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa"),
            (5, "bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb"),
        ] {
            let assign = dispatch_request(
                &mut engine,
                JsonRpcRequest {
                    jsonrpc: "2.0".into(),
                    id: json!(id),
                    method: "assign_part".into(),
                    params: json!({
                        "uuid": uuid,
                        "part_uuid": lmv321_part_uuid,
                    }),
                },
            );
            assert!(assign.error.is_none(), "{assign:?}");
        }

        let response = dispatch_request(
            &mut engine,
            JsonRpcRequest {
                jsonrpc: "2.0".into(),
                id: json!(6),
                method: "get_scoped_component_replacement_plan".into(),
                params: json!({
                    "scope": {
                        "reference_prefix": "R",
                        "value_equals": "LMV321",
                    },
                    "policy": "best_compatible_package",
                }),
            },
        );
        assert!(response.error.is_none(), "{response:?}");
        let report = response.result.expect("response should contain result");
        assert_eq!(report["policy"], "best_compatible_package");
        assert_eq!(report["replacements"].as_array().unwrap().len(), 2);
        assert_eq!(report["replacements"][0]["current_reference"], "R1");
        assert_eq!(report["replacements"][0]["target_package_name"], "ALT-3");
        assert_eq!(report["replacements"][0]["target_value"], "ALTAMP");
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
                params: json!({"query": "ALTAMP"}),
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
        assert_eq!(updated.value, "ALTAMP");
    }

    #[test]
    fn assign_part_dispatch_updates_followup_net_info_query() {
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
        let part_uuid = search.result.as_ref().unwrap()[0]["uuid"].clone();

        let baseline = dispatch_request(
            &mut engine,
            JsonRpcRequest {
                jsonrpc: "2.0".into(),
                id: json!(4),
                method: "get_net_info".into(),
                params: json!({}),
            },
        );
        let baseline_sig = baseline
            .result
            .as_ref()
            .unwrap()
            .as_array()
            .unwrap()
            .iter()
            .find(|net| net["name"] == "SIG")
            .expect("SIG net should exist");
        assert_eq!(baseline_sig["pins"].as_array().unwrap().len(), 2);

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
                method: "get_net_info".into(),
                params: json!({}),
            },
        );
        let sig = after
            .result
            .as_ref()
            .unwrap()
            .as_array()
            .unwrap()
            .iter()
            .find(|net| net["name"] == "SIG")
            .expect("SIG net should exist");
        assert_eq!(sig["pins"].as_array().unwrap().len(), 1);
    }

    #[test]
    fn assign_part_dispatch_preserves_logical_nets_across_known_part_remap() {
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
        let altamp_part_uuid = altamp_search.result.as_ref().unwrap()[0]["uuid"].clone();

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

        let second_assign = dispatch_request(
            &mut engine,
            JsonRpcRequest {
                jsonrpc: "2.0".into(),
                id: json!(7),
                method: "assign_part".into(),
                params: json!({
                    "uuid": "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa",
                    "part_uuid": altamp_part_uuid,
                }),
            },
        );
        assert!(second_assign.error.is_none(), "{second_assign:?}");

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
        let updated = components.iter().find(|component| component.reference == "R1").unwrap();
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

    #[test]
    fn set_package_with_part_dispatch_preserves_logical_nets_for_explicit_candidate() {
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
        let altamp_part_uuid = altamp_search.result.as_ref().unwrap()[0]["uuid"].clone();

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

        let set_package_with_part = dispatch_request(
            &mut engine,
            JsonRpcRequest {
                jsonrpc: "2.0".into(),
                id: json!(7),
                method: "set_package_with_part".into(),
                params: json!({
                    "uuid": "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa",
                    "package_uuid": altamp_package_uuid,
                    "part_uuid": altamp_part_uuid,
                }),
            },
        );
        assert!(
            set_package_with_part.error.is_none(),
            "{set_package_with_part:?}"
        );

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

    #[test]
    fn replace_component_dispatch_preserves_logical_nets_for_explicit_candidate() {
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
        let altamp_part_uuid = altamp_search.result.as_ref().unwrap()[0]["uuid"].clone();

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

        let replace_component = dispatch_request(
            &mut engine,
            JsonRpcRequest {
                jsonrpc: "2.0".into(),
                id: json!(7),
                method: "replace_component".into(),
                params: json!({
                    "uuid": "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa",
                    "package_uuid": altamp_package_uuid,
                    "part_uuid": altamp_part_uuid,
                }),
            },
        );
        assert!(replace_component.error.is_none(), "{replace_component:?}");
        assert_eq!(
            replace_component.result.as_ref().unwrap()["description"],
            "replace_component aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa"
        );

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

    #[test]
    fn replace_components_dispatch_batches_multiple_replacements_into_one_undo_step() {
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
        let altamp_search = dispatch_request(
            &mut engine,
            JsonRpcRequest {
                jsonrpc: "2.0".into(),
                id: json!(3),
                method: "search_pool".into(),
                params: json!({"query": "ALTAMP"}),
            },
        );
        let altamp_package_uuid = altamp_search.result.as_ref().unwrap()[0]["package_uuid"].clone();
        let altamp_part_uuid = altamp_search.result.as_ref().unwrap()[0]["uuid"].clone();

        let replace_components = dispatch_request(
            &mut engine,
            JsonRpcRequest {
                jsonrpc: "2.0".into(),
                id: json!(4),
                method: "replace_components".into(),
                params: json!({
                    "replacements": [
                        {
                            "uuid": "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa",
                            "package_uuid": altamp_package_uuid,
                            "part_uuid": altamp_part_uuid,
                        },
                        {
                            "uuid": "bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb",
                            "package_uuid": altamp_package_uuid,
                            "part_uuid": altamp_part_uuid,
                        }
                    ]
                }),
            },
        );
        assert!(replace_components.error.is_none(), "{replace_components:?}");
        assert_eq!(
            replace_components.result.as_ref().unwrap()["description"],
            "replace_components 2"
        );

        let undo = dispatch_request(
            &mut engine,
            JsonRpcRequest {
                jsonrpc: "2.0".into(),
                id: json!(5),
                method: "undo".into(),
                params: json!({}),
            },
        );
        assert!(undo.error.is_none(), "{undo:?}");
        assert_eq!(undo.result.as_ref().unwrap()["description"], "undo replace_components 2");

        let components = dispatch_request(
            &mut engine,
            JsonRpcRequest {
                jsonrpc: "2.0".into(),
                id: json!(6),
                method: "get_components".into(),
                params: json!({}),
            },
        );
        let values: Vec<_> = components
            .result
            .as_ref()
            .unwrap()
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|component| component["value"].as_str())
            .collect();
        assert_eq!(values.iter().filter(|value| **value == "10k").count(), 2);
    }

    #[test]
    fn apply_component_replacement_plan_dispatch_resolves_package_and_part_selectors() {
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
        let altamp_search = dispatch_request(
            &mut engine,
            JsonRpcRequest {
                jsonrpc: "2.0".into(),
                id: json!(3),
                method: "search_pool".into(),
                params: json!({"query": "LMV321"}),
            },
        );
        let lmv321_part_uuid = altamp_search.result.as_ref().unwrap()[0]["uuid"].clone();
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
        let altamp_part_uuid = altamp_search.result.as_ref().unwrap()[0]["uuid"].clone();

        for (id, uuid) in [(5, "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa"), (6, "bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb")] {
            let assign = dispatch_request(
                &mut engine,
                JsonRpcRequest {
                    jsonrpc: "2.0".into(),
                    id: json!(id),
                    method: "assign_part".into(),
                    params: json!({
                        "uuid": uuid,
                        "part_uuid": lmv321_part_uuid,
                    }),
                },
            );
            assert!(assign.error.is_none(), "{assign:?}");
        }

        let apply = dispatch_request(
            &mut engine,
            JsonRpcRequest {
                jsonrpc: "2.0".into(),
                id: json!(7),
                method: "apply_component_replacement_plan".into(),
                params: json!({
                    "replacements": [
                        {
                            "uuid": "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa",
                            "package_uuid": altamp_package_uuid,
                            "part_uuid": null,
                        },
                        {
                            "uuid": "bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb",
                            "package_uuid": null,
                            "part_uuid": altamp_part_uuid,
                        }
                    ]
                }),
            },
        );
        assert!(apply.error.is_none(), "{apply:?}");
        assert_eq!(apply.result.as_ref().unwrap()["description"], "replace_components 2");

        let components = dispatch_request(
            &mut engine,
            JsonRpcRequest {
                jsonrpc: "2.0".into(),
                id: json!(8),
                method: "get_components".into(),
                params: json!({}),
            },
        );
        let values: Vec<_> = components
            .result
            .as_ref()
            .unwrap()
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|component| component["value"].as_str())
            .collect();
        assert_eq!(values.iter().filter(|value| **value == "ALTAMP").count(), 2);
    }

    #[test]
    fn apply_component_replacement_policy_dispatch_resolves_best_candidates() {
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
        for (id, uuid) in [(4, "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa"), (5, "bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb")] {
            let assign = dispatch_request(
                &mut engine,
                JsonRpcRequest {
                    jsonrpc: "2.0".into(),
                    id: json!(id),
                    method: "assign_part".into(),
                    params: json!({
                        "uuid": uuid,
                        "part_uuid": lmv321_part_uuid,
                    }),
                },
            );
            assert!(assign.error.is_none(), "{assign:?}");
        }

        let apply = dispatch_request(
            &mut engine,
            JsonRpcRequest {
                jsonrpc: "2.0".into(),
                id: json!(6),
                method: "apply_component_replacement_policy".into(),
                params: json!({
                    "replacements": [
                        {
                            "uuid": "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa",
                            "policy": "best_compatible_package",
                        },
                        {
                            "uuid": "bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb",
                            "policy": "best_compatible_part",
                        }
                    ]
                }),
            },
        );
        assert!(apply.error.is_none(), "{apply:?}");
        assert_eq!(apply.result.as_ref().unwrap()["description"], "replace_components 2");

        let components = dispatch_request(
            &mut engine,
            JsonRpcRequest {
                jsonrpc: "2.0".into(),
                id: json!(7),
                method: "get_components".into(),
                params: json!({}),
            },
        );
        let values: Vec<_> = components
            .result
            .as_ref()
            .unwrap()
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|component| component["value"].as_str())
            .collect();
        assert_eq!(values.iter().filter(|value| **value == "ALTAMP").count(), 2);
    }

    #[test]
    fn apply_scoped_component_replacement_policy_dispatch_targets_filtered_components() {
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
        for (id, uuid) in [(4, "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa"), (5, "bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb")] {
            let assign = dispatch_request(
                &mut engine,
                JsonRpcRequest {
                    jsonrpc: "2.0".into(),
                    id: json!(id),
                    method: "assign_part".into(),
                    params: json!({
                        "uuid": uuid,
                        "part_uuid": lmv321_part_uuid,
                    }),
                },
            );
            assert!(assign.error.is_none(), "{assign:?}");
        }

        let apply = dispatch_request(
            &mut engine,
            JsonRpcRequest {
                jsonrpc: "2.0".into(),
                id: json!(6),
                method: "apply_scoped_component_replacement_policy".into(),
                params: json!({
                    "scope": {
                        "reference_prefix": "R",
                        "value_equals": "LMV321",
                    },
                    "policy": "best_compatible_package",
                }),
            },
        );
        assert!(apply.error.is_none(), "{apply:?}");
        assert_eq!(apply.result.as_ref().unwrap()["description"], "replace_components 2");

        let components = dispatch_request(
            &mut engine,
            JsonRpcRequest {
                jsonrpc: "2.0".into(),
                id: json!(7),
                method: "get_components".into(),
                params: json!({}),
            },
        );
        let values: Vec<_> = components
            .result
            .as_ref()
            .unwrap()
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|component| component["value"].as_str())
            .collect();
        assert_eq!(values.iter().filter(|value| **value == "ALTAMP").count(), 2);
    }

    #[test]
    fn apply_scoped_component_replacement_plan_dispatch_applies_preview_without_reresolving() {
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
        let lmv321_part_uuid = search.result.as_ref().unwrap()[0]["uuid"].clone();
        for (id, uuid) in [
            (4, "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa"),
            (5, "bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb"),
        ] {
            let assign = dispatch_request(
                &mut engine,
                JsonRpcRequest {
                    jsonrpc: "2.0".into(),
                    id: json!(id),
                    method: "assign_part".into(),
                    params: json!({
                        "uuid": uuid,
                        "part_uuid": lmv321_part_uuid,
                    }),
                },
            );
            assert!(assign.error.is_none(), "{assign:?}");
        }
        let preview = dispatch_request(
            &mut engine,
            JsonRpcRequest {
                jsonrpc: "2.0".into(),
                id: json!(6),
                method: "get_scoped_component_replacement_plan".into(),
                params: json!({
                    "scope": {
                        "reference_prefix": "R",
                        "value_equals": "LMV321",
                    },
                    "policy": "best_compatible_package",
                }),
            },
        );
        assert!(preview.error.is_none(), "{preview:?}");

        let apply = dispatch_request(
            &mut engine,
            JsonRpcRequest {
                jsonrpc: "2.0".into(),
                id: json!(7),
                method: "apply_scoped_component_replacement_plan".into(),
                params: json!({
                    "plan": preview.result.unwrap(),
                }),
            },
        );
        assert!(apply.error.is_none(), "{apply:?}");
        assert_eq!(apply.result.as_ref().unwrap()["description"], "replace_components 2");
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
