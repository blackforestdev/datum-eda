use super::*;
use crate::preferences::{PreferenceKey, active_v1_registry};

pub(super) fn key(value: &str) -> PreferenceKey {
    PreferenceKey::parse(value).unwrap()
}

pub(super) fn value_fact(id: &str, source: ResolutionSource, value: Value) -> Contribution {
    Contribution::Value(ValueFact {
        id: id.to_owned(),
        key: key("datum.viewport.snap_enabled"),
        source,
        value,
        provenance: FactProvenance {
            origin: "test".to_owned(),
            provider: None,
            package: None,
            generation: None,
            actor: "owner".to_owned(),
            role: Some("user".to_owned()),
            observed_at: "2026-08-31T00:00:00Z".to_owned(),
            effective_from: None,
            effective_until: None,
            offline_valid_until: None,
            last_successful_contact: None,
            reason: "test".to_owned(),
        },
        disclosure: ValueDisclosure::Disclosed,
        provider_state: None,
        context: None,
    })
}

fn release(level: AuthorityReleaseLevel, active: bool) -> AuthorityRelease {
    AuthorityRelease {
        id: "release-1".to_owned(),
        releasing_user: "owner".to_owned(),
        organization: "acme".to_owned(),
        machine_scope: "machine".to_owned(),
        level,
        effective_at: "2026-08-31".to_owned(),
        effective_until: None,
        attribution: "owner approval".to_owned(),
        revocable: true,
        state: if active {
            AuthorityReleaseState::Active
        } else {
            AuthorityReleaseState::Revoked
        },
    }
}

fn organization(id: &str, directive: OrganizationDirective) -> Contribution {
    organization_with_state(id, directive, ProviderGenerationState::Active)
}

fn organization_with_state(
    id: &str,
    directive: OrganizationDirective,
    generation_state: ProviderGenerationState,
) -> Contribution {
    Contribution::Organization(OrganizationFact {
        id: id.to_owned(),
        key: key("datum.viewport.snap_enabled"),
        organization: "acme".to_owned(),
        provenance: FactProvenance {
            origin: "managed-policy".to_owned(),
            provider: Some("acme-provider".to_owned()),
            package: Some("policy-1".to_owned()),
            generation: Some("7".to_owned()),
            actor: "admin".to_owned(),
            role: Some("organization administrator".to_owned()),
            observed_at: "2026-08-31T00:00:00Z".to_owned(),
            effective_from: Some("2026-08-31T00:00:00Z".to_owned()),
            effective_until: None,
            offline_valid_until: Some("2026-09-07T00:00:00Z".to_owned()),
            last_successful_contact: Some("2026-08-31T00:00:00Z".to_owned()),
            reason: "shop policy".to_owned(),
        },
        authenticated: true,
        disclosure: ValueDisclosure::Disclosed,
        generation_state,
        remaining_freedom: "user may appeal or revoke released authority".to_owned(),
        appeal_path: Some("manage AuthorityRelease".to_owned()),
        directive,
    })
}

pub(super) fn resolve(
    facts: &[Contribution],
    releases: &[AuthorityRelease],
) -> PreferenceExplanation {
    let registry = active_v1_registry();
    resolve_preference(ResolutionRequest {
        registry: &registry,
        key: key("datum.viewport.snap_enabled"),
        contributions: facts,
        authority_releases: releases,
        machine_scope: "machine",
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
fn release_scope_and_provider_authentication_are_enforced_before_control() {
    let facts = vec![organization(
        "pin",
        OrganizationDirective::Pin(Value::Bool(false)),
    )];
    let mut wrong_scope = release(AuthorityReleaseLevel::FullManagement, true);
    wrong_scope.machine_scope = "other-machine".to_owned();
    let scoped = resolve(&facts, &[wrong_scope]);
    assert_eq!(
        scoped
            .considered
            .iter()
            .find(|fact| fact.id == "pin")
            .unwrap()
            .disposition,
        ContributionDisposition::Inert
    );

    let mut unauthenticated = match organization(
        "unauthenticated",
        OrganizationDirective::Pin(Value::Bool(false)),
    ) {
        Contribution::Organization(fact) => fact,
        Contribution::Value(_) | Contribution::RuntimeDefault(_) => unreachable!(),
    };
    unauthenticated.authenticated = false;
    let refused = resolve(
        &[Contribution::Organization(unauthenticated)],
        &[release(AuthorityReleaseLevel::FullManagement, true)],
    );
    assert_eq!(
        refused
            .considered
            .iter()
            .find(|fact| fact.id == "unauthenticated")
            .unwrap()
            .disposition,
        ContributionDisposition::Refused
    );
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
            OrganizationDirective::Constrain(ValueConstraint::AllowedValues(vec![Value::Bool(
                true,
            )])),
        ),
        organization(
            "constraint-b",
            OrganizationDirective::Constrain(ValueConstraint::AllowedValues(vec![Value::Bool(
                false,
            )])),
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
        machine_scope: "machine",
    });
    assert_eq!(explanation.outcome, ResolutionOutcome::UnknownKey);
    assert_eq!(explanation.value_schema_name, None);
}

#[test]
fn context_reports_applicability_without_receiving_a_rank() {
    let mut context = match value_fact("context", ResolutionSource::Context, Value::Bool(true)) {
        Contribution::Value(fact) => fact,
        Contribution::Organization(_) | Contribution::RuntimeDefault(_) => unreachable!(),
    };
    context.context = Some(ContextApplicability {
        subject: "project-7".to_owned(),
        authority: "AdoptedDraftingStandard".to_owned(),
        lifetime: "while project-7 is focused".to_owned(),
        reason: "the focused Project selects the applicable rule".to_owned(),
    });
    let facts = vec![
        value_fact("user", ResolutionSource::User, Value::Bool(false)),
        Contribution::Value(context),
    ];
    let explanation = resolve(&facts, &[]);
    assert_eq!(
        explanation.outcome,
        ResolutionOutcome::Effective {
            value: Value::Bool(false),
            winning_fact_ids: vec!["user".to_owned()],
        }
    );
    assert_eq!(
        explanation
            .considered
            .iter()
            .find(|fact| fact.id == "context")
            .unwrap()
            .disposition,
        ContributionDisposition::Applicable
    );
}

#[test]
fn inactive_provider_states_are_visible_and_never_participate() {
    for (state, expected) in [
        (
            ProviderGenerationState::Unavailable,
            ContributionDisposition::Unavailable,
        ),
        (
            ProviderGenerationState::Expired,
            ContributionDisposition::Expired,
        ),
        (
            ProviderGenerationState::Revoked,
            ContributionDisposition::Revoked,
        ),
        (
            ProviderGenerationState::Superseded,
            ContributionDisposition::Superseded,
        ),
    ] {
        let facts = vec![organization_with_state(
            "managed",
            OrganizationDirective::Pin(Value::Bool(false)),
            state,
        )];
        let explanation = resolve(
            &facts,
            &[release(AuthorityReleaseLevel::FullManagement, true)],
        );
        let managed = explanation
            .considered
            .iter()
            .find(|fact| fact.id == "managed")
            .unwrap();
        assert_eq!(managed.disposition, expected);
        assert_eq!(managed.provider_state, Some(state));
        assert!(matches!(
            explanation.outcome,
            ResolutionOutcome::Effective {
                value: Value::Bool(true),
                ..
            }
        ));
    }
}

#[test]
fn stale_generation_remains_effective_only_as_explicit_stale_truth() {
    let facts = vec![organization_with_state(
        "stale-pin",
        OrganizationDirective::Pin(Value::Bool(false)),
        ProviderGenerationState::StaleEffective,
    )];
    let explanation = resolve(
        &facts,
        &[release(AuthorityReleaseLevel::FullManagement, true)],
    );
    let managed = explanation
        .considered
        .iter()
        .find(|fact| fact.id == "stale-pin")
        .unwrap();
    assert_eq!(managed.disposition, ContributionDisposition::Effective);
    assert_eq!(
        managed.provider_state,
        Some(ProviderGenerationState::StaleEffective)
    );
    assert!(matches!(
        explanation.outcome,
        ResolutionOutcome::Effective {
            value: Value::Bool(false),
            ..
        }
    ));
}

#[test]
fn explicit_redaction_hides_value_without_hiding_authority() {
    let mut user = match value_fact("secret", ResolutionSource::User, Value::Bool(false)) {
        Contribution::Value(fact) => fact,
        Contribution::Organization(_) | Contribution::RuntimeDefault(_) => unreachable!(),
    };
    user.disclosure = ValueDisclosure::Redacted;
    let explanation = resolve(&[Contribution::Value(user)], &[]);
    let considered = explanation
        .considered
        .iter()
        .find(|fact| fact.id == "secret")
        .unwrap();
    assert_eq!(considered.disclosure, ValueDisclosure::Redacted);
    assert_eq!(considered.value, None);
    assert_eq!(considered.disposition, ContributionDisposition::Effective);
}

#[test]
fn registered_runtime_default_requires_exact_recipe_and_schema() {
    let registry = active_v1_registry();
    let preference_key = key("datum.viewport.grid_mark_style");
    let provenance = FactProvenance {
        origin: "shared viewport profile".to_owned(),
        provider: Some("shared viewport".to_owned()),
        package: None,
        generation: Some("1".to_owned()),
        actor: "runtime-default evaluator".to_owned(),
        role: Some("subsystem owner".to_owned()),
        observed_at: "2026-08-31T00:00:00Z".to_owned(),
        effective_from: None,
        effective_until: None,
        offline_valid_until: None,
        last_successful_contact: None,
        reason: "resolved engine profile".to_owned(),
    };
    let contribution = Contribution::RuntimeDefault(RuntimeDefaultFact {
        id: "runtime-default".to_owned(),
        key: preference_key.clone(),
        recipe: "shared_viewport.grid_mark_style.v1".to_owned(),
        value: serde_json::json!({"shape":"dot","size":1,"min_spacing_px":8}),
        provenance,
        disclosure: ValueDisclosure::Disclosed,
    });
    let explanation = resolve_preference(ResolutionRequest {
        registry: &registry,
        key: preference_key,
        contributions: &[contribution],
        authority_releases: &[],
        machine_scope: "machine",
    });
    assert!(matches!(
        explanation.outcome,
        ResolutionOutcome::Effective { .. }
    ));
    assert!(
        !explanation
            .available_actions
            .contains(&AvailableAction::ProvideRuntimeDefault)
    );
}

#[test]
fn distinct_equal_authority_locks_are_a_typed_conflict() {
    let facts = vec![
        organization(
            "lock-user",
            OrganizationDirective::Lock(BTreeSet::from([ResolutionSource::User])),
        ),
        organization(
            "lock-session",
            OrganizationDirective::Lock(BTreeSet::from([ResolutionSource::Session])),
        ),
    ];
    assert!(matches!(
        resolve(
            &facts,
            &[release(AuthorityReleaseLevel::FullManagement, true)]
        )
        .outcome,
        ResolutionOutcome::UnresolvedConflict { .. }
    ));
}

#[test]
fn empty_or_nonwritable_lock_scopes_are_refused() {
    for scopes in [
        BTreeSet::new(),
        BTreeSet::from([ResolutionSource::DescriptorDefault]),
        BTreeSet::from([ResolutionSource::Context]),
        BTreeSet::from([ResolutionSource::ProjectPolicy]),
    ] {
        let explanation = resolve(
            &[organization(
                "invalid-lock",
                OrganizationDirective::Lock(scopes),
            )],
            &[release(AuthorityReleaseLevel::FullManagement, true)],
        );
        assert_eq!(
            explanation
                .considered
                .iter()
                .find(|fact| fact.id == "invalid-lock")
                .unwrap()
                .disposition,
            ContributionDisposition::Refused
        );
    }
}

#[test]
fn allowed_and_denied_constraints_with_empty_domain_conflict() {
    let facts = vec![
        organization(
            "allowed",
            OrganizationDirective::Constrain(ValueConstraint::AllowedValues(vec![Value::Bool(
                true,
            )])),
        ),
        organization(
            "denied",
            OrganizationDirective::Constrain(ValueConstraint::DeniedValues(vec![
                Value::Bool(true),
                Value::Bool(false),
            ])),
        ),
    ];
    assert!(matches!(
        resolve(&facts, &[release(AuthorityReleaseLevel::Bounded, true)]).outcome,
        ResolutionOutcome::UnresolvedConflict { .. }
    ));
}
