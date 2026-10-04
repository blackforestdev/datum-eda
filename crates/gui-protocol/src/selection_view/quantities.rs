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
