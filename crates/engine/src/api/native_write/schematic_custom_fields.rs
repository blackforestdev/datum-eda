//! Occurrence-qualified custom metadata edits retaining the raw symbol payload.
use super::{
    context::{BatchComposer, PreparedWrite, WriteProvenance},
    registry::{NativeWriteContext, NativeWriteVerb, parse_verb_params},
    schematic_selection_edit::selected_source,
};
use crate::{error::EngineError, schematic::PlacedSymbol, substrate::Operation};
use serde::Deserialize;
use serde_json::{Value, json};
use std::collections::BTreeSet;
use uuid::Uuid;

pub(crate) const VERBS: &[NativeWriteVerb] = &[NativeWriteVerb {
    id: "datum.schematic.set_symbol_field",
    build: set_symbol_field,
}];

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
enum Field {
    Custom,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Params {
    symbol: Uuid,
    instance_path: Vec<Uuid>,
    field: Field,
    field_id: Option<Uuid>,
    key: Option<String>,
    value: Option<String>,
    #[serde(default)]
    clear: bool,
}

fn invalid(message: impl Into<String>) -> EngineError {
    EngineError::Validation(message.into())
}

fn set_symbol_field(
    context: &NativeWriteContext<'_>,
    provenance: WriteProvenance,
    params: Value,
) -> Result<PreparedWrite, EngineError> {
    let params: Params = parse_verb_params("datum.schematic.set_symbol_field", params)?;
    let Field::Custom = params.field;
    if params.clear {
        if params.field_id.is_none() || params.key.is_some() || params.value.is_some() {
            return Err(invalid("Clear requires field_id and forbids key/value"));
        }
    } else {
        if params.field_id.is_none() && (params.key.is_none() || params.value.is_none()) {
            return Err(invalid("A new custom field requires key and value"));
        }
        if params.key.is_none() && params.value.is_none() {
            return Err(invalid("A custom field edit requires key or value"));
        }
        if params
            .key
            .as_ref()
            .is_some_and(|key| key.trim().is_empty() || key.chars().any(char::is_control))
        {
            return Err(invalid("Custom field key must be nonempty and single-line"));
        }
        if params
            .value
            .as_ref()
            .is_some_and(|value| value.chars().any(char::is_control))
        {
            return Err(invalid("Custom field value must be single-line"));
        }
    }
    let (source, sheet) =
        selected_source(context, "symbols", params.symbol, &params.instance_path)?;
    let original = &source.sheets[&sheet].symbols[&params.symbol];
    let mut ids = BTreeSet::new();
    if original.fields.iter().any(|field| !ids.insert(field.uuid)) {
        return Err(invalid("Custom field UUID ownership is ambiguous"));
    }

    // Materialize through the resolver, not a typed round trip: imported symbol,
    // pin and field extensions must survive edits and their canonical inverse.
    let object = context
        .model
        .objects
        .get(&params.symbol)
        .ok_or_else(|| invalid("Selected symbol source object is missing"))?;
    let shards: Vec<_> = context
        .model
        .source_shards
        .iter()
        .filter(|shard| shard.shard_id == object.source_shard_id)
        .collect();
    if shards.len() != 1 {
        return Err(invalid("Selected symbol source shard is ambiguous"));
    }
    let payload = context
        .model
        .materialized_source_shard_value_by_relative_path(&shards[0].relative_path)?;
    if payload.get("uuid") != Some(&json!(sheet)) {
        return Err(invalid("Selected symbol source sheet does not match"));
    }
    let mut symbol = payload
        .get("symbols")
        .and_then(|symbols| symbols.get(params.symbol.to_string()))
        .cloned()
        .ok_or_else(|| invalid("Selected symbol payload is missing"))?;
    if serde_json::from_value::<PlacedSymbol>(symbol.clone())? != *original {
        return Err(invalid("Selected symbol payload does not match its source"));
    }
    let before = symbol.clone();
    let fields = symbol
        .get_mut("fields")
        .and_then(Value::as_array_mut)
        .ok_or_else(|| invalid("Selected symbol fields are unavailable"))?;
    if let Some(id) = params.field_id {
        let index = original
            .fields
            .iter()
            .position(|field| field.uuid == id)
            .ok_or_else(|| invalid("Selected custom field no longer exists"))?;
        if params.clear {
            fields.remove(index);
        } else {
            let field = fields[index]
                .as_object_mut()
                .ok_or_else(|| invalid("Selected custom field payload is invalid"))?;
            if let Some(key) = params.key {
                field.insert("key".into(), Value::String(key));
            }
            if let Some(value) = params.value {
                field.insert("value".into(), Value::String(value));
            }
        }
    } else {
        let mut id = Uuid::new_v4();
        while context.model.objects.contains_key(&id) || ids.contains(&id) {
            id = Uuid::new_v4();
        }
        fields.push(json!({"uuid":id, "key":params.key, "value":params.value,
            "visible":false, "position":null}));
    }
    if symbol == before {
        return Err(invalid("Custom field is unchanged"));
    }
    BatchComposer::compose(context.model, provenance)
        .push_op(Operation::SetSchematicSymbol {
            sheet_id: sheet,
            symbol_id: params.symbol,
            symbol,
        })
        .primary_object(params.symbol)
        .finish()
}

#[cfg(test)]
#[path = "schematic_custom_fields_tests.rs"]
mod tests;
