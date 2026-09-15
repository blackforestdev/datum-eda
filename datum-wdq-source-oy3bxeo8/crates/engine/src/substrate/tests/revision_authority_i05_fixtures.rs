use uuid::Uuid;

use crate::revision::AuthorityRecordKind;

pub(super) fn rev_i05_fixture_semantics(kind: AuthorityRecordKind) -> Option<serde_json::Value> {
    Some(match kind {
        AuthorityRecordKind::ConfigurationBaseline => serde_json::json!({
            "baseline_type": "fixture",
            "scope": [{"kind": "configuration_item", "id": family_id(AuthorityRecordKind::ConfigurationItem)}],
            "members": [{
                "stable_authority_ref": {"kind": "configuration_item", "id": family_id(AuthorityRecordKind::ConfigurationItem)},
                "exact_technical_revision": "0",
                "semantic_digest": format!("sha256:{}", "4".repeat(64)),
                "role": "design",
                "inclusion_reason": "closed family fixture"
            }],
            "source_model_revision": "fixture-model",
            "accepted_transaction_tip": "fixture-tip",
            "governing_changes": [],
            "departures": [],
            "profile_refs": [],
            "establishment_attestations": [],
            "established_by": {"kind": "configuration_item", "id": family_id(AuthorityRecordKind::ConfigurationItem)},
            "established_at": 1,
            "predecessor_baselines": []
        }),
        AuthorityRecordKind::DependencySnapshot => serde_json::json!({
            "configuration": {"kind": "working", "model_revision": "fixture-model", "accepted_transaction_tip": "fixture-tip"},
            "nodes": [{
                "authority_ref": {"kind": "configuration_item", "id": family_id(AuthorityRecordKind::ConfigurationItem)},
                "exact_technical_revision": "0",
                "semantic_digest": format!("sha256:{}", "5".repeat(64)),
                "node_kind": "design"
            }],
            "edges": [],
            "evaluator_registry_revision": "fixture-evaluator-v1",
            "graph_complete": false,
            "unresolved_inputs": ["fixture intentionally incomplete"]
        }),
        AuthorityRecordKind::SemanticDelta => serde_json::json!({
            "from_configuration": {"kind": "working", "model_revision": "before", "accepted_transaction_tip": "tip-0"},
            "to_configuration": {"kind": "working", "model_revision": "after", "accepted_transaction_tip": "tip-1"},
            "subject": {"kind": "configuration_item", "id": family_id(AuthorityRecordKind::ConfigurationItem)},
            "changed_observations": ["geometry"],
            "administrative_observations": [],
            "evaluator_id": "fixture-evaluator",
            "evaluator_revision": "1"
        }),
        AuthorityRecordKind::ImpactEvaluation => serde_json::json!({
            "governing_change": family_id(AuthorityRecordKind::EngineeringChange),
            "from_configuration": {"kind": "working", "model_revision": "before", "accepted_transaction_tip": "tip-0"},
            "to_configuration": {"kind": "working", "model_revision": "after", "accepted_transaction_tip": "tip-1"},
            "dependency_snapshot": family_id(AuthorityRecordKind::DependencySnapshot),
            "evaluator_registry_revision": "fixture-evaluator-v1",
            "subjects": [{
                "subject": {"kind": "configuration_item", "id": family_id(AuthorityRecordKind::ConfigurationItem)},
                "result": "impact_unknown",
                "changed_observations": [],
                "witness_paths": [],
                "reason_code": "graph_incomplete",
                "required_actions": ["resolve_impact_unknown"]
            }],
            "graph_complete": false
        }),
        AuthorityRecordKind::EvidenceInputContext => serde_json::json!({
            "configuration": {"kind": "working", "model_revision": "fixture-model", "accepted_transaction_tip": "fixture-tip"},
            "dependency_snapshot": family_id(AuthorityRecordKind::DependencySnapshot),
            "inputs": [{
                "input_ref": {"kind": "configuration_item", "id": family_id(AuthorityRecordKind::ConfigurationItem)},
                "digest": format!("sha256:{}", "6".repeat(64))
            }],
            "policy_refs": [],
            "producer_ref": {"kind": "build_identity", "id": family_id(AuthorityRecordKind::BuildIdentity)},
            "producer_revision": "1",
            "invocation_digest": format!("sha256:{}", "7".repeat(64))
        }),
        AuthorityRecordKind::EvidenceFreshness => serde_json::json!({
            "input_context": family_id(AuthorityRecordKind::EvidenceInputContext),
            "target_configuration": {"kind": "working", "model_revision": "fixture-model", "accepted_transaction_tip": "fixture-tip"},
            "current": true,
            "differing_inputs": [],
            "unresolved_inputs": [],
            "unsupported_inputs": [],
            "reasons": []
        }),
        AuthorityRecordKind::LibraryUptakeCandidate => serde_json::json!({
            "component_instance_id": Uuid::from_u128(1000),
            "binding_id": Uuid::from_u128(1001),
            "pinned_library_ref": {"object_id": Uuid::from_u128(1002), "object_revision": 1},
            "proposed_library_ref": {"object_id": Uuid::from_u128(1002), "object_revision": 2},
            "pinned_source_resolves": true,
            "proposed_source_resolves": true,
            "semantic_delta": family_id(AuthorityRecordKind::SemanticDelta),
            "predicted_impact": family_id(AuthorityRecordKind::ImpactEvaluation),
            "required_checks": [],
            "required_regeneration": []
        }),
        AuthorityRecordKind::BaselineComparison => serde_json::json!({
            "left_baseline": family_id(AuthorityRecordKind::ConfigurationBaseline),
            "right_baseline": family_id(AuthorityRecordKind::ConfigurationBaseline),
            "aligned_members": [],
            "dependency_delta": [],
            "evidence_delta": [],
            "governing_change_coverage": [],
            "unresolved_differences": []
        }),
        AuthorityRecordKind::RegenerationPlan => serde_json::json!({
            "target_configuration": {"kind": "working", "model_revision": "fixture-model", "accepted_transaction_tip": "fixture-tip"},
            "basis_impact_evaluation": family_id(AuthorityRecordKind::ImpactEvaluation),
            "steps": [],
            "unchanged_evidence_reused": [],
            "blockers": ["fixture intentionally empty"]
        }),
        _ => return None,
    })
}

fn family_id(kind: AuthorityRecordKind) -> Uuid {
    let index = AuthorityRecordKind::ALL
        .iter()
        .position(|candidate| *candidate == kind)
        .expect("family is registered");
    Uuid::from_u128(index as u128 + 1)
}
