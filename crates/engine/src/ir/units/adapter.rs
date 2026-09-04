use super::{
    DisplayPrecision, LengthQuantity, LengthUnit, MeasurementSystem, ParseLengthRequest,
    QuantityKind, UnitSource, UnitsRefusal, parse_length,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExplicitLengthContext {
    pub project_id: Option<String>,
    pub field_id: String,
    pub quantity: LengthQuantity,
    pub resolved_unit: LengthUnit,
    pub measurement_system: MeasurementSystem,
}

#[derive(Debug, Clone, Copy)]
pub struct AutomationLengthInput<'a> {
    pub canonical_nm: Option<i64>,
    pub expression: Option<&'a str>,
    pub context: Option<&'a ExplicitLengthContext>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AutomationLengthResult {
    pub canonical_nm: i64,
    pub unit_source: UnitSource,
    pub resolved_unit: LengthUnit,
    pub context: Option<ExplicitLengthContext>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AutomationLengthRefusal {
    MissingInput,
    ConflictingCanonicalAndExpression,
    MissingBareExpressionContext,
    Units(UnitsRefusal),
}

/// Shared CLI/MCP compatibility seam. Existing `_nm` values remain exact; an
/// expression sibling is mutually exclusive. Explicit suffixes are profile-
/// independent, while a bare scalar must carry a named field context.
pub fn resolve_automation_length(
    request: AutomationLengthInput<'_>,
) -> Result<AutomationLengthResult, AutomationLengthRefusal> {
    match (request.canonical_nm, request.expression) {
        (Some(_), Some(_)) => Err(AutomationLengthRefusal::ConflictingCanonicalAndExpression),
        (None, None) => Err(AutomationLengthRefusal::MissingInput),
        (Some(value), None) => Ok(AutomationLengthResult {
            canonical_nm: value,
            unit_source: UnitSource::ExplicitSuffix,
            resolved_unit: LengthUnit::Nanometer,
            context: request.context.cloned(),
        }),
        (None, Some(expression)) => resolve_expression(expression, request.context),
    }
}

fn resolve_expression(
    expression: &str,
    context: Option<&ExplicitLengthContext>,
) -> Result<AutomationLengthResult, AutomationLengthRefusal> {
    let has_explicit_suffix = expression
        .trim_end()
        .chars()
        .last()
        .is_some_and(|character| character.is_ascii_alphabetic() || character == 'µ');
    if !has_explicit_suffix && context.is_none() {
        return Err(AutomationLengthRefusal::MissingBareExpressionContext);
    }
    let contextual_unit = context.map(|context| context.resolved_unit);
    let measurement_system = context
        .map(|context| context.measurement_system)
        .unwrap_or(MeasurementSystem::Metric);
    let parsed = parse_length(ParseLengthRequest {
        quantity: QuantityKind::Length,
        token: expression,
        contextual_unit,
        measurement_system,
        precision: DisplayPrecision::ExactNanometer,
    })
    .map_err(AutomationLengthRefusal::Units)?;
    Ok(AutomationLengthResult {
        canonical_nm: parsed.canonical_value.get(),
        unit_source: parsed.unit_source,
        resolved_unit: parsed.resolved_unit,
        context: context.cloned(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_and_expression_fields_are_mutually_exclusive() {
        assert_eq!(
            resolve_automation_length(AutomationLengthInput {
                canonical_nm: Some(1),
                expression: Some("1nm"),
                context: None,
            }),
            Err(AutomationLengthRefusal::ConflictingCanonicalAndExpression)
        );
    }

    #[test]
    fn explicit_suffix_needs_no_machine_or_project_profile() {
        let result = resolve_automation_length(AutomationLengthInput {
            canonical_nm: None,
            expression: Some("5.08mm"),
            context: None,
        })
        .unwrap();
        assert_eq!(result.canonical_nm, 5_080_000);
        assert_eq!(result.context, None);
    }

    #[test]
    fn bare_expression_requires_and_preserves_named_context() {
        assert_eq!(
            resolve_automation_length(AutomationLengthInput {
                canonical_nm: None,
                expression: Some("5.08"),
                context: None,
            }),
            Err(AutomationLengthRefusal::MissingBareExpressionContext)
        );
        let context = ExplicitLengthContext {
            project_id: Some("sensor-node".to_owned()),
            field_id: "board.track.width".to_owned(),
            quantity: LengthQuantity::BoardLayout,
            resolved_unit: LengthUnit::Millimeter,
            measurement_system: MeasurementSystem::Metric,
        };
        let result = resolve_automation_length(AutomationLengthInput {
            canonical_nm: None,
            expression: Some("5.08"),
            context: Some(&context),
        })
        .unwrap();
        assert_eq!(result.canonical_nm, 5_080_000);
        assert_eq!(result.context, Some(context));
    }
}
