//! Creation-only orderable Part intent over the canonical library writer.
use super::*;
use crate::substrate::Operation;
#[path = "library_gui_part_edit.rs"]
mod edit;

pub(super) fn build(
    context: &NativeWriteContext<'_>,
    provenance: WriteProvenance,
    request: LibraryDefinitionRequest,
    create: bool,
) -> Result<PreparedWrite, EngineError> {
    if !create {
        return edit::build(context, provenance, request);
    }
    const FIELDS: &[&str] = &[
        "manufacturer",
        "mpn",
        "value",
        "description",
        "datasheet",
        "tags",
        "entity",
        "package",
        "default_footprint",
    ];
    if request.fields.keys().any(|k| !FIELDS.contains(&k.as_str())) {
        return Err(refusal("Part creation contains an unsupported field"));
    }
    let text = |key: &str| {
        request
            .fields
            .get(key)
            .and_then(Value::as_str)
            .ok_or_else(|| refusal(format!("Part {key} must be text")))
    };
    if text("manufacturer")?.trim().is_empty() || text("mpn")?.trim().is_empty() {
        return Err(refusal("Enter a manufacturer and MPN"));
    }
    let reference = |key: &str| -> Result<Uuid, EngineError> {
        request
            .fields
            .get(key)
            .and_then(Value::as_str)
            .and_then(|s| s.parse().ok())
            .ok_or_else(|| refusal(format!("Choose an existing named Part {key}")))
    };
    let entity = reference("entity")?;
    let package = reference("package")?;
    for (id, kind, label) in [
        (entity, "entities", "Entity"),
        (package, "packages", "Package"),
    ] {
        library_definition::validate_pool_definition_reference(context.model, id, kind, label)?;
    }
    let footprint = match request.fields.get("default_footprint") {
        None | Some(Value::Null) => None,
        Some(_) => Some(reference("default_footprint")?),
    };
    if let Some(id) = footprint {
        let raw = super::super::schematic_library::materialized_pool_object(
            context.model,
            id,
            "footprints",
        )?;
        if raw
            .get("package")
            .and_then(Value::as_str)
            .and_then(|v| v.parse::<Uuid>().ok())
            != Some(package)
        {
            return Err(refusal(
                "Choose a Footprint belonging to the selected Package",
            ));
        }
    }
    let mut object = library::pool_part_payload(
        request.object_id,
        entity,
        package,
        text("mpn")?,
        text("manufacturer")?,
        text("value")?,
        text("description")?,
        text("datasheet")?,
        "Unknown",
    );
    object["default_footprint"] = json!(footprint);
    let tags: std::collections::BTreeSet<String> = match request.fields.get("tags") {
        Some(v) => serde_json::from_value(v.clone())?,
        None => Default::default(),
    };
    object["tags"] = json!(tags);
    let _: crate::pool::Part = serde_json::from_value(object.clone())?;
    let mut prepared = library::build_pool_library_write(
        context.model,
        provenance,
        Some("pool"),
        vec![PoolLibraryOperationSpec::Create {
            target: PoolLibraryObjectTarget::new("pool", "parts", request.object_id),
            object,
        }],
    )?;
    let mut references = vec![entity, package];
    references.extend(footprint);
    references.sort();
    references.dedup();
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
#[path = "library_gui_part_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "library_gui_part_edit_tests.rs"]
mod edit_tests;

#[cfg(test)]
#[path = "library_gui_part_default_footprint_tests.rs"]
mod default_footprint_tests;

#[cfg(test)]
#[path = "library_gui_part_default_map_tests.rs"]
mod default_map_tests;

#[cfg(test)]
#[path = "library_gui_part_entity_edit_tests.rs"]
mod entity_edit_tests;
