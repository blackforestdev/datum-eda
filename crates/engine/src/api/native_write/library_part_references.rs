//! Shared public manual Part admission; canonical/import/replay writers remain unchanged.
use super::{validate_pool_definition_reference, validation};
use crate::{error::EngineError, substrate::DesignModel};

/// Check the four existing Part links of a manual generic Create.
/// Canonical schema, source ownership and UUID collision errors retain priority.
pub fn validate_part_creation_references(
    model: &DesignModel,
    relative_path: &str,
    object: &serde_json::Value,
) -> Result<(), EngineError> {
    let Some(part) = part_reference_payload(object) else {
        return Ok(());
    };
    if crate::substrate::pool_creation_has_source_blocker(
        &model.objects,
        part.uuid,
        relative_path,
        object,
        "parts",
    ) {
        return Ok(());
    }
    validate_part_links(model, &part)
}

/// Check the same Part links of a manual generic Set, retaining same-owner IDs.
/// Optional null/omitted defaults and unknown fields keep their current behavior.
pub fn validate_part_replacement_references(
    model: &DesignModel,
    relative_path: &str,
    object: &serde_json::Value,
) -> Result<(), EngineError> {
    let Some(part) = part_reference_payload(object) else {
        return Ok(());
    };
    if crate::substrate::pool_replacement_has_source_blocker(
        &model.objects,
        part.uuid,
        relative_path,
        object,
        "parts",
    ) {
        return Ok(());
    }
    validate_part_links(model, &part)
}

fn part_reference_payload(object: &serde_json::Value) -> Option<crate::pool::Part> {
    if object
        .get("schema_version")
        .and_then(serde_json::Value::as_u64)
        != Some(1)
    {
        return None;
    }
    serde_json::from_value(object.clone()).ok()
}

fn validate_part_links(model: &DesignModel, part: &crate::pool::Part) -> Result<(), EngineError> {
    for (reference, kind, label) in [
        (Some(part.entity), "entities", "entity"),
        (Some(part.package), "packages", "package"),
        (part.default_footprint, "footprints", "footprint"),
        (part.default_pin_pad_map, "pin_pad_maps", "pin_pad_map"),
    ] {
        if let Some(id) = reference {
            validate_pool_definition_reference(model, id, kind, label).map_err(
                |error| match error {
                    EngineError::Validation(message) => {
                        validation(format!("{message} for part {}", part.uuid))
                    }
                    other => other,
                },
            )?;
        }
    }
    if let Some(id) = part.default_footprint {
        let footprint =
            super::super::schematic_library::materialized_pool_object(model, id, "footprints")?;
        let package = footprint
            .get("package")
            .and_then(serde_json::Value::as_str)
            .and_then(|value| value.parse::<uuid::Uuid>().ok())
            .ok_or_else(|| validation(format!("missing package for pool footprint {id}")))?;
        if package != part.package {
            return Err(validation(format!(
                "part default_footprint {id} belongs to package {package}, not part package {}",
                part.package
            )));
        }
    }
    if let Some(id) = part.default_pin_pad_map {
        let map =
            super::super::schematic_library::materialized_pool_object(model, id, "pin_pad_maps")?;
        let owner = map
            .get("part")
            .and_then(serde_json::Value::as_str)
            .and_then(|value| value.parse::<uuid::Uuid>().ok())
            .ok_or_else(|| validation(format!("missing part for pool pin_pad_map {id}")))?;
        if owner != part.uuid {
            return Err(validation(format!(
                "part default_pin_pad_map {id} belongs to part {owner}, not {}",
                part.uuid
            )));
        }
    }
    Ok(())
}

#[cfg(test)]
#[path = "library_part_footprint_pair_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "library_part_map_owner_tests.rs"]
mod map_owner_tests;
