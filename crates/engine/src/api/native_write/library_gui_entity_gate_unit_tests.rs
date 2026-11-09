//! Explicit pair changes preserve old placed chains and validate prospective links.
use super::{tests::*, *};
use crate::api::native_write::{commit_prepared, schematic_gate_placement as placement};
use crate::substrate::{DesignModel, ProjectResolver};
use std::path::Path;
fn raw(model: &DesignModel, kind: &str, id: Uuid) -> Value {
    library::pool_library_object_payload(
        model,
        &PoolLibraryObjectTarget::new("pool", kind, id).relative_path,
        id,
    )
    .unwrap()
}
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
fn pair(model: &mut DesignModel, root: &Path) -> (Uuid, Uuid) {
    let (unit, pin, symbol) = (Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4());
    let p=prepare(model,root,"units",unit,json!({"name":"Revised Unit","pins":{pin.to_string():{"name":"NEW","electrical_type":"Passive"}}})).unwrap();
    commit_prepared(model, root, p).unwrap();
    let p=prepare(model,root,"symbols",symbol,json!({"name":"Revised body","unit":unit,
        "drawings":[{"Rect":{"min":{"x":-1000000,"y":-1000000},"max":{"x":3000000,"y":1000000},"width":100000}}],
        "pin_anchors":[{"pin":pin,"position":{"x":-2000000,"y":0},"style":{"orientation":"Right","length_nm":1000000,"decoration":"none"}}]})).unwrap();
    commit_prepared(model, root, p).unwrap();
    (unit, symbol)
}
#[test]
fn saved_pair_is_atomic_raw_sparse_guarded_and_survives_history_cold() {
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
    let (next, body) = pair(&mut model, &root);
    let mut original = raw(&model, "entities", entity);
    let alias = gate.simple().to_string().to_uppercase();
    let row = original["gates"]
        .as_object_mut()
        .unwrap()
        .remove(&gate.to_string())
        .unwrap();
    original["gates"][&alias] = row;
    original["gates"][&alias]["custom"] = json!([3, 1]);
    let p = library::build_pool_library_write(
        &model,
        provenance(),
        Some("pool"),
        vec![PoolLibraryOperationSpec::Set {
            target: PoolLibraryObjectTarget::new("pool", "entities", entity),
            object: original.clone(),
        }],
    )
    .unwrap();
    commit_prepared(&mut model, &root, p).unwrap();
    let journal = crate::substrate::transaction_journal_path(&root);
    let bytes = std::fs::read(&journal).unwrap();
    for invalid in [
        json!({"gate_units":{gate.to_string():next}}),
        json!({"gate_symbols":{gate.to_string():body}}),
        json!({"gate_units":{gate.to_string():symbol},"gate_symbols":{gate.to_string():body}}),
        json!({"gate_units":{}}),
        json!({"gate_units":{gate.to_string():next,alias.clone():next}}),
    ] {
        assert!(edit(&model, &root, entity, invalid).is_err());
        assert_eq!(std::fs::read(&journal).unwrap(), bytes);
    }
    let p = edit(
        &model,
        &root,
        entity,
        json!({"gate_units":{gate.to_string():next},"gate_symbols":{gate.to_string():body}}),
    )
    .unwrap();
    for id in [unit, symbol, next, body] {
        assert!(
            p.batch.operations.iter().any(
                |op| matches!(op,Operation::GuardObjectRevision{object_id,..} if *object_id==id)
            )
        );
    }
    let mut stale = model.clone();
    stale.objects.get_mut(&next).unwrap().object_revision.0 += 1;
    assert!(stale.commit(p.batch.clone()).is_err());
    commit_prepared(&mut model, &root, p).unwrap();
    let mut expected = original.clone();
    expected["gates"][&alias]["unit"] = json!(next);
    expected["gates"][&alias]["symbol"] = json!(body);
    assert_eq!(raw(&model, "entities", entity), expected);
    model
        .commit_journal_undo(&root, provenance().into())
        .unwrap();
    assert_eq!(raw(&model, "entities", entity), original);
    model
        .commit_journal_redo(&root, provenance().into())
        .unwrap();
    model = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(raw(&model, "entities", entity), expected);
    std::fs::remove_dir_all(root).unwrap();
}
#[test]
fn repeated_pair_corrections_preserve_original_multigate_users_and_editability() {
    let (root, mut model, first, request) = placement::tests::setup("entity-unit-repeated");
    let options = placement::options(&model, first, &[]).unwrap();
    let entity = options.entity;
    // The native fixture's original gate has retained mappings. Its second
    // gate is unmapped, so correcting it preserves those compatible mappings.
    let gate = request.gate;
    let second = placement::preview(&model, &request).unwrap().symbol.uuid;
    use crate::api::native_write::schematic_delete_proposal::NativeProposalAction as Action;
    placement::tests::action(
        &mut model,
        &root,
        Action::CreateGatePlacement(request.clone()),
    );
    placement::tests::action(
        &mut model,
        &root,
        Action::AcceptApply {
            proposal_id: request.proposal_id,
        },
    );
    let old = placement::tests::raw(&model, first);
    let ci = placement::tests::binding(&model, request.component_instance);
    for _ in 0..2 {
        let (unit, symbol) = pair(&mut model, &root);
        let p = edit(
            &model,
            &root,
            entity,
            json!({"gate_units":{gate.to_string():unit},"gate_symbols":{gate.to_string():symbol}}),
        )
        .unwrap();
        commit_prepared(&mut model, &root, p).unwrap();
        assert_eq!(placement::tests::raw(&model, first), old);
        assert_eq!(
            placement::tests::binding(&model, request.component_instance),
            ci
        );
        model = ProjectResolver::new(&root).resolve().unwrap();
    }
    let p=(crate::api::native_write::registry::find_native_write_verb("datum.schematic.move_symbol").unwrap().build)(
        &NativeWriteContext{model:&model,project_root:&root},provenance(),json!({"symbol":second,"instance_path":[],"x_nm":80000000,"y_nm":45000000,"rotation_deg":90,"mirrored":true})).expect("old member remains editable");
    commit_prepared(&mut model, &root, p).unwrap();
    model
        .commit_journal_undo(&root, provenance().into())
        .unwrap();
    assert_eq!(placement::tests::raw(&model, first), old);
    std::fs::remove_dir_all(root).unwrap();
}
