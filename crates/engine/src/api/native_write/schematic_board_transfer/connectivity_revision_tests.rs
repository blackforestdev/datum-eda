//! A native three-component, six-terminal circuit exercises reviewed rewiring.
use super::{connectivity_revision as revision, *};
use crate::{
    api::native_write::{
        self as native, BatchComposer, commit_prepared,
        registry::{NativeWriteContext, find_native_write_verb},
        schematic_delete_proposal::{
            self as proposals, DeleteClass, DeleteTarget, NativeProposalAction as Action,
            NativeProposalRequest,
        },
    },
    substrate::{
        CommitSource, DesignModel, ElectricalIdentity, ElectricalSelectionSnapshot,
        ProjectResolver, SourceShardKind,
    },
};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

#[path = "connectivity_revision/test_fixture.rs"]
mod fixture;
use fixture::*;

#[test]
fn generic_proposal_gateway_rejects_forged_connectivity_receipts_without_writes() {
    use crate::substrate::{
        ProposalCreateRequest, ProposalSource, create_draft_proposal_from_batch,
    };
    let mut circuit = circuit();
    rewire(&mut circuit);
    let review = revision::preview(&circuit.model, circuit.sheet).unwrap();
    let mut altered = review.clone();
    altered.retained_copper[0].net = Some(Uuid::nil());
    for rationale in ["{}".into(), serde_json::to_string(&altered).unwrap()] {
        let id = Uuid::new_v4();
        let prepared = revision::prepare(
            &circuit.model,
            WriteProvenance::new("connectivity-test", CommitSource::Manual, revision::REASON),
            &review,
            id,
        )
        .unwrap();
        let before = durable(&circuit);
        assert!(
            create_draft_proposal_from_batch(
                &mut circuit.model,
                &circuit.root,
                ProposalCreateRequest {
                    proposal_id: Some(id),
                    batch: prepared.batch,
                    rationale,
                    source: ProposalSource::Manual,
                    checks_run: vec![],
                    finding_fingerprints: vec![],
                }
            )
            .is_err()
        );
        assert_eq!(durable(&circuit), before);
    }
}

#[test]
fn generic_review_and_apply_reject_altered_real_recorded_receipts() {
    use crate::substrate::{
        ProposalStatus, apply_accepted_proposal, preview_proposal_diff,
        preview_proposal_diff_journaled, review_proposal_status,
    };
    let mut circuit = circuit();
    rewire(&mut circuit);
    let id = draft(&mut circuit);
    let before = durable(&circuit);
    let mut corrupted = circuit.model.clone();
    let record = corrupted.proposals.get_mut(&id).unwrap();
    let mut receipt: revision::BoardConnectivityRevision =
        serde_json::from_str(&record.rationale).unwrap();
    receipt.retained_copper[0].net = Some(Uuid::nil());
    record.rationale = serde_json::to_string(&receipt).unwrap();
    assert!(preview_proposal_diff(&corrupted, id).is_err());
    assert!(preview_proposal_diff_journaled(&corrupted, &circuit.root, id).is_err());
    assert!(
        review_proposal_status(&mut corrupted, &circuit.root, id, ProposalStatus::Accepted)
            .is_err()
    );
    assert_eq!(durable(&circuit), before);
    review_proposal_status(
        &mut circuit.model,
        &circuit.root,
        id,
        ProposalStatus::Accepted,
    )
    .unwrap();
    let before = durable(&circuit);
    let mut corrupted = circuit.model.clone();
    corrupted.proposals.get_mut(&id).unwrap().rationale = "{}".into();
    assert!(preview_proposal_diff(&corrupted, id).is_err());
    assert!(preview_proposal_diff_journaled(&corrupted, &circuit.root, id).is_err());
    assert!(apply_accepted_proposal(&mut corrupted, &circuit.root, id).is_err());
    assert_eq!(durable(&circuit), before);
}

#[test]
fn ambiguous_counterpart_ownership_cannot_prepare_connectivity_changes() {
    use crate::substrate::Operation;
    let mut circuit = circuit();
    rewire(&mut circuit);
    let id = *circuit.model.component_instances.keys().next().unwrap();
    let source = &circuit.model.objects[&id];
    let shard = circuit
        .model
        .source_shards
        .iter()
        .find(|shard| shard.shard_id == source.source_shard_id)
        .unwrap();
    let mut raw = circuit
        .model
        .materialized_source_shard_value_by_relative_path(&shard.relative_path)
        .unwrap()["component_instance"]
        .clone();
    let duplicate = Uuid::new_v4();
    raw["uuid"] = json!(duplicate);
    raw["object_revision"] = json!(0);
    let prepared = BatchComposer::compose(&circuit.model, provenance())
        .push_op(Operation::CreateComponentInstance {
            component_instance_id: duplicate,
            component_instance: raw,
        })
        .finish()
        .unwrap();
    let before = durable(&circuit);
    match commit_prepared(&mut circuit.model, &circuit.root, prepared) {
        Err(_) => assert_eq!(
            durable(&circuit),
            before,
            "invalid ownership refusal must not write"
        ),
        Ok(_) => {
            let before = durable(&circuit);
            assert!(revision::preview(&circuit.model, circuit.sheet).is_err());
            assert_eq!(durable(&circuit), before);
        }
    }
}

#[test]
fn captured_components_refuse_mixed_extra_and_mismatched_current_physical_roles() {
    use crate::substrate::LibraryBindingRole;
    let mut circuit = circuit();
    let (map, footprint, source_footprint) = compatible_physical_alternatives(&mut circuit);
    rewire(&mut circuit);
    revision::preview(&circuit.model, circuit.sheet).unwrap();
    let id = *circuit.model.component_instances.keys().next().unwrap();
    let before = durable(&circuit);
    for (role, target) in [
        (LibraryBindingRole::PinPadMap, map),
        (LibraryBindingRole::Footprint, footprint),
    ] {
        for extra in [false, true] {
            let mut captured = circuit.model.clone();
            let pinned_revision = captured.objects[&target].object_revision;
            let source_footprint_revision = captured.objects[&source_footprint].object_revision;
            let component = captured.component_instances.get_mut(&id).unwrap();
            let key = component
                .library_bindings
                .iter()
                .find(|(_, binding)| binding.binding_role == role)
                .map(|(id, _)| *id)
                .unwrap_or_else(|| {
                    // Optional-role negatives only; never inject a declaration
                    // into the positive ordinary placement/transfer fixture.
                    let mut binding = component
                        .library_bindings
                        .values()
                        .find(|binding| binding.binding_role == LibraryBindingRole::PinPadMap)
                        .unwrap()
                        .clone();
                    binding.binding_role = LibraryBindingRole::Footprint;
                    binding.target_object_id = source_footprint;
                    binding.pinned_object_revision = source_footprint_revision;
                    let key = Uuid::new_v4();
                    component.library_bindings.insert(key, binding);
                    key
                });
            let mut altered = component.library_bindings[&key].clone();
            altered.target_object_id = target;
            altered.pinned_object_revision = pinned_revision;
            component
                .library_bindings
                .insert(if extra { Uuid::new_v4() } else { key }, altered);
            assert!(revision::preview(&captured, circuit.sheet).is_err());
        }
    }
    // Real source definitions and journal remain unchanged by the malformed
    // captured declarations; they never qualify an accepted design mutation.
    assert_eq!(durable(&circuit), before);
}

#[test]
fn stored_draft_and_accepted_preview_are_read_only_and_refuse_stale_source() {
    use crate::substrate::{
        ProposalStatus, preview_proposal_diff, preview_proposal_diff_journaled,
        review_proposal_status,
    };
    for accepted in [false, true] {
        let mut circuit = circuit();
        rewire(&mut circuit);
        let id = draft(&mut circuit);
        if accepted {
            review_proposal_status(
                &mut circuit.model,
                &circuit.root,
                id,
                ProposalStatus::Accepted,
            )
            .unwrap();
        }
        let before = durable(&circuit);
        let memory = preview_proposal_diff(&circuit.model, id).unwrap();
        let journaled = preview_proposal_diff_journaled(&circuit.model, &circuit.root, id).unwrap();
        assert_eq!(memory.diff, journaled.diff);
        assert_eq!(durable(&circuit), before);
        let response = act(&mut circuit, Action::Preview { proposal_id: id });
        assert_eq!(
            response.status,
            if accepted {
                ProposalStatus::Accepted
            } else {
                ProposalStatus::Draft
            }
        );
        assert_eq!(durable(&circuit), before);
        let sheet = circuit.sheet;
        write(
            &mut circuit.model,
            &circuit.root,
            "datum.schematic.draw_wire",
            json!({"sheet":sheet,"instance_path":[],
            "points":[{"x":22000000,"y":6000000},{"x":24000000,"y":6000000}]}),
        );
        let before = durable(&circuit);
        assert!(preview_proposal_diff(&circuit.model, id).is_err());
        assert!(preview_proposal_diff_journaled(&circuit.model, &circuit.root, id).is_err());
        assert_eq!(durable(&circuit), before);
        for action in [
            Action::Preview { proposal_id: id },
            Action::AcceptApply { proposal_id: id },
        ] {
            let before = durable(&circuit);
            let request = request(&circuit, action);
            assert!(proposals::execute(&mut circuit.model, &request).is_err());
            assert_eq!(durable(&circuit), before);
        }
    }
}

#[test]
fn real_applied_receipt_requires_full_copper_disclosure_and_actual_transaction() {
    let mut circuit = circuit();
    rewire(&mut circuit);
    let id = draft(&mut circuit);
    act(&mut circuit, Action::AcceptApply { proposal_id: id });
    let proposal = circuit.model.proposals[&id].clone();
    let review = revision::summary(&circuit.model, &proposal, false).unwrap();
    let mut without_copper = review.clone();
    without_copper.retained_copper.pop();
    let mut wrong_kind = review.clone();
    wrong_kind.retained_copper[0].kind = revision::RetainedCopperKind::Zone;
    let mut wrong_net = review.clone();
    wrong_net.retained_copper[0].net = Some(Uuid::nil());
    let mut without_contact = review.clone();
    without_contact.contact_basis.clear();
    let before = durable(&circuit);
    for forged in [without_copper, wrong_kind, wrong_net, without_contact] {
        let mut altered = proposal.clone();
        altered.rationale = serde_json::to_string(&forged).unwrap();
        assert!(revision::summary(&circuit.model, &altered, false).is_err());
    }
    for transaction in [None, Some(Uuid::new_v4())] {
        let mut altered = proposal.clone();
        altered.applied_transaction_id = transaction;
        assert!(revision::summary(&circuit.model, &altered, false).is_err());
    }
    assert_eq!(durable(&circuit), before);
}

#[test]
fn connectivity_revision_preserves_prior_field_history_and_rotated_physical_choice() {
    let mut circuit = circuit();
    let snapshot = ElectricalSelectionSnapshot::capture(&circuit.model).unwrap();
    let symbol = snapshot.schematic_source().unwrap().sheets[&circuit.sheet]
        .symbols
        .values()
        .next()
        .unwrap();
    let edit = proposals::FieldRowEdit {
        symbol: symbol.uuid,
        reference: "R99".into(),
        value: symbol.value.clone(),
    };
    let id = Uuid::new_v4();
    let sheet = circuit.sheet;
    act(
        &mut circuit,
        Action::CreateFieldBatch {
            proposal_id: id,
            sheet,
            instance_path: vec![],
            edits: vec![edit],
        },
    );
    act(&mut circuit, Action::AcceptApply { proposal_id: id });
    let review = super::field_revision::preview(&circuit.model, sheet, &[]).unwrap();
    let id = Uuid::new_v4();
    act(
        &mut circuit,
        Action::CreateBoardFieldRevision {
            proposal_id: id,
            sheet,
            review: Some(review),
        },
    );
    act(&mut circuit, Action::AcceptApply { proposal_id: id });
    let package = circuit
        .model
        .component_instances
        .values()
        .next()
        .unwrap()
        .placed_package_refs[0];
    let prepared = native::board_components::build_edit_board_package(
        &circuit.model,
        provenance(),
        package,
        native::board_components::BoardPackageEdit::Rotation { rotation: 90 },
    )
    .unwrap();
    commit_prepared(&mut circuit.model, &circuit.root, prepared).unwrap();
    rewire(&mut circuit);
    let before = board(&circuit.model);
    let review = revision::preview(&circuit.model, circuit.sheet).unwrap();
    let id = draft(&mut circuit);
    act(&mut circuit, Action::AcceptApply { proposal_id: id });
    let cold = ProjectResolver::new(&circuit.root).resolve().unwrap();
    let after = board(&cold);
    for key in ["packages", "component_pads", "tracks", "vias", "nets"] {
        assert_eq!(after[key], before[key]);
    }
    assert_eq!(
        after["packages"][package.to_string()]["rotation"],
        json!(90)
    );
    let projection =
        crate::substrate::component_pad_source::component_pad_projection(&cold, &after).unwrap();
    assert!(projection.statuses.values().all(|status| *status
        == crate::substrate::component_pad_source::ComponentPadMaterializationStatus::Current));
    assert_eq!(
        projection.pads[&review.changes[0].pad].net,
        Some(review.changes[0].board_net)
    );
}

#[test]
fn new_board_correspondence_and_changed_library_sources_refuse_without_writes() {
    let mut circuit = circuit();
    rewire_to(&mut circuit, 4000000);
    let before = durable(&circuit);
    assert!(revision::preview(&circuit.model, circuit.sheet).is_err());
    assert_eq!(durable(&circuit), before);
    let mut circuit = self::circuit();
    rewire(&mut circuit);
    use native::library::{PoolLibraryObjectTarget as Target, PoolLibraryOperationSpec as Spec};
    let id = circuit
        .model
        .objects
        .values()
        .find(|object| object.domain == "pool" && object.kind == "units")
        .unwrap()
        .object_id;
    let mut raw =
        native::schematic_library::materialized_pool_object(&circuit.model, id, "units").unwrap();
    raw["name"] = json!("Changed logical library");
    let prepared = native::library::build_pool_library_write(
        &circuit.model,
        provenance(),
        Some("pool"),
        vec![Spec::Set {
            target: Target::new("pool", "units", id),
            object: raw,
        }],
    )
    .unwrap();
    commit_prepared(&mut circuit.model, &circuit.root, prepared).unwrap();
    let before = durable(&circuit);
    assert!(revision::preview(&circuit.model, circuit.sheet).is_err());
    assert_eq!(durable(&circuit), before);
}

#[path = "connectivity_revision/zone_history_tests.rs"]
mod zone_history_tests;

#[test]
fn unfilled_zone_has_source_bound_unknowns_through_draft_and_accepted_preview() {
    use crate::substrate::{
        ProposalStatus, preview_proposal_diff, preview_proposal_diff_journaled,
        review_proposal_status,
    };
    let mut circuit = circuit();
    let zone = add_zone(&mut circuit);
    rewire(&mut circuit);
    let review = revision::preview(&circuit.model, circuit.sheet).unwrap();
    assert_eq!(review.zone_fills.len(), 1);
    assert_eq!(review.zone_fills[0].source_id, zone);
    assert!(!review.zone_fills[0].current);
    assert!(review.zone_fills[0].fill_source_shard.is_none());
    assert!(review.zone_fills[0].fill_source_hash.is_none());
    let id = draft(&mut circuit);
    for accepted in [false, true] {
        if accepted {
            review_proposal_status(
                &mut circuit.model,
                &circuit.root,
                id,
                ProposalStatus::Accepted,
            )
            .unwrap();
        }
        let before = durable(&circuit);
        preview_proposal_diff(&circuit.model, id).unwrap();
        preview_proposal_diff_journaled(&circuit.model, &circuit.root, id).unwrap();
        let response = act(&mut circuit, Action::Preview { proposal_id: id });
        assert_eq!(response.board_connectivity_revision, Some(review.clone()));
        assert_eq!(durable(&circuit), before);
    }
    act(&mut circuit, Action::AcceptApply { proposal_id: id });
    assert!(circuit.model.current_zone_fill(zone).is_none());
}

#[test]
fn existing_net_revision_reject_apply_and_cold_preserve_physical_sources() {
    let mut circuit = circuit();
    assert_eq!(circuit.model.component_instances.len(), 3);
    let original = board(&circuit.model);
    rewire(&mut circuit);
    let before = board(&circuit.model);
    for key in [
        "packages",
        "component_pads",
        "tracks",
        "vias",
        "zones",
        "nets",
    ] {
        assert_eq!(
            before[key], original[key],
            "source changed during schematic edit: {key}"
        );
    }
    let review = revision::preview(&circuit.model, circuit.sheet).unwrap();
    assert_eq!(review.changes.len(), 1);
    assert_eq!(review.relationships.len(), 2);
    assert_eq!(review.retained_copper.len(), 4);
    let id = Uuid::new_v4();
    let draft = act(
        &mut circuit,
        Action::CreateBoardConnectivityRevision {
            proposal_id: id,
            sheet: review.sheet,
            review: review.clone(),
        },
    );
    assert_eq!(draft.board_connectivity_revision, Some(review));
    act(&mut circuit, Action::Reject { proposal_id: id });
    assert_eq!(board(&circuit.model), before);
    let review = revision::preview(&circuit.model, circuit.sheet).unwrap();
    let id = Uuid::new_v4();
    act(
        &mut circuit,
        Action::CreateBoardConnectivityRevision {
            proposal_id: id,
            sheet: review.sheet,
            review: review.clone(),
        },
    );
    act(&mut circuit, Action::AcceptApply { proposal_id: id });
    let after = board(&circuit.model);
    let snapshot = ElectricalSelectionSnapshot::capture(&circuit.model).unwrap();
    let source = snapshot.board_source().unwrap();
    assert!(
        snapshot
            .board_net_components(&source, review.changes[0].board_net)
            .unwrap()
            .contacts
            .unwrap()
            .iter()
            .any(|contact| contact.left_net != contact.right_net)
    );
    for key in [
        "packages",
        "component_pads",
        "tracks",
        "vias",
        "zones",
        "nets",
    ] {
        assert_eq!(
            after[key], before[key],
            "connectivity acceptance changed authored {key}"
        );
    }
    let cold = ProjectResolver::new(&circuit.root).resolve().unwrap();
    for row in &review.relationships {
        assert_eq!(
            cold.electrical_identities.get(&row.record.id),
            Some(&row.record)
        );
    }
    circuit
        .model
        .commit_journal_undo(&circuit.root, provenance().into())
        .unwrap();
    for row in &review.relationships {
        assert_eq!(
            circuit.model.electrical_identities.get(&row.previous.id),
            Some(&row.previous)
        );
    }
    circuit
        .model
        .commit_journal_redo(&circuit.root, provenance().into())
        .unwrap();
    let cold = ProjectResolver::new(&circuit.root).resolve().unwrap();
    assert_eq!(board(&cold), after);
}
