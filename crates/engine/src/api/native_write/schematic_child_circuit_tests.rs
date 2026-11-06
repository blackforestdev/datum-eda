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
fn install_library(model: &mut DesignModel, root: &Path) {
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
        library::build_pool_library_write(model, provenance(), Some("pool"), objects).unwrap();
    commit_prepared(model, root, write).unwrap();
}
#[test]
fn unique_child_field_table_and_annotation_share_canonical_review_and_history() {
    use crate::api::native_write::{
        schematic_annotation,
        schematic_delete_proposal::{
            self as proposal, NativeProposalAction as Action, NativeProposalRequest,
        },
        schematic_field_batch::{self, FieldRowEdit},
    };
    let (root, mut model, _, parent) = topology_tests::fixture("unique_child_bulk_fields");
    install_library(&mut model, &root);
    let child = ChildSheetRequest {
        proposal_id: Uuid::new_v4(),
        parent,
        instance_path: vec![],
        sheet: Uuid::new_v4(),
        definition: Uuid::new_v4(),
        instance: Uuid::new_v4(),
        name: "Child fields".into(),
        position: Point::new(0, 0),
    };
    let write = schematic_child_sheet::build(&model, &child).unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    let path = vec![child.instance];
    let mut symbols = Vec::new();
    for (reference, x) in [("U_CHILD20", 10_000_000), ("U_CHILD10", 40_000_000)] {
        let write = prepare(
            &model,
            &root,
            "datum.schematic.place_symbol",
            json!({"sheet":child.sheet,
            "instance_path":path,"library_symbol":"30000000-0000-4000-8000-000000000001",
            "reference":reference,"value":"OPAMP","x_nm":x,"y_nm":20_000_000}),
        )
        .unwrap();
        symbols.push(write.primary_object_id.unwrap());
        commit_prepared(&mut model, &root, write).unwrap();
    }
    let owners: Vec<_> = symbols.iter().map(|s| binding(&model, *s, &path)).collect();
    let original = schematic(&model).unwrap();
    let edits: Vec<_> = original.sheets[&child.sheet]
        .symbols
        .values()
        .map(|s| FieldRowEdit {
            symbol: s.uuid,
            reference: s.reference.clone(),
            value: format!("{} literal", s.reference),
        })
        .collect();
    for wrong in [vec![], vec![Uuid::new_v4()]] {
        assert!(schematic_field_batch::preview(&model, child.sheet, &wrong, &edits).is_err());
        assert!(schematic_annotation::preview(&model, child.sheet, &wrong).is_err());
    }
    let preview = schematic_field_batch::preview(&model, child.sheet, &path, &edits).unwrap();
    assert_eq!(preview.instance_path, path);
    assert_eq!(preview.changes.len(), 2);
    let execute = |model: &mut DesignModel, action| {
        let request = NativeProposalRequest {
            project_root: root.clone(),
            expected_model_revision: model.model_revision.0.clone(),
            expected_transaction_tip: crate::revision::transaction_tip(&model.journal),
            action,
        };
        proposal::execute(model, &request).unwrap()
    };
    let id = Uuid::new_v4();
    execute(
        &mut model,
        Action::CreateFieldBatch {
            proposal_id: id,
            sheet: child.sheet,
            instance_path: path.clone(),
            edits: edits.clone(),
        },
    );
    assert_eq!(
        schematic(&model).unwrap(),
        original,
        "draft changes no design source"
    );
    execute(&mut model, Action::Reject { proposal_id: id });
    assert_eq!(
        schematic(&model).unwrap(),
        original,
        "cancel preserves all source"
    );
    let id = Uuid::new_v4();
    execute(
        &mut model,
        Action::CreateFieldBatch {
            proposal_id: id,
            sheet: child.sheet,
            instance_path: path.clone(),
            edits,
        },
    );
    execute(&mut model, Action::AcceptApply { proposal_id: id });
    let fields = schematic(&model).unwrap();
    assert_eq!(fields.sheets[&parent], original.sheets[&parent]);
    model = ProjectResolver::new(&root).resolve().unwrap();
    model
        .commit_journal_undo(&root, provenance().into())
        .unwrap();
    model = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(schematic(&model).unwrap(), original);
    model
        .commit_journal_redo(&root, provenance().into())
        .unwrap();
    model = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(schematic(&model).unwrap(), fields);
    let annotation = schematic_annotation::preview(&model, child.sheet, &path).unwrap();
    assert_eq!(annotation.instance_path, path);
    assert_eq!(annotation.symbol_count, 2);
    let id = Uuid::new_v4();
    execute(
        &mut model,
        Action::CreateAnnotation {
            proposal_id: id,
            sheet: child.sheet,
            instance_path: path.clone(),
        },
    );
    assert_eq!(schematic(&model).unwrap(), fields);
    execute(&mut model, Action::AcceptApply { proposal_id: id });
    let after = schematic(&model).unwrap();
    assert_eq!(after.sheets[&parent], original.sheets[&parent]);
    assert_eq!(
        after.sheets[&child.sheet]
            .symbols
            .values()
            .map(|s| s.reference.as_str())
            .collect::<BTreeSet<_>>(),
        BTreeSet::from(["U_CHILD1", "U_CHILD2"])
    );
    model = ProjectResolver::new(&root).resolve().unwrap();
    // Each accepted design edit is one history action; intervening Proposal
    // metadata retains the established canonical history semantics.
    model
        .commit_journal_undo(&root, provenance().into())
        .unwrap();
    model = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(schematic(&model).unwrap(), fields);
    model
        .commit_journal_redo(&root, provenance().into())
        .unwrap();
    model = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(schematic(&model).unwrap(), after);
    for (symbol, owner) in symbols.iter().zip(owners) {
        assert_eq!(binding(&model, *symbol, &path), owner);
    }
}
#[test]
fn unique_child_point_annotations_preserve_source_and_cold_history() {
    let (root, mut model, _, parent) = topology_tests::fixture("unique_child_annotations");
    let child = ChildSheetRequest {
        proposal_id: Uuid::new_v4(),
        parent,
        instance_path: vec![],
        sheet: Uuid::new_v4(),
        definition: Uuid::new_v4(),
        instance: Uuid::new_v4(),
        name: "Annotated child".into(),
        position: Point::new(0, 0),
    };
    let write = schematic_child_sheet::build(&model, &child).unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    let before = schematic(&model).unwrap();
    let mut writes = 0;
    for (index, kind) in ["local", "global", "power", "hierarchical"]
        .into_iter()
        .enumerate()
    {
        let params = |path: Vec<Uuid>| {
            json!({"sheet":child.sheet,"instance_path":path,
            "name":format!("CHILD_{kind}"),"kind":kind,"x_nm":(index+1)*10_000_000,"y_nm":20_000_000})
        };
        for wrong in [vec![], vec![Uuid::new_v4()]] {
            assert!(prepare(&model, &root, "datum.schematic.place_label", params(wrong)).is_err());
        }
        let unchanged = schematic(&model).unwrap();
        let write = prepare(
            &model,
            &root,
            "datum.schematic.place_label",
            params(vec![child.instance]),
        )
        .unwrap();
        assert_eq!(
            schematic(&model).unwrap(),
            unchanged,
            "label preview creates no source"
        );
        commit_prepared(&mut model, &root, write).unwrap();
        writes += 1;
    }
    let port_params = |path: Vec<Uuid>| {
        json!({"sheet":child.sheet,"instance_path":path,
        "name":"NESTED_IN","direction":"input","x_nm":0,"y_nm":0})
    };
    for wrong in [vec![], vec![Uuid::new_v4()]] {
        assert!(
            prepare(
                &model,
                &root,
                "datum.schematic.place_port",
                port_params(wrong)
            )
            .is_err()
        );
    }
    let write = prepare(
        &model,
        &root,
        "datum.schematic.place_port",
        port_params(vec![child.instance]),
    )
    .unwrap();
    let port = write.primary_object_id.unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    writes += 1;
    let write = prepare(
        &model,
        &root,
        "datum.schematic.place_port",
        json!({"sheet":child.sheet,
        "port":port,"instance_path":[child.instance],"name":"NESTED_SIGNAL",
        "direction":"bidirectional","x_nm":1_000_000,"y_nm":2_000_000}),
    )
    .unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    writes += 1;
    let after = schematic(&model).unwrap();
    assert_eq!(after.sheets[&parent], before.sheets[&parent]);
    let source = &after.sheets[&child.sheet];
    for (name, kind) in [
        ("local", LabelKind::Local),
        ("global", LabelKind::Global),
        ("power", LabelKind::Power),
        ("hierarchical", LabelKind::Hierarchical),
    ] {
        let label = source
            .labels
            .values()
            .find(|l| l.name == format!("CHILD_{name}"))
            .unwrap();
        assert_eq!(label.kind, kind);
    }
    assert_eq!(source.ports[&port].name, "NESTED_SIGNAL");
    assert_eq!(
        source.ports[&port].position,
        Point::new(1_000_000, 2_000_000)
    );
    model = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(schematic(&model).unwrap(), after);
    for _ in 0..writes {
        model
            .commit_journal_undo(&root, provenance().into())
            .unwrap();
        model = ProjectResolver::new(&root).resolve().unwrap();
    }
    assert_eq!(schematic(&model).unwrap(), before);
    for _ in 0..writes {
        model
            .commit_journal_redo(&root, provenance().into())
            .unwrap();
        model = ProjectResolver::new(&root).resolve().unwrap();
    }
    assert_eq!(schematic(&model).unwrap(), after);
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
    install_library(&mut model, &root);
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
