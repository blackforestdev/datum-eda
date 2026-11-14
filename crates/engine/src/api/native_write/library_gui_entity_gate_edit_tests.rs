//! Same-Unit gate corrections preserve raw source and existing component chains.
use super::{tests::*, *};
use crate::api::native_write::{commit_prepared, schematic_gate_placement as placement};
use crate::substrate::{DesignModel, ProjectResolver};
use std::path::Path;

fn edit(
    model: &DesignModel,
    root: &Path,
    entity: Uuid,
    fields: Value,
) -> Result<PreparedWrite, EngineError> {
    super::super::build_definition(
        &NativeWriteContext {
            model,
            project_root: root,
        },
        provenance(),
        LibraryDefinitionRequest {
            kind: "entities".into(),
            object_id: entity,
            pool_path: "pool".into(),
            fields: fields.as_object().unwrap().clone(),
        },
        false,
    )
}
fn raw(model: &DesignModel, kind: &str, id: Uuid) -> Value {
    let path = PoolLibraryObjectTarget::new("pool", kind, id).relative_path;
    library::pool_library_object_payload(model, &path, id).unwrap()
}
fn alternate(model: &mut DesignModel, root: &Path, symbol: Uuid) -> Uuid {
    let id = Uuid::new_v4();
    let mut body = raw(model, "symbols", symbol);
    body["uuid"] = json!(id);
    body["name"] = json!("Alternate named body");
    body["drawings"] = json!([{"Rect":{"min":{"x":-1000000,"y":-1000000},"max":{"x":3000000,"y":1000000},"width":100000}}]);
    let p = library::build_pool_library_write(
        model,
        provenance(),
        Some("pool"),
        vec![PoolLibraryOperationSpec::Create {
            target: PoolLibraryObjectTarget::new("pool", "symbols", id),
            object: body,
        }],
    )
    .unwrap();
    commit_prepared(model, root, p).unwrap();
    id
}
#[test]
fn gate_symbol_patch_preserves_raw_identity_preview_and_cold_history() {
    let (root, mut model, unit, symbol) = fixture();
    let (entity, gate) = (Uuid::new_v4(), Uuid::new_v4());
    let p = prepare(
        &model,
        &root,
        "entities",
        entity,
        fields(gate, unit, symbol),
    )
    .unwrap();
    commit_prepared(&mut model, &root, p).unwrap();
    let replacement = alternate(&mut model, &root, symbol);
    let mut before = raw(&model, "entities", entity);
    let gate_key = gate.simple().to_string().to_uppercase();
    let gate_raw = before["gates"]
        .as_object_mut()
        .unwrap()
        .remove(&gate.to_string())
        .unwrap();
    before["gates"][&gate_key] = gate_raw;
    before["custom"] = json!([3, 1]);
    before["tags"] = json!(["z", "a", "z"]);
    before["gates"][&gate_key]["custom"] = json!({"untouched":true});
    let p = library::build_pool_library_write(
        &model,
        provenance(),
        Some("pool"),
        vec![PoolLibraryOperationSpec::Set {
            target: PoolLibraryObjectTarget::new("pool", "entities", entity),
            object: before.clone(),
        }],
    )
    .unwrap();
    commit_prepared(&mut model, &root, p).unwrap();
    let path = crate::substrate::transaction_journal_path(&root);
    let journal = std::fs::read(&path).unwrap();
    let p = edit(
        &model,
        &root,
        entity,
        json!({"gate_symbols":{gate.to_string():replacement}}),
    )
    .unwrap();
    assert_eq!(std::fs::read(&path).unwrap(), journal);
    for id in [unit, symbol, replacement] {
        assert!(p.batch.operations.iter().any(
            |op| matches!(op, Operation::GuardObjectRevision { object_id, .. } if *object_id == id)
        ));
    }
    let mut changed_reference = model.clone();
    changed_reference
        .objects
        .get_mut(&replacement)
        .unwrap()
        .object_revision
        .0 += 1;
    let refusal = changed_reference.commit(p.batch.clone()).unwrap_err();
    assert!(
        refusal.to_string().contains("object revision mismatch"),
        "{refusal}"
    );
    assert_eq!(std::fs::read(&path).unwrap(), journal);
    let stale = p.clone();
    commit_prepared(&mut model, &root, p).unwrap();
    let mut expected = before.clone();
    expected["gates"][&gate_key]["symbol"] = json!(replacement);
    assert_eq!(raw(&model, "entities", entity), expected);
    assert!(commit_prepared(&mut model, &root, stale).is_err());
    model
        .commit_journal_undo(&root, provenance().into())
        .unwrap();
    assert_eq!(raw(&model, "entities", entity), before);
    model
        .commit_journal_redo(&root, provenance().into())
        .unwrap();
    let cold = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(raw(&cold, "entities", entity), expected);
    for intent in [
        json!({"gate_symbols":{Uuid::new_v4().to_string():symbol}}),
        json!({"gate_symbols":{gate.to_string():unit}}),
        json!({"gate_symbols":{gate.to_string():Uuid::new_v4()}}),
        json!({"gate_symbols":{}}),
        json!({"gate_symbols":{gate.to_string():symbol},"gates":{}}),
    ] {
        let old = std::fs::read(&path).unwrap();
        assert!(edit(&cold, &root, entity, intent).is_err());
        assert_eq!(std::fs::read(&path).unwrap(), old);
    }
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn old_multi_gate_members_and_later_gate_retain_their_own_entity_chains() {
    use crate::api::native_write::schematic_delete_proposal::NativeProposalAction as Action;
    use placement::tests::{action, raw as sheet, setup};
    let (root, mut model, first, mut request) = setup("saved-gate-correction");
    let entity = "40000000-0000-4000-8000-000000000001"
        .parse::<Uuid>()
        .unwrap();
    let first_gate = "40000000-0000-4000-8000-000000000101"
        .parse::<Uuid>()
        .unwrap();
    let symbol = request.library_symbol;
    let assigned_gate = request.gate;
    let part = request_part(&placement::tests::binding(
        &model,
        request.component_instance,
    ));
    let assignment = action(
        &mut model,
        &root,
        Action::CreateAssignment {
            component: None,
            proposal_id: Uuid::new_v4(),
            symbol: first,
            instance_path: vec![],
            part,
            gate: assigned_gate,
        },
    );
    action(
        &mut model,
        &root,
        Action::AcceptApply {
            proposal_id: assignment.proposal_id,
        },
    );
    model
        .commit_journal_undo(&root, provenance().into())
        .unwrap();
    model
        .commit_journal_redo(&root, provenance().into())
        .unwrap();
    model = ProjectResolver::new(&root).resolve().unwrap();
    request.gate = first_gate;
    // Place a second canonical member before correction. The first already has
    // canonical birth evidence, independently of its subsequent assignment.
    let second = placement::preview(&model, &request).unwrap().symbol.uuid;
    action(
        &mut model,
        &root,
        Action::CreateGatePlacement(request.clone()),
    );
    action(
        &mut model,
        &root,
        Action::AcceptApply {
            proposal_id: request.proposal_id,
        },
    );
    // The second member's original birth remains the adoption after genuine
    // canonical creation compensation; the compensation is not another epoch.
    model
        .commit_journal_undo(&root, provenance().into())
        .unwrap();
    model
        .commit_journal_redo(&root, provenance().into())
        .unwrap();
    model = ProjectResolver::new(&root).resolve().unwrap();
    // A retracted normal deletion restores this same captured chain; it is
    // neither a second birth nor a latest-definition adoption.
    let deleted = action(
        &mut model,
        &root,
        Action::CreateDelete {
            proposal_id: Uuid::new_v4(),
            sheet: request.sheet,
            instance_path: vec![],
            targets: vec![
                crate::api::native_write::schematic_delete_proposal::DeleteTarget {
                    class: crate::api::native_write::schematic_delete_proposal::DeleteClass::Symbol,
                    id: second,
                },
            ],
        },
    );
    action(
        &mut model,
        &root,
        Action::AcceptApply {
            proposal_id: deleted.proposal_id,
        },
    );
    model
        .commit_journal_undo(&root, provenance().into())
        .unwrap();
    model
        .commit_journal_redo(&root, provenance().into())
        .unwrap();
    model
        .commit_journal_undo(&root, provenance().into())
        .unwrap();
    model = ProjectResolver::new(&root).resolve().unwrap();
    let replacement = alternate(&mut model, &root, symbol);
    let third_gate = Uuid::new_v4();
    let mut entity_raw = raw(&model, "entities", entity);
    let mut third = entity_raw["gates"][first_gate.to_string()].clone();
    third["uuid"] = json!(third_gate);
    third["name"] = json!("Third fixture gate");
    entity_raw["gates"][third_gate.to_string()] = third;
    let p = library::build_pool_library_write(
        &model,
        provenance(),
        Some("pool"),
        vec![PoolLibraryOperationSpec::Set {
            target: PoolLibraryObjectTarget::new("pool", "entities", entity),
            object: entity_raw,
        }],
    )
    .unwrap();
    commit_prepared(&mut model, &root, p).unwrap();
    let old_sheet = sheet(&model, first);
    let old_ci = placement::tests::binding(&model, request.component_instance);
    let intent = json!({"gate_symbols":{first_gate.to_string():replacement,assigned_gate.to_string():replacement,third_gate.to_string():replacement}});
    let mut broken = model.clone();
    let changed = broken
        .journal
        .iter_mut()
        .rev()
        .find_map(|tx| {
            tx.operations.iter_mut().find_map(|op| match op {
                Operation::SetSchematicSymbol {
                    symbol_id, symbol, ..
                } if *symbol_id == first => Some(symbol),
                _ => None,
            })
        })
        .unwrap();
    changed["reference"] = json!("Unauthenticated postimage");
    let journal_path = crate::substrate::transaction_journal_path(&root);
    let journal = std::fs::read(&journal_path).unwrap();
    assert!(edit(&broken, &root, entity, intent.clone()).is_err());
    assert_eq!(std::fs::read(&journal_path).unwrap(), journal);
    let p = edit(&model, &root, entity, intent).unwrap();
    commit_prepared(&mut model, &root, p).unwrap();
    assert_eq!(sheet(&model, first), old_sheet);
    assert_eq!(
        placement::tests::binding(&model, request.component_instance),
        old_ci
    );
    let mut cold = ProjectResolver::new(&root).resolve().unwrap();
    for (index, member) in [first, second].into_iter().enumerate() {
        let p = (crate::api::native_write::registry::find_native_write_verb(
            "datum.schematic.move_symbol",
        )
        .unwrap()
        .build)(
            &NativeWriteContext {
                model: &cold,
                project_root: &root,
            },
            provenance(),
            json!({"symbol":member,"instance_path":[],"x_nm":80_000_000 + index as i64 * 30_000_000,"y_nm":45_000_000,"rotation_deg":90,"mirrored":true}),
        );
        let p = p.expect("old member remains editable");
        let before = sheet(&cold, first);
        commit_prepared(&mut cold, &root, p).unwrap();
        cold.commit_journal_undo(&root, provenance().into())
            .unwrap();
        assert_eq!(sheet(&cold, first), before);
        cold.commit_journal_redo(&root, provenance().into())
            .unwrap();
        cold = ProjectResolver::new(&root).resolve().unwrap();
    }
    let choices = placement::options(&cold, first, &[]).unwrap();
    assert_eq!(choices.choices.len(), 1);
    assert_eq!(choices.choices[0].library_symbol, replacement);
    request.gate = third_gate;
    request.reference = "U3".into();
    request.library_symbol = replacement;
    request.proposal_id = Uuid::new_v4();
    // A fresh explicit component captures the corrected current Entity chain.
    let selection =
        crate::api::native_write::schematic_library::part_placement::PartGateSelection::capture(
            &cold,
            request_part(&old_ci),
            first_gate,
            replacement,
        )
        .unwrap();
    assert_eq!(selection.symbol, replacement);
    assert_eq!(selection.entity, entity);
    let expected = placement::preview(&cold, &request).unwrap().symbol;
    let retained = sheet(&cold, first)["symbols"].clone();
    action(
        &mut cold,
        &root,
        Action::CreateGatePlacement(request.clone()),
    );
    action(
        &mut cold,
        &root,
        Action::AcceptApply {
            proposal_id: request.proposal_id,
        },
    );
    let after = sheet(&cold, first);
    assert_eq!(
        after["symbols"][expected.uuid.to_string()],
        serde_json::to_value(expected).unwrap()
    );
    for (id, raw) in retained.as_object().unwrap() {
        assert_eq!(&after["symbols"][id], raw);
    }
    let cold = ProjectResolver::new(&root).resolve().unwrap();
    assert!(
        placement::options(&cold, first, &[])
            .unwrap()
            .choices
            .is_empty()
    );
    std::fs::remove_dir_all(root).unwrap();
}
fn request_part(ci: &Value) -> Uuid {
    ci["part_ref"]["object_id"]
        .as_str()
        .unwrap()
        .parse()
        .unwrap()
}

#[test]
fn compatible_references_and_other_unit_refusal_preserve_read_sources() {
    let (root, mut model, local_unit, local_symbol) = fixture();
    let (unit, old, new, entity, gate) = (
        Uuid::new_v4(),
        Uuid::new_v4(),
        Uuid::new_v4(),
        Uuid::new_v4(),
        Uuid::new_v4(),
    );
    let mut unit_raw = raw(&model, "units", local_unit);
    unit_raw["uuid"] = json!(unit);
    let pin = Uuid::new_v4();
    let mut pin_raw = unit_raw["pins"]
        .as_object()
        .unwrap()
        .values()
        .next()
        .unwrap()
        .clone();
    pin_raw["uuid"] = json!(pin);
    unit_raw["pins"] = json!({pin.to_string():pin_raw});
    let mut old_raw = raw(&model, "symbols", local_symbol);
    old_raw["uuid"] = json!(old);
    old_raw["unit"] = json!(unit);
    old_raw["pin_anchors"][0]["pin"] = json!(pin);
    old_raw["name"] = json!("Referenced old body");
    let mut new_raw = old_raw.clone();
    new_raw["uuid"] = json!(new);
    new_raw["name"] = json!("Referenced corrected body");
    let p = library::build_pool_library_write(
        &model,
        provenance(),
        Some("pool"),
        vec![
            ("units", unit, unit_raw),
            ("symbols", old, old_raw),
            ("symbols", new, new_raw),
        ]
        .into_iter()
        .map(|(kind, id, object)| PoolLibraryOperationSpec::Create {
            target: PoolLibraryObjectTarget::new("pool", kind, id),
            object,
        })
        .collect(),
    )
    .unwrap();
    commit_prepared(&mut model, &root, p).unwrap();
    let p = prepare(&model, &root, "entities", entity, fields(gate, unit, old)).unwrap();
    commit_prepared(&mut model, &root, p).unwrap();
    let paths = [
        format!("pool/units/{unit}.json"),
        format!("pool/symbols/{old}.json"),
        format!("pool/symbols/{new}.json"),
    ];
    let retained: Vec<_> = paths
        .iter()
        .map(|p| std::fs::read(root.join(p)).unwrap())
        .collect();
    let journal_path = crate::substrate::transaction_journal_path(&root);
    let before = std::fs::read(&journal_path).unwrap();
    assert!(
        edit(
            &model,
            &root,
            entity,
            json!({"gate_symbols":{gate.to_string():local_symbol}})
        )
        .is_err()
    );
    assert_eq!(std::fs::read(&journal_path).unwrap(), before);
    let p = edit(
        &model,
        &root,
        entity,
        json!({"gate_symbols":{gate.to_string():new}}),
    )
    .unwrap();
    commit_prepared(&mut model, &root, p).unwrap();
    let cold = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(
        raw(&cold, "entities", entity)["gates"][gate.to_string()]["symbol"],
        json!(new)
    );
    for (p, expected) in paths.iter().zip(retained) {
        assert_eq!(std::fs::read(root.join(p)).unwrap(), expected);
    }
    std::fs::remove_dir_all(root).unwrap();
}
