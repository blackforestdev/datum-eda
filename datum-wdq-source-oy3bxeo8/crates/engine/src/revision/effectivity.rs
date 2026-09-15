use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use super::{AuthorityDiagnostic, EffectivityId};

pub const EFFECTIVITY_NODE_KINDS: &[&str] = &["selector", "all_of", "any_of", "not"];

pub const EFFECTIVITY_SELECTOR_FAMILIES: &[EffectivitySelectorFamily] = &[
    EffectivitySelectorFamily::ProductOrAssembly,
    EffectivitySelectorFamily::ConfigurationItem,
    EffectivitySelectorFamily::Variant,
    EffectivitySelectorFamily::BoardOrSubassembly,
    EffectivitySelectorFamily::SerialRange,
    EffectivitySelectorFamily::LotOrBatch,
    EffectivitySelectorFamily::Build,
    EffectivitySelectorFamily::Customer,
    EffectivitySelectorFamily::Site,
    EffectivitySelectorFamily::Contract,
    EffectivitySelectorFamily::Date,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EffectivitySelectorFamily {
    ProductOrAssembly,
    ConfigurationItem,
    Variant,
    BoardOrSubassembly,
    SerialRange,
    LotOrBatch,
    Build,
    Customer,
    Site,
    Contract,
    Date,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EffectivitySelector {
    pub family: EffectivitySelectorFamily,
    pub values: BTreeSet<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "node", content = "value", rename_all = "snake_case")]
pub enum EffectivityExpression {
    Selector(EffectivitySelector),
    AllOf(Vec<EffectivityExpression>),
    AnyOf(Vec<EffectivityExpression>),
    Not(Box<EffectivityExpression>),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EffectivityData {
    pub expression: EffectivityExpression,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub supersedes: Option<EffectivityId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resolved_population_snapshot: Option<BTreeSet<String>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EffectivityResolutionContext {
    pub population: BTreeSet<String>,
    pub attributes: BTreeMap<EffectivitySelectorFamily, BTreeMap<String, BTreeSet<String>>>,
    pub supported_families: BTreeSet<EffectivitySelectorFamily>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EffectivityResolution {
    Universal,
    Resolved {
        members: BTreeSet<String>,
        exact_snapshot: Option<BTreeSet<String>>,
    },
    Refused(AuthorityDiagnostic),
}

pub fn resolve_effectivity(
    effectivity: Option<&EffectivityData>,
    context: &EffectivityResolutionContext,
) -> EffectivityResolution {
    let Some(effectivity) = effectivity else {
        return EffectivityResolution::Universal;
    };
    match evaluate(&effectivity.expression, context) {
        Ok(members) => EffectivityResolution::Resolved {
            members,
            exact_snapshot: effectivity.resolved_population_snapshot.clone(),
        },
        Err(diagnostic) => EffectivityResolution::Refused(diagnostic),
    }
}

fn evaluate(
    expression: &EffectivityExpression,
    context: &EffectivityResolutionContext,
) -> Result<BTreeSet<String>, AuthorityDiagnostic> {
    match expression {
        EffectivityExpression::Selector(selector) => evaluate_selector(selector, context),
        EffectivityExpression::AllOf(children) => {
            if children.is_empty() {
                return Err(diag(
                    "revision_effectivity_ambiguous",
                    "AllOf requires at least one child",
                ));
            }
            let mut children = children.iter();
            let mut result = evaluate(children.next().expect("checked non-empty"), context)?;
            for child in children {
                let next = evaluate(child, context)?;
                result = result.intersection(&next).cloned().collect();
            }
            Ok(result)
        }
        EffectivityExpression::AnyOf(children) => {
            if children.is_empty() {
                return Err(diag(
                    "revision_effectivity_ambiguous",
                    "AnyOf requires at least one child",
                ));
            }
            let mut result = BTreeSet::new();
            for child in children {
                result.extend(evaluate(child, context)?);
            }
            Ok(result)
        }
        EffectivityExpression::Not(child) => Ok(context
            .population
            .difference(&evaluate(child, context)?)
            .cloned()
            .collect()),
    }
}

fn evaluate_selector(
    selector: &EffectivitySelector,
    context: &EffectivityResolutionContext,
) -> Result<BTreeSet<String>, AuthorityDiagnostic> {
    if !context.supported_families.contains(&selector.family) {
        return Err(diag(
            "revision_effectivity_unsupported_selector",
            "selector family is unsupported in this resolution context",
        ));
    }
    if selector.values.is_empty() {
        return Err(diag(
            "revision_effectivity_unknown_selector",
            "selector has no requested values",
        ));
    }
    let Some(by_value) = context.attributes.get(&selector.family) else {
        return Err(diag(
            "revision_effectivity_unresolvable_selector",
            "resolution context has no values for selector family",
        ));
    };
    let mut result = BTreeSet::new();
    for value in &selector.values {
        let Some(members) = by_value.get(value) else {
            return Err(diag(
                "revision_effectivity_unknown_selector",
                "selector value is unknown in this resolution context",
            ));
        };
        result.extend(members.iter().cloned());
    }
    if !result.is_subset(&context.population) {
        return Err(diag(
            "revision_effectivity_ambiguous_population",
            "selector resolves outside the declared population",
        ));
    }
    Ok(result)
}

pub(crate) fn validate_effectivity(data: &EffectivityData) -> Vec<AuthorityDiagnostic> {
    let mut diagnostics = Vec::new();
    validate_expression(&data.expression, &mut diagnostics);
    diagnostics
}

fn validate_expression(
    expression: &EffectivityExpression,
    diagnostics: &mut Vec<AuthorityDiagnostic>,
) {
    match expression {
        EffectivityExpression::Selector(selector) if selector.values.is_empty() => {
            diagnostics.push(diag(
                "revision_effectivity_unknown_selector",
                "selector values cannot be empty",
            ));
        }
        EffectivityExpression::AllOf(children) | EffectivityExpression::AnyOf(children) => {
            if children.is_empty() {
                diagnostics.push(diag(
                    "revision_effectivity_ambiguous",
                    "boolean effectivity node cannot be empty",
                ));
            }
            for child in children {
                validate_expression(child, diagnostics);
            }
        }
        EffectivityExpression::Not(child) => validate_expression(child, diagnostics),
        EffectivityExpression::Selector(_) => {}
    }
}

fn diag(code: &str, message: &str) -> AuthorityDiagnostic {
    AuthorityDiagnostic {
        code: code.to_string(),
        message: message.to_string(),
    }
}
