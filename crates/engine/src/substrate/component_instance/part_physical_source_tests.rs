//! Actual component placement/assignment, source drift and persisted history.
use super::*;
use native::schematic_delete_proposal::{
    self as proposals, NativeProposalAction as Action, NativeProposalRequest,
};

fn place(f: &mut F, complete: bool) -> (Uuid, Option<Uuid>) {
    let part = f.other;
    place_part(f, part, complete)
}
fn place_part(f: &mut F, part: Uuid, complete: bool) -> (Uuid, Option<Uuid>) {
    let component = CompletePartSelection::capture(&f.model, part, f.a, f.symbol).unwrap();
    let mut request = params(f, None);
    if complete {
        request["component"] = serde_json::to_value(&component).unwrap();
        request["value"] = json!(component.value);
    }
    let prepared = prepare(f, "datum.schematic.place_symbol", request).unwrap();
    let placed = prepared.primary_object_id.unwrap();
    commit_prepared(&mut f.model, &f.root, prepared).unwrap();
    let ci = f
        .model
        .component_instances
        .values()
        .find(|ci| ci.placed_symbol_refs.contains(&placed))
        .map(|ci| ci.id);
    (placed, ci)
}
fn replace(f: &mut F, kind: &str, id: Uuid, value: Value) {
    use native::library::{PoolLibraryObjectTarget as Target, PoolLibraryOperationSpec as Spec};
    let prepared = native::library::build_pool_library_write(
        &f.model,
        provenance(),
        Some("pool"),
        vec![Spec::Set {
            target: Target::new("pool", kind, id),
            object: value,
        }],
    )
    .unwrap();
    commit_prepared(&mut f.model, &f.root, prepared).unwrap();
}
fn source(f: &F, symbol: Uuid) -> Value {
    let options = native::schematic_assignment::options(&f.model, symbol, &[]).unwrap();
    native::schematic_symbol_payload::source_symbol_payload(
        &f.model,
        options.sheet,
        &options.original,
    )
    .unwrap()
}
fn action(f: &mut F, action: Action) -> proposals::NativeProposalResponse {
    let request = NativeProposalRequest {
        project_root: f.root.clone(),
        expected_model_revision: f.model.model_revision.0.clone(),
        expected_transaction_tip: crate::revision::transaction_tip(&f.model.journal),
        action,
    };
    proposals::execute(&mut f.model, &request).unwrap()
}

#[test]
fn physical_choice_survives_pose_undo_redo_and_cold_but_refuses_independent_map_swap() {
    let mut f = fixture();
    let (placed, ci) = place(&mut f, true);
    let ci = ci.unwrap();
    let choice = CompletePartSelection::capture(&f.model, f.other, f.a, f.symbol).unwrap();
    choice.verify_declared(&f.model, ci).unwrap();
    write(
        &mut f,
        "datum.schematic.move_symbol",
        json!({"symbol":placed,"instance_path":[],"x_nm":12000000,"y_nm":22000000}),
    );
    choice.verify_declared(&f.model, ci).unwrap();
    f.model
        .commit_journal_undo(&f.root, provenance().into())
        .unwrap();
    f.model
        .commit_journal_redo(&f.root, provenance().into())
        .unwrap();
    f.model = ProjectResolver::new(&f.root).resolve().unwrap();
    choice.verify_declared(&f.model, ci).unwrap();
    let mut map = materialized_pool_object(&f.model, choice.pin_pad_map, "pin_pad_maps").unwrap();
    let rows = map["mappings"].as_object_mut().unwrap();
    let keys: Vec<_> = rows.keys().cloned().collect();
    let first = rows[&keys[0]].clone();
    let second = rows[&keys[1]].clone();
    rows.insert(keys[0].clone(), second);
    rows.insert(keys[1].clone(), first);
    replace(&mut f, "pin_pad_maps", choice.pin_pad_map, map);
    f.model = ProjectResolver::new(&f.root).resolve().unwrap();
    assert!(f.model.component_instances.contains_key(&ci));
    let current = CompletePartSelection::capture(&f.model, f.other, f.a, f.symbol).unwrap();
    let before = journal(&f);
    assert!(current.verify_declared(&f.model, ci).is_err());
    assert_eq!(journal(&f), before);
}

#[test]
fn physical_choice_refuses_independent_footprint_or_padstack_changes() {
    for kind in ["footprints", "padstacks"] {
        let mut f = fixture();
        let (_, ci) = place(&mut f, true);
        let ci = ci.unwrap();
        let choice = CompletePartSelection::capture(&f.model, f.other, f.a, f.symbol).unwrap();
        let id = if kind == "footprints" {
            choice.footprint
        } else {
            let footprint: crate::pool::Footprint = serde_json::from_value(
                materialized_pool_object(&f.model, choice.footprint, "footprints").unwrap(),
            )
            .unwrap();
            footprint.pads.values().next().unwrap().padstack
        };
        let mut raw = materialized_pool_object(&f.model, id, kind).unwrap();
        if kind == "footprints" {
            let pad = raw["pads"]
                .as_object_mut()
                .unwrap()
                .values_mut()
                .next()
                .unwrap();
            pad["position"]["x"] = json!(500000);
        } else {
            raw["aperture"]["circle"]["diameter_nm"] = json!(2000000);
        }
        replace(&mut f, kind, id, raw);
        f.model = ProjectResolver::new(&f.root).resolve().unwrap();
        assert!(f.model.component_instances.contains_key(&ci));
        let current = CompletePartSelection::capture(&f.model, f.other, f.a, f.symbol).unwrap();
        assert!(current.verify_declared(&f.model, ci).is_err(), "{kind}");
    }
}

#[test]
fn complete_assignment_updates_value_preserves_wiring_and_authenticates_applied_adoption() {
    let mut f = fixture();
    let (placed, owner) = place(&mut f, false);
    assert!(owner.is_none());
    let original = source(&f, placed);
    let position = original["pins"][0]["position"].clone();
    let x = position["x"].as_i64().unwrap();
    let y = position["y"].as_i64().unwrap();
    let sheet = f.sheet;
    write(
        &mut f,
        "datum.schematic.draw_wire",
        json!({"sheet":sheet,"instance_path":[],"points":[{"x":x,"y":y},{"x":x+3000000,"y":y}]}),
    );
    let wires = crate::substrate::ElectricalSelectionSnapshot::capture(&f.model)
        .unwrap()
        .schematic_source()
        .unwrap()
        .sheets[&sheet]
        .wires
        .clone();
    let nets = f.model.electrical_identities.clone();
    let component = CompletePartSelection::capture(&f.model, f.other, f.a, f.symbol).unwrap();
    assert_ne!(original["value"], json!(component.value));
    let proposal_id = Uuid::new_v4();
    let part = f.other;
    let gate = f.a;
    let draft = action(
        &mut f,
        Action::CreateAssignment {
            proposal_id,
            symbol: placed,
            instance_path: vec![],
            part,
            gate,
            component: Some(component.clone()),
        },
    );
    assert_eq!(source(&f, placed), original);
    let summary = draft.assignment.unwrap();
    let applied = action(&mut f, Action::AcceptApply { proposal_id });
    assert_eq!(applied.assignment.as_ref(), Some(&summary));
    f.model = ProjectResolver::new(&f.root).resolve().unwrap();
    let mut expected = original.clone();
    expected["part"] = json!(part);
    expected["entity"] = json!(f.entity);
    expected["gate"] = json!(gate);
    expected["value"] = json!(component.value);
    assert_eq!(source(&f, placed), expected);
    assert_eq!(
        crate::substrate::ElectricalSelectionSnapshot::capture(&f.model)
            .unwrap()
            .schematic_source()
            .unwrap()
            .sheets[&sheet]
            .wires,
        wires
    );
    assert_eq!(f.model.electrical_identities, nets);
    component
        .verify_declared(&f.model, summary.component_instance)
        .unwrap();
    for _ in 0..2 {
        f.model
            .commit_journal_undo(&f.root, provenance().into())
            .unwrap();
        f.model
            .commit_journal_redo(&f.root, provenance().into())
            .unwrap();
        f.model = ProjectResolver::new(&f.root).resolve().unwrap();
        component
            .verify_declared(&f.model, summary.component_instance)
            .unwrap();
    }
    let mut revised = materialized_pool_object(&f.model, part, "parts").unwrap();
    revised["value"] = json!("Later library value");
    replace(&mut f, "parts", part, revised);
    let recorded =
        native::schematic_assignment::summary(&f.model, &f.model.proposals[&proposal_id], false)
            .unwrap();
    assert_eq!(recorded, summary);
}

#[test]
fn part_only_reassignment_adopts_the_new_physical_epoch_instead_of_the_old_entity_epoch() {
    let mut f = fixture();
    let b = CompletePartSelection::capture(&f.model, f.other, f.a, f.symbol).unwrap();
    let map = materialized_pool_object(&f.model, b.pin_pad_map, "pin_pad_maps").unwrap();
    let first = f.part;
    write(
        &mut f,
        "datum.library.create",
        json!({"kind":"pin_pad_maps","object_id":Uuid::new_v4(),"pool_path":"pool","fields":{"part":first,"footprint":b.footprint,"mappings":map["mappings"],"assign_default":true}}),
    );
    let (placed, ci) = place_part(&mut f, first, true);
    let ci = ci.unwrap();
    let a = CompletePartSelection::capture(&f.model, first, f.a, f.symbol).unwrap();
    a.verify_declared(&f.model, ci).unwrap();
    let mut body = materialized_pool_object(&f.model, b.footprint, "footprints").unwrap();
    body["pads"]
        .as_object_mut()
        .unwrap()
        .values_mut()
        .next()
        .unwrap()["position"]["x"] = json!(500000);
    replace(&mut f, "footprints", b.footprint, body.clone());
    let current_b = CompletePartSelection::capture(&f.model, f.other, f.a, f.symbol).unwrap();
    assert_ne!(
        a.source_revisions[&b.footprint],
        current_b.source_revisions[&b.footprint]
    );
    let proposal_id = Uuid::new_v4();
    let gate = f.a;
    let part = f.other;
    action(
        &mut f,
        Action::CreateAssignment {
            proposal_id,
            symbol: placed,
            instance_path: vec![],
            part,
            gate,
            component: Some(current_b.clone()),
        },
    );
    action(&mut f, Action::AcceptApply { proposal_id });
    current_b.verify_declared(&f.model, ci).unwrap();
    for _ in 0..2 {
        f.model
            .commit_journal_undo(&f.root, provenance().into())
            .unwrap();
        assert_eq!(f.model.component_instances[&ci].part_ref, Some(first));
        f.model
            .commit_journal_redo(&f.root, provenance().into())
            .unwrap();
        f.model = ProjectResolver::new(&f.root).resolve().unwrap();
        assert_eq!(f.model.component_instances[&ci].part_ref, Some(part));
        current_b.verify_declared(&f.model, ci).unwrap();
    }
    body["pads"]
        .as_object_mut()
        .unwrap()
        .values_mut()
        .next()
        .unwrap()["position"]["x"] = json!(900000);
    replace(&mut f, "footprints", b.footprint, body);
    let changed = CompletePartSelection::capture(&f.model, f.other, f.a, f.symbol).unwrap();
    assert!(changed.verify_declared(&f.model, ci).is_err());
}

#[test]
fn same_part_gate_reassignment_retains_physical_sources_and_refuses_library_drift() {
    for drift in [false, true] {
        let mut f = fixture();
        let (placed, ci) = place(&mut f, true);
        let ci = ci.unwrap();
        let original = CompletePartSelection::capture(&f.model, f.other, f.a, f.symbol).unwrap();
        if drift {
            let mut footprint =
                materialized_pool_object(&f.model, original.footprint, "footprints").unwrap();
            footprint["name"] = json!("Independent library revision");
            replace(&mut f, "footprints", original.footprint, footprint);
        }
        let component = CompletePartSelection::capture(&f.model, f.other, f.b, f.symbol).unwrap();
        let before = journal(&f);
        let proposal_id = Uuid::new_v4();
        let request = NativeProposalRequest {
            project_root: f.root.clone(),
            expected_model_revision: f.model.model_revision.0.clone(),
            expected_transaction_tip: crate::revision::transaction_tip(&f.model.journal),
            action: Action::CreateAssignment {
                proposal_id,
                symbol: placed,
                instance_path: vec![],
                part: f.other,
                gate: f.b,
                component: Some(component.clone()),
            },
        };
        let result = proposals::execute(&mut f.model, &request);
        if drift {
            assert!(
                result
                    .unwrap_err()
                    .to_string()
                    .contains("physical definition changed")
            );
            assert_eq!(
                journal(&f),
                before,
                "Refusal must precede Proposal creation"
            );
            assert!(!f.model.proposals.contains_key(&proposal_id));
        } else {
            result.unwrap();
            action(&mut f, Action::AcceptApply { proposal_id });
            f.model = ProjectResolver::new(&f.root).resolve().unwrap();
            component.verify_declared(&f.model, ci).unwrap();
        }
    }
}
