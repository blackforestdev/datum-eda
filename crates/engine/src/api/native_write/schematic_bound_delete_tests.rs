//! Real assigned native sources, reviewed deletion, raw history and refusals.
use super::tests::{action, create, request, source, target, verb};
use super::*;
use crate::api::native_write::{
    BatchComposer, commit_prepared, schematic_assignment::tests::fixture,
    schematic_edit::prepare_native_birth,
};
use crate::substrate::{
    ObjectRevision, ProjectResolver, Relationship, RelationshipKind, RevisionedRef,
};
use serde_json::{Value, json};
fn provenance() -> WriteProvenance {
    WriteProvenance::new("test", CommitSource::Manual, "bound delete")
}
fn raw(model: &DesignModel, id: Uuid) -> Value {
    let shard = model
        .source_shards
        .iter()
        .find(|s| s.shard_id == model.objects[&id].source_shard_id)
        .unwrap();
    model
        .materialized_source_shard_value_by_relative_path(&shard.relative_path)
        .unwrap()
}
pub(crate) fn setup(name: &str) -> (PathBuf, DesignModel, Uuid, Uuid, Uuid, Uuid) {
    let (root, mut model, symbol, part) = fixture(name);
    let draft = action(
        &mut model,
        &root,
        NativeProposalAction::CreateAssignment {
            proposal_id: Uuid::new_v4(),
            symbol,
            instance_path: vec![],
            part,
            gate: "40000000-0000-4000-8000-000000000101".parse().unwrap(),
        },
    );
    let ci = action(
        &mut model,
        &root,
        NativeProposalAction::AcceptApply {
            proposal_id: draft.proposal_id,
        },
    )
    .assignment
    .unwrap()
    .component_instance;
    verb(
        &mut model,
        &root,
        "datum.schematic.set_symbol_value",
        json!({"symbol":symbol,"instance_path":[],"value":"before deletion"}),
    );
    verb(
        &mut model,
        &root,
        "datum.schematic.move_symbol",
        json!({"symbol":symbol,"instance_path":[],"x_nm":12_000_000,"y_nm":21_000_000,"rotation_deg":90}),
    );
    let previous = raw(&model, ci)["component_instance"].clone();
    let mut next = previous.clone();
    next["object_revision"] = json!(model.objects[&ci].object_revision.0 + 1);
    for b in next["library_bindings"]
        .as_object_mut()
        .unwrap()
        .values_mut()
    {
        b["provenance"] = json!({"retain":["raw",7]});
    }
    let write = BatchComposer::compose(&model, provenance())
        .push_op(Operation::SetComponentInstance {
            component_instance_id: ci,
            previous_component_instance: previous,
            component_instance: next,
        })
        .finish()
        .unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    let sheet = source(&model)
        .sheets
        .values()
        .find(|s| s.symbols.contains_key(&symbol))
        .unwrap()
        .uuid;
    (root, model, sheet, symbol, ci, part)
}
fn binding(model: &DesignModel, ci: Uuid, symbol: Uuid, part: Uuid) -> Value {
    assert_eq!(model.component_instances[&ci].part_ref, Some(part));
    let mut value = raw(model, ci)["component_instance"].clone();
    assert_eq!(
        value["placed_symbol_refs"][0]["object_revision"],
        json!(model.objects[&symbol].object_revision.0)
    );
    assert_eq!(
        value["object_revision"],
        json!(model.objects[&ci].object_revision.0)
    );
    value["object_revision"] = Value::Null;
    value["placed_symbol_refs"][0]["object_revision"] = Value::Null;
    value
}
#[test]
fn native_bound_delete_review_raw_history_and_reopen() {
    let (root, mut model, sheet, symbol, ci, part) = setup("bound-delete-history");
    let item = source(&model).sheets[&sheet].symbols[&symbol].clone();
    let marker = Uuid::from_u128(1);
    let wire = Uuid::from_u128(u128::MAX - 1);
    let write=BatchComposer::compose(&model,provenance())
        .push_op(Operation::CreateSchematicNoConnect {sheet_id:sheet,noconnect_id:marker,noconnect:json!({"uuid":marker,"symbol":symbol,"pin":item.pins[1].uuid,"position":item.pins[1].position,"extension":"marker"})})
        .push_op(Operation::CreateSchematicWire {sheet_id:sheet,wire_id:wire,wire:json!({"uuid":wire,"from":item.pins[0].position,"to":{"x":item.pins[0].position.x+1_000_000,"y":item.pins[0].position.y},"extension":"wire"})}).finish().unwrap();
    let write = prepare_native_birth(&model, write).unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    let before = raw(&model, symbol);
    let component = binding(&model, ci, symbol, part);
    let pool: Vec<_> = model
        .source_shards
        .iter()
        .filter(|s| s.kind == crate::substrate::SourceShardKind::Pool)
        .map(|s| (s.relative_path.clone(), std::fs::read(&s.path).unwrap()))
        .collect();
    assert_eq!(bound_symbol_owner(&model, symbol, &[]).unwrap(), Some(ci));
    let selected = vec![
        target(DeleteClass::Symbol, symbol),
        target(DeleteClass::NoConnect, marker),
    ];
    let cancel = action(
        &mut model,
        &root,
        create(Uuid::new_v4(), sheet, selected.clone()),
    );
    assert_eq!(cancel.selected_count, Some(2));
    assert_eq!(cancel.owned_count, Some(1));
    action(
        &mut model,
        &root,
        NativeProposalAction::Reject {
            proposal_id: cancel.proposal_id,
        },
    );
    assert_eq!(raw(&model, symbol), before);
    assert_eq!(binding(&model, ci, symbol, part), component);
    let draft = action(&mut model, &root, create(Uuid::new_v4(), sheet, selected));
    let count = model.journal.len();
    let applied = action(
        &mut model,
        &root,
        NativeProposalAction::AcceptApply {
            proposal_id: draft.proposal_id,
        },
    );
    assert_eq!(applied.status, ProposalStatus::Applied);
    assert_eq!(
        model.journal.len(),
        count + 2,
        "accept metadata then one design transaction"
    );
    assert!(!model.objects.contains_key(&symbol));
    assert!(!model.component_instances.contains_key(&ci));
    let after = raw(&model, wire);
    assert_eq!(after["wires"], before["wires"]);
    assert!(
        !after["noconnects"]
            .as_object()
            .unwrap()
            .contains_key(&marker.to_string())
    );
    model = ProjectResolver::new(&root).resolve().unwrap();
    for _ in 0..2 {
        model
            .commit_journal_undo(&root, provenance().into())
            .unwrap();
        model = ProjectResolver::new(&root).resolve().unwrap();
        assert_eq!(raw(&model, symbol), before);
        assert_eq!(binding(&model, ci, symbol, part), component);
        assert_eq!(
            model.proposals[&draft.proposal_id].status,
            ProposalStatus::Accepted
        );
        model
            .commit_journal_redo(&root, provenance().into())
            .unwrap();
        model = ProjectResolver::new(&root).resolve().unwrap();
        assert_eq!(raw(&model, wire), after);
        assert!(!model.objects.contains_key(&ci));
    }
    // The same owned binding composes with a complete heterogeneous selection.
    model
        .commit_journal_undo(&root, provenance().into())
        .unwrap();
    model = ProjectResolver::new(&root).resolve().unwrap();
    let draft = action(
        &mut model,
        &root,
        create(
            Uuid::new_v4(),
            sheet,
            vec![
                target(DeleteClass::Symbol, symbol),
                target(DeleteClass::Wire, wire),
            ],
        ),
    );
    action(
        &mut model,
        &root,
        NativeProposalAction::AcceptApply {
            proposal_id: draft.proposal_id,
        },
    );
    assert!(!model.objects.contains_key(&symbol));
    assert!(!model.objects.contains_key(&wire));
    model
        .commit_journal_undo(&root, provenance().into())
        .unwrap();
    model = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(raw(&model, symbol), before);
    assert_eq!(binding(&model, ci, symbol, part), component);
    for (path, bytes) in pool {
        assert_eq!(std::fs::read(root.join(path)).unwrap(), bytes);
    }
    std::fs::remove_dir_all(root).unwrap();
}
#[test]
fn native_bound_delete_refuses_external_stale_and_wrong_occurrence_atomically() {
    let (root, mut model, sheet, symbol, ci, _) = setup("bound-delete-refusal");
    let selected = vec![target(DeleteClass::Symbol, symbol)];
    assert!(bound_symbol_owner(&model, symbol, &[Uuid::new_v4()]).is_err());
    let draft = action(
        &mut model,
        &root,
        create(Uuid::new_v4(), sheet, selected.clone()),
    );
    verb(
        &mut model,
        &root,
        "datum.schematic.set_symbol_value",
        json!({"symbol":symbol,"instance_path":[],"value":"changed"}),
    );
    let req = request(
        &model,
        &root,
        NativeProposalAction::AcceptApply {
            proposal_id: draft.proposal_id,
        },
    );
    assert!(execute(&mut model, &req).is_err());
    let relationship = Relationship {
        id: Uuid::new_v4(),
        kind: RelationshipKind::SchematicOnly,
        from: vec![RevisionedRef {
            object_id: ci,
            object_revision: model.objects[&ci].object_revision,
        }],
        to: vec![],
        authored_intent: vec![],
        object_revision: ObjectRevision(0),
    };
    let write = BatchComposer::compose(&model, provenance())
        .push_op(Operation::CreateRelationship {
            relationship_id: relationship.id,
            relationship: serde_json::to_value(relationship).unwrap(),
        })
        .finish()
        .unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    let count = model.journal.len();
    let before = raw(&model, symbol);
    let req = request(&model, &root, create(Uuid::new_v4(), sheet, selected));
    assert!(
        execute(&mut model, &req)
            .unwrap_err()
            .to_string()
            .contains("dependencies")
    );
    assert_eq!(model.journal.len(), count);
    assert_eq!(raw(&model, symbol), before);
    std::fs::remove_dir_all(root).unwrap();
}

#[path = "schematic_bound_delete_refusal_tests.rs"]
mod refusals;
