//! Shared manual library definition validation and payload construction.
//!
//! These builders preserve the existing CLI contract, including optional copper
//! apertures, positive optional drills, unrestricted positive pad layers and
//! project-wide pool references. They introduce no SMD or manufacturing policy.
//! Callers retain pool-path resolution and commit the payloads through
//! `library::build_pool_library_write`; these functions never write source files.

use uuid::Uuid;

use crate::error::EngineError;
use crate::substrate::DesignModel;

use super::library::pool_padstack_payload;

/// Independently supplied aperture arguments, retaining conflicting-input errors.
#[derive(Debug, Clone, Copy, Default)]
pub struct PadstackDefinitionInput<'a> {
    pub aperture: Option<&'a str>,
    pub diameter_nm: Option<i64>,
    pub width_nm: Option<i64>,
    pub height_nm: Option<i64>,
    pub drill_nm: Option<i64>,
}

/// Build the historical padstack payload after validating its manual inputs.
/// Names and absent apertures/drills retain their existing accepted behavior.
pub fn validated_padstack_payload(
    padstack_id: Uuid,
    name: &str,
    input: PadstackDefinitionInput<'_>,
) -> Result<serde_json::Value, EngineError> {
    if input.drill_nm.is_some_and(|value| value <= 0) {
        return Err(validation("padstack drill-nm must be positive"));
    }
    let aperture = match input.aperture {
        None => {
            if input.diameter_nm.is_some() || input.width_nm.is_some() || input.height_nm.is_some()
            {
                return Err(validation(
                    "padstack aperture dimensions require --aperture circle or rect",
                ));
            }
            serde_json::Value::Null
        }
        Some("circle") => {
            if input.width_nm.is_some() || input.height_nm.is_some() {
                return Err(validation(
                    "circle padstack aperture does not accept width-nm or height-nm",
                ));
            }
            let diameter = positive_required_dimension(input.diameter_nm, "diameter-nm")?;
            serde_json::json!({"circle": {"diameter_nm": diameter}})
        }
        Some("rect") => {
            if input.diameter_nm.is_some() {
                return Err(validation(
                    "rect padstack aperture does not accept diameter-nm",
                ));
            }
            let width = positive_required_dimension(input.width_nm, "width-nm")?;
            let height = positive_required_dimension(input.height_nm, "height-nm")?;
            serde_json::json!({"rect": {"width_nm": width, "height_nm": height}})
        }
        Some(kind) => {
            return Err(validation(format!(
                "unsupported padstack aperture {kind}; expected circle or rect"
            )));
        }
    };
    Ok(pool_padstack_payload(
        padstack_id,
        name,
        aperture,
        input.drill_nm,
    ))
}

/// One pad insertion or replacement, identified by its existing stable UUID.
#[derive(Debug, Clone, Copy)]
pub struct FootprintPadDefinitionInput<'a> {
    pub pad_id: Uuid,
    pub padstack_id: Uuid,
    pub name: &'a str,
    pub x_nm: i64,
    pub y_nm: i64,
    pub layer: i32,
}

impl FootprintPadDefinitionInput<'_> {
    /// Validate scalar inputs before resolving references, preserving error order.
    pub fn validate(&self) -> Result<(), EngineError> {
        if self.name.trim().is_empty() {
            return Err(validation("footprint pad name must be non-empty"));
        }
        if self.layer <= 0 {
            return Err(validation("footprint pad layer must be positive"));
        }
        Ok(())
    }
}

/// Require a pool object of the specified kind in the resolved Project.
/// This intentionally does not restrict the reference to the destination pool.
pub fn validate_pool_definition_reference(
    model: &DesignModel,
    object_id: Uuid,
    kind: &str,
    label: &str,
) -> Result<(), EngineError> {
    if model
        .objects
        .get(&object_id)
        .filter(|object| object.domain == "pool" && object.kind == kind)
        .is_none()
    {
        return Err(validation(format!("missing pool {label} {object_id}")));
    }
    Ok(())
}

/// Check the existing Package/Padstack links of a manual generic Footprint edit.
/// Schema and shape errors remain owned by canonical pool admission; an invalid
/// shape is deferred there so its established diagnostic precedes link errors.
/// This checks the resolved Project, without imposing destination-pool policy.
pub fn validate_footprint_replacement_references(
    model: &DesignModel,
    object: &serde_json::Value,
) -> Result<(), EngineError> {
    if object
        .get("schema_version")
        .and_then(serde_json::Value::as_u64)
        != Some(1)
    {
        return Ok(());
    }
    let Ok(footprint) = serde_json::from_value::<crate::pool::Footprint>(object.clone()) else {
        return Ok(());
    };
    validate_pool_definition_reference(model, footprint.package, "packages", "package")?;
    let mut pads: Vec<_> = footprint.pads.iter().collect();
    pads.sort_by_key(|(id, _)| **id);
    for (_, pad) in pads {
        validate_pool_definition_reference(model, pad.padstack, "padstacks", "padstack")?;
    }
    Ok(())
}

/// Check existing Package/Padstack links of a manual generic Footprint create.
/// Canonical shape, source-path and root/leaf UUID errors retain precedence.
/// This only supplements admission; it neither creates objects nor changes
/// the source owner's collision policy or the replacement helper's behavior.
pub fn validate_footprint_creation_references(
    model: &DesignModel,
    relative_path: &str,
    object: &serde_json::Value,
) -> Result<(), EngineError> {
    if object
        .get("schema_version")
        .and_then(serde_json::Value::as_u64)
        != Some(1)
    {
        return Ok(());
    }
    let Ok(footprint) = serde_json::from_value::<crate::pool::Footprint>(object.clone()) else {
        return Ok(());
    };
    if crate::substrate::pool_creation_has_source_blocker(
        &model.objects,
        footprint.uuid,
        relative_path,
        object,
        "footprints",
    ) {
        return Ok(());
    }
    validate_footprint_replacement_references(model, object)
}

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
    Ok(())
}

/// Replace only a Symbol's fields after admitting the existing field schema.
/// Keep the raw array so order, duplicates, unknown metadata and absent defaults survive.
pub fn symbol_with_fields(
    mut object: serde_json::Value,
    fields: serde_json::Value,
) -> Result<serde_json::Value, EngineError> {
    serde_json::from_value::<Vec<crate::pool::LibrarySymbolField>>(fields.clone())
        .map_err(|error| validation(format!("invalid symbol fields: {error}")))?;
    object
        .as_object_mut()
        .ok_or_else(|| validation("symbol payload must be an object"))?
        .insert("fields".to_string(), fields);
    Ok(object)
}

/// Check the existing Unit link of a manual generic Symbol Create.
/// Canonical schema, source ownership and UUID collision errors retain priority.
pub fn validate_symbol_creation_references(
    model: &DesignModel,
    relative_path: &str,
    object: &serde_json::Value,
) -> Result<(), EngineError> {
    validate_symbol_definition(model, relative_path, object, false)
}

/// Check the same Unit link of a manual generic Symbol Set.
pub fn validate_symbol_replacement_references(
    model: &DesignModel,
    relative_path: &str,
    object: &serde_json::Value,
) -> Result<(), EngineError> {
    validate_symbol_definition(model, relative_path, object, true)
}

fn validate_symbol_definition(
    model: &DesignModel,
    relative_path: &str,
    object: &serde_json::Value,
    replacement: bool,
) -> Result<(), EngineError> {
    if object
        .get("schema_version")
        .and_then(serde_json::Value::as_u64)
        != Some(1)
    {
        return Ok(());
    }
    let Ok(symbol) = serde_json::from_value::<crate::pool::Symbol>(object.clone()) else {
        return Ok(());
    };
    let blocked = if replacement {
        crate::substrate::pool_replacement_has_source_blocker(
            &model.objects,
            symbol.uuid,
            relative_path,
            object,
            "symbols",
        )
    } else {
        crate::substrate::pool_creation_has_source_blocker(
            &model.objects,
            symbol.uuid,
            relative_path,
            object,
            "symbols",
        )
    };
    if blocked {
        return Ok(());
    }
    validate_pool_definition_reference(model, symbol.unit, "units", "unit").map_err(|error| {
        match error {
            EngineError::Validation(message) => {
                validation(format!("{message} for symbol {}", symbol.uuid))
            }
            other => other,
        }
    })
}

/// Insert or replace a pad in an already materialized, UUID-checked footprint.
/// The caller validates the padstack reference against its resolved model before
/// materializing this payload. Unrelated and unknown payload fields are retained.
pub fn footprint_with_pad(
    footprint_id: Uuid,
    mut object: serde_json::Value,
    input: FootprintPadDefinitionInput<'_>,
) -> Result<serde_json::Value, EngineError> {
    input.validate()?;
    let object_map = object
        .as_object_mut()
        .ok_or_else(|| validation(format!("footprint {footprint_id} payload is not an object")))?;
    let pads = object_map
        .entry("pads".to_string())
        .or_insert_with(|| serde_json::json!({}));
    let pads = pads.as_object_mut().ok_or_else(|| {
        validation(format!(
            "footprint {footprint_id} pads field is not an object"
        ))
    })?;
    pads.insert(
        input.pad_id.to_string(),
        serde_json::json!({
            "uuid": input.pad_id,
            "name": input.name,
            "position": {"x": input.x_nm, "y": input.y_nm},
            "padstack": input.padstack_id,
            "layer": input.layer
        }),
    );
    Ok(object)
}

fn validation(message: impl Into<String>) -> EngineError {
    EngineError::Validation(message.into())
}

fn positive_required_dimension(value: Option<i64>, name: &str) -> Result<i64, EngineError> {
    match value {
        Some(value) if value > 0 => Ok(value),
        Some(_) => Err(validation(format!("padstack {name} must be positive"))),
        None => Err(validation(format!("padstack {name} is required"))),
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_support::resolved_model_with_board_package;
    use super::*;

    fn error_message(error: EngineError) -> String {
        match error {
            EngineError::Validation(message) => message,
            other => panic!("unexpected error: {other}"),
        }
    }

    #[test]
    fn padstack_preserves_optional_aperture_drill_and_exact_payload() {
        let id = Uuid::new_v4();
        for drill_nm in [None, Some(1)] {
            let payload = validated_padstack_payload(
                id,
                "",
                PadstackDefinitionInput {
                    drill_nm,
                    ..Default::default()
                },
            )
            .unwrap();
            assert_eq!(
                payload,
                serde_json::json!({
                    "schema_version": 1, "uuid": id, "name": "",
                    "aperture": null, "drill_nm": drill_nm
                })
            );
        }
        for (input, aperture) in [
            (
                PadstackDefinitionInput {
                    aperture: Some("circle"),
                    diameter_nm: Some(1),
                    drill_nm: Some(1),
                    ..Default::default()
                },
                serde_json::json!({"circle": {"diameter_nm": 1}}),
            ),
            (
                PadstackDefinitionInput {
                    aperture: Some("rect"),
                    width_nm: Some(i64::MAX),
                    height_nm: Some(1),
                    ..Default::default()
                },
                serde_json::json!({"rect": {"width_nm": i64::MAX, "height_nm": 1}}),
            ),
        ] {
            let payload = validated_padstack_payload(id, "Entered", input).unwrap();
            assert_eq!(payload["aperture"], aperture);
            assert_eq!(payload["drill_nm"], serde_json::json!(input.drill_nm));
            assert!(payload.get("layer_span").is_none());
            assert!(payload.get("mask_policy").is_none());
            assert!(payload.get("paste_policy").is_none());
        }
    }

    #[test]
    fn padstack_rejects_conflicts_missing_and_nonpositive_dimensions_in_order() {
        let id = Uuid::new_v4();
        let default = PadstackDefinitionInput::default();
        let cases = [
            (
                PadstackDefinitionInput {
                    drill_nm: Some(0),
                    aperture: Some("unknown"),
                    ..default
                },
                "padstack drill-nm must be positive",
            ),
            (
                PadstackDefinitionInput {
                    drill_nm: Some(-1),
                    ..default
                },
                "padstack drill-nm must be positive",
            ),
            (
                PadstackDefinitionInput {
                    diameter_nm: Some(1),
                    ..default
                },
                "padstack aperture dimensions require --aperture circle or rect",
            ),
            (
                PadstackDefinitionInput {
                    aperture: Some("circle"),
                    width_nm: Some(1),
                    ..default
                },
                "circle padstack aperture does not accept width-nm or height-nm",
            ),
            (
                PadstackDefinitionInput {
                    aperture: Some("circle"),
                    ..default
                },
                "padstack diameter-nm is required",
            ),
            (
                PadstackDefinitionInput {
                    aperture: Some("circle"),
                    diameter_nm: Some(0),
                    ..default
                },
                "padstack diameter-nm must be positive",
            ),
            (
                PadstackDefinitionInput {
                    aperture: Some("rect"),
                    diameter_nm: Some(1),
                    ..default
                },
                "rect padstack aperture does not accept diameter-nm",
            ),
            (
                PadstackDefinitionInput {
                    aperture: Some("rect"),
                    ..default
                },
                "padstack width-nm is required",
            ),
            (
                PadstackDefinitionInput {
                    aperture: Some("rect"),
                    width_nm: Some(-1),
                    ..default
                },
                "padstack width-nm must be positive",
            ),
            (
                PadstackDefinitionInput {
                    aperture: Some("rect"),
                    width_nm: Some(1),
                    ..default
                },
                "padstack height-nm is required",
            ),
            (
                PadstackDefinitionInput {
                    aperture: Some("rect"),
                    width_nm: Some(1),
                    height_nm: Some(0),
                    ..default
                },
                "padstack height-nm must be positive",
            ),
            (
                PadstackDefinitionInput {
                    aperture: Some("Circle"),
                    ..default
                },
                "unsupported padstack aperture Circle; expected circle or rect",
            ),
        ];
        for (input, expected) in cases {
            assert_eq!(
                error_message(validated_padstack_payload(id, "P", input).unwrap_err()),
                expected
            );
        }
    }

    #[test]
    fn footprint_pad_replaces_same_uuid_without_losing_unrelated_fields() {
        let footprint_id = Uuid::new_v4();
        let pad_id = Uuid::new_v4();
        let other_pad = Uuid::new_v4();
        let input = FootprintPadDefinitionInput {
            pad_id,
            padstack_id: Uuid::new_v4(),
            name: " 1 ",
            x_nm: i64::MIN,
            y_nm: i64::MAX,
            layer: i32::MAX,
        };
        let prior = serde_json::json!({
            "uuid": footprint_id, "custom": {"retained": true},
            "pads": {pad_id.to_string(): {"name": "old"}, other_pad.to_string(): {"name": "2"}}
        });
        let next = footprint_with_pad(footprint_id, prior.clone(), input).unwrap();
        assert_eq!(next["custom"], prior["custom"]);
        assert_eq!(
            next["pads"][other_pad.to_string()],
            prior["pads"][other_pad.to_string()]
        );
        assert_eq!(next["pads"].as_object().unwrap().len(), 2);
        assert_eq!(
            next["pads"][pad_id.to_string()],
            serde_json::json!({
                "uuid": pad_id, "name": " 1 ", "padstack": input.padstack_id,
                "position": {"x": i64::MIN, "y": i64::MAX}, "layer": i32::MAX
            })
        );
        let without_pads = footprint_with_pad(footprint_id, serde_json::json!({}), input).unwrap();
        assert_eq!(
            without_pads["pads"][pad_id.to_string()],
            next["pads"][pad_id.to_string()]
        );
    }

    #[test]
    fn footprint_pad_refuses_invalid_scalars_and_payload_shape() {
        let id = Uuid::new_v4();
        let mut input = FootprintPadDefinitionInput {
            pad_id: Uuid::new_v4(),
            padstack_id: Uuid::new_v4(),
            name: " \t ",
            x_nm: 0,
            y_nm: 0,
            layer: 0,
        };
        assert_eq!(
            error_message(input.validate().unwrap_err()),
            "footprint pad name must be non-empty"
        );
        input.name = "1";
        for layer in [0, -1] {
            input.layer = layer;
            assert_eq!(
                error_message(input.validate().unwrap_err()),
                "footprint pad layer must be positive"
            );
        }
        input.layer = 1;
        assert_eq!(
            error_message(footprint_with_pad(id, serde_json::Value::Null, input).unwrap_err()),
            format!("footprint {id} payload is not an object")
        );
        assert_eq!(
            error_message(
                footprint_with_pad(id, serde_json::json!({"pads": []}), input).unwrap_err()
            ),
            format!("footprint {id} pads field is not an object")
        );
    }

    #[test]
    fn reference_validation_requires_both_pool_domain_and_requested_kind() {
        let (root, model, _board, package) = resolved_model_with_board_package("definition_ref");
        // A real board package must not masquerade as a pool definition.
        assert_eq!(
            error_message(
                validate_pool_definition_reference(&model, package, "packages", "package")
                    .unwrap_err()
            ),
            format!("missing pool package {package}")
        );
        let absent = Uuid::new_v4();
        assert_eq!(
            error_message(
                validate_pool_definition_reference(&model, absent, "padstacks", "padstack")
                    .unwrap_err()
            ),
            format!("missing pool padstack {absent}")
        );
        std::fs::remove_dir_all(root).unwrap();
    }
}
