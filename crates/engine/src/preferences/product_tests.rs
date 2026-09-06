use std::path::PathBuf;

use serde_json::{Value, json};
use uuid::Uuid;

use super::*;

struct TestLocations(PathBuf);

impl TestLocations {
    fn new(name: &str) -> Self {
        Self(std::env::temp_dir().join(format!(
            "datum-preference-product-{name}-{}",
            Uuid::new_v4()
        )))
    }

    fn provider(&self) -> FixedPreferenceLocationProvider {
        FixedPreferenceLocationProvider(PreferenceLocations {
            configuration_base: self.0.clone(),
            repository_root: self.0.join("datum/preferences"),
            legacy_console_path: self.0.join("datum/gui-preferences.json"),
        })
    }
}

impl Drop for TestLocations {
    fn drop(&mut self) {
        if self.0.starts_with(std::env::temp_dir()) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
}

fn service(name: &str) -> (TestLocations, GlobalPreferencesProductService) {
    let locations = TestLocations::new(name);
    let service = GlobalPreferencesProductService::open(&locations.provider(), "product-test")
        .expect("test product service opens");
    (locations, service)
}

fn actor(kind: PreferenceActorKindV1) -> PreferenceActorV1 {
    PreferenceActorV1 {
        kind,
        session_id: "test-session".to_owned(),
        local_actor_id: "test-user".to_owned(),
        invocation_id: Uuid::new_v4(),
    }
}

#[test]
fn product_catalog_and_active_only_search_have_exact_inventory() {
    let (_locations, service) = service("catalog");
    assert_eq!(service.context().active_descriptor_count, 11);
    assert_eq!(service.context().reserved_descriptor_count, 45);
    let PreferenceQueryResultV1::Describe(describe) =
        service.query(PreferenceQueryV1::Describe).unwrap()
    else {
        panic!("expected Describe result")
    };
    assert_eq!(
        describe
            .sections
            .iter()
            .map(|section| section.id.as_str())
            .collect::<Vec<_>>(),
        ["appearance", "units"]
    );
    assert_eq!(describe.controls.len(), 11);

    let PreferenceQueryResultV1::Search(alias) = service
        .query(PreferenceQueryV1::Search {
            query: "console_duration".to_owned(),
        })
        .unwrap()
    else {
        panic!("expected Search result")
    };
    assert_eq!(alias.matches.len(), 1);
    assert_eq!(
        alias.matches[0].match_kind,
        PreferenceMatchKindV1::RegisteredAlias
    );
    assert_eq!(
        alias.matches[0].matched_vocabulary.as_deref(),
        Some("console_duration")
    );

    let PreferenceQueryResultV1::Search(reserved) = service
        .query(PreferenceQueryV1::Search {
            query: "datum.pcb.layer_color_scheme".to_owned(),
        })
        .unwrap()
    else {
        panic!("expected Search result")
    };
    assert!(reserved.matches.is_empty());
    let refusal = service
        .query(PreferenceQueryV1::Get {
            key: "datum.pcb.layer_color_scheme".to_owned(),
        })
        .unwrap_err();
    assert_eq!(refusal.code, PreferenceErrorCodeV1::ReservedPreferenceKey);
}

#[test]
fn direct_mutation_requires_trusted_human_and_preserves_project_independence() {
    let (locations, mut service) = service("actor");
    let request = PreferenceMutationRequestV1::SetUser {
        key: "datum.accessibility.reduced_motion".to_owned(),
        value: Value::Bool(true),
        expected: HeadExpectationV1::Missing,
        request_id: Uuid::new_v4(),
        reason: "accessibility choice".to_owned(),
    };
    let refusal = service
        .mutate(request.clone(), &actor(PreferenceActorKindV1::McpAgent))
        .unwrap_err();
    assert_eq!(refusal.code, PreferenceErrorCodeV1::ProposalRequired);
    assert!(!locations.0.join("datum/preferences").exists());

    let result = service
        .mutate(request, &actor(PreferenceActorKindV1::HumanGui))
        .unwrap();
    assert!(result.changed);
    assert_eq!(
        result.generation.as_ref().map(|value| value.generation),
        Some(0)
    );
    assert_eq!(result.value.user_value, Some(json!(true)));
    assert!(result.receipt.is_some());
    assert!(!locations.0.join("project.json").exists());
}

#[test]
fn factory_seed_preview_never_creates_or_reads_a_repository() {
    let (locations, service) = service("factory-preview");
    let PreferenceQueryResultV1::PreviewProjectUnitsSeed(preview) = service
        .query(PreferenceQueryV1::PreviewProjectUnitsSeed {
            source: ProjectUnitsSourceV1::Factory {
                profile_id: "datum.units.factory.v1".to_owned(),
            },
        })
        .unwrap()
    else {
        panic!("expected seed preview")
    };
    assert_eq!(
        preview.profile.as_object().map(|value| value.len()),
        Some(8)
    );
    assert!(preview.pinned_generation.is_none());
    assert!(!locations.0.join("datum/preferences").exists());
}

#[test]
fn wire_schemas_refuse_unknown_fields_and_round_trip_symbolic_errors() {
    let error = PreferenceErrorV1 {
        code: PreferenceErrorCodeV1::StaleGeneration,
        message: "changed".to_owned(),
        details: Default::default(),
        current_context: PreferenceContextV1 {
            scope: "global_this_device".to_owned(),
            repository_status: PreferenceRepositoryStatusV1::DefaultsOnly,
            generation: None,
            active_catalog_digest: "sha256:test".to_owned(),
            active_descriptor_count: 11,
            reserved_descriptor_count: 45,
        },
        preserved_draft: Some(json!(true)),
        preserved_proposal: None,
    };
    let encoded = serde_json::to_value(&error).unwrap();
    assert_eq!(encoded["code"], "stale_generation");
    assert_eq!(
        serde_json::from_value::<PreferenceErrorV1>(encoded).unwrap(),
        error
    );
    assert!(
        serde_json::from_value::<PreferenceSchemaRefV1>(json!({
            "name": "datum.preferences.get",
            "version": 1,
            "future": true
        }))
        .is_err()
    );
}

#[test]
fn proposal_prepare_validate_and_reject_are_portable_and_side_effect_free() {
    let (locations, service) = service("proposal-pure");
    let prepared = service
        .prepare_proposal(
            PreferenceMutationRequestV1::SetUser {
                key: "datum.accessibility.reduced_motion".to_owned(),
                value: json!(true),
                expected: HeadExpectationV1::Missing,
                request_id: Uuid::new_v4(),
                reason: "reduce animation".to_owned(),
            },
            "agent proposes an accessibility preference".to_owned(),
            &actor(PreferenceActorKindV1::McpAgent),
        )
        .unwrap();
    assert!(prepared.proposal.proposal_digest.starts_with("sha256:"));
    assert_eq!(prepared.current_value.user_value, None);
    assert!(!locations.0.join("datum/preferences").exists());

    let validated = service
        .validate_proposal(prepared.proposal.clone())
        .unwrap();
    assert!(validated.valid);
    assert_eq!(validated.current_generation, None);
    let rejected = service.reject_proposal(prepared.proposal).unwrap();
    assert!(rejected.rejected);
    assert!(!locations.0.join("datum/preferences").exists());
}

#[test]
fn proposal_digest_tampering_and_stale_authority_are_typed_refusals() {
    let (_locations, mut service) = service("proposal-refusal");
    let requesting_actor = actor(PreferenceActorKindV1::ScriptAgent);
    let prepared = service
        .prepare_proposal(
            PreferenceMutationRequestV1::SetUser {
                key: "datum.accessibility.reduced_motion".to_owned(),
                value: json!(true),
                expected: HeadExpectationV1::Missing,
                request_id: Uuid::new_v4(),
                reason: "reduce animation".to_owned(),
            },
            "automation requests review".to_owned(),
            &requesting_actor,
        )
        .unwrap();

    let mut tampered = prepared.proposal.clone();
    tampered.rationale = "different rationale".to_owned();
    let refusal = service.validate_proposal(tampered).unwrap_err();
    assert_eq!(refusal.code, PreferenceErrorCodeV1::ProposalInvalid);
    assert!(refusal.preserved_proposal.is_some());

    service
        .mutate(
            PreferenceMutationRequestV1::SetUser {
                key: "datum.accessibility.high_contrast_noncolor".to_owned(),
                value: json!(true),
                expected: HeadExpectationV1::Missing,
                request_id: Uuid::new_v4(),
                reason: "contrast choice".to_owned(),
            },
            &actor(PreferenceActorKindV1::HumanGui),
        )
        .unwrap();
    let refusal = service.validate_proposal(prepared.proposal).unwrap_err();
    assert_eq!(refusal.code, PreferenceErrorCodeV1::ProposalStale);
    assert!(refusal.preserved_proposal.is_some());
}

#[test]
fn mutation_idempotency_survives_restart_and_replays_the_original_generation() {
    let locations = TestLocations::new("durable-idempotency");
    let mut service = GlobalPreferencesProductService::open(&locations.provider(), "writer-one")
        .expect("test product service opens");
    let request_id = Uuid::new_v4();
    let trusted_actor = actor(PreferenceActorKindV1::HumanGui);
    let request = PreferenceMutationRequestV1::SetUser {
        key: "datum.accessibility.reduced_motion".to_owned(),
        value: json!(true),
        expected: HeadExpectationV1::Missing,
        request_id,
        reason: "reduce animation".to_owned(),
    };
    let original = service
        .mutate(request.clone(), &trusted_actor)
        .expect("first request commits");
    let original_receipt = original.receipt.as_ref().expect("changed receipt");
    assert_eq!(original_receipt.request_id, Some(request_id));
    assert_eq!(original_receipt.actor_kind.as_deref(), Some("human_gui"));
    assert_eq!(
        original_receipt.actor_session_id.as_deref(),
        Some("test-session")
    );
    assert_eq!(
        original_receipt.invocation_id,
        Some(trusted_actor.invocation_id)
    );

    service
        .mutate(
            PreferenceMutationRequestV1::SetUser {
                key: "datum.accessibility.high_contrast_noncolor".to_owned(),
                value: json!(true),
                expected: HeadExpectationV1::Generation(
                    original.generation.clone().expect("generation zero"),
                ),
                request_id: Uuid::new_v4(),
                reason: "contrast choice".to_owned(),
            },
            &actor(PreferenceActorKindV1::HumanGui),
        )
        .expect("later request commits");
    assert_eq!(service.context().generation.as_ref().unwrap().generation, 1);
    drop(service);

    let mut reopened = GlobalPreferencesProductService::open(&locations.provider(), "writer-two")
        .expect("repository reopens");
    let replay = reopened
        .mutate(request.clone(), &trusted_actor)
        .expect("identical retry replays");
    assert!(replay.changed);
    assert_eq!(replay.generation.as_ref().unwrap().generation, 0);
    assert_eq!(replay.value.user_value, Some(json!(true)));
    assert_eq!(replay.receipt, original.receipt);
    assert_eq!(
        reopened.context().generation.as_ref().unwrap().generation,
        1
    );

    let conflict = reopened
        .mutate(
            PreferenceMutationRequestV1::SetUser {
                key: "datum.accessibility.reduced_motion".to_owned(),
                value: json!(false),
                expected: HeadExpectationV1::Missing,
                request_id,
                reason: "reduce animation".to_owned(),
            },
            &trusted_actor,
        )
        .unwrap_err();
    assert_eq!(conflict.code, PreferenceErrorCodeV1::IdempotencyConflict);
    assert_eq!(
        reopened.context().generation.as_ref().unwrap().generation,
        1
    );
}

#[test]
fn mcp_acceptance_broker_binds_human_session_repository_and_invocation() {
    let (_locations, service) = service("acceptance-broker");
    let mcp_actor = actor(PreferenceActorKindV1::McpAgent);
    let prepared = service
        .prepare_proposal(
            PreferenceMutationRequestV1::SetUser {
                key: "datum.accessibility.reduced_motion".to_owned(),
                value: json!(true),
                expected: HeadExpectationV1::Missing,
                request_id: Uuid::new_v4(),
                reason: "reduce animation".to_owned(),
            },
            "request local human review".to_owned(),
            &mcp_actor,
        )
        .unwrap();
    let repository_identity = "missing:test-repository";
    let daemon_id = Uuid::new_v4();
    let mut broker = PreferenceAcceptanceBroker::new(daemon_id);
    broker
        .register_prepared(&prepared.proposal, repository_identity)
        .unwrap();
    let request = AuthorizeMcpPreferenceApplyV1 {
        proposal_id: prepared.proposal.proposal_id,
        proposal_digest: prepared.proposal.proposal_digest.clone(),
        originating_mcp_session: mcp_actor.session_id.clone(),
    };
    let apply_invocation = Uuid::new_v4();
    let human = actor(PreferenceActorKindV1::HumanGui);
    let public = broker
        .authorize_mcp_apply(
            &request,
            &human,
            repository_identity,
            apply_invocation,
            1_000,
        )
        .unwrap();
    assert!(public.authorized);
    assert_eq!(public.expires_at_unix_ms, 301_000);
    let encoded = serde_json::to_value(&public).unwrap();
    assert!(encoded.get("handle").is_none());
    assert!(encoded.get("acceptance_id").is_none());

    assert_eq!(
        broker.authorization_for_apply(
            &prepared.proposal,
            &mcp_actor.session_id,
            repository_identity,
            Uuid::new_v4(),
            2_000,
        ),
        Err(PreferenceAcceptanceRefusal::InvocationMismatch)
    );
    let handle = broker
        .authorization_for_apply(
            &prepared.proposal,
            &mcp_actor.session_id,
            repository_identity,
            apply_invocation,
            2_000,
        )
        .unwrap();
    assert_ne!(handle.acceptance_id(), Uuid::nil());
    assert_eq!(handle.accepting_actor(), &human);
    broker.consume(&handle).unwrap();
    assert_eq!(
        broker.consume(&handle),
        Err(PreferenceAcceptanceRefusal::Consumed)
    );
}

#[test]
fn mcp_acceptance_broker_expires_and_closes_without_portable_authority() {
    let (_locations, service) = service("acceptance-expiry");
    let mcp_actor = actor(PreferenceActorKindV1::McpAgent);
    let prepared = service
        .prepare_proposal(
            PreferenceMutationRequestV1::ResetUser {
                key: "datum.accessibility.reduced_motion".to_owned(),
                expected: HeadExpectationV1::Missing,
                request_id: Uuid::new_v4(),
                reason: "reset animation".to_owned(),
            },
            "request local human review".to_owned(),
            &mcp_actor,
        )
        .unwrap();
    let mut broker = PreferenceAcceptanceBroker::new(Uuid::new_v4());
    broker
        .register_prepared(&prepared.proposal, "missing:test-repository")
        .unwrap();
    let invocation = Uuid::new_v4();
    broker
        .authorize_mcp_apply(
            &AuthorizeMcpPreferenceApplyV1 {
                proposal_id: prepared.proposal.proposal_id,
                proposal_digest: prepared.proposal.proposal_digest.clone(),
                originating_mcp_session: mcp_actor.session_id.clone(),
            },
            &actor(PreferenceActorKindV1::HumanCli),
            "missing:test-repository",
            invocation,
            10,
        )
        .unwrap();
    assert_eq!(
        broker.authorization_for_apply(
            &prepared.proposal,
            &mcp_actor.session_id,
            "missing:test-repository",
            invocation,
            300_011,
        ),
        Err(PreferenceAcceptanceRefusal::Expired)
    );
    broker.close_session(&mcp_actor.session_id);
    assert_eq!(
        broker.authorization_for_apply(
            &prepared.proposal,
            &mcp_actor.session_id,
            "missing:test-repository",
            invocation,
            20,
        ),
        Err(PreferenceAcceptanceRefusal::Expired)
    );
}
