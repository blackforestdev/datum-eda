use super::resolver_tests::{key, resolve, value_fact};
use super::*;

#[test]
fn explanation_semantic_golden_is_complete_and_stable() {
    let facts = vec![value_fact(
        "user",
        ResolutionSource::User,
        Value::Bool(false),
    )];
    let actual = resolve(&facts, &[]);
    let descriptor_provenance = FactProvenance {
        origin: "datum.viewport.snap_enabled".to_owned(),
        provider: Some("shared viewport".to_owned()),
        package: None,
        generation: Some("1".to_owned()),
        actor: "descriptor".to_owned(),
        role: Some("subsystem owner".to_owned()),
        observed_at: "registry snapshot".to_owned(),
        effective_from: None,
        effective_until: None,
        offline_valid_until: None,
        last_successful_contact: None,
        reason: "literal descriptor default".to_owned(),
    };
    assert_eq!(
            actual,
            PreferenceExplanation {
                key: key("datum.viewport.snap_enabled"),
                machine_scope: "machine".to_owned(),
                value_schema_name: Some("bool".to_owned()),
                setting_class: Some(SettingClass::WorkflowDefault),
                eligible_sources: BTreeSet::from([
                    ResolutionSource::DescriptorDefault,
                    ResolutionSource::Installation,
                    ResolutionSource::Organization,
                    ResolutionSource::User,
                    ResolutionSource::Session,
                    ResolutionSource::Context,
                ]),
                outcome: ResolutionOutcome::Effective {
                    value: Value::Bool(false),
                    winning_fact_ids: vec!["user".to_owned()],
                },
                considered: vec![
                    ConsideredFact {
                        id: "descriptor-default".to_owned(),
                        source: ResolutionSource::DescriptorDefault,
                        value: Some(Value::Bool(true)),
                        directive: None,
                        control: None,
                        disposition: ContributionDisposition::Losing,
                        reason: "loses to higher-ranked User contribution".to_owned(),
                        provenance: descriptor_provenance,
                        disclosure: ValueDisclosure::Disclosed,
                        provider_state: None,
                        authority_release: None,
                        remaining_freedom: None,
                        appeal_path: None,
                        reactivation_condition: None,
                        context: None,
                    },
                    match &facts[0] {
                        Contribution::Value(fact) => ConsideredFact {
                            id: "user".to_owned(),
                            source: ResolutionSource::User,
                            value: Some(Value::Bool(false)),
                            directive: None,
                            control: None,
                            disposition: ContributionDisposition::Effective,
                            reason: "wins the descriptor-declared ordinary ranking".to_owned(),
                            provenance: fact.provenance.clone(),
                            disclosure: ValueDisclosure::Disclosed,
                            provider_state: None,
                            authority_release: None,
                            remaining_freedom: None,
                            appeal_path: None,
                            reactivation_condition: None,
                            context: None,
                        },
                        Contribution::Organization(_) | Contribution::RuntimeDefault(_) => {
                            unreachable!()
                        }
                    },
                ],
                absent_sources: BTreeSet::from([
                    ResolutionSource::Installation,
                    ResolutionSource::Organization,
                    ResolutionSource::Session,
                    ResolutionSource::Context,
                ]),
                evaluation_stages: vec![
                    "1 descriptor identity, source eligibility, and value validation".to_owned(),
                    "2 active AuthorityRelease, constraints, locks, pins, and conflicts".to_owned(),
                    "3 report Context applicability, then rank Session > User > Recommendation > Installation > default".to_owned(),
                ],
                available_actions: BTreeSet::from([AvailableAction::ResetUserContribution]),
            }
        );
}
