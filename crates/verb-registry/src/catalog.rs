//! Deterministic JSON projection of the verb registry.
//!
//! Emits `mcp-server/datum_tool_catalog.json`: verbs sorted by id, object keys
//! sorted (serde_json's default `BTreeMap`-backed `Map`), trailing newline.

use serde_json::{Map, Value, json};

use crate::{ArgvToken, Dispatch, ParamSpec, ParamType, VerbSpec, verbs};

pub const CATALOG_VERSION: u64 = 1;

fn param_schema(param: &ParamSpec) -> Value {
    let base = match param.ty {
        ParamType::Str | ParamType::Uuid => "string",
        ParamType::Int => "integer",
        ParamType::Bool => "boolean",
        ParamType::StrList => "array",
        ParamType::Json => "object",
    };
    let type_value = if param.required {
        json!(base)
    } else {
        json!([base, "null"])
    };
    let mut schema = Map::new();
    schema.insert("type".to_string(), type_value);
    if param.ty == ParamType::StrList {
        schema.insert("items".to_string(), json!({"type": "string"}));
    }
    Value::Object(schema)
}

fn input_schema(verb: &VerbSpec) -> Value {
    if let Some(raw) = verb.schema_json_override {
        return serde_json::from_str(raw)
            .unwrap_or_else(|err| panic!("{} schema_json_override invalid: {err}", verb.id));
    }
    let mut properties = Map::new();
    for param in verb.params {
        properties.insert(param.name.to_string(), param_schema(param));
    }
    let required: Vec<Value> = verb
        .params
        .iter()
        .filter(|param| param.required)
        .map(|param| json!(param.name))
        .collect();
    let mut schema = Map::new();
    schema.insert("type".to_string(), json!("object"));
    schema.insert("properties".to_string(), Value::Object(properties));
    if !required.is_empty() {
        schema.insert("required".to_string(), Value::Array(required));
    }
    Value::Object(schema)
}

fn argv_token_json(token: &ArgvToken) -> Value {
    match *token {
        ArgvToken::Lit(value) => json!({"lit": value}),
        ArgvToken::Param(param) => json!({"param": param}),
        ArgvToken::Flag { flag, param } => json!({"flag": flag, "param": param}),
        ArgvToken::Switch { flag, param } => json!({"switch": flag, "param": param}),
        ArgvToken::Repeated { flag, param } => json!({"repeated": flag, "param": param}),
    }
}

fn dispatch_json(dispatch: &Dispatch) -> Value {
    match *dispatch {
        Dispatch::Cli { method, argv } => json!({
            "kind": "cli",
            "method": method,
            "argv": argv.iter().map(argv_token_json).collect::<Vec<_>>(),
        }),
        Dispatch::DaemonRpc { method } => json!({"kind": "daemon", "method": method}),
    }
}

fn dispatch_defaults(verb: &VerbSpec) -> Value {
    let mut defaults = Map::new();
    for param in verb.params {
        if let Some(raw) = param.default_json {
            let value: Value = serde_json::from_str(raw).unwrap_or_else(|err| {
                panic!("{} param {} default_json invalid: {err}", verb.id, param.name)
            });
            defaults.insert(param.name.to_string(), value);
        }
    }
    Value::Object(defaults)
}

fn verb_json(verb: &VerbSpec) -> Value {
    json!({
        "name": verb.id,
        "description": verb.summary,
        "inputSchema": input_schema(verb),
        "status": verb.status.as_str(),
        "replacements": verb.replacements,
        "retirement": verb
            .retirement
            .map(|note| json!({"status": note.status, "criteria": note.criteria}))
            .unwrap_or(Value::Null),
        "dispatch": dispatch_json(&verb.dispatch),
        "dispatch_args": verb.params.iter().map(|param| param.name).collect::<Vec<_>>(),
        "dispatch_defaults": dispatch_defaults(verb),
        "write_surface": verb
            .write_surface
            .map(|surface| json!({"class": surface.class, "evidence": surface.evidence}))
            .unwrap_or(Value::Null),
    })
}

/// The full catalog document as a JSON value.
pub fn catalog_json() -> Value {
    json!({
        "catalog_version": CATALOG_VERSION,
        "verbs": verbs().iter().map(verb_json).collect::<Vec<_>>(),
    })
}

/// The catalog rendered exactly as persisted (pretty, trailing newline).
pub fn catalog_string() -> String {
    let mut rendered = serde_json::to_string_pretty(&catalog_json())
        .expect("catalog serialization cannot fail");
    rendered.push('\n');
    rendered
}
