use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::ir::geometry::LayerId;

/// Rule scope expression tree.
/// The data model is expression-based from M0. The evaluator starts
/// with leaf nodes in M2 and expands to combinators in M6+.
/// See docs/CANONICAL_IR.md §6, specs/ENGINE_SPEC.md §1.5.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RuleScope {
    // Leaf nodes — M2 evaluator
    All,
    Net(Uuid),
    NetClass(Uuid),
    Layer(LayerId),

    // Combinators — M6+ evaluator (parse in M2, error on eval)
    And(Box<RuleScope>, Box<RuleScope>),
    Or(Box<RuleScope>, Box<RuleScope>),
    Not(Box<RuleScope>),

    // Extended functions — M6+ evaluator
    InComponent(Uuid),
    HasPackage(String),
    NetNameRegex(String),
    IsDiffpair,
    IsVia,
    IsPad,
    IsSMD,
    InArea(Uuid),
}

/// Rule type — what kind of constraint this rule defines.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RuleType {
    ClearanceCopper,
    TrackWidth,
    ViaHole,
    ViaAnnularRing,
    HoleSize,
    SilkClearance,
    Connectivity,
    // M5+: Impedance, LengthMatch, DiffpairGap, DiffpairSkew
}

/// Rule parameters — type-specific values.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RuleParams {
    Clearance { min: i64 },
    TrackWidth { min: i64, preferred: i64, max: i64 },
    ViaHole { min: i64, max: i64 },
    ViaAnnularRing { min: i64 },
    HoleSize { min: i64, max: i64 },
    SilkClearance { min: i64 },
    Connectivity,
}

/// A design rule.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rule {
    pub uuid: Uuid,
    pub name: String,
    pub scope: RuleScope,
    pub priority: u32,
    pub enabled: bool,
    pub rule_type: RuleType,
    pub parameters: RuleParams,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rule_scope_serde_round_trip() {
        let scope = RuleScope::And(
            Box::new(RuleScope::NetClass(Uuid::nil())),
            Box::new(RuleScope::Layer(1)),
        );
        let json = serde_json::to_string(&scope).unwrap();
        let deserialized: RuleScope = serde_json::from_str(&json).unwrap();
        assert_eq!(scope, deserialized);
    }

    #[test]
    fn rule_serde_round_trip() {
        let rule = Rule {
            uuid: Uuid::new_v4(),
            name: "default clearance".into(),
            scope: RuleScope::All,
            priority: 1,
            enabled: true,
            rule_type: RuleType::ClearanceCopper,
            parameters: RuleParams::Clearance { min: 100_000 }, // 0.1mm
        };
        let json = serde_json::to_string_pretty(&rule).unwrap();
        let deserialized: Rule = serde_json::from_str(&json).unwrap();
        assert_eq!(rule, deserialized);
    }
}
