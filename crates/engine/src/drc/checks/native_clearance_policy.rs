//! Per-check native policy projection over the already captured DesignModel.
//! This has no persistence or write path and never narrows authored priorities.
use super::*;
use crate::board::nominal_geometry::GeometryError;
use crate::error::EngineError;
use crate::rules::native_policy::PolicyRules;
use crate::substrate::DesignModel;
use std::collections::{BTreeMap, BTreeSet};

pub(in crate::drc) fn has_native_clearance_policy(
    model: &DesignModel,
) -> Result<bool, EngineError> {
    crate::rules::native_policy::has_rule_kind(model, "clearance_copper")
}

pub(super) enum ComparisonError {
    Geometry(GeometryError),
    Policy(String),
}

impl std::fmt::Debug for ComparisonError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Geometry(error) => std::fmt::Debug::fmt(error, f),
            Self::Policy(reason) => write!(f, "native clearance policy: {reason}"),
        }
    }
}

impl From<GeometryError> for ComparisonError {
    fn from(error: GeometryError) -> Self {
        Self::Geometry(error)
    }
}

type NetMinimum = Result<Option<i64>, String>;

pub(super) struct ClearancePolicies<'a> {
    board: &'a Board,
    native: Option<BTreeMap<(Uuid, LayerId), NetMinimum>>,
}

impl<'a> ClearancePolicies<'a> {
    pub(super) fn capture(board: &'a Board, model: Option<&DesignModel>) -> Self {
        let native = model.map(|model| {
            // Decode once per check; select once per actual Net/layer rather
            // than decoding the entire Rule source for every geometric pair.
            let rules = PolicyRules::from_model(model);
            let layers = board
                .tracks
                .values()
                .map(|track| track.layer)
                .collect::<BTreeSet<_>>();
            let mut policies = BTreeMap::new();
            for net in board.nets.values() {
                for layer in &layers {
                    let minimum =
                        rules
                            .as_ref()
                            .map_err(|error| error.to_string())
                            .and_then(|rules| {
                                (|| -> Result<Option<i64>, EngineError> {
                                    if let Some(rule) =
                                        rules.select("clearance_copper", net, *layer)?
                                    {
                                        let minimum =
                                            rule["params"]["min_nm"].as_i64().ok_or_else(|| {
                                                EngineError::Validation(
                                                    "Native clearance minimum is not an integer"
                                                        .into(),
                                                )
                                            })?;
                                        return Ok(Some(minimum));
                                    }
                                    // Preserve the existing checker interpretation of a
                                    // class when no applicable native Rule overrides it.
                                    Ok(board
                                        .net_classes
                                        .get(&net.class)
                                        .map(|class| class.clearance))
                                })()
                                .map_err(|error| error.to_string())
                            });
                    policies.insert((net.uuid, *layer), minimum);
                }
            }
            policies
        });
        Self { board, native }
    }

    pub(super) fn required(
        &self,
        a: Uuid,
        b: Uuid,
        layer: LayerId,
    ) -> Result<i64, ComparisonError> {
        let Some(native) = &self.native else {
            return Ok(super::required_clearance_nm(self.board, a, b));
        };
        let get = |net| {
            native
                .get(&(net, layer))
                .ok_or_else(|| {
                    ComparisonError::Policy("Net/layer has no captured source policy".into())
                })?
                .as_ref()
                .copied()
                .map_err(|reason| ComparisonError::Policy(reason.clone()))
        };
        Ok(match (get(a)?, get(b)?) {
            (Some(a), Some(b)) => a.max(b),
            (Some(value), None) | (None, Some(value)) => value,
            // The legacy checker already uses this only when both classes are
            // absent. An applicable-policy error never reaches this fallback.
            (None, None) => 100_000,
        })
    }
}

pub(in crate::drc) fn native_clearance_unavailable(board: Uuid, reason: String) -> DrcViolation {
    let objects = vec![board];
    let code = "nominal_geometry_unavailable";
    DrcViolation {
        id: stable_violation_id(code, RuleType::ClearanceCopper, None, &objects),
        code: code.into(),
        rule_type: RuleType::ClearanceCopper,
        severity: DrcSeverity::Error,
        message: format!("Native clearance cannot certify its source: {reason}"),
        location: None,
        objects,
        fingerprint: None,
        standards_basis: None,
        rule_revision: None,
        import_key: None,
        waived: false,
    }
}
