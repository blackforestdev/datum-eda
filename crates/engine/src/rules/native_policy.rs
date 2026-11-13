//! Canonical native rule selection shared by authoring and checking.
use crate::board::{Net, NetClass};
use crate::error::EngineError;
use crate::rules::ast::{Rule, RuleParams};
use crate::rules::eval::{ScopeEvalContext, eval_scope_m2};
use crate::substrate::{DesignModel, SourceShardKind};

fn invalid(reason: impl Into<String>) -> EngineError {
    EngineError::Validation(format!("Native rule policy: {}", reason.into()))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TrackWidthPolicy {
    pub minimum: Option<i64>,
    pub preferred: Option<i64>,
    pub maximum: Option<i64>,
}

pub(crate) fn track_width_policy(
    model: &DesignModel,
    board: &serde_json::Value,
    net: &Net,
    layer: i32,
) -> Result<TrackWidthPolicy, EngineError> {
    track_width_from_rules(&PolicyRules::from_model(model)?, board, net, layer)
}

pub(crate) fn track_width_from_rules(
    rules: &PolicyRules,
    board: &serde_json::Value,
    net: &Net,
    layer: i32,
) -> Result<TrackWidthPolicy, EngineError> {
    let class = if net.class.is_nil() {
        None
    } else {
        let value = board["net_classes"]
            .get(net.class.to_string())
            .ok_or_else(|| invalid("Net class source is unavailable"))?;
        let class: NetClass = serde_json::from_value(value.clone())?;
        if class.uuid != net.class {
            return Err(invalid("Net class identity differs from its source"));
        }
        Some(class)
    };
    if let Some(rule) = rules.select("track_width", net, layer)? {
        return width_parameters(rule);
    }
    if let Some(class) = class {
        if class.track_width <= 0 {
            return Err(invalid("Net class width source is invalid"));
        }
        // Retain the existing native DRC interpretation of class track_width.
        return Ok(TrackWidthPolicy {
            minimum: Some(class.track_width),
            preferred: Some(class.track_width),
            maximum: None,
        });
    }
    Ok(TrackWidthPolicy {
        minimum: None,
        preferred: None,
        maximum: None,
    })
}

pub(crate) fn width_parameters(rule: &serde_json::Value) -> Result<TrackWidthPolicy, EngineError> {
    let parameter = |name: &str| {
        rule["params"][name]
            .as_i64()
            .ok_or_else(|| invalid("Track width parameter is not an integer"))
    };
    Ok(TrackWidthPolicy {
        minimum: Some(parameter("min_nm")?),
        preferred: Some(parameter("preferred_nm")?),
        maximum: Some(parameter("max_nm")?),
    })
}

/// A transient evaluation view, never a replacement for persisted source.
/// Both admitted encodings retain their own structural validator. Only the
/// existing width/clearance consumers need a shared parameter vocabulary.
pub(crate) fn rule_for_policy(rule: &serde_json::Value) -> Result<serde_json::Value, EngineError> {
    if rule.get("params").is_some() {
        crate::substrate::validate_native_project_rule_payload(rule)?;
        Ok(rule.clone())
    } else {
        legacy_rule_for_policy(&serde_json::from_value(rule.clone())?)
    }
}

fn legacy_rule_for_policy(rule: &Rule) -> Result<serde_json::Value, EngineError> {
    crate::rules::validate::validate_rule(rule)?;
    let (kind, params) = match &rule.parameters {
        RuleParams::TrackWidth {
            min,
            preferred,
            max,
        } => (
            "track_width",
            serde_json::json!({"min_nm":min,"preferred_nm":preferred,"max_nm":max}),
        ),
        RuleParams::Clearance { min } => ("clearance_copper", serde_json::json!({"min_nm":min})),
        // Other legacy kinds remain validated, nonmatching records; no new
        // evaluator or kind admission is introduced by this compatibility view.
        _ => return Ok(serde_json::to_value(rule)?),
    };
    Ok(serde_json::json!({
        "uuid":rule.uuid,"name":rule.name,"scope":rule.scope,
        "priority":u64::from(rule.priority),"enabled":rule.enabled,
        "rule_type":kind,"params":params,
    }))
}

/// Validated transient views captured once for a check. Source bytes and
/// model authority stay unchanged; there is no persistent policy cache.
pub(crate) struct PolicyRules {
    rules: Vec<serde_json::Value>,
}

impl PolicyRules {
    pub(crate) fn from_model(model: &DesignModel) -> Result<Self, EngineError> {
        let shard = model
            .source_shards
            .iter()
            .find(|s| s.kind == SourceShardKind::RulesRoot)
            .ok_or_else(|| invalid("Model has no Rules source"))?;
        let source = model.capture_source_shard_value(shard.shard_id)?;
        let rules = source["rules"]
            .as_array()
            .ok_or_else(|| invalid("Rules source has no rule array"))?;
        Ok(Self {
            rules: rules
                .iter()
                .map(rule_for_policy)
                .collect::<Result<_, _>>()?,
        })
    }

    pub(crate) fn from_typed(rules: &[Rule]) -> Result<Self, EngineError> {
        Ok(Self {
            rules: rules
                .iter()
                .map(legacy_rule_for_policy)
                .collect::<Result<_, _>>()?,
        })
    }

    pub(crate) fn select(
        &self,
        kind: &str,
        net: &Net,
        layer: i32,
    ) -> Result<Option<&serde_json::Value>, EngineError> {
        let context = ScopeEvalContext {
            net: Some(net.uuid),
            net_class: Some(net.class),
            layer: Some(layer),
        };
        let mut applicable = Vec::new();
        for rule in &self.rules {
            if rule["enabled"] == true
                && rule["rule_type"] == kind
                && eval_scope_m2(&serde_json::from_value(rule["scope"].clone())?, context)?
            {
                applicable.push(rule);
            }
        }
        applicable.sort_by_key(|r| std::cmp::Reverse(r["priority"].as_u64()));
        if let Some(rule) = applicable.first() {
            if applicable
                .get(1)
                .is_some_and(|other| other["priority"] == rule["priority"])
            {
                return Err(invalid(format!(
                    "{kind} rules have ambiguous equal priority"
                )));
            }
            return Ok(Some(*rule));
        }
        Ok(None)
    }
}

/// Consumer admission uses the same decoding and validation as selection.
pub(crate) fn has_rule_kind(model: &DesignModel, kind: &str) -> Result<bool, EngineError> {
    Ok(PolicyRules::from_model(model)?
        .rules
        .iter()
        .any(|rule| rule["rule_type"] == kind))
}

/// Routing captures one policy view for its source-qualified request. Checks
/// that visit multiple objects reuse PolicyRules directly for the whole run.
pub(crate) fn selected_rule(
    model: &DesignModel,
    kind: &str,
    net: &Net,
    layer: i32,
) -> Result<Option<serde_json::Value>, EngineError> {
    let rules = PolicyRules::from_model(model)?;
    Ok(rules.select(kind, net, layer)?.cloned())
}
