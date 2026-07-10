use uuid::Uuid;

use crate::rules::{ast::RuleScope, validate::validate_scope};

use super::{EngineError, Operation};

pub(super) fn apply_rules_operation(
    rules_value: &mut serde_json::Value,
    operation: &Operation,
) -> Result<bool, EngineError> {
    match operation {
        Operation::SetProjectRules {
            rules_root_id,
            rules,
        } => {
            require_rules_root(rules_value, rules_root_id)?;
            set_rules_field(
                rules_value,
                serde_json::Value::Array(rules_with_default_revisions(rules)?),
            )?;
            bump_rules_root_revision(rules_value)?;
            Ok(true)
        }
        Operation::CreateProjectRule {
            rules_root_id,
            rule_id,
            rule,
        } => {
            require_rules_root(rules_value, rules_root_id)?;
            require_rule_payload_id(rule, rule_id)?;
            let mut rules = rules_field(rules_value)?;
            if rules
                .iter()
                .any(|existing| rule_payload_id(existing).ok() == Some(*rule_id))
            {
                return Err(EngineError::Validation(format!(
                    "project rule {rule_id} already exists"
                )));
            }
            rules.push(rule_with_revision(rule, rule_revision(rule).unwrap_or(0))?);
            set_rules_field(rules_value, serde_json::Value::Array(rules))?;
            bump_rules_root_revision(rules_value)?;
            Ok(true)
        }
        Operation::SetProjectRule {
            rules_root_id,
            rule_id,
            rule,
        } => {
            require_rules_root(rules_value, rules_root_id)?;
            require_rule_payload_id(rule, rule_id)?;
            let mut rules = rules_field(rules_value)?;
            let mut replaced = false;
            for existing in &mut rules {
                if rule_payload_id(existing).ok() == Some(*rule_id) {
                    let revision = rule_revision(existing).unwrap_or(0) + 1;
                    *existing = rule_with_revision(rule, revision)?;
                    replaced = true;
                    break;
                }
            }
            if !replaced {
                return Err(EngineError::Validation(format!(
                    "project rule {rule_id} not found"
                )));
            }
            set_rules_field(rules_value, serde_json::Value::Array(rules))?;
            bump_rules_root_revision(rules_value)?;
            Ok(true)
        }
        Operation::DeleteProjectRule {
            rules_root_id,
            rule_id,
            ..
        } => {
            require_rules_root(rules_value, rules_root_id)?;
            let mut rules = rules_field(rules_value)?;
            let before_len = rules.len();
            rules.retain(|existing| rule_payload_id(existing).ok() != Some(*rule_id));
            if rules.len() == before_len {
                return Err(EngineError::Validation(format!(
                    "project rule {rule_id} not found"
                )));
            }
            set_rules_field(rules_value, serde_json::Value::Array(rules))?;
            bump_rules_root_revision(rules_value)?;
            Ok(true)
        }
        _ => Ok(false),
    }
}

pub(super) fn inverse_rules_operation(
    rules_value: &mut serde_json::Value,
    operation: &Operation,
    inverse_operations: &mut Vec<Operation>,
) -> Result<(), EngineError> {
    match operation {
        Operation::SetProjectRules {
            rules_root_id,
            rules,
        } => {
            let previous_rules = rules_field(rules_value)?;
            inverse_operations.push(Operation::SetProjectRules {
                rules_root_id: *rules_root_id,
                rules: previous_rules,
            });
            set_rules_field(
                rules_value,
                serde_json::Value::Array(rules_with_default_revisions(rules)?),
            )?;
            bump_rules_root_revision(rules_value)?;
        }
        Operation::CreateProjectRule {
            rules_root_id,
            rule_id,
            rule,
        } => {
            inverse_operations.push(Operation::DeleteProjectRule {
                rules_root_id: *rules_root_id,
                rule_id: *rule_id,
                rule: rule.clone(),
            });
            apply_rules_operation(rules_value, operation)?;
        }
        Operation::SetProjectRule {
            rules_root_id,
            rule_id,
            ..
        } => {
            let previous = find_rule(rules_value, rule_id)?;
            inverse_operations.push(Operation::SetProjectRule {
                rules_root_id: *rules_root_id,
                rule_id: *rule_id,
                rule: previous,
            });
            apply_rules_operation(rules_value, operation)?;
        }
        Operation::DeleteProjectRule {
            rules_root_id,
            rule_id,
            ..
        } => {
            let rule = find_rule(rules_value, rule_id)?;
            inverse_operations.push(Operation::CreateProjectRule {
                rules_root_id: *rules_root_id,
                rule_id: *rule_id,
                rule,
            });
            apply_rules_operation(rules_value, operation)?;
        }
        _ => {}
    }
    Ok(())
}

fn rules_field(rules_value: &serde_json::Value) -> Result<Vec<serde_json::Value>, EngineError> {
    let value = rules_value
        .get("rules")
        .ok_or_else(|| EngineError::Validation("rules root missing rules array".to_string()))?;
    value.as_array().cloned().ok_or_else(|| {
        EngineError::Validation("rules root rules field is not an array".to_string())
    })
}

fn set_rules_field(
    rules_value: &mut serde_json::Value,
    rules: serde_json::Value,
) -> Result<(), EngineError> {
    let object = rules_value
        .as_object_mut()
        .ok_or_else(|| EngineError::Validation("rules root is not an object".to_string()))?;
    object.insert("rules".to_string(), rules);
    Ok(())
}

fn require_rules_root(
    rules_value: &serde_json::Value,
    rules_root_id: &Uuid,
) -> Result<(), EngineError> {
    let actual = rules_value
        .get("uuid")
        .and_then(serde_json::Value::as_str)
        .and_then(|value| Uuid::parse_str(value).ok())
        .ok_or_else(|| {
            EngineError::Validation("rules root missing uuid for granular edit".to_string())
        })?;
    if &actual != rules_root_id {
        return Err(EngineError::Validation(format!(
            "rules root id mismatch: expected {rules_root_id}, found {actual}"
        )));
    }
    Ok(())
}

fn require_rule_payload_id(rule: &serde_json::Value, rule_id: &Uuid) -> Result<(), EngineError> {
    let actual = rule_payload_id(rule)?;
    if &actual != rule_id {
        return Err(EngineError::Validation(format!(
            "rule payload id mismatch: expected {rule_id}, found {actual}"
        )));
    }
    Ok(())
}

fn rule_payload_id(rule: &serde_json::Value) -> Result<Uuid, EngineError> {
    let uuid = rule
        .get("uuid")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| EngineError::Validation("project rule missing uuid".to_string()))
        .and_then(|value| {
            Uuid::parse_str(value)
                .map_err(|error| EngineError::Validation(format!("invalid rule uuid: {error}")))
        })?;
    if uuid.is_nil() {
        return Err(EngineError::Validation(
            "project rule uuid must not be nil".to_string(),
        ));
    }
    Ok(uuid)
}

fn rule_revision(rule: &serde_json::Value) -> Option<u64> {
    rule.get("object_revision")
        .and_then(serde_json::Value::as_u64)
}

fn rule_with_revision(
    rule: &serde_json::Value,
    revision: u64,
) -> Result<serde_json::Value, EngineError> {
    validate_native_project_rule_payload(rule)?;
    let mut value = rule.clone();
    let object = value
        .as_object_mut()
        .ok_or_else(|| EngineError::Validation("project rule is not an object".to_string()))?;
    object.insert(
        "object_revision".to_string(),
        serde_json::Value::Number(serde_json::Number::from(revision)),
    );
    Ok(value)
}

fn rules_with_default_revisions(
    rules: &[serde_json::Value],
) -> Result<Vec<serde_json::Value>, EngineError> {
    rules
        .iter()
        .map(|rule| rule_with_revision(rule, rule_revision(rule).unwrap_or(0)))
        .collect()
}

pub fn validate_native_project_rule_payload(rule: &serde_json::Value) -> Result<(), EngineError> {
    let object = rule
        .as_object()
        .ok_or_else(|| EngineError::Validation("project rule is not an object".to_string()))?;
    rule_payload_id(rule)?;
    let name = object
        .get("name")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| EngineError::Validation("project rule missing name".to_string()))?;
    if name.trim().is_empty() {
        return Err(EngineError::Validation(
            "project rule name must not be empty".to_string(),
        ));
    }
    if object
        .get("priority").is_none_or(|value| value.as_u64().is_none())
    {
        return Err(EngineError::Validation(
            "project rule priority must be an unsigned integer".to_string(),
        ));
    }
    if object
        .get("enabled").is_none_or(|value| value.as_bool().is_none())
    {
        return Err(EngineError::Validation(
            "project rule enabled must be boolean".to_string(),
        ));
    }
    let scope = object
        .get("scope")
        .ok_or_else(|| EngineError::Validation("project rule missing scope".to_string()))?;
    validate_native_project_rule_scope(scope)?;
    let rule_type = object
        .get("rule_type")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| EngineError::Validation("project rule missing rule_type".to_string()))?;
    let params = object
        .get("params")
        .and_then(serde_json::Value::as_object)
        .ok_or_else(|| EngineError::Validation("project rule missing params object".to_string()))?;
    validate_rule_params(rule_type, params)
}

fn validate_native_project_rule_scope(scope: &serde_json::Value) -> Result<(), EngineError> {
    if scope.as_str().is_some_and(|value| value.trim().is_empty()) || scope.is_null() {
        return Err(EngineError::Validation(
            "project rule scope must not be empty".to_string(),
        ));
    }
    let scope = serde_json::from_value::<RuleScope>(scope.clone())
        .map_err(|error| EngineError::Validation(format!("invalid project rule scope: {error}")))?;
    validate_scope(&scope)
        .map_err(|error| EngineError::Validation(format!("invalid project rule scope: {error}")))
}

fn validate_rule_params(
    rule_type: &str,
    params: &serde_json::Map<String, serde_json::Value>,
) -> Result<(), EngineError> {
    match rule_type {
        "clearance_copper" => require_positive_param(params, "min_nm"),
        "track_width" => {
            let min = positive_param(params, "min_nm")?;
            let preferred = positive_param(params, "preferred_nm")?;
            let max = positive_param(params, "max_nm")?;
            if min > preferred || preferred > max {
                return Err(EngineError::Validation(
                    "project rule track_width params must satisfy min_nm <= preferred_nm <= max_nm"
                        .to_string(),
                ));
            }
            Ok(())
        }
        "via_hole" | "hole_size" => {
            let min = positive_param(params, "min_nm")?;
            let max = positive_param(params, "max_nm")?;
            if min > max {
                return Err(EngineError::Validation(format!(
                    "project rule {rule_type} params must satisfy min_nm <= max_nm"
                )));
            }
            Ok(())
        }
        "via_annular_ring" | "silk_clearance" => require_positive_param(params, "min_nm"),
        "process_aperture" => {
            require_non_negative_param(params, "min_mask_expansion_nm")?;
            require_non_negative_param(params, "min_paste_reduction_nm")
        }
        "connectivity" => Ok(()),
        _ => Err(EngineError::Validation(format!(
            "unsupported project rule_type {rule_type}"
        ))),
    }
}

fn require_positive_param(
    params: &serde_json::Map<String, serde_json::Value>,
    field: &str,
) -> Result<(), EngineError> {
    positive_param(params, field).map(|_| ())
}

fn positive_param(
    params: &serde_json::Map<String, serde_json::Value>,
    field: &str,
) -> Result<i64, EngineError> {
    let value = numeric_param(params, field)?;
    if value <= 0 {
        return Err(EngineError::Validation(format!(
            "project rule param {field} must be > 0"
        )));
    }
    Ok(value)
}

fn require_non_negative_param(
    params: &serde_json::Map<String, serde_json::Value>,
    field: &str,
) -> Result<(), EngineError> {
    let value = numeric_param(params, field)?;
    if value < 0 {
        return Err(EngineError::Validation(format!(
            "project rule param {field} must be >= 0"
        )));
    }
    Ok(())
}

fn numeric_param(
    params: &serde_json::Map<String, serde_json::Value>,
    field: &str,
) -> Result<i64, EngineError> {
    params
        .get(field)
        .and_then(serde_json::Value::as_i64)
        .ok_or_else(|| {
            EngineError::Validation(format!("project rule missing numeric param {field}"))
        })
}

fn find_rule(
    rules_value: &serde_json::Value,
    rule_id: &Uuid,
) -> Result<serde_json::Value, EngineError> {
    rules_field(rules_value)?
        .into_iter()
        .find(|rule| rule_payload_id(rule).ok() == Some(*rule_id))
        .ok_or_else(|| EngineError::Validation(format!("project rule {rule_id} not found")))
}

fn bump_rules_root_revision(rules_value: &mut serde_json::Value) -> Result<(), EngineError> {
    let object = rules_value
        .as_object_mut()
        .ok_or_else(|| EngineError::Validation("rules root is not an object".to_string()))?;
    let revision = object
        .get("object_revision")
        .and_then(serde_json::Value::as_u64)
        .ok_or_else(|| {
            EngineError::Validation(
                "rules root missing object_revision for granular edit".to_string(),
            )
        })?;
    object.insert(
        "object_revision".to_string(),
        serde_json::Value::Number(serde_json::Number::from(revision + 1)),
    );
    Ok(())
}
