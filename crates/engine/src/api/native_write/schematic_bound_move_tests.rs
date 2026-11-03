//! Real assigned native library Symbols through the ordinary transform writer and resolver.
use super::*;
use crate::api::native_write::{
    commit_prepared,
    registry::find_native_write_verb,
    schematic_assignment::tests::fixture,
    schematic_delete_proposal::{
        self as proposal, NativeProposalAction as Action, NativeProposalRequest,
    },
    schematic_edit::prepare_native_birth,
};
use crate::substrate::{CommitSource, DesignModel, ProjectResolver, electrical_test_partitions};
use serde_json::{Value, json};
use std::path::Path;
fn provenance() -> WriteProvenance {
    WriteProvenance::new("test", CommitSource::Manual, "bound Symbol transform")
}
fn action(
    model: &mut DesignModel,
    root: &Path,
    action: Action,
) -> proposal::NativeProposalResponse {
    let request = NativeProposalRequest {
        project_root: root.into(),
        expected_model_revision: model.model_revision.0.clone(),
        expected_transaction_tip: crate::revision::transaction_tip(&model.journal),
        action,
    };
    proposal::execute(model, &request).unwrap()
}
fn assign(model: &mut DesignModel, root: &Path, symbol: Uuid, part: Uuid) -> Uuid {
    let draft = action(
        model,
        root,
        Action::CreateAssignment {
            proposal_id: Uuid::new_v4(),
            symbol,
            instance_path: vec![],
            part,
            gate: "40000000-0000-4000-8000-000000000101".parse().unwrap(),
        },
    );
    let id = draft.assignment.unwrap().component_instance;
    action(
        model,
        root,
        Action::AcceptApply {
            proposal_id: draft.proposal_id,
        },
    );
    id
}
fn native(model: &mut DesignModel, root: &Path, verb: &str, params: Value) -> Uuid {
    let write = (find_native_write_verb(verb).unwrap().build)(
        &NativeWriteContext {
            model,
            project_root: root,
        },
        provenance(),
        params,
    )
    .unwrap();
    let id = write.primary_object_id.unwrap();
    commit_prepared(model, root, write).unwrap();
    id
}
fn ci(model: &DesignModel, symbol: Uuid, id: Uuid, part: Uuid) -> Value {
    let resolved = &model.component_instances[&id];
    assert_eq!(resolved.part_ref, Some(part));
    assert_eq!(resolved.placed_symbol_refs, vec![symbol]);
    assert!(resolved.placed_package_refs.is_empty());
    let object = &model.objects[&id];
    let shard = model
        .source_shards
        .iter()
        .find(|s| s.shard_id == object.source_shard_id)
        .unwrap();
    let mut value = model
        .materialized_source_shard_value_by_relative_path(&shard.relative_path)
        .unwrap()["component_instance"]
        .clone();
    assert_eq!(
        value["placed_symbol_refs"][0]["object_revision"],
        json!(model.objects[&symbol].object_revision.0)
    );
    assert_eq!(value["object_revision"], json!(object.object_revision.0));
    value.as_object_mut().unwrap().remove("object_revision");
    value["placed_symbol_refs"][0]
        .as_object_mut()
        .unwrap()
        .remove("object_revision");
    value
}
fn raw(model: &DesignModel, sheet: Uuid, symbol: Uuid) -> Value {
    source_sheet_payload(model, sheet, symbol).unwrap()
}
#[test]
fn native_bound_move_singleton_group_raw_bindings_and_atomic_history() {
    let (root, mut model, first, part) = fixture("bound-move-history");
    let snapshot = ElectricalSelectionSnapshot::capture(&model).unwrap();
    let basis = SymbolMoveBasis::capture(&snapshot, first, &[]).unwrap();
    let sheet = basis.sheet_id;
    let second = native(
        &mut model,
        &root,
        "datum.schematic.place_symbol",
        json!({
            "sheet":sheet,"library_symbol":"30000000-0000-4000-8000-000000000001",
            "reference":"U2","value":"OPAMP","x_nm":50_000_000,"y_nm":20_000_000
        }),
    );
    let a = assign(&mut model, &root, first, part);
    let b = assign(&mut model, &root, second, part);
    let snapshot = ElectricalSelectionSnapshot::capture(&model).unwrap();
    let source = &snapshot.schematic_source().unwrap().sheets[&sheet];
    let from = source.symbols[&first].pins[0].position;
    let to = source.symbols[&second].pins[1].position;
    let wire = Uuid::new_v4();
    let write = BatchComposer::compose(&model, provenance())
        .push_op(Operation::CreateSchematicWire {
            sheet_id: sheet,
            wire_id: wire,
            wire: json!({"uuid":wire,"from":{"x":from.x,"y":from.y,"extension":"from"},
            "to":{"x":to.x,"y":to.y,"extension":"to"},"extension":"wire"}),
        })
        .finish()
        .unwrap();
    let write = prepare_native_birth(&model, write).unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    let shard = model
        .source_shards
        .iter()
        .find(|s| s.shard_id == model.objects[&a].source_shard_id)
        .unwrap();
    let previous = model
        .materialized_source_shard_value_by_relative_path(&shard.relative_path)
        .unwrap()["component_instance"]
        .clone();
    let mut extended = previous.clone();
    extended["object_revision"] = json!(model.objects[&a].object_revision.0 + 1);
    extended["placed_symbol_roles"][first.to_string()]["label"] = json!("Retained role");
    for binding in extended["library_bindings"]
        .as_object_mut()
        .unwrap()
        .values_mut()
    {
        binding["provenance"] = json!({"extension":{"source":"retain verbatim"}});
    }
    let write = BatchComposer::compose(&model, provenance())
        .push_op(Operation::SetComponentInstance {
            component_instance_id: a,
            previous_component_instance: previous,
            component_instance: extended,
        })
        .finish()
        .unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    let semantics = [ci(&model, first, a, part), ci(&model, second, b, part)];
    let identities = model.electrical_identities.clone();
    let partitions = electrical_test_partitions(&model, &[]).unwrap();
    let requests = [
        json!({"symbol":first,"instance_path":[],"x_nm":11_000_000,"y_nm":21_000_000}),
        json!({"symbol":first,"instance_path":[],"x_nm":11_000_000,"y_nm":21_000_000,"rotation_deg":90}),
        json!({"symbol":first,"instance_path":[],"x_nm":11_000_000,"y_nm":21_000_000,"mirrored":true}),
        json!({"symbols":[first,second],"instance_path":[],"reference":{"x":11_000_000,"y":21_000_000},
            "x_nm":12_000_000,"y_nm":22_000_000,"rotation_deg":90,"mirrored":true}),
    ];
    for params in requests {
        let before = raw(&model, sheet, first);
        let len = model.journal.len();
        let write = move_symbol(
            &NativeWriteContext {
                model: &model,
                project_root: &root,
            },
            provenance(),
            params,
        )
        .unwrap();
        assert_eq!(
            raw(&model, sheet, first),
            before,
            "prepare/cancel is read only"
        );
        commit_prepared(&mut model, &root, write).unwrap();
        assert_eq!(model.journal.len(), len + 1);
        model = ProjectResolver::new(&root).resolve().unwrap();
        let after = raw(&model, sheet, first);
        assert_eq!(
            after["symbols"][first.to_string()]["extension"],
            before["symbols"][first.to_string()]["extension"]
        );
        assert_eq!(
            after["symbols"][first.to_string()]["fields"][0]["position"]["extension"],
            json!(true)
        );
        assert_eq!(after["wires"][wire.to_string()]["extension"], json!("wire"));
        assert_eq!(
            [ci(&model, first, a, part), ci(&model, second, b, part)],
            semantics
        );
        assert_eq!(model.electrical_identities, identities);
        assert_eq!(electrical_test_partitions(&model, &[]).unwrap(), partitions);
        model
            .commit_journal_undo(&root, provenance().into())
            .unwrap();
        model = ProjectResolver::new(&root).resolve().unwrap();
        assert_eq!(raw(&model, sheet, first), before);
        assert_eq!(
            [ci(&model, first, a, part), ci(&model, second, b, part)],
            semantics
        );
        model
            .commit_journal_redo(&root, provenance().into())
            .unwrap();
        model = ProjectResolver::new(&root).resolve().unwrap();
        assert_eq!(raw(&model, sheet, first), after);
        assert_eq!(
            [ci(&model, first, a, part), ci(&model, second, b, part)],
            semantics
        );
        assert_eq!(electrical_test_partitions(&model, &[]).unwrap(), partitions);
    }
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn native_bound_move_refuses_stale_physical_and_multiple_symbol_owners() {
    use crate::api::native_write::board_components::{
        BoardPackagePlacement, build_place_board_package,
    };
    use crate::board::PlacedPackage;
    for mode in ["stale", "physical", "multiple"] {
        let (root, mut model, symbol, part) = fixture("bound-move-refusal");
        let id = assign(&mut model, &root, symbol, part);
        let snapshot = ElectricalSelectionSnapshot::capture(&model).unwrap();
        let basis = SymbolMoveBasis::capture(&snapshot, symbol, &[]).unwrap();
        let other = if mode == "physical" {
            let placement = BoardPackagePlacement {
                package: PlacedPackage {
                    uuid: Uuid::new_v4(),
                    part,
                    package: "50000000-0000-4000-8000-000000000001".parse().unwrap(),
                    reference: "U1".into(),
                    value: "OPAMP".into(),
                    position: Point::zero(),
                    rotation: 0,
                    layer: 1,
                    locked: false,
                },
                materialized: json!({}),
            };
            let write = build_place_board_package(&model, provenance(), &placement).unwrap();
            commit_prepared(&mut model, &root, write).unwrap();
            placement.package.uuid
        } else {
            native(
                &mut model,
                &root,
                "datum.schematic.place_symbol",
                json!({"sheet":basis.sheet_id,
                "library_symbol":"30000000-0000-4000-8000-000000000001","reference":"U2",
                "value":"OPAMP","x_nm":50_000_000,"y_nm":20_000_000}),
            )
        };
        let shard = model
            .source_shards
            .iter()
            .find(|s| s.shard_id == model.objects[&id].source_shard_id)
            .unwrap();
        let previous = model
            .materialized_source_shard_value_by_relative_path(&shard.relative_path)
            .unwrap()["component_instance"]
            .clone();
        let write = if mode == "stale" {
            // Reproduce the pre-fix path: source epoch advances without its CI ref.
            let mut payload =
                raw(&model, basis.sheet_id, symbol)["symbols"][symbol.to_string()].clone();
            payload["extension"] = json!("advanced outside transform");
            BatchComposer::compose(&model, provenance())
                .push_op(Operation::SetSchematicSymbol {
                    sheet_id: basis.sheet_id,
                    symbol_id: symbol,
                    symbol: payload,
                })
                .finish()
                .unwrap()
        } else {
            let mut next = previous.clone();
            next["object_revision"] = json!(model.objects[&id].object_revision.0 + 1);
            let field = if mode == "physical" {
                "placed_package_refs"
            } else {
                "placed_symbol_refs"
            };
            next[field].as_array_mut().unwrap().push(json!({"object_id":other,"object_revision":model.objects[&other].object_revision.0}));
            BatchComposer::compose(&model, provenance())
                .push_op(Operation::SetComponentInstance {
                    component_instance_id: id,
                    previous_component_instance: previous,
                    component_instance: next,
                })
                .finish()
                .unwrap()
        };
        commit_prepared(&mut model, &root, write).unwrap();
        model = ProjectResolver::new(&root).resolve().unwrap();
        if mode == "stale" {
            assert!(!model.component_instances.contains_key(&id));
        }
        let before = raw(&model, basis.sheet_id, symbol);
        let len = model.journal.len();
        for params in [
            json!({"symbol":symbol,"instance_path":[],"x_nm":11_000_000,"y_nm":20_000_000}),
            json!({"symbols":[symbol,other],"instance_path":[],"reference":{"x":0,"y":0},"x_nm":1_000_000,"y_nm":0}),
        ] {
            assert!(
                move_symbol(
                    &NativeWriteContext {
                        model: &model,
                        project_root: &root
                    },
                    provenance(),
                    params
                )
                .is_err()
            );
            assert_eq!(model.journal.len(), len);
            assert_eq!(raw(&model, basis.sheet_id, symbol), before);
        }
        std::fs::remove_dir_all(root).unwrap();
    }
}
