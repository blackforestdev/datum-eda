//! Finite native library definition adapters. Raw payloads remain authoritative.
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};
use uuid::Uuid;

use super::{
    PreparedWrite, WriteProvenance,
    library::{self, PoolLibraryObjectTarget, PoolLibraryOperationSpec},
    library_definition, library_footprint,
    registry::{NativeWriteContext, NativeWriteVerb, parse_verb_params},
};
use crate::{error::EngineError, substrate::SourceShardKind};

#[path = "library_gui_package.rs"]
mod package;

/// GUI intent contains only explicitly edited fields, never a normalized object.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LibraryDefinitionRequest {
    pub kind: String,
    pub object_id: Uuid,
    pub pool_path: String,
    pub fields: Map<String, Value>,
}

pub const VERBS: &[NativeWriteVerb] = &[
    NativeWriteVerb {
        id: "datum.library.create",
        build: create,
    },
    NativeWriteVerb {
        id: "datum.library.edit",
        build: edit,
    },
];

fn refusal(message: impl Into<String>) -> EngineError {
    EngineError::Validation(message.into())
}

fn create(
    context: &NativeWriteContext<'_>,
    provenance: WriteProvenance,
    params: Value,
) -> Result<PreparedWrite, EngineError> {
    build_definition(
        context,
        provenance,
        parse_verb_params("datum.library.create", params)?,
        true,
    )
}
fn edit(
    context: &NativeWriteContext<'_>,
    provenance: WriteProvenance,
    params: Value,
) -> Result<PreparedWrite, EngineError> {
    build_definition(
        context,
        provenance,
        parse_verb_params("datum.library.edit", params)?,
        false,
    )
}

pub fn build_definition(
    context: &NativeWriteContext<'_>,
    provenance: WriteProvenance,
    request: LibraryDefinitionRequest,
    create: bool,
) -> Result<PreparedWrite, EngineError> {
    if request.pool_path != "pool" {
        return Err(refusal(
            "Library authoring requires the canonical project-owned pool target",
        ));
    }
    if request.kind == "packages" {
        return package::build(context, provenance, request, create);
    }
    let allowed: &[&str] = match request.kind.as_str() {
        "padstacks" => &[
            "name",
            "aperture",
            "drill_nm",
            "plated",
            "layer_span",
            "mask_policy",
            "paste_policy",
        ],
        "footprints" => &[
            "name",
            "package",
            "pads",
            "courtyard",
            "silkscreen",
            "fab",
            "process_basis",
        ],
        _ => {
            return Err(refusal(
                "This library kind is available for inspection; authoring supports Footprint, Padstack and Package creation",
            ));
        }
    };
    if request
        .fields
        .keys()
        .any(|key| !allowed.contains(&key.as_str()))
    {
        return Err(refusal(
            "Library edit contains an unsupported field; identity and approval are not editable here",
        ));
    }
    let target = PoolLibraryObjectTarget::new("pool", &request.kind, request.object_id);
    if !create {
        let matches = context
            .model
            .source_shards
            .iter()
            .filter(|s| s.kind == SourceShardKind::Pool)
            .filter(|s| {
                context
                    .model
                    .materialized_source_shard_value_by_relative_path(&s.relative_path)
                    .ok()
                    .and_then(|v| v.get("uuid").and_then(Value::as_str).map(str::to_owned))
                    .is_some_and(|id| id == request.object_id.to_string())
            })
            .count();
        if matches != 1 {
            return Err(refusal(
                "Library edit identity has missing or ambiguous source candidates",
            ));
        }
    }
    let mut object = if create {
        match request.kind.as_str() {
            "padstacks" => json!({"schema_version":1,"uuid":request.object_id,"name":""}),
            _ => {
                let package = request
                    .fields
                    .get("package")
                    .and_then(Value::as_str)
                    .and_then(|s| Uuid::parse_str(s).ok())
                    .ok_or_else(|| {
                        refusal("Choose an existing named Package before creating a Footprint")
                    })?;
                library_definition::validate_pool_definition_reference(
                    context.model,
                    package,
                    "packages",
                    "package",
                )?;
                library_footprint::pool_footprint_payload(request.object_id, package, "")
            }
        }
    } else {
        let source = context
            .model
            .objects
            .get(&request.object_id)
            .filter(|o| o.domain == "pool" && o.kind == request.kind)
            .ok_or_else(|| refusal("Library edit target has no matching canonical source"))?;
        if !context.model.source_shards.iter().any(|s| {
            s.shard_id == source.source_shard_id
                && s.kind == SourceShardKind::Pool
                && s.relative_path == target.relative_path
        }) {
            return Err(refusal(
                "Library edit target is not owned by the canonical project pool path",
            ));
        }
        library::pool_library_object_payload(
            context.model,
            &target.relative_path,
            request.object_id,
        )?
    };
    let raw = object
        .as_object_mut()
        .ok_or_else(|| refusal("Library payload is not an object"))?;
    for (field, value) in request.fields {
        if field == "aperture" {
            // Admission applies to the submitted shape, before merging into
            // retained raw fields; malformed edits cannot become silent no-ops.
            let _: crate::pool::PadstackAperture = serde_json::from_value(value.clone())?;
        }
        // Selected pad records retain raw siblings just like their owning object.
        if field == "pads" && !create {
            merge_pads(raw, value)?;
        } else if field == "aperture"
            && let (Some(previous), Some(next)) = (
                raw.get_mut(&field).and_then(Value::as_object_mut),
                value.as_object(),
            )
        {
            let shape = if next.contains_key("circle") {
                "circle"
            } else {
                "rect"
            };
            if previous.contains_key(shape) {
                if let (Some(before), Some(after)) = (
                    previous.get_mut(shape).and_then(Value::as_object_mut),
                    next.get(shape).and_then(Value::as_object),
                ) {
                    before.extend(after.clone());
                }
            } else {
                for (old_shape, dimensions) in [
                    ("circle", &["diameter_nm"][..]),
                    ("rect", &["width_nm", "height_nm"][..]),
                ] {
                    if previous
                        .get(old_shape)
                        .and_then(Value::as_object)
                        .is_some_and(|fields| {
                            fields.keys().any(|key| !dimensions.contains(&key.as_str()))
                        })
                    {
                        return Err(refusal(
                            "Changing copper shape would discard raw geometry extension fields",
                        ));
                    }
                }
                previous.remove("circle");
                previous.remove("rect");
                previous.extend(next.clone());
            }
        } else {
            raw.insert(field, value);
        }
    }
    if object
        .get("name")
        .and_then(Value::as_str)
        .is_none_or(|n| n.trim().is_empty())
    {
        return Err(refusal("Library name must be non-empty"));
    }
    if request.kind == "padstacks" {
        validate_padstack(&object)?;
    } else {
        let _: crate::pool::Footprint = serde_json::from_value(object.clone())?;
        library_definition::validate_footprint_replacement_references(context.model, &object)?;
    }
    let operation = if create {
        PoolLibraryOperationSpec::Create { target, object }
    } else {
        PoolLibraryOperationSpec::Set { target, object }
    };
    library::build_pool_library_write(
        context.model,
        provenance,
        create.then_some("pool"),
        vec![operation],
    )
}

fn merge_pads(raw: &mut Map<String, Value>, next: Value) -> Result<(), EngineError> {
    let edits = next
        .as_object()
        .ok_or_else(|| refusal("Footprint pads must be a UUID-keyed map"))?;
    let current = raw
        .entry("pads")
        .or_insert_with(|| json!({}))
        .as_object_mut()
        .ok_or_else(|| refusal("Footprint pads source is not a map"))?;
    for (id, edit) in edits {
        Uuid::parse_str(id).map_err(|_| refusal("Footprint Pad identity is invalid"))?;
        if edit.is_null() {
            if current.remove(id).is_none() {
                return Err(refusal("Cannot remove a missing Footprint Pad"));
            }
            continue;
        }
        let fields = edit
            .as_object()
            .ok_or_else(|| refusal("Footprint Pad edit must be an object"))?;
        if fields
            .keys()
            .any(|k| !["uuid", "name", "position", "padstack", "layer"].contains(&k.as_str()))
        {
            return Err(refusal("Unsupported Footprint Pad field"));
        }
        let pad = current
            .entry(id.clone())
            .or_insert_with(|| json!({"uuid":id}))
            .as_object_mut()
            .ok_or_else(|| refusal("Footprint Pad source is not an object"))?;
        if fields.get("uuid").is_some_and(|v| v.as_str() != Some(id)) {
            return Err(refusal(
                "Footprint Pad UUID must match its retained identity",
            ));
        }
        for (k, v) in fields {
            if k == "position"
                && let (Some(previous), Some(next)) =
                    (pad.get_mut(k).and_then(Value::as_object_mut), v.as_object())
            {
                previous.extend(next.clone());
            } else {
                pad.insert(k.clone(), v.clone());
            }
        }
    }
    Ok(())
}

fn validate_padstack(raw: &Value) -> Result<(), EngineError> {
    let stack: crate::pool::Padstack = serde_json::from_value(raw.clone())?;
    use crate::pool::{PadstackAperture, PadstackMaskPolicy, PadstackPastePolicy};
    let dimensions: Vec<i64> = match stack.aperture {
        Some(PadstackAperture::Circle { diameter_nm }) => vec![diameter_nm],
        Some(PadstackAperture::Rect {
            width_nm,
            height_nm,
        }) => vec![width_nm, height_nm],
        None => return Err(refusal("Choose supported Circle or Rect copper geometry")),
    };
    if dimensions.iter().any(|d| *d <= 0) || stack.drill_nm.is_some_and(|d| d <= 0) {
        return Err(refusal(
            "Copper dimensions and any declared drill must be positive",
        ));
    }
    // Validate only declared concrete policies with the shared PM061 kernel.
    // Inherited policies need a Footprint basis and remain unresolved here.
    // Suppressed companion policies are validation inputs, never authored defaults.
    if stack.mask_policy != PadstackMaskPolicy::Inherit {
        let mut input = raw.clone();
        input["paste_policy"] = json!("none");
        crate::pool::process::resolve_padstack_process(&input, &json!({}), None)?;
    }
    if stack.paste_policy != PadstackPastePolicy::Inherit {
        let mut input = raw.clone();
        input["mask_policy"] = json!("tented");
        crate::pool::process::resolve_padstack_process(&input, &json!({}), None)?;
    }
    Ok(())
}

#[cfg(test)]
#[path = "library_gui_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "library_gui_package_tests.rs"]
mod package_tests;
