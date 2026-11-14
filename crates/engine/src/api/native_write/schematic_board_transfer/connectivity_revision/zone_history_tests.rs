//! Complete native zoned repair, history and current-copper lifecycle.
use super::*;
use crate::substrate::Operation;

fn assert_historical_fill_currency(model: &DesignModel, zone: Uuid, raw: &Value) {
    let recorded: crate::substrate::ZoneFill = serde_json::from_value(raw.clone()).unwrap();
    assert_eq!(recorded.state, crate::substrate::ZoneFillState::Filled);
    let same_basis = recorded.model_revision == model.model_revision
        && recorded.source_zone_revision == model.objects[&zone].object_revision;
    assert_eq!(model.current_zone_fill(zone).is_some(), same_basis);
    if !same_basis {
        assert_eq!(
            model.zone_fills[&zone].state,
            crate::substrate::ZoneFillState::Stale
        );
    }
    eprintln!(
        "Historical fill currency: recorded model {:?}/zone {:?}, current model {:?}/zone {:?}, current={same_basis}",
        recorded.model_revision,
        recorded.source_zone_revision,
        model.model_revision,
        model.objects[&zone].object_revision
    );
}

#[test]
#[ignore = "requires explicitly qualified matching DATUM_CONNECTIVITY_DRC_CLI and artifact SHA256"]
fn zoned_revision_discloses_stale_contacts_then_repairs_refills_and_restores_cold_history() {
    let mut circuit = circuit();
    let zone = add_filled_zone(&mut circuit);
    rewire(&mut circuit);
    assert!(circuit.model.current_zone_fill(zone).is_none());
    let review = revision::preview(&circuit.model, circuit.sheet).unwrap();
    assert_eq!(review.zone_fills.len(), 1);
    assert_eq!(review.zone_fills[0].source_id, zone);
    assert!(!review.zone_fills[0].current);
    assert!(review.zone_fills[0].invalidated_by_revision);
    assert!(
        review
            .contact_basis
            .iter()
            .any(|contact| contact.unavailable.is_some())
    );
    assert_eq!(review.retained_copper.len(), 5);
    assert!(
        review
            .retained_copper
            .iter()
            .all(|row| row.impact == revision::CopperImpact::ConservativeUnknownContacts)
    );
    let snapshot = ElectricalSelectionSnapshot::capture(&circuit.model).unwrap();
    assert!(
        snapshot
            .global_net(&circuit.model.model_revision, review.changes[0].logical_net)
            .is_err(),
        "pending correspondence cannot become a complete Global Net"
    );
    for variant in 0..3 {
        let mut forged = review.clone();
        if variant == 0 {
            forged.zone_fills.clear();
        } else if variant == 1 {
            forged.zone_fills[0].current = true;
            forged
                .contact_basis
                .iter_mut()
                .for_each(|contact| contact.unavailable = None);
        } else {
            forged.zone_fills[0].fill_source_hash = Some("unreviewed generated fill".into());
        }
        let before = durable(&circuit);
        assert!(
            revision::prepare(
                &circuit.model,
                WriteProvenance::new("connectivity-test", CommitSource::Manual, revision::REASON),
                &forged,
                Uuid::new_v4()
            )
            .is_err()
        );
        assert_eq!(durable(&circuit), before);
    }
    let before = board(&circuit.model);
    let rejected = draft(&mut circuit);
    act(
        &mut circuit,
        Action::Reject {
            proposal_id: rejected,
        },
    );
    assert_eq!(board(&circuit.model), before);
    let id = draft(&mut circuit);
    let review = revision::preview(&circuit.model, circuit.sheet).unwrap();
    act(&mut circuit, Action::AcceptApply { proposal_id: id });
    let accepted = board(&circuit.model);
    for key in [
        "packages",
        "component_pads",
        "tracks",
        "vias",
        "zones",
        "nets",
    ] {
        assert_eq!(
            accepted[key], before[key],
            "revision altered authored {key}"
        );
    }
    assert!(circuit.model.current_zone_fill(zone).is_none());
    let snapshot = ElectricalSelectionSnapshot::capture(&circuit.model).unwrap();
    let source = snapshot.board_source().unwrap();
    assert!(
        matches!(snapshot.board_net_components(&source,source.zones[&zone].net),
        Err(crate::substrate::ElectricalQueryFailure::Physical(crate::connectivity::PhysicalQueryFailure::UnavailableFill { source_id, .. })) if source_id==zone)
    );
    assert!(
        connectivity_codes(&circuit)
            .iter()
            .any(|code| code == "nominal_connectivity_unavailable")
    );
    // Historical generated copper is not admissible as a new ordinary fill.
    let historical = circuit
        .model
        .journal
        .iter()
        .find_map(|tx| {
            tx.operations.iter().find_map(|op| match op {
                Operation::SetZoneFill {
                    zone_id, zone_fill, ..
                } if *zone_id == zone => Some(zone_fill.clone()),
                _ => None,
            })
        })
        .unwrap();
    let before_refusal = durable(&circuit);
    let write = BatchComposer::compose(&circuit.model, provenance())
        .push_op(Operation::SetZoneFill {
            zone_id: zone,
            previous_zone_fill: Some(historical.clone()),
            zone_fill: historical,
        })
        .finish()
        .unwrap();
    assert!(commit_prepared(&mut circuit.model, &circuit.root, write).is_err());
    assert_eq!(durable(&circuit), before_refusal);
    let proposal = circuit.model.proposals[&id].clone();
    let mut forged = review.clone();
    forged.zone_fills.clear();
    let mut altered = proposal.clone();
    altered.rationale = serde_json::to_string(&forged).unwrap();
    assert!(revision::summary(&circuit.model, &altered, false).is_err());
    let accepted_length = circuit.model.journal.len();
    repair(&mut circuit, zone);
    assert!(
        connectivity_codes(&circuit).is_empty(),
        "ordinary repair/refill must restore nominal connectivity"
    );
    let repaired = board(&circuit.model);
    let fill_shard = circuit
        .model
        .source_shards
        .iter()
        .find(|shard| shard.kind == SourceShardKind::ZoneFill)
        .unwrap()
        .shard_id;
    let repaired_fill_source = circuit
        .model
        .capture_source_shard_value(fill_shard)
        .unwrap();
    let first_restored_fill = circuit
        .model
        .journal
        .last()
        .unwrap()
        .inverse_operations
        .iter()
        .find_map(|operation| match operation {
            Operation::SetZoneFill {
                zone_id, zone_fill, ..
            } if *zone_id == zone => Some(zone_fill.clone()),
            _ => None,
        })
        .unwrap();
    let repair_steps = circuit.model.journal.len() - accepted_length;
    for step in 0..repair_steps {
        circuit
            .model
            .commit_journal_undo(&circuit.root, provenance().into())
            .unwrap();
        if step == 0 {
            let cold = ProjectResolver::new(&circuit.root).resolve().unwrap();
            assert_eq!(
                cold.capture_source_shard_value(fill_shard).unwrap(),
                first_restored_fill
            );
            assert_historical_fill_currency(&cold, zone, &first_restored_fill);
        }
    }
    assert_eq!(board(&circuit.model), accepted);
    for row in &review.relationships {
        assert_eq!(
            circuit.model.electrical_identities.get(&row.record.id),
            Some(&row.record)
        );
    }
    circuit
        .model
        .commit_journal_undo(&circuit.root, provenance().into())
        .unwrap();
    let cold = ProjectResolver::new(&circuit.root).resolve().unwrap();
    for row in &review.relationships {
        assert_eq!(
            cold.electrical_identities.get(&row.previous.id),
            Some(&row.previous)
        );
    }
    circuit
        .model
        .commit_journal_redo(&circuit.root, provenance().into())
        .unwrap();
    for _ in 0..repair_steps {
        circuit
            .model
            .commit_journal_redo(&circuit.root, provenance().into())
            .unwrap();
    }
    let cold = ProjectResolver::new(&circuit.root).resolve().unwrap();
    assert_eq!(board(&cold), repaired);
    assert_eq!(
        cold.capture_source_shard_value(fill_shard).unwrap(),
        repaired_fill_source
    );
    for row in &review.relationships {
        assert_eq!(
            cold.electrical_identities.get(&row.record.id),
            Some(&row.record)
        );
    }
    // Currency follows the recorded basis, which may match again after Redo.
    // History itself neither recertifies evidence nor forces it to be stale.
    assert_historical_fill_currency(&cold, zone, &repaired_fill_source);
    circuit.model = cold;
    refill(&mut circuit, zone);
    circuit.model = ProjectResolver::new(&circuit.root).resolve().unwrap();
    assert!(circuit.model.current_zone_fill(zone).is_some());
    assert!(connectivity_codes(&circuit).is_empty());
}
