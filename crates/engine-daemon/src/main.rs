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
    ReplaceComponentInput, RotateComponentInput, ScopedComponentReplacementOverride,
    ScopedComponentReplacementPlanEdit, ScopedComponentReplacementPolicyInput, SetDesignRuleInput,
    SetNetClassInput, SetPackageInput, SetPackageWithPartInput, SetReferenceInput, SetValueInput,
    ViolationDomain,
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
struct ScopedComponentReplacementOverrideParams {
    component_uuid: uuid::Uuid,
    target_package_uuid: uuid::Uuid,
    target_part_uuid: uuid::Uuid,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct EditScopedComponentReplacementPlanParams {
    plan: eda_engine::api::ScopedComponentReplacementPlan,
    exclude_component_uuids: Vec<uuid::Uuid>,
    overrides: Vec<ScopedComponentReplacementOverrideParams>,
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

fn serialized_success_response<T: Serialize>(id: Value, payload: T) -> JsonRpcResponse {
    match serde_json::to_value(payload) {
        Ok(result) => success_response(id, result),
        Err(err) => error_response(id, -32603, &format!("failed to serialize result: {err}")),
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

    #[path = "main_tests_component_mutation_core.rs"]
    mod component_mutation_core;
    #[path = "main_tests_dispatch_followups_transport.rs"]
    mod dispatch_followups_transport;
    #[path = "main_tests_dispatch_replacements.rs"]
    mod dispatch_replacements;
    #[path = "main_tests_mutation_basics.rs"]
    mod mutation_basics;
    #[path = "main_tests_package_replacements.rs"]
    mod package_replacements;
    #[path = "main_tests_package_replacements_apply.rs"]
    mod package_replacements_apply;
    #[path = "main_tests_query_check.rs"]
    mod query_check;
    #[path = "main_tests_query_check_runs.rs"]
    mod query_check_runs;
    #[path = "main_tests_session_pool.rs"]
    mod session_pool;
    #[path = "main_tests_session_pool_replacements.rs"]
    mod session_pool_replacements;
}
