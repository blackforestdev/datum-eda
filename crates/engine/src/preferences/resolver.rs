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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthorityRelease {
    pub releasing_user: String,
    pub organization: String,
    pub machine_scope: String,
    pub level: AuthorityReleaseLevel,
    pub effective_at: String,
    pub attribution: String,
    pub revocable: bool,
    pub active: bool,
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
    pub provider: Option<String>,
    pub actor: String,
    pub reason: String,
    /// A Context value participates only after its owning subsystem explicitly
    /// identifies it as the applicable already-authoritative value.
    pub context_is_applicable_authority: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrganizationFact {
    pub id: String,
    pub key: PreferenceKey,
    pub organization: String,
    pub package: String,
    pub actor: String,
    pub reason: String,
    pub directive: OrganizationDirective,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Contribution {
    Value(ValueFact),
    Organization(OrganizationFact),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContributionDisposition {
    Effective,
    Losing,
    Retained,
    Inert,
    Refused,
    Conflicting,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConsideredFact {
    pub id: String,
    pub source: ResolutionSource,
    pub value: Option<Value>,
    pub directive: Option<DirectiveKind>,
    pub disposition: ContributionDisposition,
    pub reason: String,
    pub provider: Option<String>,
    pub actor: String,
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
    pub value_schema_name: Option<String>,
    pub setting_class: Option<SettingClass>,
    pub eligible_sources: BTreeSet<ResolutionSource>,
    pub outcome: ResolutionOutcome,
    pub considered: Vec<ConsideredFact>,
    pub absent_sources: BTreeSet<ResolutionSource>,
    pub evaluation_stages: Vec<String>,
    pub available_actions: BTreeSet<String>,
}

pub struct ResolutionRequest<'a> {
    pub registry: &'a DescriptorRegistry,
    pub key: PreferenceKey,
    pub contributions: &'a [Contribution],
    pub authority_releases: &'a [AuthorityRelease],
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
            value_schema_name: None,
            setting_class: None,
            eligible_sources: BTreeSet::new(),
            outcome: ResolutionOutcome::UnknownKey,
            considered: Vec::new(),
            absent_sources: BTreeSet::new(),
            evaluation_stages: vec!["descriptor identity: unknown; no type guessed".to_owned()],
            available_actions: BTreeSet::from(["preserve opaque record".to_owned()]),
        };
    };

    let mut considered = Vec::new();
    let mut candidates = Vec::new();
    let mut controls = Vec::new();
    let mut present_sources = BTreeSet::new();
    let mut actions = BTreeSet::new();

    if let Some(default) = &descriptor.default_value {
        present_sources.insert(ResolutionSource::DescriptorDefault);
        let index = considered.len();
        considered.push(ConsideredFact {
            id: "descriptor-default".to_owned(),
            source: ResolutionSource::DescriptorDefault,
            value: Some(default.clone()),
            directive: None,
            disposition: ContributionDisposition::Losing,
            reason: "immutable descriptor fact admitted before ranking".to_owned(),
            provider: Some(descriptor.owner.clone()),
            actor: "descriptor".to_owned(),
        });
        candidates.push(Candidate {
            id: "descriptor-default".to_owned(),
            source: ResolutionSource::DescriptorDefault,
            value: default.clone(),
            considered_index: index,
            rank: 0,
        });
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
                &mut present_sources,
                &mut considered,
                &mut candidates,
                &mut controls,
                &mut actions,
            ),
        }
    }

    let conflict = control_conflict(&controls);
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
        value_schema_name: Some(descriptor.value_schema_name.clone()),
        setting_class: Some(descriptor.class),
        eligible_sources: descriptor.allowed_sources.clone(),
        outcome,
        considered,
        absent_sources,
        evaluation_stages: vec![
            "1 descriptor identity, source eligibility, and value validation".to_owned(),
            "2 active AuthorityRelease, constraints, locks, pins, and conflicts".to_owned(),
            "3 Context applicability then Session > User > Recommendation > Installation > default"
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
    actions: &mut BTreeSet<String>,
) {
    present_sources.insert(fact.source);
    let index = considered.len();
    let mut disposition = ContributionDisposition::Losing;
    let reason;
    let rank = source_rank(fact.source);
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
    } else if fact.source == ResolutionSource::Context && !fact.context_is_applicable_authority {
        disposition = ContributionDisposition::Inert;
        reason = "Context has no universal rank; owning subsystem did not select applicability"
            .to_owned();
    } else {
        reason = "eligible validated value admitted to ordinary ranking".to_owned();
        candidates.push(Candidate {
            id: fact.id.clone(),
            source: fact.source,
            value: fact.value.clone(),
            considered_index: index,
            rank,
        });
        if fact.source == ResolutionSource::User {
            actions.insert("reset user contribution".to_owned());
        }
        if fact.source == ResolutionSource::Session {
            actions.insert("end Session override".to_owned());
        }
    }
    considered.push(ConsideredFact {
        id: fact.id.clone(),
        source: fact.source,
        value: Some(fact.value.clone()),
        directive: None,
        disposition,
        reason,
        provider: fact.provider.clone(),
        actor: fact.actor.clone(),
    });
}

#[allow(clippy::too_many_arguments)]
fn consider_organization(
    descriptor: &super::PreferenceDescriptor,
    fact: &OrganizationFact,
    key: &PreferenceKey,
    releases: &[AuthorityRelease],
    present_sources: &mut BTreeSet<ResolutionSource>,
    considered: &mut Vec<ConsideredFact>,
    candidates: &mut Vec<Candidate>,
    controls: &mut Vec<ActiveControl>,
    actions: &mut BTreeSet<String>,
) {
    present_sources.insert(ResolutionSource::Organization);
    let index = considered.len();
    let kind = fact.directive.kind();
    let active_release = releases
        .iter()
        .filter(|release| release.active && release.organization == fact.organization)
        .max_by_key(|release| release.level);
    let required = required_release(kind, descriptor.class);
    let mut disposition = ContributionDisposition::Losing;
    let reason;
    if fact.key != *key
        || !descriptor
            .allowed_sources
            .contains(&ResolutionSource::Organization)
    {
        disposition = ContributionDisposition::Refused;
        reason = "organization source is ineligible for this descriptor".to_owned();
    } else if !descriptor.allowed_directives.contains(&kind) {
        disposition = ContributionDisposition::Refused;
        reason = "descriptor does not admit this organization directive".to_owned();
    } else if fact
        .directive
        .value()
        .is_some_and(|value| !descriptor.validates(value))
    {
        disposition = ContributionDisposition::Refused;
        reason = "directive value violates the authoritative descriptor schema".to_owned();
    } else if matches!(
        &fact.directive,
        OrganizationDirective::Constrain(constraint)
            if !constraint_matches_descriptor(constraint, descriptor)
    ) {
        disposition = ContributionDisposition::Refused;
        reason = "constraint domain is incompatible with the descriptor schema".to_owned();
    } else if active_release.is_none_or(|release| release.level < required) {
        disposition = ContributionDisposition::Inert;
        reason = format!("pending request exceeds active AuthorityRelease {required:?}");
        actions.insert("review organization authority request".to_owned());
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
        actions.insert("manage AuthorityRelease".to_owned());
    }
    considered.push(ConsideredFact {
        id: fact.id.clone(),
        source: ResolutionSource::Organization,
        value: fact.directive.value().cloned(),
        directive: Some(kind),
        disposition,
        reason,
        provider: Some(format!("{}:{}", fact.organization, fact.package)),
        actor: fact.actor.clone(),
    });
}

fn constraint_matches_descriptor(
    constraint: &ValueConstraint,
    descriptor: &super::PreferenceDescriptor,
) -> bool {
    match constraint {
        ValueConstraint::IntegerRange { .. } => {
            matches!(
                &descriptor.value_schema,
                super::ValueSchema::IntegerRange { .. }
            )
        }
        ValueConstraint::AllowedValues(values) | ValueConstraint::DeniedValues(values) => {
            !values.is_empty() && values.iter().all(|value| descriptor.validates(value))
        }
    }
}

fn required_release(kind: DirectiveKind, class: SettingClass) -> AuthorityReleaseLevel {
    match kind {
        DirectiveKind::Recommend => AuthorityReleaseLevel::RecommendationsOnly,
        DirectiveKind::Constrain => AuthorityReleaseLevel::Bounded,
        DirectiveKind::Pin | DirectiveKind::Lock
            if matches!(
                class,
                SettingClass::Capability | SettingClass::ProjectPolicySeed
            ) =>
        {
            AuthorityReleaseLevel::BacksideManagement
        }
        DirectiveKind::Pin | DirectiveKind::Lock => AuthorityReleaseLevel::FullManagement,
    }
}

fn source_rank(source: ResolutionSource) -> u8 {
    match source {
        ResolutionSource::Context => 5,
        ResolutionSource::Session => 4,
        ResolutionSource::User => 3,
        ResolutionSource::Organization => 2,
        ResolutionSource::Installation => 1,
        ResolutionSource::DescriptorDefault => 0,
        ResolutionSource::ProjectPolicy => 5,
    }
}

fn control_conflict(controls: &[ActiveControl]) -> Option<(Vec<String>, String)> {
    let pins: Vec<_> = controls
        .iter()
        .filter_map(|control| match &control.directive {
            OrganizationDirective::Pin(value) => Some((control.id.clone(), value)),
            _ => None,
        })
        .collect();
    let pins_conflict = pins
        .first()
        .is_some_and(|(_, first)| pins.iter().any(|(_, value)| *value != *first));
    if pins_conflict {
        let mut ids: Vec<_> = pins.into_iter().map(|(id, _)| id).collect();
        ids.sort();
        return Some((
            ids,
            "equal-authority Pins specify incompatible values".to_owned(),
        ));
    }
    let ranges: Vec<_> = controls
        .iter()
        .filter_map(|control| match control.directive {
            OrganizationDirective::Constrain(ValueConstraint::IntegerRange { min, max }) => {
                Some((control.id.clone(), min, max))
            }
            _ => None,
        })
        .collect();
    if !ranges.is_empty() {
        let lower = ranges.iter().map(|(_, min, _)| *min).max().unwrap();
        let upper = ranges.iter().map(|(_, _, max)| *max).min().unwrap();
        if lower > upper {
            let mut ids: Vec<_> = ranges.into_iter().map(|(id, _, _)| id).collect();
            ids.sort();
            return Some((
                ids,
                "equal-authority constraints have an empty intersection".to_owned(),
            ));
        }
    }
    let allowed_sets: Vec<_> = controls
        .iter()
        .filter_map(|control| match &control.directive {
            OrganizationDirective::Constrain(ValueConstraint::AllowedValues(values)) => {
                Some((control.id.clone(), values))
            }
            _ => None,
        })
        .collect();
    if let Some((_, first)) = allowed_sets.first() {
        let has_common_value = first.iter().any(|value| {
            allowed_sets
                .iter()
                .all(|(_, allowed)| allowed.contains(value))
        });
        if !has_common_value {
            let mut ids: Vec<_> = allowed_sets.into_iter().map(|(id, _)| id).collect();
            ids.sort();
            return Some((
                ids,
                "equal-authority constraints have an empty allowed-value intersection".to_owned(),
            ));
        }
    }
    None
}

fn mark_conflicting(considered: &mut [ConsideredFact], ids: &[String]) {
    for fact in considered {
        if ids.contains(&fact.id) {
            fact.disposition = ContributionDisposition::Conflicting;
        }
    }
}

fn resolve_with_controls(
    considered: &mut [ConsideredFact],
    candidates: &mut Vec<Candidate>,
    controls: &[ActiveControl],
    actions: &mut BTreeSet<String>,
) -> ResolutionOutcome {
    for control in controls {
        if matches!(
            control.directive,
            OrganizationDirective::Constrain(_) | OrganizationDirective::Lock(_)
        ) {
            considered[control.considered_index].disposition = ContributionDisposition::Effective;
            considered[control.considered_index].reason =
                "active control applies before ordinary value ranking".to_owned();
        }
    }
    let constraints: Vec<_> = controls
        .iter()
        .filter_map(|control| match &control.directive {
            OrganizationDirective::Constrain(constraint) => Some(constraint),
            _ => None,
        })
        .collect();
    for candidate in candidates.iter() {
        if constraints
            .iter()
            .any(|constraint| !constraint.allows(&candidate.value))
        {
            considered[candidate.considered_index].disposition = ContributionDisposition::Inert;
            considered[candidate.considered_index].reason =
                "retained value is outside an active organization constraint".to_owned();
        }
    }
    candidates.retain(|candidate| {
        constraints
            .iter()
            .all(|constraint| constraint.allows(&candidate.value))
    });

    let pins: Vec<_> = controls
        .iter()
        .filter_map(|control| match &control.directive {
            OrganizationDirective::Pin(value) => Some((control, value)),
            _ => None,
        })
        .collect();
    if let Some((_, value)) = pins.first() {
        let pinned_value = (*value).clone();
        if constraints
            .iter()
            .any(|constraint| !constraint.allows(value))
        {
            let mut ids: Vec<_> = pins.iter().map(|(control, _)| control.id.clone()).collect();
            ids.extend(
                controls
                    .iter()
                    .filter(|control| {
                        matches!(control.directive, OrganizationDirective::Constrain(_))
                    })
                    .map(|control| control.id.clone()),
            );
            ids.sort();
            mark_conflicting(considered, &ids);
            return ResolutionOutcome::UnresolvedConflict {
                fact_ids: ids,
                reason: "Pin violates an applicable constraint".to_owned(),
            };
        }
        for candidate in candidates.iter() {
            considered[candidate.considered_index].disposition = ContributionDisposition::Retained;
            considered[candidate.considered_index].reason =
                "retained value resumes when the active Pin lifts".to_owned();
        }
        let mut ids = Vec::new();
        for (control, _) in &pins {
            considered[control.considered_index].disposition = ContributionDisposition::Effective;
            ids.push(control.id.clone());
        }
        ids.sort();
        actions.insert("appeal or revoke managed Pin".to_owned());
        return ResolutionOutcome::Effective {
            value: pinned_value,
            winning_fact_ids: ids,
        };
    }

    let Some(highest_rank) = candidates.iter().map(|candidate| candidate.rank).max() else {
        return ResolutionOutcome::NoValue;
    };
    let winners: Vec<_> = candidates
        .iter()
        .filter(|candidate| candidate.rank == highest_rank)
        .collect();
    let first_value = &winners[0].value;
    if winners
        .iter()
        .any(|candidate| candidate.value != *first_value)
    {
        let mut ids: Vec<_> = winners
            .iter()
            .map(|candidate| candidate.id.clone())
            .collect();
        ids.sort();
        mark_conflicting(considered, &ids);
        return ResolutionOutcome::UnresolvedConflict {
            fact_ids: ids,
            reason: "equal-rank values differ; arrival order cannot choose".to_owned(),
        };
    }
    let winner_ids: BTreeSet<_> = winners
        .iter()
        .map(|candidate| candidate.id.clone())
        .collect();
    let winner_source = winners[0].source;
    for candidate in candidates.iter() {
        if winner_ids.contains(&candidate.id) {
            considered[candidate.considered_index].disposition = ContributionDisposition::Effective;
            considered[candidate.considered_index].reason =
                "wins the descriptor-declared ordinary ranking".to_owned();
        } else {
            considered[candidate.considered_index].disposition = ContributionDisposition::Losing;
            considered[candidate.considered_index].reason =
                format!("loses to higher-ranked {:?} contribution", winner_source);
        }
    }
    ResolutionOutcome::Effective {
        value: first_value.clone(),
        winning_fact_ids: winner_ids.into_iter().collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::preferences::{PreferenceKey, active_v1_registry};

    fn key(value: &str) -> PreferenceKey {
        PreferenceKey::parse(value).unwrap()
    }

    fn value_fact(id: &str, source: ResolutionSource, value: Value) -> Contribution {
        Contribution::Value(ValueFact {
            id: id.to_owned(),
            key: key("datum.viewport.snap_enabled"),
            source,
            value,
            provider: None,
            actor: "owner".to_owned(),
            reason: "test".to_owned(),
            context_is_applicable_authority: false,
        })
    }

    fn release(level: AuthorityReleaseLevel, active: bool) -> AuthorityRelease {
        AuthorityRelease {
            releasing_user: "owner".to_owned(),
            organization: "acme".to_owned(),
            machine_scope: "machine".to_owned(),
            level,
            effective_at: "2026-08-31".to_owned(),
            attribution: "owner approval".to_owned(),
            revocable: true,
            active,
        }
    }

    fn organization(id: &str, directive: OrganizationDirective) -> Contribution {
        Contribution::Organization(OrganizationFact {
            id: id.to_owned(),
            key: key("datum.viewport.snap_enabled"),
            organization: "acme".to_owned(),
            package: "policy-1".to_owned(),
            actor: "admin".to_owned(),
            reason: "shop policy".to_owned(),
            directive,
        })
    }

    fn resolve(facts: &[Contribution], releases: &[AuthorityRelease]) -> PreferenceExplanation {
        let registry = active_v1_registry();
        resolve_preference(ResolutionRequest {
            registry: &registry,
            key: key("datum.viewport.snap_enabled"),
            contributions: facts,
            authority_releases: releases,
        })
    }

    #[test]
    fn user_outranks_recommendation_installation_and_default() {
        let facts = vec![
            value_fact(
                "install",
                ResolutionSource::Installation,
                Value::Bool(false),
            ),
            organization(
                "recommend",
                OrganizationDirective::Recommend(Value::Bool(false)),
            ),
            value_fact("user", ResolutionSource::User, Value::Bool(true)),
        ];
        let explanation = resolve(
            &facts,
            &[release(AuthorityReleaseLevel::RecommendationsOnly, true)],
        );
        assert_eq!(
            explanation.outcome,
            ResolutionOutcome::Effective {
                value: Value::Bool(true),
                winning_fact_ids: vec!["user".to_owned()]
            }
        );
        assert_eq!(explanation.considered.len(), 4);
        assert!(
            explanation
                .absent_sources
                .contains(&ResolutionSource::Session)
        );
    }

    #[test]
    fn pin_retains_user_and_revocation_restores_it() {
        let facts = vec![
            value_fact("user", ResolutionSource::User, Value::Bool(true)),
            organization("pin", OrganizationDirective::Pin(Value::Bool(false))),
        ];
        let pinned = resolve(
            &facts,
            &[release(AuthorityReleaseLevel::FullManagement, true)],
        );
        assert!(matches!(
            pinned.outcome,
            ResolutionOutcome::Effective {
                value: Value::Bool(false),
                ..
            }
        ));
        assert_eq!(
            pinned
                .considered
                .iter()
                .find(|fact| fact.id == "user")
                .unwrap()
                .disposition,
            ContributionDisposition::Retained
        );

        let revoked = resolve(
            &facts,
            &[release(AuthorityReleaseLevel::FullManagement, false)],
        );
        assert!(matches!(
            revoked.outcome,
            ResolutionOutcome::Effective {
                value: Value::Bool(true),
                ..
            }
        ));
    }

    #[test]
    fn insufficient_release_keeps_request_visible_and_inert() {
        let facts = vec![organization(
            "pin",
            OrganizationDirective::Pin(Value::Bool(false)),
        )];
        let explanation = resolve(
            &facts,
            &[release(AuthorityReleaseLevel::RecommendationsOnly, true)],
        );
        assert_eq!(
            explanation
                .considered
                .iter()
                .find(|fact| fact.id == "pin")
                .unwrap()
                .disposition,
            ContributionDisposition::Inert
        );
        assert!(matches!(
            explanation.outcome,
            ResolutionOutcome::Effective {
                value: Value::Bool(true),
                ..
            }
        ));
    }

    #[test]
    fn incompatible_equal_authority_pins_are_typed_conflict() {
        let facts = vec![
            organization("pin-a", OrganizationDirective::Pin(Value::Bool(false))),
            organization("pin-b", OrganizationDirective::Pin(Value::Bool(true))),
        ];
        let explanation = resolve(
            &facts,
            &[release(AuthorityReleaseLevel::FullManagement, true)],
        );
        assert!(matches!(
            explanation.outcome,
            ResolutionOutcome::UnresolvedConflict { .. }
        ));
    }

    #[test]
    fn incompatible_equal_authority_constraints_are_typed_conflict() {
        let facts = vec![
            organization(
                "constraint-a",
                OrganizationDirective::Constrain(ValueConstraint::AllowedValues(vec![
                    Value::Bool(true),
                ])),
            ),
            organization(
                "constraint-b",
                OrganizationDirective::Constrain(ValueConstraint::AllowedValues(vec![
                    Value::Bool(false),
                ])),
            ),
        ];
        let explanation = resolve(&facts, &[release(AuthorityReleaseLevel::Bounded, true)]);
        assert!(matches!(
            explanation.outcome,
            ResolutionOutcome::UnresolvedConflict { .. }
        ));
    }

    #[test]
    fn validation_and_source_eligibility_precede_ranking() {
        let facts = vec![
            value_fact(
                "invalid",
                ResolutionSource::User,
                Value::String("yes".to_owned()),
            ),
            value_fact(
                "project",
                ResolutionSource::ProjectPolicy,
                Value::Bool(false),
            ),
        ];
        let explanation = resolve(&facts, &[]);
        for id in ["invalid", "project"] {
            assert_eq!(
                explanation
                    .considered
                    .iter()
                    .find(|fact| fact.id == id)
                    .unwrap()
                    .disposition,
                ContributionDisposition::Refused
            );
        }
        assert!(matches!(
            explanation.outcome,
            ResolutionOutcome::Effective {
                value: Value::Bool(true),
                ..
            }
        ));
    }

    #[test]
    fn session_outranks_user_but_not_an_active_pin() {
        let facts = vec![
            value_fact("user", ResolutionSource::User, Value::Bool(false)),
            value_fact("session", ResolutionSource::Session, Value::Bool(true)),
        ];
        assert!(matches!(
            resolve(&facts, &[]).outcome,
            ResolutionOutcome::Effective {
                value: Value::Bool(true),
                ..
            }
        ));
        let mut pinned_facts = facts;
        pinned_facts.push(organization(
            "pin",
            OrganizationDirective::Pin(Value::Bool(false)),
        ));
        assert!(matches!(
            resolve(
                &pinned_facts,
                &[release(AuthorityReleaseLevel::FullManagement, true)]
            )
            .outcome,
            ResolutionOutcome::Effective {
                value: Value::Bool(false),
                ..
            }
        ));
    }

    #[test]
    fn explanation_is_input_order_independent() {
        let mut facts = vec![
            value_fact("z-user", ResolutionSource::User, Value::Bool(true)),
            value_fact(
                "a-install",
                ResolutionSource::Installation,
                Value::Bool(false),
            ),
        ];
        let first = resolve(&facts, &[]);
        facts.reverse();
        let second = resolve(&facts, &[]);
        assert_eq!(first, second);
    }

    #[test]
    fn unknown_key_never_guesses_a_descriptor() {
        let registry = active_v1_registry();
        let explanation = resolve_preference(ResolutionRequest {
            registry: &registry,
            key: key("datum.unknown.future_key"),
            contributions: &[],
            authority_releases: &[],
        });
        assert_eq!(explanation.outcome, ResolutionOutcome::UnknownKey);
        assert_eq!(explanation.value_schema_name, None);
    }
}
