//! Real generic Proposal creation/preview/apply and pre-apply tamper controls.
use super::{
    commit_prepared,
    context::{BatchComposer, WriteProvenance},
    design_clipboard::{
        ClipboardClass, ClipboardPlacement, ClipboardTarget, DesignClipboardBundle,
    },
    design_clipboard_edit::{ClipboardContactPolicy, DesignEditIntent},
    design_clipboard_receipt,
    electrical::topology_tests,
    registry::NativeWriteContext,
    schematic_delete_proposal::{NativeProposalAction, NativeProposalRequest, execute},
    schematic_edit::prepare_native_birth,
    schematic_symbol_payload::source_sheet_payload,
};
use crate::{
    ir::geometry::Point,
    substrate::{CommitSource, DesignModel, Operation, ProjectResolver, SchematicMarkerKind},
};
use serde_json::json;
use std::path::{Path, PathBuf};
use uuid::Uuid;
fn provenance() -> WriteProvenance {
    WriteProvenance::new("test", CommitSource::Test, "clipboard proposal control")
}
fn fixture() -> (PathBuf, DesignModel, Uuid, Uuid, Uuid, Uuid) {
    let (root, mut model, _, sheet) = topology_tests::fixture("clipboard_receipt");
    let wire = Uuid::new_v4();
    let marker = Uuid::new_v4();
    let sentinel = Uuid::new_v4();
    let prepared=BatchComposer::compose(&model,provenance()).push_ops(vec![
        Operation::CreateSchematicWire{sheet_id:sheet,wire_id:wire,wire:json!({"uuid":wire,"from":{"x":0,"y":0,"future":"keep"},"to":{"x":1000,"y":0},"vendor":"${W}"})},
        Operation::CreateSchematicWire{sheet_id:sheet,wire_id:sentinel,wire:json!({"uuid":sentinel,"from":{"x":10000,"y":0},"to":{"x":11000,"y":0}})},
        Operation::PlaceSchematicMarker{sheet_id:sheet,marker_id:marker,marker_kind:SchematicMarkerKind::Junction,marker:json!({"uuid":marker,"position":{"x":0,"y":0,"future":"keep"},"vendor":"${J}"})},
    ]).finish().unwrap();
    let prepared = prepare_native_birth(&model, prepared).unwrap();
    commit_prepared(&mut model, &root, prepared).unwrap();
    model = ProjectResolver::new(&root).resolve().unwrap();
    (root, model, sheet, wire, marker, sentinel)
}
fn act(
    model: &mut DesignModel,
    root: &Path,
    action: NativeProposalAction,
) -> Result<super::schematic_delete_proposal::NativeProposalResponse, crate::error::EngineError> {
    let request = NativeProposalRequest {
        project_root: root.into(),
        expected_model_revision: model.model_revision.0.clone(),
        expected_transaction_tip: crate::revision::transaction_tip(&model.journal),
        action,
    };
    execute(model, &request)
}
fn paste(model: &DesignModel, sheet: Uuid, wire: Uuid) -> DesignEditIntent {
    let bundle = DesignClipboardBundle::capture(
        model,
        sheet,
        &[],
        &[ClipboardTarget {
            class: ClipboardClass::Wire,
            id: wire,
        }],
        Point::zero(),
    )
    .unwrap();
    DesignEditIntent::PlaceBundle {
        bundle,
        placement: ClipboardPlacement {
            position: Point::new(2_731, 5_019),
            rotation_deg: 0,
            mirrored: false,
        },
        edit_seed: Uuid::new_v4(),
        contact_policy: ClipboardContactPolicy::DrawWireEndpoints,
    }
}

#[test]
fn design_edit_proposal_paste_uses_cached_physical_and_semantic_ids_across_preview_apply_cold_history()
 {
    let (root, mut model, sheet, wire, _, sentinel) = fixture();
    let before = source_sheet_payload(&model, sheet, sentinel).unwrap();
    let identities = model.electrical_identities.clone();
    let proposal_id = Uuid::new_v4();
    let edit = paste(&model, sheet, wire);
    act(
        &mut model,
        &root,
        NativeProposalAction::CreateDesignEdit {
            proposal_id,
            sheet,
            instance_path: vec![],
            edit,
        },
    )
    .unwrap();
    model = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(
        source_sheet_payload(&model, sheet, sentinel).unwrap(),
        before,
        "draft has no source writes"
    );
    let cached = model.proposals[&proposal_id].batch.clone();
    design_clipboard_receipt::validate_recorded(&model.proposals[&proposal_id]).unwrap();
    design_clipboard_receipt::validate_current(
        &NativeWriteContext {
            model: &model,
            project_root: &root,
        },
        &model.proposals[&proposal_id],
    )
    .unwrap();
    act(
        &mut model,
        &root,
        NativeProposalAction::Preview { proposal_id },
    )
    .unwrap();
    assert_eq!(
        model.proposals[&proposal_id].batch, cached,
        "preview never regenerates physical or semantic identities"
    );
    act(
        &mut model,
        &root,
        NativeProposalAction::AcceptApply { proposal_id },
    )
    .unwrap();
    model = ProjectResolver::new(&root).resolve().unwrap();
    let after = source_sheet_payload(&model, sheet, sentinel).unwrap();
    assert_eq!(after["wires"].as_object().unwrap().len(), 3);
    let after_identities = model.electrical_identities.clone();
    for redo in [false, true] {
        if redo {
            model
                .commit_journal_redo(&root, provenance().into())
                .unwrap();
        } else {
            model
                .commit_journal_undo(&root, provenance().into())
                .unwrap();
        }
        model = ProjectResolver::new(&root).resolve().unwrap();
        assert_eq!(
            source_sheet_payload(&model, sheet, sentinel).unwrap(),
            if redo { after.clone() } else { before.clone() }
        );
        assert_eq!(
            model.electrical_identities,
            if redo {
                after_identities.clone()
            } else {
                identities.clone()
            }
        );
    }
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn design_edit_receipt_rejects_changed_extra_and_identity_operations_before_apply() {
    let (root, mut model, sheet, wire, _, sentinel) = fixture();
    let proposal_id = Uuid::new_v4();
    let edit = paste(&model, sheet, wire);
    act(
        &mut model,
        &root,
        NativeProposalAction::CreateDesignEdit {
            proposal_id,
            sheet,
            instance_path: vec![],
            edit,
        },
    )
    .unwrap();
    let valid = model.proposals[&proposal_id].clone();
    let before = source_sheet_payload(&model, sheet, sentinel).unwrap();
    let count = model.journal.len();
    for mode in 0..3 {
        let mut tampered = valid.clone();
        match mode {
            0 => {
                let op = tampered
                    .batch
                    .operations
                    .iter_mut()
                    .find(|op| matches!(op, Operation::CreateSchematicWire { .. }))
                    .unwrap();
                if let Operation::CreateSchematicWire { wire, .. } = op {
                    wire["to"]["x"] = json!(99999);
                }
            }
            1 => {
                let op = tampered
                    .batch
                    .operations
                    .iter()
                    .find(|op| matches!(op, Operation::CreateSchematicWire { .. }))
                    .unwrap()
                    .clone();
                tampered.batch.operations.push(op);
            }
            _ => {
                assert!(
                    tampered
                        .batch
                        .operations
                        .iter()
                        .any(|op| matches!(op, Operation::CreateElectricalIdentity { .. })),
                    "recorded final birth allocation exists"
                );
                tampered
                    .batch
                    .operations
                    .retain(|op| !matches!(op, Operation::CreateElectricalIdentity { .. }));
            }
        }
        assert!(design_clipboard_receipt::validate_recorded(&tampered).is_err());
        // In-memory hostile metadata input is fed to the real gateway; no test
        // writes or private source mutation bypass the canonical design path.
        model.proposals.insert(proposal_id, tampered);
        assert!(
            act(
                &mut model,
                &root,
                NativeProposalAction::AcceptApply { proposal_id }
            )
            .is_err()
        );
        assert_eq!(model.journal.len(), count);
        assert_eq!(
            source_sheet_payload(&model, sheet, sentinel).unwrap(),
            before
        );
    }
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn design_edit_cut_receipt_checks_current_raw_source_even_if_receipt_and_batch_are_tampered_together()
 {
    let (root, mut model, sheet, wire, marker, sentinel) = fixture();
    let proposal_id = Uuid::new_v4();
    let edit = DesignEditIntent::Cut {
        targets: vec![
            ClipboardTarget {
                class: ClipboardClass::Wire,
                id: wire,
            },
            ClipboardTarget {
                class: ClipboardClass::Junction,
                id: marker,
            },
        ],
    };
    let before = source_sheet_payload(&model, sheet, sentinel).unwrap();
    act(
        &mut model,
        &root,
        NativeProposalAction::CreateDesignEdit {
            proposal_id,
            sheet,
            instance_path: vec![],
            edit,
        },
    )
    .unwrap();
    let valid = model.proposals[&proposal_id].clone();
    let mut tampered = valid.clone();
    let mut receipt: serde_json::Value = serde_json::from_str(&tampered.rationale).unwrap();
    let captured = receipt["cut_source"]["items"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|item| item["id"] == json!(wire))
        .unwrap();
    captured["raw"]["vendor"] = json!("forged");
    tampered.rationale = serde_json::to_string(&receipt).unwrap();
    for op in &mut tampered.batch.operations {
        if let Operation::DeleteSchematicWire {
            wire: payload,
            wire_id,
            ..
        } = op
        {
            if *wire_id == wire {
                payload["vendor"] = json!("forged");
            }
        }
    }
    design_clipboard_receipt::validate_recorded(&tampered).unwrap();
    assert!(
        design_clipboard_receipt::validate_current(
            &NativeWriteContext {
                model: &model,
                project_root: &root
            },
            &tampered
        )
        .is_err()
    );
    let count = model.journal.len();
    model.proposals.insert(proposal_id, tampered);
    assert!(
        act(
            &mut model,
            &root,
            NativeProposalAction::AcceptApply { proposal_id }
        )
        .is_err()
    );
    assert_eq!(model.journal.len(), count);
    assert_eq!(
        source_sheet_payload(&model, sheet, sentinel).unwrap(),
        before
    );
    model.proposals.insert(proposal_id, valid);
    act(
        &mut model,
        &root,
        NativeProposalAction::AcceptApply { proposal_id },
    )
    .unwrap();
    model = ProjectResolver::new(&root).resolve().unwrap();
    assert!(
        source_sheet_payload(&model, sheet, sentinel).unwrap()["wires"]
            .get(wire.to_string())
            .is_none()
    );
    model
        .commit_journal_undo(&root, provenance().into())
        .unwrap();
    model = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(
        source_sheet_payload(&model, sheet, sentinel).unwrap(),
        before
    );
    std::fs::remove_dir_all(root).unwrap();
}

fn forged_draft(
    model: &mut DesignModel,
    root: &Path,
    id: Uuid,
    mut prepared: super::context::PreparedWrite,
    rationale: String,
) {
    prepared.batch.expected_model_revision = Some(model.model_revision.clone());
    crate::substrate::create_draft_proposal_from_batch(
        model,
        root,
        crate::substrate::ProposalCreateRequest {
            proposal_id: Some(id),
            batch: prepared.batch,
            rationale,
            source: crate::substrate::ProposalSource::Manual,
            checks_run: vec![],
            finding_fingerprints: vec![],
        },
    )
    .unwrap();
}

#[test]
fn design_edit_generic_draft_cannot_hide_raw_or_historical_foreign_identity() {
    for historical in [false, true] {
        let (root, mut model, sheet, wire, _, sentinel) = fixture();
        let id = Uuid::new_v4();
        let edit = paste(&model, sheet, wire);
        let (prepared, rationale) = design_clipboard_receipt::prepare(
            &NativeWriteContext {
                model: &model,
                project_root: &root,
            },
            provenance(),
            id,
            sheet,
            &[],
            &edit,
        )
        .unwrap();
        let created = prepared
            .batch
            .operations
            .iter()
            .find_map(|op| match op {
                Operation::CreateSchematicWire { wire_id, .. } => Some(*wire_id),
                _ => None,
            })
            .unwrap();
        let before = source_sheet_payload(&model, sheet, sentinel).unwrap();
        let old = before["wires"][sentinel.to_string()].clone();
        let mut raw = old.clone();
        raw["opaque_foreign_reference"] = json!(created);
        // Canonical raw-only edit leaves typed geometry unchanged. Undo leaves
        // the reference exclusively in actual physical operation history.
        let write = BatchComposer::compose(&model, provenance())
            .push_ops(vec![
                Operation::DeleteSchematicWire {
                    sheet_id: sheet,
                    wire_id: sentinel,
                    wire: old,
                },
                Operation::CreateSchematicWire {
                    sheet_id: sheet,
                    wire_id: sentinel,
                    wire: raw,
                },
            ])
            .finish()
            .unwrap();
        let write = prepare_native_birth(&model, write).unwrap();
        commit_prepared(&mut model, &root, write).unwrap();
        if historical {
            model
                .commit_journal_undo(&root, provenance().into())
                .unwrap();
        }
        model = ProjectResolver::new(&root).resolve().unwrap();
        if historical {
            assert_eq!(
                source_sheet_payload(&model, sheet, sentinel).unwrap(),
                before
            );
        }
        forged_draft(&mut model, &root, id, prepared, rationale);
        model = ProjectResolver::new(&root).resolve().unwrap();
        design_clipboard_receipt::validate_recorded(&model.proposals[&id]).unwrap();
        let error = design_clipboard_receipt::validate_current(
            &NativeWriteContext {
                model: &model,
                project_root: &root,
            },
            &model.proposals[&id],
        )
        .unwrap_err();
        assert!(
            error.to_string().contains("identity already exists"),
            "{error}"
        );
        let source = source_sheet_payload(&model, sheet, sentinel).unwrap();
        let journal = model.journal.clone();
        assert!(
            act(
                &mut model,
                &root,
                NativeProposalAction::AcceptApply { proposal_id: id }
            )
            .is_err()
        );
        assert_eq!(
            model.journal, journal,
            "rejected forged paste has no journal write"
        );
        assert_eq!(
            source_sheet_payload(&model, sheet, sentinel).unwrap(),
            source
        );
        std::fs::remove_dir_all(root).unwrap();
    }
}

#[test]
fn design_edit_own_metadata_exclusion_requires_exact_body_and_preserves_other_proposals() {
    let (root, mut model, sheet, wire, _, _) = fixture();
    let id = Uuid::new_v4();
    let edit = paste(&model, sheet, wire);
    act(
        &mut model,
        &root,
        NativeProposalAction::CreateDesignEdit {
            proposal_id: id,
            sheet,
            instance_path: vec![],
            edit,
        },
    )
    .unwrap();
    let own = model.proposals[&id].clone();
    let check = |model: &DesignModel| {
        design_clipboard_receipt::validate_current(
            &NativeWriteContext {
                model,
                project_root: &root,
            },
            &model.proposals[&id],
        )
    };
    check(&model).unwrap();
    crate::substrate::review_proposal_status(
        &mut model,
        &root,
        id,
        crate::substrate::ProposalStatus::Accepted,
    )
    .unwrap();
    model = ProjectResolver::new(&root).resolve().unwrap();
    check(&model).unwrap();
    // An exact own-ID/path is insufficient when an old metadata body differs.
    let mut altered = model.clone();
    let mut found = false;
    for transaction in &mut altered.journal {
        for op in &mut transaction.operations {
            if let Operation::CreateProposalMetadata {
                proposal_id,
                proposal,
                ..
            } = op
            {
                if *proposal_id == id {
                    proposal["rationale"] = json!("different cached authority");
                    found = true;
                }
            }
        }
    }
    assert!(found);
    assert!(check(&altered).is_err());
    let mut foreign = own.clone();
    foreign.proposal_id = Uuid::new_v4();
    model.proposals.insert(foreign.proposal_id, foreign);
    assert!(
        check(&model).is_err(),
        "other cached records retain all physical-ID references"
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn design_edit_generic_cut_receipt_rechecks_foreign_sheet_bus_entry_dependencies() {
    let (root, mut model, sheet, wire, _, sentinel) = fixture();
    let id = Uuid::new_v4();
    let edit = DesignEditIntent::Cut {
        targets: vec![ClipboardTarget {
            class: ClipboardClass::Wire,
            id: wire,
        }],
    };
    let (prepared, rationale) = design_clipboard_receipt::prepare(
        &NativeWriteContext {
            model: &model,
            project_root: &root,
        },
        provenance(),
        id,
        sheet,
        &[],
        &edit,
    )
    .unwrap();
    let snapshot = crate::substrate::ElectricalSelectionSnapshot::capture(&model).unwrap();
    let schematic = snapshot.schematic_source().unwrap().uuid;
    let other = Uuid::new_v4();
    let bus = Uuid::new_v4();
    let entry = Uuid::new_v4();
    let raw = json!({"schema_version":1,"uuid":other,"name":"Foreign dependency","symbols":{},"wires":{},"labels":{},"ports":{},"junctions":{},"buses":{bus.to_string():{"uuid":bus,"name":"DATA","members":[],"segments":[]}},"bus_entries":{entry.to_string():{"uuid":entry,"bus":bus,"wire":wire,"position":{"x":0,"y":0},"size":{"x":0,"y":0}}},"noconnects":{},"texts":{},"drawings":{}});
    let write = super::schematic_sheets::build_create_schematic_sheet(
        &model,
        provenance(),
        schematic,
        other,
        "sheets/foreign.json",
        raw,
    )
    .unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    model = ProjectResolver::new(&root).resolve().unwrap();
    // Forge only the public generic Draft transport, not a private source edit.
    // The target source did not change; the new dependency is outside it.
    let mut receipt: serde_json::Value = serde_json::from_str(&rationale).unwrap();
    receipt["cut_source"]["source_revision"] = json!(model.model_revision);
    forged_draft(&mut model, &root, id, prepared, receipt.to_string());
    model = ProjectResolver::new(&root).resolve().unwrap();
    design_clipboard_receipt::validate_recorded(&model.proposals[&id]).unwrap();
    let error = design_clipboard_receipt::validate_current(
        &NativeWriteContext {
            model: &model,
            project_root: &root,
        },
        &model.proposals[&id],
    )
    .unwrap_err();
    assert!(error.to_string().contains("surviving Bus Entry"), "{error}");
    let before = source_sheet_payload(&model, sheet, sentinel).unwrap();
    let count = model.journal.len();
    assert!(
        act(
            &mut model,
            &root,
            NativeProposalAction::AcceptApply { proposal_id: id }
        )
        .is_err()
    );
    assert_eq!(model.journal.len(), count);
    assert_eq!(
        source_sheet_payload(&model, sheet, sentinel).unwrap(),
        before
    );
    std::fs::remove_dir_all(root).unwrap();
}
