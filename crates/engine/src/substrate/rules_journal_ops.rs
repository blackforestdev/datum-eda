use uuid::Uuid;

use super::{EngineError, Operation};

pub(super) fn apply_rules_operation(
    rules_value: &mut serde_json::Value,
    operation: &Operation,
) -> Result<bool, EngineError> {
    match operation {
        Operation::SetProjectRules { rules } => {
            set_rules_field(rules_value, serde_json::Value::Array(rules.clone()))?;
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
            rules.push(rule.clone());
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
                    *existing = rule.clone();
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
        Operation::SetProjectRules { rules } => {
            let previous_rules = rules_field(rules_value)?;
            inverse_operations.push(Operation::SetProjectRules {
                rules: previous_rules,
            });
            set_rules_field(rules_value, serde_json::Value::Array(rules.clone()))?;
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
    rule.get("uuid")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| EngineError::Validation("project rule missing uuid".to_string()))
        .and_then(|value| {
            Uuid::parse_str(value)
                .map_err(|error| EngineError::Validation(format!("invalid rule uuid: {error}")))
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
