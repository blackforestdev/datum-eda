//! Ordinary proposal lifecycle and cold history; physical identity is preserved.
use super::*;
use crate::api::native_write::schematic_delete_proposal::{
    self as proposals, FieldRowEdit, NativeProposalAction as Action, NativeProposalRequest,
    NativeProposalResponse,
};
use crate::substrate::{
    ElectricalIdentity, NetCorrespondenceStatus, component_pad_source, net_correspondence_status,
};

fn action(f: &mut F, action: Action) -> NativeProposalResponse {
    let request = NativeProposalRequest {
        project_root: f.root.clone(),
        expected_model_revision: f.model.model_revision.0.clone(),
        expected_transaction_tip: crate::revision::transaction_tip(&f.model.journal),
        action,
    };
    proposals::execute(&mut f.model, &request).unwrap()
}

fn authoritative_bytes(f: &F) -> BTreeMap<String, Vec<u8>> {
    let mut result = bytes(f);
    let mut directories = vec![f.root.join(".datum/revision")];
    while let Some(directory) = directories.pop() {
        for entry in std::fs::read_dir(directory).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                directories.push(path);
            } else {
                result.insert(
                    path.strip_prefix(&f.root).unwrap().to_string_lossy().into(),
                    std::fs::read(path).unwrap(),
                );
            }
        }
    }
    result
}

fn preview_draft_and_accepted(f: &mut F, id: Uuid) {
    use crate::substrate::{ProposalStatus, review_proposal_status};
    for status in [ProposalStatus::Draft, ProposalStatus::Accepted] {
        if status == ProposalStatus::Accepted {
            review_proposal_status(&mut f.model, &f.root, id, status).unwrap();
        }
        let before = authoritative_bytes(f);
        let response = action(f, Action::Preview { proposal_id: id });
        assert_eq!(response.status, status);
        assert_eq!(
            authoritative_bytes(f),
            before,
            "persisted Preview never changes canonical design/journal/revision authority"
        );
    }
}

fn board(f: &F) -> Value {
    f.model
        .materialized_source_shard_value(SourceShardKind::BoardRoot)
        .unwrap()
}

fn bound() -> F {
    let mut f = fixture_mode(true, true);
    f.request.choices.clear();
    let prepared = prepare_transfer(&f.model, provenance(), &f.request).unwrap();
    commit_prepared(&mut f.model, &f.root, prepared).unwrap();
    f.model = ProjectResolver::new(&f.root).resolve().unwrap();
    f
}

fn moved_rotated_routed() -> F {
    use native::board_components::{BoardPackageEdit, build_edit_board_package};
    use native::board_routing::route::capture_route_basis;
    let mut f = bound();
    let package = f
        .model
        .component_instances
        .values()
        .next()
        .unwrap()
        .placed_package_refs[0];
    write(
        &mut f.model,
        &f.root,
        "datum.pcb.move_component",
        json!({"component":package,"x_nm":2_000_000,"y_nm":3_000_000}),
    );
    f.model = ProjectResolver::new(&f.root).resolve().unwrap();
    let prepared = build_edit_board_package(
        &f.model,
        provenance(),
        package,
        BoardPackageEdit::Rotation { rotation: 90 },
    )
    .unwrap();
    commit_prepared(&mut f.model, &f.root, prepared).unwrap();
    f.model = ProjectResolver::new(&f.root).resolve().unwrap();
    let source = board(&f);
    assert_eq!(
        source["packages"][package.to_string()]["rotation"],
        json!(90)
    );
    let projection = component_pad_source::component_pad_projection(&f.model, &source).unwrap();
    let pads: Vec<_> = projection.pads.values().cloned().collect();
    assert_eq!(pads.len(), 2);
    let rule = json!({"uuid":Uuid::new_v4(),"name":"Explicit test clearance",
        "scope":{"Net":pads[0].net.unwrap()},"priority":0,"enabled":true,
        "rule_type":"clearance_copper","params":{"min_nm":100_000}});
    write(
        &mut f.model,
        &f.root,
        "datum.project.create_rule",
        json!({"rule":rule}),
    );
    f.model = ProjectResolver::new(&f.root).resolve().unwrap();
    let basis = capture_route_basis(&f.model, pads[0].uuid, 1).unwrap();
    write(
        &mut f.model,
        &f.root,
        "datum.pcb.draw_track",
        json!({"basis":basis,
        "end_pad":pads[1].uuid,"points":[pads[0].position,pads[1].position],
        "width_nm":200_000,"edit_seed":Uuid::new_v4()}),
    );
    f.model = ProjectResolver::new(&f.root).resolve().unwrap();
    assert!(!board(&f)["tracks"].as_object().unwrap().is_empty());
    f
}

fn edit_source(f: &mut F) {
    edit_source_to(f, "AMP1", "10k");
}

fn edit_source_to(f: &mut F, reference: &str, value: &str) {
    let source = ElectricalSelectionSnapshot::capture(&f.model).unwrap();
    let symbol = source.schematic_source().unwrap().sheets[&f.sheet]
        .symbols
        .values()
        .next()
        .unwrap();
    let edit = FieldRowEdit {
        symbol: symbol.uuid,
        reference: reference.into(),
        value: value.into(),
    };
    let id = Uuid::new_v4();
    let sheet = f.sheet;
    action(
        f,
        Action::CreateFieldBatch {
            proposal_id: id,
            sheet,
            instance_path: vec![],
            edits: vec![edit],
        },
    );
    preview_draft_and_accepted(f, id);
    action(f, Action::AcceptApply { proposal_id: id });
    f.model = ProjectResolver::new(&f.root).resolve().unwrap();
}

fn electrical(f: &F) -> BTreeMap<Uuid, Option<Uuid>> {
    for record in f.model.electrical_identities.values() {
        if let ElectricalIdentity::NetRelationship { logical_net, .. } = &record.identity {
            assert_eq!(
                net_correspondence_status(&f.model, *logical_net),
                NetCorrespondenceStatus::Complete
            );
        }
    }
    let projected = component_pad_source::component_pad_projection(&f.model, &board(f)).unwrap();
    assert!(projected.unavailable.is_empty());
    assert_eq!(projected.query_pads.len(), 2);
    projected
        .query_pads
        .into_iter()
        .map(|(id, pad)| {
            assert!(
                pad.terminal().net.is_some(),
                "every physical pad retains its actual net"
            );
            (id, pad.terminal().net)
        })
        .collect()
}

fn physical(mut board: Value) -> Value {
    for package in board["packages"].as_object_mut().unwrap().values_mut() {
        package.as_object_mut().unwrap().remove("reference");
        package.as_object_mut().unwrap().remove("value");
    }
    board
}

#[test]
fn field_revision_preview_rejection_and_stale_capture_preserve_the_board() {
    let mut f = bound();
    let physical_before = board(&f);
    let nets = electrical(&f);
    edit_source(&mut f);
    assert_eq!(electrical(&f), nets);
    let before = bytes(&f);
    let review = field_revision::preview(&f.model, f.sheet, &[]).unwrap();
    assert_eq!(review.changes.len(), 1);
    assert_eq!(review.changes[0].reference, "AMP1");
    assert_eq!(review.changes[0].value, "10k");
    assert_eq!(bytes(&f), before);
    assert!(field_revision::preview(&f.model, f.sheet, &[Uuid::new_v4()]).is_err());
    let id = Uuid::new_v4();
    let sheet = f.sheet;
    let draft = action(
        &mut f,
        Action::CreateBoardFieldRevision {
            proposal_id: id,
            sheet,
            review: Some(review.clone()),
        },
    );
    assert_eq!(draft.board_field_revision.as_ref(), Some(&review));
    action(&mut f, Action::Reject { proposal_id: id });
    f.model = ProjectResolver::new(&f.root).resolve().unwrap();
    assert_eq!(board(&f), physical_before);
    assert_eq!(electrical(&f), nets);
    edit_source_to(&mut f, "AMP2", "20k");
    let before = bytes(&f);
    let request = NativeProposalRequest {
        project_root: f.root.clone(),
        expected_model_revision: f.model.model_revision.0.clone(),
        expected_transaction_tip: crate::revision::transaction_tip(&f.model.journal),
        action: Action::CreateBoardFieldRevision {
            proposal_id: Uuid::new_v4(),
            sheet,
            review: Some(review),
        },
    };
    assert!(proposals::execute(&mut f.model, &request).is_err());
    assert_eq!(bytes(&f), before);
}

#[test]
fn reference_value_revision_acceptance_and_repeated_cold_history_keep_pad_nets() {
    let mut f = moved_rotated_routed();
    let original = board(&f);
    let nets = electrical(&f);
    edit_source(&mut f);
    assert_eq!(board(&f), original);
    assert_eq!(electrical(&f), nets);
    let id = Uuid::new_v4();
    let sheet = f.sheet;
    // A headless caller may omit its snapshot, but still creates only a Draft.
    let draft = action(
        &mut f,
        Action::CreateBoardFieldRevision {
            proposal_id: id,
            sheet,
            review: None,
        },
    );
    assert_eq!(draft.status, crate::substrate::ProposalStatus::Draft);
    assert_eq!(board(&f), original);
    preview_draft_and_accepted(&mut f, id);
    action(&mut f, Action::AcceptApply { proposal_id: id });
    f.model = ProjectResolver::new(&f.root).resolve().unwrap();
    let accepted = board(&f);
    assert_ne!(accepted, original);
    assert_eq!(physical(accepted.clone()), physical(original.clone()));
    assert_eq!(electrical(&f), nets);
    for redo in [false, true, false, true] {
        let tip = crate::revision::transaction_tip(&f.model.journal).unwrap();
        native::history::commit_history(
            &mut f.model,
            &f.root,
            native::history::HistoryGuards { expected_tip: tip },
            redo,
            provenance(),
        )
        .unwrap();
        f.model = ProjectResolver::new(&f.root).resolve().unwrap();
        assert_eq!(
            board(&f),
            if redo {
                accepted.clone()
            } else {
                original.clone()
            }
        );
        assert_eq!(electrical(&f), nets);
        assert_eq!(f.model.component_instances.len(), 2);
        if !redo {
            let before = authoritative_bytes(&f);
            f.model.assert_current_metadata_redo_admission();
            assert_eq!(
                authoritative_bytes(&f),
                before,
                "prospective valid/forged Redo checks never write the design or journal"
            );
        }
    }
    // A later move must also authenticate the earlier metadata/compensation
    // epochs before the new component anchor, without recursively using itself.
    let package: Uuid = accepted["packages"]
        .as_object()
        .unwrap()
        .iter()
        .find(|(_, value)| value["reference"] == json!("AMP1"))
        .unwrap()
        .0
        .parse()
        .unwrap();
    let component_bytes: BTreeMap<_, _> = bytes(&f)
        .into_iter()
        .filter(|(path, _)| path.starts_with(".datum/component_instances/"))
        .collect();
    write(
        &mut f.model,
        &f.root,
        "datum.pcb.move_component",
        json!({"component":package,"x_nm":4_000_000,"y_nm":4_000_000}),
    );
    f.model = ProjectResolver::new(&f.root).resolve().unwrap();
    assert_eq!(electrical(&f), nets);
    assert_eq!(board(&f)["tracks"], accepted["tracks"]);
    assert_eq!(board(&f)["component_pads"], accepted["component_pads"]);
    let current_bytes: BTreeMap<_, _> = bytes(&f)
        .into_iter()
        .filter(|(path, _)| path.starts_with(".datum/component_instances/"))
        .collect();
    assert_eq!(current_bytes, component_bytes);
}

#[test]
fn pcb_field_revision_rejects_unreviewed_operations_missing_guards_and_foreign_receipts() {
    let mut f = bound();
    edit_source(&mut f);
    let id = Uuid::new_v4();
    let sheet = f.sheet;
    action(
        &mut f,
        Action::CreateBoardFieldRevision {
            proposal_id: id,
            sheet,
            review: None,
        },
    );
    let stored = f.model.proposals[&id].clone();
    let reviewed = field_revision::validate_recorded(&stored);
    assert!(
        reviewed.is_ok(),
        "canonical draft validates before adversarial changes"
    );
    let row = serde_json::from_str::<field_revision::BoardFieldRevision>(&stored.rationale)
        .unwrap()
        .changes
        .remove(0);
    let before = bytes(&f);
    for case in 0..4 {
        let mut forged = stored.clone();
        match case {
            0 => forged.batch.operations.push(crate::substrate::Operation::SetBoardPackagePosition {
                package_id: row.package, x: 1, y: 2,
            }),
            1 => forged.batch.operations.retain(|op| !matches!(op,
                crate::substrate::Operation::GuardObjectRevision { object_id, .. } if *object_id == row.package)),
            2 => forged.project_id = Uuid::new_v4(),
            _ => {
                let mut receipt: field_revision::BoardFieldRevision = serde_json::from_str(&forged.rationale).unwrap();
                receipt.changes[0].value = "unreviewed".into();
                forged.rationale = serde_json::to_string(&receipt).unwrap();
            }
        }
        assert!(
            field_revision::validate_recorded(&forged).is_err(),
            "forged case {case}"
        );
        assert_eq!(bytes(&f), before);
    }
}

#[test]
fn current_library_graph_revisions_cannot_be_hidden_by_unchanged_component_metadata() {
    use native::library::{PoolLibraryObjectTarget as Target, PoolLibraryOperationSpec as Spec};
    for kind in ["entities", "units", "packages"] {
        let mut f = bound();
        edit_source(&mut f);
        let id = f
            .model
            .objects
            .values()
            .find(|object| object.domain == "pool" && object.kind == kind)
            .unwrap()
            .object_id;
        let mut raw =
            native::schematic_library::materialized_pool_object(&f.model, id, kind).unwrap();
        raw["name"] = json!("Revised source");
        let prepared = native::library::build_pool_library_write(
            &f.model,
            provenance(),
            Some("pool"),
            vec![Spec::Set {
                target: Target::new("pool", kind, id),
                object: raw,
            }],
        )
        .unwrap();
        commit_prepared(&mut f.model, &f.root, prepared).unwrap();
        f.model = ProjectResolver::new(&f.root).resolve().unwrap();
        let before = bytes(&f);
        assert!(
            field_revision::preview(&f.model, f.sheet, &[]).is_err(),
            "changed {kind}"
        );
        assert_eq!(bytes(&f), before);
    }
}

#[test]
fn public_component_move_rotate_and_cold_history_preserve_authored_identity_and_pad_nets() {
    use native::board_components::{BoardPackageEdit, build_edit_board_package};
    let mut f = bound();
    let package = f
        .model
        .component_instances
        .values()
        .next()
        .unwrap()
        .placed_package_refs[0];
    let before = board(&f);
    let nets = electrical(&f);
    let component_bytes: BTreeMap<_, _> = bytes(&f)
        .into_iter()
        .filter(|(path, _)| path.starts_with(".datum/component_instances/"))
        .collect();
    assert_eq!(component_bytes.len(), 2);
    write(
        &mut f.model,
        &f.root,
        "datum.pcb.move_component",
        json!({"component":package,"x_nm":2_000_000,"y_nm":3_000_000}),
    );
    f.model = ProjectResolver::new(&f.root).resolve().unwrap();
    assert_eq!(f.model.component_instances.len(), 2);
    assert_eq!(electrical(&f), nets);
    let moved = board(&f);
    assert_ne!(
        moved["packages"][package.to_string()]["position"],
        before["packages"][package.to_string()]["position"]
    );
    let prepared = build_edit_board_package(
        &f.model,
        provenance(),
        package,
        BoardPackageEdit::Rotation { rotation: 90 },
    )
    .unwrap();
    commit_prepared(&mut f.model, &f.root, prepared).unwrap();
    f.model = ProjectResolver::new(&f.root).resolve().unwrap();
    let rotated = board(&f);
    assert_eq!(
        rotated["packages"][package.to_string()]["rotation"],
        json!(90)
    );
    for redo in [false, true, false, true] {
        let tip = crate::revision::transaction_tip(&f.model.journal).unwrap();
        native::history::commit_history(
            &mut f.model,
            &f.root,
            native::history::HistoryGuards { expected_tip: tip },
            redo,
            provenance(),
        )
        .unwrap();
        f.model = ProjectResolver::new(&f.root).resolve().unwrap();
        assert_eq!(f.model.component_instances.len(), 2);
        assert_eq!(electrical(&f), nets);
        let current = board(&f);
        assert_eq!(current["component_pads"], before["component_pads"]);
        assert_eq!(
            current["packages"][package.to_string()],
            if redo {
                rotated["packages"][package.to_string()].clone()
            } else {
                moved["packages"][package.to_string()].clone()
            }
        );
        let current_bytes: BTreeMap<_, _> = bytes(&f)
            .into_iter()
            .filter(|(path, _)| path.starts_with(".datum/component_instances/"))
            .collect();
        assert_eq!(current_bytes, component_bytes);
    }
}

#[test]
fn draft_preview_rejects_forged_stale_partial_and_cross_project_batches_without_disk_writes() {
    use crate::substrate::{
        ProposalCreateRequest, ProposalSource, create_draft_proposal_from_batch,
    };
    let mut f = bound();
    let snapshot = ElectricalSelectionSnapshot::capture(&f.model).unwrap();
    let symbol = snapshot.schematic_source().unwrap().sheets[&f.sheet]
        .symbols
        .values()
        .next()
        .unwrap()
        .uuid;
    let canonical = native::schematic_field_batch::build(
        &f.model,
        f.sheet,
        &[],
        &[FieldRowEdit {
            symbol,
            reference: "AMP1".into(),
            value: "10k".into(),
        }],
    )
    .unwrap()
    .batch;
    let before = bytes(&f);
    for case in 0..4 {
        let mut batch = canonical.clone();
        match case {
            0 => batch
                .operations
                .push(crate::substrate::Operation::SetBoardPackagePosition {
                    package_id: f
                        .model
                        .component_instances
                        .values()
                        .next()
                        .unwrap()
                        .placed_package_refs[0],
                    x: 1,
                    y: 2,
                }),
            1 => {
                batch.expected_model_revision =
                    Some(crate::substrate::ModelRevision("stale-source".into()))
            }
            2 => {
                let index = batch
                    .operations
                    .iter()
                    .position(|op| {
                        matches!(op, crate::substrate::Operation::SetComponentInstance { .. })
                    })
                    .unwrap();
                batch.operations.truncate(index + 1);
            }
            _ => {
                let guard = batch
                    .operations
                    .iter_mut()
                    .find(|op| {
                        matches!(op, crate::substrate::Operation::GuardObjectRevision { .. })
                    })
                    .unwrap();
                if let crate::substrate::Operation::GuardObjectRevision {
                    expected_object_revision,
                    ..
                } = guard
                {
                    expected_object_revision.0 += 1;
                }
            }
        }
        let request = ProposalCreateRequest {
            proposal_id: Some(Uuid::new_v4()),
            batch,
            rationale: "Untrusted candidate".into(),
            source: ProposalSource::Manual,
            checks_run: vec![],
            finding_fingerprints: vec![],
        };
        assert!(
            create_draft_proposal_from_batch(&mut f.model, &f.root, request).is_err(),
            "forged preview case {case}"
        );
        assert_eq!(bytes(&f), before);
    }
    let mut foreign = bound();
    let foreign_before = bytes(&foreign);
    let request = ProposalCreateRequest {
        proposal_id: Some(Uuid::new_v4()),
        batch: canonical,
        rationale: "Cross-project candidate".into(),
        source: ProposalSource::Manual,
        checks_run: vec![],
        finding_fingerprints: vec![],
    };
    assert!(create_draft_proposal_from_batch(&mut foreign.model, &foreign.root, request).is_err());
    assert_eq!(bytes(&foreign), foreign_before);
    assert_eq!(bytes(&f), before);
}
