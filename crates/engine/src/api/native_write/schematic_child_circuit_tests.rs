//! Unique child authoring uses real native library definitions and canonical history.
use super::*;
use crate::api::native_write::{
    commit_prepared,
    electrical::topology_tests,
    library::{self, PoolLibraryObjectTarget as Target, PoolLibraryOperationSpec as Spec},
    registry::find_native_write_verb,
    schematic_child_sheet::{self, ChildSheetRequest},
};
use crate::substrate::{
    CommitSource, ComponentCounterpartStatus, ElectricalOccurrence, ProjectResolver,
};
use serde_json::{Value, json};
use std::path::Path;
fn provenance() -> WriteProvenance {
    WriteProvenance::new("test", CommitSource::Manual, "unique child circuit")
}
fn prepare(
    model: &DesignModel,
    root: &Path,
    verb: &str,
    params: Value,
) -> Result<PreparedWrite, EngineError> {
    (find_native_write_verb(verb).unwrap().build)(
        &NativeWriteContext {
            model,
            project_root: root,
        },
        provenance(),
        params,
    )
}
fn binding(model: &DesignModel, symbol: Uuid, path: &[Uuid]) -> Uuid {
    let occurrence = ElectricalOccurrence {
        class: "symbols".into(),
        source_id: symbol,
        instance_path: path.into(),
    };
    let snapshot = ElectricalSelectionSnapshot::capture(model).unwrap();
    let context = snapshot
        .component_selection_context(&model.model_revision, &BTreeSet::from([occurrence.clone()]))
        .unwrap();
    assert!(context.unavailable.is_empty(), "{:?}", context.unavailable);
    assert_eq!(
        context.bindings[&occurrence].counterpart_status,
        ComponentCounterpartStatus::NotDeclared
    );
    context.bindings[&occurrence].component_instance
}
#[test]
fn unique_child_text_placement_refusal_and_cold_history() {
    let (root, mut model, _, parent) = topology_tests::fixture("unique_child_text");
    let child = ChildSheetRequest {
        proposal_id: Uuid::new_v4(),
        parent,
        instance_path: vec![],
        sheet: Uuid::new_v4(),
        definition: Uuid::new_v4(),
        instance: Uuid::new_v4(),
        name: "Notes".into(),
        position: Point::new(0, 0),
    };
    let write = schematic_child_sheet::build(&model, &child).unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    let before = schematic(&model).unwrap();
    let params = |path: Vec<Uuid>| {
        json!({"sheet":child.sheet,"instance_path":path,
        "value":"  Literal ${CHILD} note  ","x_nm":10_000_000,"y_nm":20_000_000,"rotation_deg":90})
    };
    for path in [vec![], vec![Uuid::new_v4()]] {
        assert!(prepare(&model, &root, "datum.schematic.place_text", params(path)).is_err());
    }
    let write = prepare(
        &model,
        &root,
        "datum.schematic.place_text",
        params(vec![child.instance]),
    )
    .unwrap();
    let id = write.primary_object_id.unwrap();
    assert_eq!(
        schematic(&model).unwrap(),
        before,
        "preparing writes no source"
    );
    commit_prepared(&mut model, &root, write).unwrap();
    let after = schematic(&model).unwrap();
    assert_eq!(after.sheets[&parent], before.sheets[&parent]);
    let text = &after.sheets[&child.sheet].texts[&id];
    assert_eq!(text.text, "  Literal ${CHILD} note  ");
    assert_eq!(text.position, Point::new(10_000_000, 20_000_000));
    assert_eq!(text.rotation, 90);
    model = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(schematic(&model).unwrap(), after);
    model
        .commit_journal_undo(&root, provenance().into())
        .unwrap();
    model = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(schematic(&model).unwrap(), before);
    model
        .commit_journal_redo(&root, provenance().into())
        .unwrap();
    model = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(schematic(&model).unwrap(), after);
}
#[test]
fn unique_child_circuit_place_wire_fields_refusal_and_cold_history() {
    let (root, mut model, _, parent) = topology_tests::fixture("unique_child_circuit");
    let baseline = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../test-harness/testdata/library/native_authored_baseline_v1/pool");
    let mut objects = vec![];
    for kind in [
        "units",
        "symbols",
        "entities",
        "packages",
        "padstacks",
        "footprints",
        "parts",
        "pin_pad_maps",
    ] {
        for entry in std::fs::read_dir(baseline.join(kind)).unwrap() {
            let object: Value =
                serde_json::from_slice(&std::fs::read(entry.unwrap().path()).unwrap()).unwrap();
            objects.push(Spec::Create {
                target: Target::new(
                    "pool",
                    kind,
                    object["uuid"].as_str().unwrap().parse().unwrap(),
                ),
                object,
            });
        }
    }
    let write =
        library::build_pool_library_write(&model, provenance(), Some("pool"), objects).unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    let child = ChildSheetRequest {
        proposal_id: Uuid::new_v4(),
        parent,
        instance_path: vec![],
        sheet: Uuid::new_v4(),
        definition: Uuid::new_v4(),
        instance: Uuid::new_v4(),
        name: "Amplifier".into(),
        position: Point::new(0, 0),
    };
    let write = schematic_child_sheet::build(&model, &child).unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    let original = schematic(&model).unwrap();
    let path = vec![child.instance];
    let library_symbol = "30000000-0000-4000-8000-000000000001";
    let place = |reference: &str, x: i64, path: &[Uuid]| json!({"sheet":child.sheet,"instance_path":path,"library_symbol":library_symbol,"reference":reference,"value":"OPAMP","x_nm":x,"y_nm":20_000_000});
    let wire = |path: &[Uuid]| json!({"sheet":child.sheet,"instance_path":path,"points":[{"x":0,"y":0},{"x":1_000_000,"y":0}]});
    let tip = model.journal.len();
    for wrong in [vec![], vec![Uuid::new_v4()]] {
        assert!(
            prepare(
                &model,
                &root,
                "datum.schematic.place_symbol",
                place("U_CHILD1", 10_000_000, &wrong)
            )
            .is_err()
        );
        assert!(prepare(&model, &root, "datum.schematic.draw_wire", wire(&wrong)).is_err());
    }
    assert_eq!(model.journal.len(), tip);
    let stale = prepare(&model, &root, "datum.schematic.draw_wire", wire(&path)).unwrap();
    let mut symbols = vec![];
    let mut owners = vec![];
    for (name, x) in [("U_CHILD1", 10_000_000), ("U_CHILD2", 40_000_000)] {
        let write = prepare(
            &model,
            &root,
            "datum.schematic.place_symbol",
            place(name, x, &path),
        )
        .unwrap();
        let symbol = write.primary_object_id.unwrap();
        commit_prepared(&mut model, &root, write).unwrap();
        symbols.push(symbol);
        owners.push(binding(&model, symbol, &path));
    }
    let before_stale = schematic(&model).unwrap();
    assert!(commit_prepared(&mut model, &root, stale).is_err());
    assert_eq!(schematic(&model).unwrap(), before_stale);
    let pins: Vec<_> = symbols
        .iter()
        .map(|id| schematic(&model).unwrap().sheets[&child.sheet].symbols[id].pins[0].position)
        .collect();
    let write = prepare(
        &model,
        &root,
        "datum.schematic.draw_wire",
        json!({"sheet":child.sheet,"instance_path":path,"points":pins}),
    )
    .unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    let write=prepare(&model,&root,"datum.schematic.set_symbol_reference",json!({"symbol":symbols[0],"instance_path":path,"reference":"U_CHILD3","value":"Amplifier stage"})).unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    let expected = schematic(&model).unwrap();
    assert_eq!(expected.sheets[&parent], original.sheets[&parent]);
    assert_eq!(expected.sheets[&child.sheet].wires.len(), 1);
    let identities: BTreeSet<_> = model.electrical_identities.keys().copied().collect();
    for _ in 0..2 {
        model = ProjectResolver::new(&root).resolve().unwrap();
        for _ in 0..4 {
            model
                .commit_journal_undo(&root, provenance().into())
                .unwrap();
            model = ProjectResolver::new(&root).resolve().unwrap();
        }
        assert_eq!(schematic(&model).unwrap(), original);
        for _ in 0..4 {
            model
                .commit_journal_redo(&root, provenance().into())
                .unwrap();
            model = ProjectResolver::new(&root).resolve().unwrap();
        }
        assert_eq!(schematic(&model).unwrap(), expected);
        assert_eq!(
            model
                .electrical_identities
                .keys()
                .copied()
                .collect::<BTreeSet<_>>(),
            identities
        );
        for (symbol, owner) in symbols.iter().zip(&owners) {
            assert_eq!(binding(&model, *symbol, &path), *owner);
        }
    }
    // The same real child and its bound owners support the existing pose lifecycle.
    let partitions = crate::substrate::electrical_test_partitions(&model, &[]).unwrap();
    let moves = [
        json!({"symbol":symbols[0],"instance_path":path,"x_nm":11_000_000,"y_nm":21_000_000}),
        json!({"symbol":symbols[0],"instance_path":path,"x_nm":11_000_000,"y_nm":21_000_000,"rotation_deg":90}),
        json!({"symbol":symbols[0],"instance_path":path,"x_nm":11_000_000,"y_nm":21_000_000,"mirrored":true}),
        json!({"symbols":symbols,"instance_path":path,"reference":{"x":11_000_000,"y":21_000_000},
            "x_nm":12_000_000,"y_nm":22_000_000,"rotation_deg":90,"mirrored":true}),
    ];
    for params in moves {
        let before = schematic(&model).unwrap();
        let write = prepare(&model, &root, "datum.schematic.move_symbol", params).unwrap();
        assert_eq!(
            schematic(&model).unwrap(),
            before,
            "preview/cancel writes nothing"
        );
        commit_prepared(&mut model, &root, write).unwrap();
        model = ProjectResolver::new(&root).resolve().unwrap();
        let after = schematic(&model).unwrap();
        assert_eq!(after.sheets[&parent], original.sheets[&parent]);
        assert_eq!(
            after.sheets[&child.sheet].wires.keys().collect::<Vec<_>>(),
            before.sheets[&child.sheet].wires.keys().collect::<Vec<_>>()
        );
        assert_eq!(
            crate::substrate::electrical_test_partitions(&model, &[]).unwrap(),
            partitions
        );
        for (symbol, owner) in symbols.iter().zip(&owners) {
            assert_eq!(binding(&model, *symbol, &path), *owner);
            assert_eq!(
                after.sheets[&child.sheet].symbols[symbol]
                    .pins
                    .iter()
                    .map(|p| p.uuid)
                    .collect::<Vec<_>>(),
                before.sheets[&child.sheet].symbols[symbol]
                    .pins
                    .iter()
                    .map(|p| p.uuid)
                    .collect::<Vec<_>>()
            );
        }
        model
            .commit_journal_undo(&root, provenance().into())
            .unwrap();
        model = ProjectResolver::new(&root).resolve().unwrap();
        assert_eq!(schematic(&model).unwrap(), before);
        model
            .commit_journal_redo(&root, provenance().into())
            .unwrap();
        model = ProjectResolver::new(&root).resolve().unwrap();
        assert_eq!(schematic(&model).unwrap(), after);
        assert_eq!(
            model
                .electrical_identities
                .keys()
                .copied()
                .collect::<BTreeSet<_>>(),
            identities
        );
    }
    let refused_source = schematic(&model).unwrap();
    let refused_tip = model.journal.len();
    for wrong in [vec![], vec![Uuid::new_v4()]] {
        assert!(
            prepare(
                &model,
                &root,
                "datum.schematic.move_symbol",
                json!({"symbol":symbols[0],"instance_path":wrong,"x_nm":0,"y_nm":0})
            )
            .is_err()
        );
    }
    assert_eq!(schematic(&model).unwrap(), refused_source);
    assert_eq!(model.journal.len(), refused_tip);
    use crate::api::native_write::schematic_delete_proposal::{
        self as proposal, DeleteClass, DeleteTarget, NativeProposalAction as Action,
        NativeProposalRequest,
    };
    let act = |model: &mut DesignModel, action| {
        let request = NativeProposalRequest {
            project_root: root.clone(),
            expected_model_revision: model.model_revision.0.clone(),
            expected_transaction_tip: crate::revision::transaction_tip(&model.journal),
            action,
        };
        proposal::execute(model, &request).unwrap()
    };
    let before_delete = schematic(&model).unwrap();
    let draft = act(
        &mut model,
        Action::CreateDelete {
            proposal_id: Uuid::new_v4(),
            sheet: child.sheet,
            instance_path: path.clone(),
            targets: vec![DeleteTarget {
                class: DeleteClass::Symbol,
                id: symbols[0],
            }],
        },
    );
    act(
        &mut model,
        Action::AcceptApply {
            proposal_id: draft.proposal_id,
        },
    );
    let after_delete = schematic(&model).unwrap();
    assert!(
        !after_delete.sheets[&child.sheet]
            .symbols
            .contains_key(&symbols[0])
    );
    assert_eq!(
        after_delete.sheets[&child.sheet].symbols[&symbols[1]],
        before_delete.sheets[&child.sheet].symbols[&symbols[1]]
    );
    assert_eq!(after_delete.sheets[&parent], before_delete.sheets[&parent]);
    assert_eq!(binding(&model, symbols[1], &path), owners[1]);
    model
        .commit_journal_undo(&root, provenance().into())
        .unwrap();
    model = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(schematic(&model).unwrap(), before_delete);
    assert_eq!(binding(&model, symbols[0], &path), owners[0]);
    model
        .commit_journal_redo(&root, provenance().into())
        .unwrap();
    model = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(schematic(&model).unwrap(), after_delete);
    model
        .commit_journal_undo(&root, provenance().into())
        .unwrap();
    model = ProjectResolver::new(&root).resolve().unwrap();
    let mut repeated = expected.sheet_instances[&child.instance].clone();
    repeated.uuid = Uuid::new_v4();
    repeated.name = "Repeated amplifier".into();
    let write = crate::api::native_write::schematic_sheets::build_create_schematic_sheet_instance(
        &model,
        provenance(),
        expected.uuid,
        repeated.uuid,
        serde_json::to_value(&repeated).unwrap(),
    )
    .unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    let refused_source = schematic(&model).unwrap();
    let refused_tip = model.journal.len();
    assert!(
        prepare(
            &model,
            &root,
            "datum.schematic.place_symbol",
            place("U_CHILD4", 70_000_000, &path)
        )
        .is_err()
    );
    assert!(prepare(&model, &root, "datum.schematic.draw_wire", wire(&path)).is_err());
    assert!(
        prepare(
            &model,
            &root,
            "datum.schematic.set_symbol_reference",
            json!({"symbol":symbols[0],"instance_path":path,"reference":"U_CHILD4"})
        )
        .is_err()
    );
    assert!(
        prepare(
            &model,
            &root,
            "datum.schematic.move_symbol",
            json!({"symbol":symbols[0],"instance_path":path,"x_nm":0,"y_nm":0})
        )
        .is_err()
    );
    assert_eq!(schematic(&model).unwrap(), refused_source);
    assert_eq!(model.journal.len(), refused_tip);
    std::fs::remove_dir_all(root).unwrap();
}
