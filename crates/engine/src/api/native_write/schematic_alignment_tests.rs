//! Real native source, body edges, connected writes and complete refusals.
use super::*;
use crate::api::native_write::{
    commit_prepared,
    genesis::{GenesisSpec, bootstrap_native_project},
    registry::find_native_write_verb,
    test_support::temp_project_root,
};
use crate::substrate::{CommitSource, ProjectResolver};
use serde_json::{Value, json};
use std::path::{Path, PathBuf};

fn provenance() -> WriteProvenance {
    WriteProvenance::new("test", CommitSource::Manual, "Align native Symbol bodies")
}
fn write(model: &mut DesignModel, root: &Path, verb: &str, params: Value) -> Uuid {
    let p = (find_native_write_verb(verb).unwrap().build)(
        &NativeWriteContext {
            model,
            project_root: root,
        },
        provenance(),
        params,
    )
    .unwrap();
    let id = p.primary_object_id.unwrap_or_default();
    commit_prepared(model, root, p).unwrap();
    id
}
fn fixture(name: &str) -> (PathBuf, DesignModel, Vec<Uuid>) {
    let root = temp_project_root(name);
    bootstrap_native_project(
        &root,
        GenesisSpec {
            project_name: "Body alignment".into(),
            existing_ids: None,
        },
    )
    .unwrap();
    let mut model = ProjectResolver::new(&root).resolve().unwrap();
    let (unit, pin, sheet) = (Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4());
    write(
        &mut model,
        &root,
        "datum.library.create",
        json!({"kind":"units","pool_path":"pool","object_id":unit,"fields":{"name":"Passive","pins":{pin.to_string():{"name":"Signal","electrical_type":"Passive"}}}}),
    );
    write(
        &mut model,
        &root,
        "datum.schematic.create_sheet",
        json!({"sheet":sheet,"name":"Root"}),
    );
    let mut symbols = Vec::new();
    for (i, (x, y, left, right)) in [
        (0, 0, -1000000, 1000000),
        (10000000, 4000000, -3000000, 2000000),
    ]
    .into_iter()
    .enumerate()
    {
        let library = Uuid::new_v4();
        write(
            &mut model,
            &root,
            "datum.library.create",
            json!({"kind":"symbols","pool_path":"pool","object_id":library,"fields":{"name":format!("Body{i}"),"unit":unit,"drawings":[{"Rect":{"min":{"x":left,"y":-500000},"max":{"x":right,"y":500000},"width":100000}}],"pin_anchors":[{"pin":pin,"position":{"x":-2000000,"y":0},"style":{"orientation":"Right","length_nm":1000000,"decoration":"none"}}]}}),
        );
        symbols.push(write(&mut model, &root, "datum.schematic.place_symbol", json!({"sheet":sheet,"instance_path":[],"library_symbol":library,"reference":format!("U{}",i+1),"value":"A long caption does not define body bounds","x_nm":x,"y_nm":y})));
    }
    write(
        &mut model,
        &root,
        "datum.schematic.draw_wire",
        json!({"sheet":sheet,"instance_path":[],"points":[{"x":-2000000,"y":0},{"x":8000000,"y":4000000}]}),
    );
    (root, model, symbols)
}
fn request(symbols: Vec<Uuid>) -> SymbolAlignmentRequest {
    SymbolAlignmentRequest {
        symbols,
        instance_path: vec![],
        op: AlignmentOperation::Align,
        axis: AlignmentAxis::Horizontal,
        position: AlignmentPosition::Left,
        reference: AlignmentReference::SelectionBounds,
    }
}

#[test]
fn one_locked_member_refuses_the_complete_group_without_writes() {
    use crate::{api::native_write::context::BatchComposer, substrate::Operation};
    let (root, mut model, symbols) = fixture("symbol_body_alignment_locked");
    let (basis, _) = plan_alignment(&model, &request(symbols.clone())).unwrap();
    let shard = model
        .source_shards
        .iter()
        .find(|shard| {
            model
                .materialized_source_shard_value_by_relative_path(&shard.relative_path)
                .is_ok_and(|raw| raw["uuid"] == json!(basis.sheet_id))
        })
        .unwrap();
    let raw = model
        .materialized_source_shard_value_by_relative_path(&shard.relative_path)
        .unwrap();
    let mut symbol = raw["symbols"][symbols[1].to_string()].clone();
    symbol["locked"] = json!(true);
    let p = BatchComposer::compose(&model, provenance())
        .push_op(Operation::SetSchematicSymbol {
            sheet_id: basis.sheet_id,
            symbol_id: symbols[1],
            symbol,
        })
        .finish()
        .unwrap();
    commit_prepared(&mut model, &root, p).unwrap();
    let before: Vec<_> = model
        .source_shards
        .iter()
        .map(|s| (s.path.clone(), std::fs::read(&s.path).unwrap()))
        .collect();
    let journal = std::fs::read(root.join(".datum/journal/transactions.jsonl")).unwrap();
    let error = plan_alignment(&model, &request(symbols)).unwrap_err();
    assert!(error.to_string().contains("locked Symbol"), "{error}");
    for (path, bytes) in before {
        assert_eq!(std::fs::read(path).unwrap(), bytes);
    }
    assert_eq!(
        std::fs::read(root.join(".datum/journal/transactions.jsonl")).unwrap(),
        journal
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn unequal_body_alignment_preserves_contacts_and_authenticates_cold_source() {
    let (root, mut model, symbols) = fixture("symbol_body_alignment");
    let snapshot = ElectricalSelectionSnapshot::capture(&model).unwrap();
    let sheet = snapshot
        .schematic_source()
        .unwrap()
        .sheets
        .values()
        .next()
        .unwrap();
    let sheet_id = sheet.uuid;
    let library = sheet.symbols[&symbols[0]].lib_id.clone().unwrap();
    for i in 0..6 {
        write(
            &mut model,
            &root,
            "datum.schematic.place_symbol",
            json!({
                "sheet":sheet_id,"instance_path":[],"library_symbol":library,
                "reference":format!("U{}",i+3),"value":"Unselected foreign geometry",
                "x_nm":40000000+i*5000000,"y_nm":50000000
            }),
        );
    }
    let request = request(symbols);
    let (basis, destination) = plan_alignment(&model, &request).unwrap();
    let snapshot = ElectricalSelectionSnapshot::capture(&model).unwrap();
    assert_eq!(
        basis,
        SymbolMoveSetBasis::capture(&snapshot, &request.symbols, &[]).unwrap()
    );
    let old_wires = snapshot.schematic_source().unwrap().sheets[&basis.sheet_id]
        .wires
        .clone();
    let first = &destination.symbols[0];
    let second = &destination.symbols[1];
    assert_eq!(
        body_bounds(&model, first).unwrap().min[0],
        body_bounds(&model, second).unwrap().min[0]
    );
    assert_ne!(
        first.position.x, second.position.x,
        "Body edges, not origins, align"
    );
    assert_eq!(destination.wires.len(), 1);
    let wire = &destination.wires[0];
    assert!(old_wires.contains_key(&wire.uuid));
    for symbol in &destination.symbols {
        assert!(wire.from == symbol.pins[0].position || wire.to == symbol.pins[0].position);
    }
    let p = align_symbols(
        &NativeWriteContext {
            model: &model,
            project_root: &root,
        },
        provenance(),
        serde_json::to_value(&request).unwrap(),
    )
    .unwrap();
    commit_prepared(&mut model, &root, p).unwrap();
    let cold = ProjectResolver::new(&root).resolve().unwrap();
    let (cold_basis, cold_destination) = plan_alignment(&cold, &request).unwrap();
    assert_eq!(cold_basis.symbols, cold_destination.symbols);
    assert!(
        align_symbols(
            &NativeWriteContext {
                model: &cold,
                project_root: &root
            },
            provenance(),
            serde_json::to_value(&request).unwrap()
        )
        .is_err()
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn invalid_complete_scope_and_axis_refuse_before_any_write() {
    let (root, model, symbols) = fixture("symbol_body_alignment_refusal");
    let journal = std::fs::read(root.join(".datum/journal/transactions.jsonl")).unwrap();
    for selected in [
        vec![],
        vec![symbols[0]],
        vec![symbols[0], symbols[0]],
        vec![symbols[0], Uuid::new_v4()],
    ] {
        assert!(plan_alignment(&model, &request(selected)).is_err());
    }
    let mut request = request(symbols);
    request.position = AlignmentPosition::Top;
    assert!(plan_alignment(&model, &request).is_err());
    assert_eq!(
        std::fs::read(root.join(".datum/journal/transactions.jsonl")).unwrap(),
        journal
    );
    std::fs::remove_dir_all(root).unwrap();
}
