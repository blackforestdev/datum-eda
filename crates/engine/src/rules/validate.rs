use crate::error::EngineError;

use super::ast::{Rule, RuleParams, RuleScope, RuleType};

/// Structural rule validation for the M0 foundation layer.
///
/// This validates rule shape and internal consistency only. Validation that
/// depends on board objects, pool objects, or imported references belongs to
/// later milestones once those models exist in code.
pub fn validate_rule(rule: &Rule) -> Result<(), EngineError> {
    if rule.name.trim().is_empty() {
        return Err(EngineError::Validation(
            "rule name must not be empty".into(),
        ));
    }

    validate_scope(&rule.scope)?;
    validate_rule_params(rule)?;
    Ok(())
}

pub fn validate_scope(scope: &RuleScope) -> Result<(), EngineError> {
    match scope {
        RuleScope::All => Ok(()),
        RuleScope::Net(uuid)
        | RuleScope::NetClass(uuid)
        | RuleScope::InComponent(uuid)
        | RuleScope::InArea(uuid) => {
            if uuid.is_nil() {
                return Err(EngineError::Validation(
                    "rule scope UUID reference must not be nil".into(),
                ));
            }
            Ok(())
        }
        RuleScope::Layer(_) => Ok(()),
        RuleScope::And(a, b) | RuleScope::Or(a, b) => {
            validate_scope(a)?;
            validate_scope(b)
        }
        RuleScope::Not(inner) => validate_scope(inner),
        RuleScope::HasPackage(glob) | RuleScope::NetNameRegex(glob) => {
            if glob.trim().is_empty() {
                return Err(EngineError::Validation(
                    "string-based rule scope operand must not be empty".into(),
                ));
            }
            Ok(())
        }
        RuleScope::IsDiffpair | RuleScope::IsVia | RuleScope::IsPad | RuleScope::IsSMD => Ok(()),
    }
}

fn validate_rule_params(rule: &Rule) -> Result<(), EngineError> {
    match (&rule.rule_type, &rule.parameters) {
        (RuleType::ClearanceCopper, RuleParams::Clearance { min }) => {
            require_positive("clearance min", *min)
        }
        (
            RuleType::TrackWidth,
            RuleParams::TrackWidth {
                min,
                preferred,
                max,
            },
        ) => {
            require_positive("track width min", *min)?;
            require_positive("track width preferred", *preferred)?;
            require_positive("track width max", *max)?;
            if min > preferred || preferred > max {
                return Err(EngineError::Validation(
                    "track width parameters must satisfy min <= preferred <= max".into(),
                ));
            }
            Ok(())
        }
        (RuleType::ViaHole, RuleParams::ViaHole { min, max })
        | (RuleType::HoleSize, RuleParams::HoleSize { min, max }) => {
            require_positive("hole min", *min)?;
            require_positive("hole max", *max)?;
            if min > max {
                return Err(EngineError::Validation(
                    "hole parameters must satisfy min <= max".into(),
                ));
            }
            Ok(())
        }
        (RuleType::ViaAnnularRing, RuleParams::ViaAnnularRing { min })
        | (RuleType::SilkClearance, RuleParams::SilkClearance { min }) => {
            require_positive("minimum value", *min)
        }
        (RuleType::Connectivity, RuleParams::Connectivity) => Ok(()),
        _ => Err(EngineError::Validation(
            "rule_type and parameters variant do not match".into(),
        )),
    }
}

fn require_positive(name: &str, value: i64) -> Result<(), EngineError> {
    if value <= 0 {
        return Err(EngineError::Validation(format!("{name} must be > 0")));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use super::*;

    #[test]
    fn rejects_empty_rule_name() {
        let rule = Rule {
            uuid: Uuid::new_v4(),
            name: "  ".into(),
            scope: RuleScope::All,
            priority: 1,
            enabled: true,
            rule_type: RuleType::Connectivity,
            parameters: RuleParams::Connectivity,
        };

        assert!(validate_rule(&rule).is_err());
    }

    #[test]
    fn rejects_mismatched_rule_params() {
        let rule = Rule {
            uuid: Uuid::new_v4(),
            name: "bad".into(),
            scope: RuleScope::All,
            priority: 1,
            enabled: true,
            rule_type: RuleType::Connectivity,
            parameters: RuleParams::Clearance { min: 100 },
        };

        assert!(validate_rule(&rule).is_err());
    }

    #[test]
    fn validates_nested_scope_shape() {
        let scope = RuleScope::And(
            Box::new(RuleScope::Net(Uuid::new_v4())),
            Box::new(RuleScope::Not(Box::new(RuleScope::Layer(1)))),
        );
        assert!(validate_scope(&scope).is_ok());
    }
}
