use super::*;

use eda_engine::ir::units::{
    AutomationLengthInput, AutomationLengthRefusal, ExplicitLengthContext, LengthQuantity,
    LengthUnit, MeasurementSystem, UnitSource, resolve_automation_length,
};

#[derive(Debug, Serialize)]
struct LengthContextView {
    project_id: Option<String>,
    field_id: String,
    quantity: &'static str,
    resolved_unit: &'static str,
    measurement_system: &'static str,
}

#[derive(Debug, Serialize)]
struct ResolveLengthReport {
    schema: &'static str,
    ok: bool,
    authored_token: Option<String>,
    canonical_nm: Option<i64>,
    unit_source: Option<&'static str>,
    resolved_unit: Option<&'static str>,
    context: Option<LengthContextView>,
    refusal: Option<String>,
}

pub(crate) fn execute_units_command(
    format: &OutputFormat,
    action: UnitsCommands,
) -> Result<(String, i32)> {
    match action {
        UnitsCommands::ResolveLength(args) => resolve_length(format, args),
    }
}

fn resolve_length(format: &OutputFormat, args: ResolveLengthArgs) -> Result<(String, i32)> {
    let context = match explicit_context(&args) {
        Ok(context) => context,
        Err(refusal) => return report_refusal(format, args.expression, refusal),
    };
    match resolve_automation_length(AutomationLengthInput {
        canonical_nm: args.canonical_nm,
        expression: args.expression.as_deref(),
        context: context.as_ref(),
    }) {
        Ok(result) => {
            let report = ResolveLengthReport {
                schema: "datum.units.resolve_length.v1",
                ok: true,
                authored_token: args.expression,
                canonical_nm: Some(result.canonical_nm),
                unit_source: Some(match result.unit_source {
                    UnitSource::ExplicitSuffix => "explicit_suffix",
                    UnitSource::Context => "context",
                }),
                resolved_unit: Some(unit_token(result.resolved_unit)),
                context: result.context.map(context_view),
                refusal: None,
            };
            Ok((render_output(format, &report), 0))
        }
        Err(refusal) => report_refusal(format, args.expression, format_refusal(&refusal)),
    }
}

fn explicit_context(
    args: &ResolveLengthArgs,
) -> std::result::Result<Option<ExplicitLengthContext>, String> {
    let supplied = [
        args.quantity.is_some(),
        args.unit.is_some(),
        args.system.is_some(),
        args.field.is_some(),
    ];
    if supplied.iter().all(|value| !value) {
        return Ok(None);
    }
    if supplied.iter().any(|value| !value) {
        return Err(
            "incomplete_context: quantity, unit, system, and field must be supplied together"
                .to_owned(),
        );
    }
    let quantity = match args.quantity.as_deref().unwrap() {
        "board" => LengthQuantity::BoardLayout,
        "drill" => LengthQuantity::DrillHole,
        "schematic" => LengthQuantity::SchematicGeometry,
        value => return Err(format!("unsupported_quantity: {value}")),
    };
    let resolved_unit = match args.unit.as_deref().unwrap() {
        "nm" => LengthUnit::Nanometer,
        "um" | "µm" => LengthUnit::Micrometer,
        "mm" => LengthUnit::Millimeter,
        "mil" => LengthUnit::Mil,
        "in" => LengthUnit::Inch,
        value => return Err(format!("unsupported_unit: {value}")),
    };
    let measurement_system = match args.system.as_deref().unwrap() {
        "metric" => MeasurementSystem::Metric,
        "imperial" => MeasurementSystem::Imperial,
        value => return Err(format!("unsupported_measurement_system: {value}")),
    };
    Ok(Some(ExplicitLengthContext {
        project_id: args.project_id.clone(),
        field_id: args.field.clone().unwrap(),
        quantity,
        resolved_unit,
        measurement_system,
    }))
}

fn report_refusal(
    format: &OutputFormat,
    authored_token: Option<String>,
    refusal: String,
) -> Result<(String, i32)> {
    let report = ResolveLengthReport {
        schema: "datum.units.resolve_length.v1",
        ok: false,
        authored_token,
        canonical_nm: None,
        unit_source: None,
        resolved_unit: None,
        context: None,
        refusal: Some(refusal),
    };
    Ok((render_output(format, &report), 2))
}

fn format_refusal(refusal: &AutomationLengthRefusal) -> String {
    match refusal {
        AutomationLengthRefusal::MissingInput => "missing_input".to_owned(),
        AutomationLengthRefusal::ConflictingCanonicalAndExpression => {
            "conflicting_canonical_nm_and_expression".to_owned()
        }
        AutomationLengthRefusal::MissingBareExpressionContext => {
            "missing_bare_expression_context".to_owned()
        }
        AutomationLengthRefusal::Units(refusal) => format!("units::{:?}", refusal.reason),
    }
}

fn context_view(context: ExplicitLengthContext) -> LengthContextView {
    LengthContextView {
        project_id: context.project_id,
        field_id: context.field_id,
        quantity: match context.quantity {
            LengthQuantity::BoardLayout => "board",
            LengthQuantity::DrillHole => "drill",
            LengthQuantity::SchematicGeometry => "schematic",
        },
        resolved_unit: unit_token(context.resolved_unit),
        measurement_system: match context.measurement_system {
            MeasurementSystem::Metric => "metric",
            MeasurementSystem::Imperial => "imperial",
        },
    }
}

fn unit_token(unit: LengthUnit) -> &'static str {
    match unit {
        LengthUnit::Nanometer => "nm",
        LengthUnit::Micrometer => "um",
        LengthUnit::Millimeter => "mm",
        LengthUnit::Mil => "mil",
        LengthUnit::Inch => "in",
    }
}
