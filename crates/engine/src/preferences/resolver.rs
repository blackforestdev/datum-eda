use std::collections::BTreeSet;

use serde_json::Value;

use super::{DescriptorRegistry, DirectiveKind, PreferenceKey, ResolutionSource, SettingClass};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum AuthorityReleaseLevel {
    None,
    RecommendationsOnly,
    Bounded,
    BacksideManagement,
    FullManagement,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum AuthorityReleaseState {
    Active,
    Expired,
    Revoked,
    Superseded,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthorityRelease {
    pub id: String,
    pub releasing_user: String,
    pub organization: String,
    pub machine_scope: String,
    pub level: AuthorityReleaseLevel,
    pub effective_at: String,
    pub effective_until: Option<String>,
    pub attribution: String,
    pub revocable: bool,
    pub state: AuthorityReleaseState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderGenerationState {
    Active,
    StaleEffective,
    Unavailable,
    Expired,
    Revoked,
    Superseded,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueDisclosure {
    Disclosed,
    Redacted,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FactProvenance {
    pub origin: String,
    pub provider: Option<String>,
    pub package: Option<String>,
    pub generation: Option<String>,
    pub actor: String,
    pub role: Option<String>,
    pub observed_at: String,
    pub effective_from: Option<String>,
    pub effective_until: Option<String>,
    pub offline_valid_until: Option<String>,
    pub last_successful_contact: Option<String>,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextApplicability {
    pub subject: String,
    pub authority: String,
    pub lifetime: String,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValueConstraint {
    IntegerRange { min: i64, max: i64 },
    AllowedValues(Vec<Value>),
    DeniedValues(Vec<Value>),
}

impl ValueConstraint {
    fn allows(&self, value: &Value) -> bool {
        match self {
            Self::IntegerRange { min, max } => value
                .as_i64()
                .is_some_and(|candidate| candidate >= *min && candidate <= *max),
            Self::AllowedValues(values) => values.contains(value),
            Self::DeniedValues(values) => !values.contains(value),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OrganizationDirective {
    Recommend(Value),
    Constrain(ValueConstraint),
    Pin(Value),
    Lock(BTreeSet<ResolutionSource>),
}

impl OrganizationDirective {
    fn kind(&self) -> DirectiveKind {
        match self {
            Self::Recommend(_) => DirectiveKind::Recommend,
            Self::Constrain(_) => DirectiveKind::Constrain,
            Self::Pin(_) => DirectiveKind::Pin,
            Self::Lock(_) => DirectiveKind::Lock,
        }
    }

    fn value(&self) -> Option<&Value> {
        match self {
            Self::Recommend(value) | Self::Pin(value) => Some(value),
            Self::Constrain(_) | Self::Lock(_) => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValueFact {
    pub id: String,
    pub key: PreferenceKey,
    pub source: ResolutionSource,
    pub value: Value,
    pub provenance: FactProvenance,
    pub disclosure: ValueDisclosure,
    pub provider_state: Option<ProviderGenerationState>,
    /// Context is applicability metadata, never an ordinary ranked value.
    pub context: Option<ContextApplicability>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrganizationFact {
    pub id: String,
    pub key: PreferenceKey,
    pub organization: String,
    pub provenance: FactProvenance,
    pub authenticated: bool,
    pub disclosure: ValueDisclosure,
    pub generation_state: ProviderGenerationState,
    pub remaining_freedom: String,
    pub appeal_path: Option<String>,
    pub directive: OrganizationDirective,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeDefaultFact {
    pub id: String,
    pub key: PreferenceKey,
    pub recipe: String,
    pub value: Value,
    pub provenance: FactProvenance,
    pub disclosure: ValueDisclosure,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Contribution {
    Value(ValueFact),
    Organization(OrganizationFact),
    RuntimeDefault(RuntimeDefaultFact),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContributionDisposition {
    Effective,
    Losing,
    Retained,
    Inert,
    Refused,
    Conflicting,
    Applicable,
    Unavailable,
    Stale,
    Expired,
    Revoked,
    Superseded,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConsideredFact {
    pub id: String,
    pub source: ResolutionSource,
    pub value: Option<Value>,
    pub directive: Option<DirectiveKind>,
    pub control: Option<OrganizationDirective>,
    pub disposition: ContributionDisposition,
    pub reason: String,
    pub provenance: FactProvenance,
    pub disclosure: ValueDisclosure,
    pub provider_state: Option<ProviderGenerationState>,
    pub authority_release: Option<AuthorityRelease>,
    pub remaining_freedom: Option<String>,
    pub appeal_path: Option<String>,
    pub reactivation_condition: Option<String>,
    pub context: Option<ContextApplicability>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum AvailableAction {
    PreserveOpaqueRecord,
    ProvideRuntimeDefault,
    ResetUserContribution,
    EndSessionOverride,
    ReviewAuthorityRequest,
    ManageAuthorityRelease,
    AppealOrRevokeManagedPin,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResolutionOutcome {
    Effective {
        value: Value,
        winning_fact_ids: Vec<String>,
    },
    NoValue,
    UnresolvedConflict {
        fact_ids: Vec<String>,
        reason: String,
    },
    UnknownKey,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreferenceExplanation {
    pub key: PreferenceKey,
    pub machine_scope: String,
    pub value_schema_name: Option<String>,
    pub setting_class: Option<SettingClass>,
    pub eligible_sources: BTreeSet<ResolutionSource>,
    pub outcome: ResolutionOutcome,
    pub considered: Vec<ConsideredFact>,
    pub absent_sources: BTreeSet<ResolutionSource>,
    pub evaluation_stages: Vec<String>,
    pub available_actions: BTreeSet<AvailableAction>,
}

pub struct ResolutionRequest<'a> {
    pub registry: &'a DescriptorRegistry,
    pub key: PreferenceKey,
    pub contributions: &'a [Contribution],
    pub authority_releases: &'a [AuthorityRelease],
    pub machine_scope: &'a str,
}

#[derive(Debug, Clone)]
struct Candidate {
    id: String,
    source: ResolutionSource,
    value: Value,
    considered_index: usize,
    rank: u8,
}

#[derive(Debug, Clone)]
struct ActiveControl {
    id: String,
    directive: OrganizationDirective,
    considered_index: usize,
}

pub fn resolve_preference(request: ResolutionRequest<'_>) -> PreferenceExplanation {
    let Some(descriptor) = request.registry.get(&request.key) else {
        return PreferenceExplanation {
            key: request.key,
            machine_scope: request.machine_scope.to_owned(),
            value_schema_name: None,
            setting_class: None,
            eligible_sources: BTreeSet::new(),
            outcome: ResolutionOutcome::UnknownKey,
            considered: Vec::new(),
            absent_sources: BTreeSet::new(),
            evaluation_stages: vec!["descriptor identity: unknown; no type guessed".to_owned()],
            available_actions: BTreeSet::from([AvailableAction::PreserveOpaqueRecord]),
        };
    };

    let mut considered = Vec::new();
    let mut candidates = Vec::new();
    let mut controls = Vec::new();
    let mut present_sources = BTreeSet::new();
    let mut actions = BTreeSet::new();

    match &descriptor.default_value {
        super::DescriptorDefault::Literal(default) => {
            present_sources.insert(ResolutionSource::DescriptorDefault);
            let index = considered.len();
            considered.push(ConsideredFact {
                id: "descriptor-default".to_owned(),
                source: ResolutionSource::DescriptorDefault,
                value: Some(default.clone()),
                directive: None,
                control: None,
                disposition: ContributionDisposition::Losing,
                reason: "immutable literal descriptor fact admitted before ranking".to_owned(),
                provenance: descriptor_provenance(descriptor, "literal descriptor default"),
                disclosure: ValueDisclosure::Disclosed,
                provider_state: None,
                authority_release: None,
                remaining_freedom: None,
                appeal_path: None,
                reactivation_condition: None,
                context: None,
            });
            candidates.push(Candidate {
                id: "descriptor-default".to_owned(),
                source: ResolutionSource::DescriptorDefault,
                value: default.clone(),
                considered_index: index,
                rank: 0,
            });
        }
        super::DescriptorDefault::Runtime { recipe }
            if !request.contributions.iter().any(|contribution| {
                matches!(contribution, Contribution::RuntimeDefault(fact) if fact.key == request.key && fact.recipe == *recipe)
            }) =>
        {
            considered.push(ConsideredFact {
                id: "descriptor-default".to_owned(),
                source: ResolutionSource::DescriptorDefault,
                value: None,
                directive: None,
                control: None,
                disposition: ContributionDisposition::Unavailable,
                reason: format!(
                    "runtime descriptor-default recipe {recipe} has no evaluated value"
                ),
                provenance: descriptor_provenance(descriptor, "runtime descriptor default"),
                disclosure: ValueDisclosure::Disclosed,
                provider_state: None,
                authority_release: None,
                remaining_freedom: None,
                appeal_path: None,
                reactivation_condition: Some(
                    "evaluate the registered runtime-default recipe".to_owned(),
                ),
                context: None,
            });
            actions.insert(AvailableAction::ProvideRuntimeDefault);
        }
        super::DescriptorDefault::Runtime { .. } | super::DescriptorDefault::Absent => {}
    }

    for contribution in request.contributions {
        match contribution {
            Contribution::Value(fact) => consider_value(
                descriptor,
                fact,
                &request.key,
                &mut present_sources,
                &mut considered,
                &mut candidates,
                &mut actions,
            ),
            Contribution::Organization(fact) => consider_organization(
                descriptor,
                fact,
                &request.key,
                request.authority_releases,
                request.machine_scope,
                &mut present_sources,
                &mut considered,
                &mut candidates,
                &mut controls,
                &mut actions,
            ),
            Contribution::RuntimeDefault(fact) => consider_runtime_default(
                descriptor,
                fact,
                &request.key,
                &mut present_sources,
                &mut considered,
                &mut candidates,
            ),
        }
    }

    let conflict = control_conflict(&controls, descriptor);
    let outcome = if let Some((ids, reason)) = conflict {
        mark_conflicting(&mut considered, &ids);
        ResolutionOutcome::UnresolvedConflict {
            fact_ids: ids,
            reason,
        }
    } else {
        resolve_with_controls(&mut considered, &mut candidates, &controls, &mut actions)
    };
    let absent_sources = descriptor
        .allowed_sources
        .difference(&present_sources)
        .copied()
        .collect();
    considered.sort_by(|left, right| left.id.cmp(&right.id));

    PreferenceExplanation {
        key: request.key,
        machine_scope: request.machine_scope.to_owned(),
        value_schema_name: Some(descriptor.value_schema_name.clone()),
        setting_class: Some(descriptor.class),
        eligible_sources: descriptor.allowed_sources.clone(),
        outcome,
        considered,
        absent_sources,
        evaluation_stages: vec![
            "1 descriptor identity, source eligibility, and value validation".to_owned(),
            "2 active AuthorityRelease, constraints, locks, pins, and conflicts".to_owned(),
            "3 report Context applicability, then rank Session > User > Recommendation > Installation > default"
                .to_owned(),
        ],
        available_actions: actions,
    }
}

#[allow(clippy::too_many_arguments)]
fn consider_value(
    descriptor: &super::PreferenceDescriptor,
    fact: &ValueFact,
    key: &PreferenceKey,
    present_sources: &mut BTreeSet<ResolutionSource>,
    considered: &mut Vec<ConsideredFact>,
    candidates: &mut Vec<Candidate>,
    actions: &mut BTreeSet<AvailableAction>,
) {
    present_sources.insert(fact.source);
    let index = considered.len();
    let mut disposition = ContributionDisposition::Losing;
    let reason;
    let eligible = fact.key == *key
        && fact.source != ResolutionSource::DescriptorDefault
        && descriptor.allowed_sources.contains(&fact.source);
    if !eligible {
        disposition = ContributionDisposition::Refused;
        reason = "key/source/class is ineligible before precedence".to_owned();
    } else if !descriptor.validates(&fact.value) {
        disposition = ContributionDisposition::Refused;
        reason = "value violates the authoritative descriptor schema".to_owned();
    } else if fact.source == ResolutionSource::Organization {
        disposition = ContributionDisposition::Refused;
        reason = "organization values require a typed Recommend or Pin directive".to_owned();
    } else if fact.source == ResolutionSource::Context {
        if fact.context.is_some() {
            disposition = ContributionDisposition::Applicable;
            reason =
                "descriptor-approved Context reports read-only applicability and receives no rank"
                    .to_owned();
        } else {
            disposition = ContributionDisposition::Refused;
            reason =
                "Context requires a named subject, authority, lifetime, and applicability reason"
                    .to_owned();
        }
    } else if fact.source == ResolutionSource::ProjectPolicy {
        disposition = ContributionDisposition::Refused;
        reason = "ProjectPolicy resolves on the separate Project-authority path".to_owned();
    } else {
        reason = "eligible validated value admitted to ordinary ranking".to_owned();
        let rank = source_rank(fact.source)
            .expect("eligible ordinary preference sources have descriptor-declared rank");
        candidates.push(Candidate {
            id: fact.id.clone(),
            source: fact.source,
            value: fact.value.clone(),
            considered_index: index,
            rank,
        });
        if fact.source == ResolutionSource::User {
            actions.insert(AvailableAction::ResetUserContribution);
        }
        if fact.source == ResolutionSource::Session {
            actions.insert(AvailableAction::EndSessionOverride);
        }
    }
    considered.push(ConsideredFact {
        id: fact.id.clone(),
        source: fact.source,
        value: (fact.disclosure == ValueDisclosure::Disclosed).then(|| fact.value.clone()),
        directive: None,
        control: None,
        disposition,
        reason,
        provenance: fact.provenance.clone(),
        disclosure: fact.disclosure,
        provider_state: fact.provider_state,
        authority_release: None,
        remaining_freedom: None,
        appeal_path: None,
        reactivation_condition: None,
        context: fact.context.clone(),
    });
}

#[allow(clippy::too_many_arguments)]
fn consider_organization(
    descriptor: &super::PreferenceDescriptor,
    fact: &OrganizationFact,
    key: &PreferenceKey,
    releases: &[AuthorityRelease],
    machine_scope: &str,
    present_sources: &mut BTreeSet<ResolutionSource>,
    considered: &mut Vec<ConsideredFact>,
    candidates: &mut Vec<Candidate>,
    controls: &mut Vec<ActiveControl>,
    actions: &mut BTreeSet<AvailableAction>,
) {
    present_sources.insert(ResolutionSource::Organization);
    let index = considered.len();
    let kind = fact.directive.kind();
    let active_release = releases
        .iter()
        .filter(|release| {
            release.state == AuthorityReleaseState::Active
                && release.organization == fact.organization
                && release.machine_scope == machine_scope
                && release.revocable
        })
        .max_by_key(|release| release.level);
    let required = descriptor
        .directive_policy(kind)
        .map(|policy| policy.minimum_release);
    let mut disposition = ContributionDisposition::Losing;
    let reason;
    if fact.key != *key
        || !descriptor
            .allowed_sources
            .contains(&ResolutionSource::Organization)
    {
        disposition = ContributionDisposition::Refused;
        reason = "organization source is ineligible for this descriptor".to_owned();
    } else if required.is_none() {
        disposition = ContributionDisposition::Refused;
        reason = "descriptor does not admit this organization directive".to_owned();
    } else if fact
        .directive
        .value()
        .is_some_and(|value| !descriptor.validates(value))
    {
        disposition = ContributionDisposition::Refused;
        reason = "directive value violates the authoritative descriptor schema".to_owned();
    } else if !fact.authenticated {
        disposition = ContributionDisposition::Refused;
        reason = "managed generation lacks authenticated provider identity".to_owned();
    } else if matches!(
        &fact.directive,
        OrganizationDirective::Constrain(constraint)
            if !constraint_matches_descriptor(constraint, descriptor)
    ) {
        disposition = ContributionDisposition::Refused;
        reason = "constraint domain is incompatible with the descriptor schema".to_owned();
    } else if matches!(
        &fact.directive,
        OrganizationDirective::Lock(scopes)
            if scopes.is_empty()
                || scopes.iter().any(|source| {
                    !descriptor.allowed_sources.contains(source)
                        || !matches!(
                            source,
                            ResolutionSource::Installation
                                | ResolutionSource::User
                                | ResolutionSource::Session
                        )
                })
    ) {
        disposition = ContributionDisposition::Refused;
        reason = "Lock must name at least one descriptor-eligible writable scope".to_owned();
    } else if matches!(
        fact.generation_state,
        ProviderGenerationState::Unavailable
            | ProviderGenerationState::Expired
            | ProviderGenerationState::Revoked
            | ProviderGenerationState::Superseded
    ) {
        disposition = match fact.generation_state {
            ProviderGenerationState::Unavailable => ContributionDisposition::Unavailable,
            ProviderGenerationState::Expired => ContributionDisposition::Expired,
            ProviderGenerationState::Revoked => ContributionDisposition::Revoked,
            ProviderGenerationState::Superseded => ContributionDisposition::Superseded,
            ProviderGenerationState::Active | ProviderGenerationState::StaleEffective => {
                unreachable!()
            }
        };
        reason = "managed generation is inactive under its authenticated provider state".to_owned();
    } else if active_release.is_none_or(|release| release.level < required.unwrap()) {
        disposition = ContributionDisposition::Inert;
        reason = format!(
            "pending request exceeds active AuthorityRelease {:?}",
            required.unwrap()
        );
        actions.insert(AvailableAction::ReviewAuthorityRequest);
    } else {
        reason = "directive admitted by descriptor and active AuthorityRelease".to_owned();
        match &fact.directive {
            OrganizationDirective::Recommend(value) => candidates.push(Candidate {
                id: fact.id.clone(),
                source: ResolutionSource::Organization,
                value: value.clone(),
                considered_index: index,
                rank: 2,
            }),
            OrganizationDirective::Constrain(_)
            | OrganizationDirective::Pin(_)
            | OrganizationDirective::Lock(_) => controls.push(ActiveControl {
                id: fact.id.clone(),
                directive: fact.directive.clone(),
                considered_index: index,
            }),
        }
        actions.insert(AvailableAction::ManageAuthorityRelease);
    }
    let release = active_release.cloned();
    considered.push(ConsideredFact {
        id: fact.id.clone(),
        source: ResolutionSource::Organization,
        value: (fact.disclosure == ValueDisclosure::Disclosed)
            .then(|| fact.directive.value().cloned())
            .flatten(),
        directive: Some(kind),
        control: Some(fact.directive.clone()),
        disposition,
        reason,
        provenance: fact.provenance.clone(),
        disclosure: fact.disclosure,
        provider_state: Some(fact.generation_state),
        authority_release: release,
        remaining_freedom: Some(fact.remaining_freedom.clone()),
        appeal_path: fact.appeal_path.clone(),
        reactivation_condition: match fact.generation_state {
            ProviderGenerationState::Unavailable => {
                Some("obtain and validate an authenticated generation".to_owned())
            }
            ProviderGenerationState::Expired => {
                Some("obtain a generation with a valid effective interval".to_owned())
            }
            ProviderGenerationState::Revoked => None,
            ProviderGenerationState::Superseded => {
                Some("evaluate the authenticated successor generation".to_owned())
            }
            ProviderGenerationState::Active | ProviderGenerationState::StaleEffective => None,
        },
        context: None,
    });
}

#[path = "resolver_controls.rs"]
mod resolver_controls;

use resolver_controls::{
    consider_runtime_default, constraint_matches_descriptor, control_conflict,
    descriptor_provenance, mark_conflicting, resolve_with_controls, source_rank,
};

#[cfg(test)]
#[path = "resolver_golden_tests.rs"]
mod resolver_golden_tests;
#[cfg(test)]
#[path = "resolver_tests.rs"]
mod resolver_tests;
