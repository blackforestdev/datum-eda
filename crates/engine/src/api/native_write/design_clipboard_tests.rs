//! Shared design clipboard source admission, untrusted payloads and canonical edits.
use super::{
    commit_prepared,
    context::{BatchComposer, PreparedWrite, WriteProvenance},
    design_clipboard::{
        ClipboardClass, ClipboardPlacement, ClipboardTarget, DesignClipboardBundle,
    },
    design_clipboard_edit::{self, ClipboardContactPolicy, DesignEditIntent},
    electrical::topology_tests,
    registry::NativeWriteContext,
    schematic_edit::prepare_native_birth,
    schematic_symbol_payload::source_sheet_payload,
};
use crate::{
    ir::geometry::Point,
    substrate::{
        CommitSource, DesignModel, ElectricalSelectionSnapshot, Operation, ProjectResolver,
        SchematicMarkerKind,
    },
};
use serde_json::json;
use std::path::{Path, PathBuf};
use uuid::Uuid;
fn provenance() -> WriteProvenance {
    WriteProvenance::new("test", CommitSource::Test, "shared design clipboard")
}
fn persist(model: &mut DesignModel, root: &Path, prepared: PreparedWrite) {
    commit_prepared(model, root, prepared).unwrap();
    *model = ProjectResolver::new(root).resolve().unwrap();
}
fn create_wire(sheet: Uuid, id: Uuid, from: Point, to: Point, locked: bool) -> Operation {
    Operation::CreateSchematicWire {
        sheet_id: sheet,
        wire_id: id,
        wire: json!({"uuid":id,
        "from":{"x":from.x,"y":from.y,"future_point":"from"},"to":{"x":to.x,"y":to.y,"future_point":"to"},
        "locked":locked,"vendor":{"literal":"${KEEP}","opaque_uuid":id}}),
    }
}
fn fixture(locked: bool) -> (PathBuf, DesignModel, Uuid, Uuid, Uuid, Uuid) {
    let (root, mut model, _, sheet) = topology_tests::fixture("design_clipboard");
    let wire = Uuid::new_v4();
    let junction = Uuid::new_v4();
    let sentinel = Uuid::new_v4();
    let prepared=BatchComposer::compose(&model,provenance()).push_ops(vec![
        create_wire(sheet,wire,Point::zero(),Point::new(1_000,0),locked),
        create_wire(sheet,sentinel,Point::new(100_000,0),Point::new(110_000,0),false),
        Operation::PlaceSchematicMarker{sheet_id:sheet,marker_id:junction,marker_kind:SchematicMarkerKind::Junction,
            marker:json!({"uuid":junction,"position":{"x":0,"y":0,"future_point":"junction"},"vendor":"${J}"})},
    ]).finish().unwrap();
    let prepared = prepare_native_birth(&model, prepared).unwrap();
    persist(&mut model, &root, prepared);
    (root, model, sheet, wire, junction, sentinel)
}
fn targets(wire: Uuid, junction: Uuid) -> Vec<ClipboardTarget> {
    vec![
        ClipboardTarget {
            class: ClipboardClass::Wire,
            id: wire,
        },
        ClipboardTarget {
            class: ClipboardClass::Junction,
            id: junction,
        },
    ]
}
fn build(
    model: &DesignModel,
    root: &Path,
    sheet: Uuid,
    path: &[Uuid],
    edit: DesignEditIntent,
) -> Result<PreparedWrite, crate::error::EngineError> {
    design_clipboard_edit::build(
        &NativeWriteContext {
            model,
            project_root: root,
        },
        provenance(),
        sheet,
        path,
        &edit,
    )
}

#[test]
fn design_copy_locked_source_is_readonly_and_shared_transform_preserves_opaque_raw() {
    let (root, model, sheet, wire, junction, sentinel) = fixture(true);
    let before = source_sheet_payload(&model, sheet, sentinel).unwrap();
    let count = model.journal.len();
    let selected = targets(wire, junction);
    let bundle =
        DesignClipboardBundle::capture(&model, sheet, &[], &selected, Point::zero()).unwrap();
    assert_eq!(
        source_sheet_payload(&model, sheet, sentinel).unwrap(),
        before
    );
    assert_eq!(model.journal.len(), count);
    let placement = ClipboardPlacement {
        position: Point::new(2_731, 5_019),
        rotation_deg: 90,
        mirrored: true,
    };
    let seed = Uuid::new_v4();
    let moved = bundle.placed(placement, seed).unwrap();
    assert_eq!(bundle.placed(placement, seed).unwrap(), moved);
    let copied = moved
        .iter()
        .find(|item| item.class == ClipboardClass::Wire)
        .unwrap();
    assert_ne!(copied.id, wire);
    assert_eq!(
        copied.raw["from"],
        json!({"x":2731,"y":5019,"future_point":"from"})
    );
    assert_eq!(
        copied.raw["to"],
        json!({"x":2731,"y":4019,"future_point":"to"})
    );
    assert_eq!(
        copied.raw["vendor"],
        before["wires"][wire.to_string()]["vendor"]
    );
    assert_eq!(
        copied.raw["locked"], true,
        "Copy preserves locks without mutating their owner"
    );
    let error = build(
        &model,
        &root,
        sheet,
        &[],
        DesignEditIntent::Cut { targets: selected },
    )
    .unwrap_err();
    assert!(error.to_string().contains("locked"));
    assert_eq!(
        source_sheet_payload(&model, sheet, sentinel).unwrap(),
        before
    );
    assert_eq!(model.journal.len(), count);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn design_clipboard_untrusted_shape_identity_pose_and_source_paths_refuse_without_writes() {
    let (root, model, sheet, wire, junction, sentinel) = fixture(false);
    let selected = targets(wire, junction);
    let before = source_sheet_payload(&model, sheet, sentinel).unwrap();
    let count = model.journal.len();
    assert!(
        DesignClipboardBundle::capture(&model, sheet, &[Uuid::new_v4()], &selected, Point::zero())
            .is_err()
    );
    assert!(
        DesignClipboardBundle::capture(
            &model,
            sheet,
            &[],
            &[selected[0].clone(), selected[0].clone()],
            Point::zero()
        )
        .is_err()
    );
    let bundle =
        DesignClipboardBundle::capture(&model, sheet, &[], &selected, Point::zero()).unwrap();
    let placement = ClipboardPlacement {
        position: Point::zero(),
        rotation_deg: 0,
        mirrored: false,
    };
    let mut unsupported = serde_json::to_value(&bundle).unwrap();
    unsupported["items"][0]["class"] = json!("symbol");
    assert!(serde_json::from_value::<DesignClipboardBundle>(unsupported).is_err());
    let mut altered = bundle.clone();
    altered.items[0].raw["uuid"] = json!(Uuid::new_v4());
    assert!(altered.placed(placement, Uuid::new_v4()).is_err());
    let mut altered = bundle.clone();
    altered.items.push(altered.items[0].clone());
    assert!(altered.placed(placement, Uuid::new_v4()).is_err());
    assert!(bundle.placed(placement, Uuid::nil()).is_err());
    assert!(
        bundle
            .placed(
                ClipboardPlacement {
                    rotation_deg: 45,
                    ..placement
                },
                Uuid::new_v4()
            )
            .is_err()
    );
    assert!(
        bundle
            .placed(
                ClipboardPlacement {
                    position: Point::new(i64::MAX, 0),
                    ..placement
                },
                Uuid::new_v4()
            )
            .is_err()
    );
    let source = ElectricalSelectionSnapshot::capture(&model).unwrap();
    let target = &source.schematic_source().unwrap().sheets[&sheet];
    assert!(
        design_clipboard_edit::preview_placement(
            target,
            &bundle,
            &placement,
            &Uuid::new_v4(),
            ClipboardContactPolicy::DrawWireEndpoints
        )
        .is_err(),
        "a duplicate-in-place bundle must not add hidden overlapping wire copies"
    );
    let shifted = ClipboardPlacement {
        position: Point::new(20_000, 20_000),
        ..placement
    };
    let seed = Uuid::new_v4();
    let items = bundle.placed(shifted, seed).unwrap();
    let new_wire = items
        .iter()
        .find(|item| item.class == ClipboardClass::Wire)
        .unwrap()
        .id;
    let mut collision = target.clone();
    collision.wires.insert(
        new_wire,
        crate::schematic::SchematicWire {
            uuid: new_wire,
            from: Point::new(30_000, 0),
            to: Point::new(31_000, 0),
        },
    );
    assert!(
        design_clipboard_edit::preview_placement(
            &collision,
            &bundle,
            &shifted,
            &seed,
            ClipboardContactPolicy::DrawWireEndpoints
        )
        .is_err(),
        "pure preview must refuse known target identity overwrite without claiming global freshness"
    );
    assert_eq!(
        source_sheet_payload(&model, sheet, sentinel).unwrap(),
        before
    );
    assert_eq!(model.journal.len(), count);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn canonical_cross_project_paste_uses_fresh_ids_explicit_t_and_one_cold_history_batch() {
    let (source_root, source, source_sheet, wire, _, source_sentinel) = fixture(false);
    let source_before = source_sheet_payload(&source, source_sheet, source_sentinel).unwrap();
    let bundle = DesignClipboardBundle::capture(
        &source,
        source_sheet,
        &[],
        &[ClipboardTarget {
            class: ClipboardClass::Wire,
            id: wire,
        }],
        Point::zero(),
    )
    .unwrap();
    let (root, mut model, _, sheet) = topology_tests::fixture("clipboard_paste_target");
    let fixed = Uuid::new_v4();
    let prepared = BatchComposer::compose(&model, provenance())
        .push_op(create_wire(
            sheet,
            fixed,
            Point::new(2_000, -1_000),
            Point::new(2_000, 1_000),
            true,
        ))
        .finish()
        .unwrap();
    let prepared = prepare_native_birth(&model, prepared).unwrap();
    persist(&mut model, &root, prepared);
    let before = source_sheet_payload(&model, sheet, fixed).unwrap();
    let identities = model.electrical_identities.clone();
    let count = model.journal.len();
    let seed = Uuid::new_v4();
    let placement = ClipboardPlacement {
        position: Point::new(2_000, 517),
        rotation_deg: 0,
        mirrored: false,
    };
    let edit = DesignEditIntent::PlaceBundle {
        bundle: bundle.clone(),
        placement,
        edit_seed: seed,
        contact_policy: ClipboardContactPolicy::DrawWireEndpoints,
    };
    let placed = bundle.placed(placement, seed).unwrap();
    let pasted = placed[0].id;
    let captured = ElectricalSelectionSnapshot::capture(&model).unwrap();
    let preview = design_clipboard_edit::preview_placement(
        &captured.schematic_source().unwrap().sheets[&sheet],
        &bundle,
        &placement,
        &seed,
        ClipboardContactPolicy::DrawWireEndpoints,
    )
    .unwrap();
    let prepared = build(&model, &root, sheet, &[], edit.clone()).unwrap();
    assert_eq!(source_sheet_payload(&model, sheet, fixed).unwrap(), before);
    assert_eq!(model.journal.len(), count);
    persist(&mut model, &root, prepared);
    assert_eq!(model.journal.len(), count + 1);
    let after = source_sheet_payload(&model, sheet, fixed).unwrap();
    assert_eq!(
        ElectricalSelectionSnapshot::capture(&model)
            .unwrap()
            .schematic_source()
            .unwrap()
            .sheets[&sheet],
        preview,
        "pure clipboard ghost matches canonical members including explicit T markers"
    );
    assert_eq!(
        after["wires"][fixed.to_string()],
        before["wires"][fixed.to_string()],
        "Paste never rewrites even a locked target edge"
    );
    assert_eq!(after["wires"][pasted.to_string()], placed[0].raw);
    let markers = after["junctions"].as_object().unwrap();
    assert_eq!(markers.len(), 1);
    assert_eq!(
        markers.values().next().unwrap()["position"],
        json!({"x":2000,"y":517})
    );
    assert_eq!(
        ElectricalSelectionSnapshot::capture(&model)
            .unwrap()
            .net_ids()
            .len(),
        1,
        "pasted endpoint joins existing scalar group"
    );
    assert!(
        build(&model, &root, sheet, &[], edit).is_err(),
        "same physical gesture map cannot collide with existing objects"
    );
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
            source_sheet_payload(&model, sheet, fixed).unwrap(),
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
    assert_eq!(
        source_sheet_payload(&source, source_sheet, source_sentinel).unwrap(),
        source_before
    );
    std::fs::remove_dir_all(root).unwrap();
    std::fs::remove_dir_all(source_root).unwrap();
}

#[test]
fn canonical_cut_reuses_delete_and_cold_undo_restores_exact_extensions_and_identity() {
    let (root, mut model, sheet, wire, junction, sentinel) = fixture(false);
    let before = source_sheet_payload(&model, sheet, sentinel).unwrap();
    let identities = model.electrical_identities.clone();
    let edit = DesignEditIntent::Cut {
        targets: targets(wire, junction),
    };
    let prepared = build(&model, &root, sheet, &[], edit).unwrap();
    persist(&mut model, &root, prepared);
    let after = source_sheet_payload(&model, sheet, sentinel).unwrap();
    let mut expected = before.clone();
    expected["wires"]
        .as_object_mut()
        .unwrap()
        .remove(&wire.to_string());
    expected["junctions"]
        .as_object_mut()
        .unwrap()
        .remove(&junction.to_string());
    assert_eq!(after, expected);
    model
        .commit_journal_undo(&root, provenance().into())
        .unwrap();
    model = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(
        source_sheet_payload(&model, sheet, sentinel).unwrap(),
        before
    );
    assert_eq!(model.electrical_identities, identities);
    model
        .commit_journal_redo(&root, provenance().into())
        .unwrap();
    model = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(
        source_sheet_payload(&model, sheet, sentinel).unwrap(),
        after
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn canonical_existing_transform_preserves_physical_ids_raw_identity_and_cold_history() {
    use super::design_clipboard_edit::ExistingTransformContactPolicy::PreserveFixedContacts;
    use super::schematic_delete_proposal::{NativeProposalAction, NativeProposalRequest, execute};
    let (root, mut model, sheet, wire, junction, sentinel) = fixture(false);
    let before = source_sheet_payload(&model, sheet, sentinel).unwrap();
    let identities = model.electrical_identities.clone();
    let selected = targets(wire, junction);
    let placement = ClipboardPlacement {
        position: Point::new(2_731, 5_019),
        rotation_deg: 90,
        mirrored: true,
    };
    let snapshot = ElectricalSelectionSnapshot::capture(&model).unwrap();
    let preview = design_clipboard_edit::preview_transform(
        &snapshot.schematic_source().unwrap().sheets[&sheet],
        &selected,
        Point::zero(),
        placement,
        PreserveFixedContacts,
    )
    .unwrap();
    let proposal_id = Uuid::new_v4();
    let edit = DesignEditIntent::TransformExisting {
        targets: selected,
        reference: Point::zero(),
        placement,
        contact_policy: PreserveFixedContacts,
    };
    for action in [
        NativeProposalAction::CreateDesignEdit {
            proposal_id,
            sheet,
            instance_path: vec![],
            edit,
        },
        NativeProposalAction::AcceptApply { proposal_id },
    ] {
        let request = NativeProposalRequest {
            project_root: root.clone(),
            expected_model_revision: model.model_revision.0.clone(),
            expected_transaction_tip: crate::revision::transaction_tip(&model.journal),
            action,
        };
        execute(&mut model, &request).unwrap();
    }
    model = ProjectResolver::new(&root).resolve().unwrap();
    let after = source_sheet_payload(&model, sheet, sentinel).unwrap();
    let mut expected = before.clone();
    expected["wires"][wire.to_string()]["from"]["x"] = json!(2731);
    expected["wires"][wire.to_string()]["from"]["y"] = json!(5019);
    expected["wires"][wire.to_string()]["to"]["x"] = json!(2731);
    expected["wires"][wire.to_string()]["to"]["y"] = json!(4019);
    expected["junctions"][junction.to_string()]["position"]["x"] = json!(2731);
    expected["junctions"][junction.to_string()]["position"]["y"] = json!(5019);
    assert_eq!(
        after, expected,
        "only existing known coordinates change; raw extensions and UUIDs remain exact"
    );
    let snapshot = ElectricalSelectionSnapshot::capture(&model).unwrap();
    assert_eq!(snapshot.schematic_source().unwrap().sheets[&sheet], preview);
    assert_eq!(
        model.electrical_identities, identities,
        "preserving transforms allocate no semantic identities"
    );
    let cached = &model.proposals[&proposal_id].batch;
    assert!(cached.operations.iter().all(|op| !matches!(
        op,
        Operation::CreateElectricalIdentity { .. } | Operation::DeleteElectricalIdentity { .. }
    )));
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
        assert_eq!(model.electrical_identities, identities);
    }
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn canonical_existing_transform_refuses_locked_incomplete_and_wrong_occurrence_without_writes() {
    use super::design_clipboard_edit::ExistingTransformContactPolicy::PreserveFixedContacts;
    for locked in [false, true] {
        let (root, model, sheet, wire, junction, sentinel) = fixture(locked);
        let before = source_sheet_payload(&model, sheet, sentinel).unwrap();
        let count = model.journal.len();
        let placement = ClipboardPlacement {
            position: Point::new(2731, 5019),
            rotation_deg: 90,
            mirrored: false,
        };
        let edit = |selected| DesignEditIntent::TransformExisting {
            targets: selected,
            reference: Point::zero(),
            placement,
            contact_policy: PreserveFixedContacts,
        };
        if locked {
            assert!(build(&model, &root, sheet, &[], edit(targets(wire, junction))).is_err());
        } else {
            assert!(
                build(
                    &model,
                    &root,
                    sheet,
                    &[],
                    edit(vec![ClipboardTarget {
                        class: ClipboardClass::Wire,
                        id: wire
                    }])
                )
                .is_err(),
                "unselected Junction is a fixed contact"
            );
        }
        assert!(
            build(
                &model,
                &root,
                sheet,
                &[Uuid::new_v4()],
                edit(targets(wire, junction))
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
}

#[test]
fn design_edit_existing_transform_rejects_tampered_raw_and_stale_receipts_before_writes() {
    use super::design_clipboard_edit::ExistingTransformContactPolicy::PreserveFixedContacts;
    use super::{
        design_clipboard_receipt,
        schematic_delete_proposal::{NativeProposalAction, NativeProposalRequest, execute},
    };
    let act = |model: &mut DesignModel, root: &Path, action| {
        let request = NativeProposalRequest {
            project_root: root.into(),
            expected_model_revision: model.model_revision.0.clone(),
            expected_transaction_tip: crate::revision::transaction_tip(&model.journal),
            action,
        };
        execute(model, &request)
    };
    let (root, mut model, sheet, wire, marker, sentinel) = fixture(false);
    let proposal_id = Uuid::new_v4();
    let edit = DesignEditIntent::TransformExisting {
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
        reference: Point::zero(),
        placement: ClipboardPlacement {
            position: Point::new(2731, 5019),
            rotation_deg: 90,
            mirrored: true,
        },
        contact_policy: PreserveFixedContacts,
    };
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
    for forged_source in [false, true] {
        let mut altered = valid.clone();
        if forged_source {
            let mut receipt: serde_json::Value = serde_json::from_str(&altered.rationale).unwrap();
            for item in receipt["transform_source"]["items"].as_array_mut().unwrap() {
                if item["id"] == json!(wire) {
                    item["raw"]["vendor"] = json!("forged");
                }
            }
            altered.rationale = receipt.to_string();
        }
        for op in &mut altered.batch.operations {
            match op {
                Operation::CreateSchematicWire {
                    wire_id, wire: raw, ..
                } if *wire_id == wire => {
                    raw["vendor"] = json!("forged");
                }
                Operation::DeleteSchematicWire {
                    wire_id, wire: raw, ..
                } if *wire_id == wire && forged_source => {
                    raw["vendor"] = json!("forged");
                }
                _ => {}
            }
        }
        if forged_source {
            design_clipboard_receipt::validate_recorded(&altered).unwrap();
        } else {
            assert!(design_clipboard_receipt::validate_recorded(&altered).is_err());
        }
        model.proposals.insert(proposal_id, altered);
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
    model.proposals.insert(proposal_id, valid);
    let fresh = Uuid::new_v4();
    let write = BatchComposer::compose(&model, provenance())
        .push_op(Operation::CreateSchematicWire {
            sheet_id: sheet,
            wire_id: fresh,
            wire: json!({"uuid":fresh,"from":{"x":50000,"y":0},"to":{"x":51000,"y":0}}),
        })
        .finish()
        .unwrap();
    let write = prepare_native_birth(&model, write).unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    let current = source_sheet_payload(&model, sheet, sentinel).unwrap();
    let count = model.journal.len();
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
        current
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[path = "schematic_design_move_tests.rs"]
mod schematic_design_move_tests;
