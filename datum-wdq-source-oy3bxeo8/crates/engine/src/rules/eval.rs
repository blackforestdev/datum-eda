use uuid::Uuid;

use crate::error::EngineError;

use super::ast::RuleScope;

/// Minimal evaluation context for milestone-gated scope evaluation.
///
/// This is intentionally small and foundation-only. Later milestones will
/// replace or extend it with richer object context once board/pool/query models
/// exist in code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScopeEvalContext {
    pub net: Option<Uuid>,
    pub net_class: Option<Uuid>,
    pub layer: Option<i32>,
}

/// Evaluate rule scopes for the M2-supported leaf nodes only.
///
/// Expression nodes that are structurally valid but not yet supported return
/// `EngineError::UnsupportedScope` per the support matrix in the spec.
pub fn eval_scope_m2(scope: &RuleScope, ctx: ScopeEvalContext) -> Result<bool, EngineError> {
    match scope {
        RuleScope::All => Ok(true),
        RuleScope::Net(uuid) => Ok(ctx.net == Some(*uuid)),
        RuleScope::NetClass(uuid) => Ok(ctx.net_class == Some(*uuid)),
        RuleScope::Layer(id) => Ok(ctx.layer == Some(*id)),
        unsupported => Err(unsupported_scope_error(unsupported)),
    }
}

fn unsupported_scope_error(scope: &RuleScope) -> EngineError {
    EngineError::UnsupportedScope {
        node: scope_node_name(scope).to_string(),
        available_from: "M6",
    }
}

fn scope_node_name(scope: &RuleScope) -> &'static str {
    match scope {
        RuleScope::All => "All",
        RuleScope::Net(_) => "Net",
        RuleScope::NetClass(_) => "NetClass",
        RuleScope::Layer(_) => "Layer",
        RuleScope::And(_, _) => "And",
        RuleScope::Or(_, _) => "Or",
        RuleScope::Not(_) => "Not",
        RuleScope::InComponent(_) => "InComponent",
        RuleScope::HasPackage(_) => "HasPackage",
        RuleScope::NetNameRegex(_) => "NetNameRegex",
        RuleScope::IsDiffpair => "IsDiffpair",
        RuleScope::IsVia => "IsVia",
        RuleScope::IsPad => "IsPad",
        RuleScope::IsSMD => "IsSMD",
        RuleScope::InArea(_) => "InArea",
    }
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use super::*;

    #[test]
    fn evaluates_leaf_nodes() {
        let net = Uuid::new_v4();
        let class = Uuid::new_v4();
        let ctx = ScopeEvalContext {
            net: Some(net),
            net_class: Some(class),
            layer: Some(2),
        };

        assert!(eval_scope_m2(&RuleScope::All, ctx).unwrap());
        assert!(eval_scope_m2(&RuleScope::Net(net), ctx).unwrap());
        assert!(eval_scope_m2(&RuleScope::NetClass(class), ctx).unwrap());
        assert!(eval_scope_m2(&RuleScope::Layer(2), ctx).unwrap());
        assert!(!eval_scope_m2(&RuleScope::Layer(1), ctx).unwrap());
    }

    #[test]
    fn rejects_unsupported_nodes() {
        let ctx = ScopeEvalContext {
            net: None,
            net_class: None,
            layer: None,
        };
        let err = eval_scope_m2(
            &RuleScope::And(Box::new(RuleScope::All), Box::new(RuleScope::Layer(1))),
            ctx,
        )
        .unwrap_err();

        match err {
            EngineError::UnsupportedScope {
                node,
                available_from,
            } => {
                assert_eq!(node, "And");
                assert_eq!(available_from, "M6");
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }
}
