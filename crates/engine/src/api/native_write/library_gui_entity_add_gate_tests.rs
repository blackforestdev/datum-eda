//! Fresh saved gates preserve existing raw memberships and join canonical history.
use super::{tests::*, *};
use crate::api::native_write::{commit_prepared, schematic_gate_placement as placement};
use crate::substrate::{DesignModel, ProjectResolver};
use std::path::Path;
fn raw(model: &DesignModel, id: Uuid) -> Value {
    library::pool_library_object_payload(
        model,
        &PoolLibraryObjectTarget::new("pool", "entities", id).relative_path,
        id,
    )
    .unwrap()
}
fn add(
    model: &DesignModel,
    root: &Path,
    entity: Uuid,
    rows: Value,
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
            fields: json!({"gate_additions":rows}).as_object().unwrap().clone(),
        },
        false,
    )
}
fn row(id: Uuid, unit: Uuid, symbol: Uuid) -> Value {
    json!({"uuid":id,"name":"Supply","unit":unit,"symbol":symbol})
}
#[test]
fn saved_add_registers_fresh_gate_preserves_alias_raw_and_guards_history_cold() {
    let (root, mut model, unit, symbol) = fixture();
    let (entity, old, new) = (Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4());
    let p = prepare(&model, &root, "entities", entity, fields(old, unit, symbol)).unwrap();
    commit_prepared(&mut model, &root, p).unwrap();
    let mut original = raw(&model, entity);
    let alias = old.simple().to_string().to_uppercase();
    let gate = original["gates"]
        .as_object_mut()
        .unwrap()
        .remove(&old.to_string())
        .unwrap();
    original["gates"][&alias] = gate;
    original["gates"][&alias]["custom"] = json!([3, 1]);
    original["custom"] = json!(true);
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
    let revision = model.objects[&old].object_revision;
    let rows = json!({new.to_string():row(new,unit,symbol)});
    let journal = crate::substrate::transaction_journal_path(&root);
    let bytes = std::fs::read(&journal).unwrap();
    let p = add(&model, &root, entity, rows.clone()).unwrap();
    assert_eq!(std::fs::read(&journal).unwrap(), bytes);
    assert_eq!(raw(&model, entity), original);
    for id in [unit, symbol] {
        assert!(
            p.batch.operations.iter().any(
                |op| matches!(op,Operation::GuardObjectRevision{object_id,..} if *object_id==id)
            )
        );
    }
    let mut changed = model.clone();
    changed.objects.get_mut(&unit).unwrap().object_revision.0 += 1;
    assert!(changed.commit(p.batch.clone()).is_err());
    let stale = p.clone();
    commit_prepared(&mut model, &root, p).unwrap();
    assert!(commit_prepared(&mut model, &root, stale).is_err());
    let mut expected = original.clone();
    expected["gates"][new.to_string()] = rows[new.to_string()].clone();
    assert_eq!(raw(&model, entity), expected);
    assert_eq!(model.objects[&old].object_revision, revision);
    assert_eq!(
        model.objects[&new].source_shard_id,
        model.objects[&entity].source_shard_id
    );
    model
        .commit_journal_undo(&root, provenance().into())
        .unwrap();
    assert_eq!(raw(&model, entity), original);
    assert!(!model.objects.contains_key(&new));
    model
        .commit_journal_redo(&root, provenance().into())
        .unwrap();
    model = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(raw(&model, entity), expected);
    assert_eq!(model.objects[&old].object_revision, revision);
    for id in [old, entity, unit, symbol, Uuid::nil()] {
        assert!(
            add(
                &model,
                &root,
                entity,
                json!({id.to_string():row(id,unit,symbol)})
            )
            .is_err()
        );
    }
    let bytes = std::fs::read(&journal).unwrap();
    for rows in [
        json!({}),
        json!({new.to_string():row(new,unit,symbol)}),
        json!({Uuid::new_v4().to_string():row(Uuid::new_v4(),unit,symbol)}),
    ] {
        assert!(add(&model, &root, entity, rows).is_err());
        assert_eq!(std::fs::read(&journal).unwrap(), bytes);
    }
    std::fs::remove_dir_all(root).unwrap();
}
#[test]
fn new_gate_is_available_for_existing_component_remaining_gate_placement() {
    use crate::api::native_write::schematic_delete_proposal::NativeProposalAction as Action;
    let (root, mut model, first, mut request) = placement::tests::setup("saved-add-remaining-gate");
    let options = placement::options(&model, first, &[]).unwrap();
    let entity = options.entity;
    let before = raw(&model, entity);
    let id = Uuid::new_v4();
    let existing = before["gates"]
        .as_object()
        .unwrap()
        .values()
        .next()
        .unwrap();
    let unit: Uuid = serde_json::from_value(existing["unit"].clone()).unwrap();
    let symbol: Uuid = serde_json::from_value(existing["symbol"].clone()).unwrap();
    let old_sheet = placement::tests::raw(&model, first);
    let old_ci = placement::tests::binding(&model, request.component_instance);
    let p = add(
        &model,
        &root,
        entity,
        json!({id.to_string():row(id,unit,symbol)}),
    )
    .unwrap();
    commit_prepared(&mut model, &root, p).unwrap();
    assert_eq!(placement::tests::raw(&model, first), old_sheet);
    assert_eq!(
        placement::tests::binding(&model, request.component_instance),
        old_ci
    );
    model = ProjectResolver::new(&root).resolve().unwrap();
    assert!(
        placement::options(&model, first, &[])
            .unwrap()
            .choices
            .iter()
            .any(|c| c.gate == id)
    );
    request.gate = id;
    request.library_symbol = symbol;
    request.reference = "U3".into();
    let fresh = placement::preview(&model, &request).unwrap().symbol.uuid;
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
    let sheet = placement::tests::raw(&model, first);
    assert_eq!(
        sheet["symbols"][first.to_string()],
        old_sheet["symbols"][first.to_string()]
    );
    assert_eq!(sheet["symbols"][fresh.to_string()]["gate"], json!(id));
    let ci = placement::tests::binding(&model, request.component_instance);
    assert_eq!(
        ci["component_instance"]["part_ref"],
        old_ci["component_instance"]["part_ref"]
    );
    assert_eq!(
        ci["component_instance"]["library_bindings"],
        old_ci["component_instance"]["library_bindings"]
    );
    std::fs::remove_dir_all(root).unwrap();
}
