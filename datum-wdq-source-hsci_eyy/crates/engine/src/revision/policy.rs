use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use super::{
    ApprovalIntent, AuthorityDiagnostic, AuthorityRecord, AuthorityRef, AuthoritySnapshot,
    Capability, ProjectRevisionPolicyId, RevisionSchemeId,
};

pub const REV_I03_POLICY_SECTIONS: &[&str] = &[
    "scheme_profile_selection",
    "per_ci_namespace_rules",
    "earlier_control",
    "build_presentation",
    "namespace_transition",
    "approval_policy",
    "effectivity_obligations",
    "controlled_terminology",
    "required_method_classes",
];

pub const FACTORY_PROFILE_NAME: &str = "SequentialAlphanumericLegacy";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EarlierControlMode {
    NoEarlierControl,
    AuthorizedChangeRequired,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BuildPresentationMode {
    Quiet,
    PhaseBuild,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NamespaceTransitionMode {
    Continuous,
    GovernedProductionIdentity,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SchemeProfileSelection {
    pub profile_name: String,
    pub scheme_id: RevisionSchemeId,
    pub scheme_version: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PerCiNamespaceRules {
    pub require_one_namespace_per_configuration_item: bool,
    pub allow_cross_item_token_reuse: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ApprovalRequirement {
    pub intent: ApprovalIntent,
    pub required_capability: Capability,
    pub quorum: u32,
    pub order: u32,
    pub independent_from_intents: BTreeSet<ApprovalIntent>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ApprovalPolicy {
    pub requirements: Vec<ApprovalRequirement>,
    pub separation_of_duty: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EffectivityObligations {
    pub required_for_intents: BTreeSet<ApprovalIntent>,
    pub exact_population_snapshot_when_enumerable: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ControlledTerminology {
    pub terms: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RequiredMethodClasses {
    pub by_intent: BTreeMap<ApprovalIntent, BTreeSet<String>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectRevisionPolicyData {
    pub policy_version: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub supersedes: Option<ProjectRevisionPolicyId>,
    pub scheme_profile_selection: SchemeProfileSelection,
    pub per_ci_namespace_rules: PerCiNamespaceRules,
    pub earlier_control: EarlierControlMode,
    pub build_presentation: BuildPresentationMode,
    pub namespace_transition: NamespaceTransitionMode,
    pub approval_policy: ApprovalPolicy,
    pub effectivity_obligations: EffectivityObligations,
    pub controlled_terminology: ControlledTerminology,
    pub required_method_classes: RequiredMethodClasses,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProjectRevisionPolicyResolution {
    Unmanaged,
    Managed {
        policy_ref: AuthorityRef,
        policy: ProjectRevisionPolicyData,
    },
    Refused(AuthorityDiagnostic),
}

pub fn resolve_project_revision_policy(
    snapshot: Option<&AuthoritySnapshot>,
) -> ProjectRevisionPolicyResolution {
    let Some(snapshot) = snapshot else {
        return ProjectRevisionPolicyResolution::Unmanaged;
    };
    let policies: Vec<_> = snapshot
        .records
        .iter()
        .filter_map(|record| match record {
            AuthorityRecord::ProjectRevisionPolicy(body) => Some(body),
            _ => None,
        })
        .collect();
    if policies.is_empty() {
        return ProjectRevisionPolicyResolution::Unmanaged;
    }
    let superseded: BTreeSet<_> = policies
        .iter()
        .filter_map(|body| body.semantics.supersedes)
        .collect();
    let active: Vec<_> = policies
        .into_iter()
        .filter(|body| !superseded.contains(&body.id))
        .collect();
    if active.len() != 1 {
        return ProjectRevisionPolicyResolution::Refused(AuthorityDiagnostic {
            code: "revision_policy_ambiguous_active_version".to_string(),
            message: "Project revision policy has no unique active version".to_string(),
        });
    }
    let body = active[0];
    ProjectRevisionPolicyResolution::Managed {
        policy_ref: AuthorityRef::ProjectRevisionPolicy(body.id),
        policy: body.semantics.clone(),
    }
}

pub(crate) fn validate_policy_data(data: &ProjectRevisionPolicyData) -> Vec<AuthorityDiagnostic> {
    let mut diagnostics = Vec::new();
    if data.policy_version == 0 || data.scheme_profile_selection.scheme_version == 0 {
        diagnostics.push(AuthorityDiagnostic {
            code: "revision_policy_invalid_version".to_string(),
            message: "policy and scheme versions must be positive".to_string(),
        });
    }
    let mut orders = BTreeSet::new();
    for requirement in &data.approval_policy.requirements {
        if requirement.quorum == 0 {
            diagnostics.push(AuthorityDiagnostic {
                code: "revision_policy_zero_quorum".to_string(),
                message: "approval quorum must be positive".to_string(),
            });
        }
        if !orders.insert((requirement.intent, requirement.order)) {
            diagnostics.push(AuthorityDiagnostic {
                code: "revision_policy_ambiguous_approval_order".to_string(),
                message: "one approval intent cannot have duplicate order positions".to_string(),
            });
        }
    }
    diagnostics
}
