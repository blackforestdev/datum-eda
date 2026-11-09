//! New logical Entities use the canonical pool writer and explicit gate links.
use super::*;
use crate::substrate::Operation;
use std::collections::HashSet;

#[path = "library_gui_entity_edit.rs"]
mod edit;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct GateIntent {
    uuid: Uuid,
    name: String,
    unit: Uuid,
    symbol: Uuid,
}

pub(super) fn build(
    context: &NativeWriteContext<'_>,
    provenance: WriteProvenance,
    request: LibraryDefinitionRequest,
    create: bool,
) -> Result<PreparedWrite, EngineError> {
    if !create {
        return edit::build(context, provenance, request);
    }
    if request
        .fields
        .keys()
        .any(|key| !["name", "prefix", "manufacturer", "gates", "tags"].contains(&key.as_str()))
    {
        return Err(refusal(
            "Entity creation supports names, prefix, manufacturer, tags and explicit gates",
        ));
    }
    let text = |key: &str| {
        request
            .fields
            .get(key)
            .and_then(Value::as_str)
            .ok_or_else(|| refusal(format!("Entity {key} must be text")))
    };
    let name = text("name")?.trim();
    if name.is_empty() {
        return Err(refusal("Enter an Entity name"));
    }
    let rows = request
        .fields
        .get("gates")
        .and_then(Value::as_object)
        .filter(|g| !g.is_empty())
        .ok_or_else(|| refusal("Add one or more named gates"))?;
    let mut identities = HashSet::new();
    let mut references = HashSet::new();
    let mut gates = Map::new();
    for (key, value) in rows {
        let id = Uuid::parse_str(key).map_err(|_| refusal("Gate identity is invalid"))?;
        if !identities.insert(id) {
            return Err(refusal("Gate identity is ambiguous"));
        }
        let gate: GateIntent = serde_json::from_value(value.clone())?;
        if id != gate.uuid {
            return Err(refusal("Gate map key must match its retained UUID"));
        }
        if gate.name.trim().is_empty() {
            return Err(refusal("Enter a name for every gate"));
        }
        library_definition::validate_pool_definition_reference(
            context.model,
            gate.unit,
            "units",
            "gate Unit",
        )?;
        let symbol = super::super::schematic_library::materialized_pool_object(
            context.model,
            gate.symbol,
            "symbols",
        )?;
        if symbol
            .get("unit")
            .and_then(Value::as_str)
            .and_then(|id| Uuid::parse_str(id).ok())
            != Some(gate.unit)
        {
            return Err(refusal(format!(
                "Gate {}: choose a Symbol belonging to its Unit",
                gate.name
            )));
        }
        references.extend([gate.unit, gate.symbol]);
        gates.insert(
            id.to_string(),
            json!({"uuid":id,"name":gate.name,"unit":gate.unit,"symbol":gate.symbol}),
        );
    }
    let tags: std::collections::BTreeSet<String> = match request.fields.get("tags") {
        Some(tags) => serde_json::from_value(tags.clone())?,
        None => Default::default(),
    };
    let object = json!({"schema_version":1,"uuid":request.object_id,"name":name,"prefix":text("prefix")?,"manufacturer":text("manufacturer")?,"tags":tags,"gates":gates});
    let _: crate::pool::Entity = serde_json::from_value(object.clone())?;
    let mut prepared = library::build_pool_library_write(
        context.model,
        provenance,
        Some("pool"),
        vec![PoolLibraryOperationSpec::Create {
            target: PoolLibraryObjectTarget::new("pool", "entities", request.object_id),
            object,
        }],
    )?;
    // Commit checks the same referenced definitions observed during preview.
    // Guards are canonical before-model evidence, not persisted gate metadata.
    let mut references: Vec<_> = references.into_iter().collect();
    references.sort();
    for id in references.into_iter().rev() {
        prepared.batch.operations.insert(
            0,
            Operation::GuardObjectRevision {
                object_id: id,
                expected_object_revision: context.model.objects[&id].object_revision,
            },
        );
    }
    Ok(prepared)
}

#[cfg(test)]
#[path = "library_gui_entity_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "library_gui_entity_edit_tests.rs"]
mod edit_tests;

#[cfg(test)]
#[path = "library_gui_entity_gate_edit_tests.rs"]
mod gate_edit_tests;

#[cfg(test)]
#[path = "library_gui_entity_gate_name_tests.rs"]
mod gate_name_tests;

#[cfg(test)]
#[path = "library_gui_entity_gate_unit_tests.rs"]
mod gate_unit_tests;

#[cfg(test)]
#[path = "library_gui_entity_add_gate_tests.rs"]
mod add_gate_tests;
