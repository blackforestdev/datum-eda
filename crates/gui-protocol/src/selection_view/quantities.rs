//! Project-bound edge formatting delegates to the engine units owner. Exact
//! Common/Mixed/Unavailable aggregation is already complete before formatting.
use super::*;
use eda_engine::ir::units::{
    FormatLengthRequest, Nanometers, QuantityKind, ResolvedQuantityUnits, format_length,
};
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum QuantityPresentation {
    Formatted { rendered: String, rounded: bool },
    Unavailable { reason: String },
}
fn kind(class: Class, field: &str) -> Option<eda_engine::ir::units::LengthQuantity> {
    use eda_engine::ir::units::LengthQuantity;
    match field {
        "drill_diameter_nm" => Some(LengthQuantity::DrillHole),
        "placement_position_nm"
        | "position_nm"
        | "terminal_position_nm"
        | "section_start_nm"
        | "section_end_nm"
        | "section_midpoint_nm"
        | "pad_width_nm"
        | "pad_height_nm"
        | "pad_diameter_nm"
        | "copper_width_nm"
        | "copper_diameter_nm"
        | "text_height_nm"
        | "text_stroke_width_nm" => Some(match class {
            Class::Footprint
            | Class::Pad
            | Class::Track
            | Class::Via
            | Class::Zone
            | Class::BoardText
            | Class::BoardGraphic
            | Class::BoardOutline => LengthQuantity::BoardLayout,
            _ => LengthQuantity::SchematicGeometry,
        }),
        _ => None,
    }
}
fn render(
    value: &Value,
    units: ResolvedQuantityUnits,
    system: eda_engine::ir::units::MeasurementSystem,
) -> Option<QuantityPresentation> {
    let format = |nm| {
        format_length(FormatLengthRequest {
            value: Nanometers::new(nm),
            quantity: QuantityKind::Length,
            display_unit: units.unit,
            measurement_system: system,
            precision: units.precision,
        })
        .ok()
    };
    let (rendered, rounded) = if let Some(nm) = value.as_i64() {
        let text = format(nm)?;
        (text.rendered, text.rounded)
    } else {
        let x = format(value.get("x")?.as_i64()?)?;
        let y = format(value.get("y")?.as_i64()?)?;
        (
            format!("({}, {})", x.rendered, y.rendered),
            x.rounded || y.rounded,
        )
    };
    Some(QuantityPresentation::Formatted { rendered, rounded })
}
pub(super) fn append(model: &eda_engine::substrate::DesignModel, scopes: &mut [ScopeInspection]) {
    let context = eda_engine::api::native_write::project::project_display_units(model)
        .map_err(|error| error.to_string())
        .and_then(|profile| profile.resolve().map_err(|error| format!("{error:?}")));
    for scope in scopes {
        for (field, value) in &scope.fields {
            let InspectionValue::Common(value) = value else {
                continue;
            };
            let display = match (&context, field.as_str()) {
                (Ok(profile), "combined_bounds_nm" | "projected_bounds_nm") => {
                    Some(bounds(value, *profile).unwrap_or_else(|| {
                        QuantityPresentation::Unavailable {
                            reason:
                                "Canonical bounds cannot be formatted by the engine units service"
                                    .into(),
                        }
                    }))
                }
                (Err(reason), "combined_bounds_nm" | "projected_bounds_nm") => {
                    Some(QuantityPresentation::Unavailable {
                        reason: format!("Project quantity context unavailable: {reason}"),
                    })
                }
                (Ok(profile), "placement_rotation") => value.as_i64().and_then(|degrees| {
                    use eda_engine::ir::units::{CanonicalAngleScale, format_decimal_degrees};
                    let angle = format_decimal_degrees(
                        degrees,
                        CanonicalAngleScale::new(1)?,
                        profile.angle_precision.places(),
                    )
                    .ok()?;
                    Some(QuantityPresentation::Formatted {
                        rendered: angle.rendered,
                        rounded: angle.rounded,
                    })
                }),
                _ => None,
            };
            if let Some(display) = display {
                scope.quantities.insert(field.clone(), display);
            }
        }
        let Some(class) = scope.scope else { continue };
        for (field, value) in &scope.fields {
            let (Some(kind), InspectionValue::Common(value)) = (kind(class, field), value) else {
                continue;
            };
            let display = match &context {
                Ok(profile) => {
                    use eda_engine::ir::units::LengthQuantity;
                    let units = match kind {
                        LengthQuantity::BoardLayout => profile.board,
                        LengthQuantity::DrillHole => profile.drill,
                        LengthQuantity::SchematicGeometry => profile.schematic,
                    };
                    render(value, units, profile.system).unwrap_or_else(|| {
                        QuantityPresentation::Unavailable {
                            reason:
                                "Canonical quantity cannot be formatted by the engine units service"
                                    .into(),
                        }
                    })
                }
                Err(reason) => QuantityPresentation::Unavailable {
                    reason: format!("Project quantity context unavailable: {reason}"),
                },
            };
            scope.quantities.insert(field.clone(), display);
        }
    }
}

fn bounds(
    value: &Value,
    profile: eda_engine::ir::units::ResolvedUnitsProfile,
) -> Option<QuantityPresentation> {
    let frames = value.as_object()?;
    let mut rows = Vec::new();
    let mut rounded = false;
    for (frame, bounds) in frames.iter().take(2) {
        let (name, units) = if frame.starts_with("board:") {
            ("Board", profile.board)
        } else {
            ("Schematic", profile.schematic)
        };
        let mut coordinates = Vec::new();
        for key in ["min_x", "min_y", "max_x", "max_y"] {
            let value = &bounds[key];
            let numerator = value["numerator"].as_str()?.parse::<i128>().ok()?;
            let denominator = value["denominator"].as_str()?.parse::<i128>().ok()?;
            if denominator <= 0 {
                return None;
            }
            let text = if numerator % denominator == 0 {
                let nm = i64::try_from(numerator / denominator).ok()?;
                let text = format_length(FormatLengthRequest {
                    value: Nanometers::new(nm),
                    quantity: QuantityKind::Length,
                    display_unit: units.unit,
                    measurement_system: profile.system,
                    precision: units.precision,
                })
                .ok()?;
                rounded |= text.rounded;
                text.rendered
            } else {
                format!("{numerator}/{denominator} nm")
            };
            coordinates.push(text);
        }
        rows.push(format!(
            "{name}: ({}, {}) to ({}, {})",
            coordinates[0], coordinates[1], coordinates[2], coordinates[3]
        ));
    }
    let rendered = if rows.is_empty() {
        "No visible bounds".into()
    } else {
        format!(
            "{}{}",
            rows.join("; "),
            if frames.len() > 2 {
                format!("; {} views total", frames.len())
            } else {
                String::new()
            }
        )
    };
    Some(QuantityPresentation::Formatted { rendered, rounded })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn bounds_use_project_units_and_preserve_exact_fractional_nanometers() {
        let source = serde_json::json!({"board:example": {
            "min_x": {"numerator":"1000000","denominator":"1"},
            "min_y": {"numerator":"2000000","denominator":"1"},
            "max_x": {"numerator":"3000000","denominator":"1"},
            "max_y": {"numerator":"1","denominator":"2"}
        }});
        let original = source.clone();
        let mut profile = eda_engine::ir::units::FACTORY_UNITS_PROFILE_V1;
        let QuantityPresentation::Formatted { rendered, .. } =
            bounds(&source, profile.resolve().unwrap()).unwrap()
        else {
            panic!();
        };
        assert!(rendered.contains("1.000mm"), "{rendered}");
        assert!(rendered.contains("1/2 nm"), "{rendered}");
        assert!(!rendered.contains('{'));
        profile.system = eda_engine::ir::units::MeasurementSystem::Imperial;
        let QuantityPresentation::Formatted { rendered, .. } =
            bounds(&source, profile.resolve().unwrap()).unwrap()
        else {
            panic!();
        };
        assert!(rendered.contains("mil"), "{rendered}");
        assert_eq!(source, original);
        for name in ["pad_width_nm", "pad_height_nm", "pad_diameter_nm"] {
            assert_eq!(
                kind(Class::Pad, name),
                Some(eda_engine::ir::units::LengthQuantity::BoardLayout)
            );
        }
    }
}
