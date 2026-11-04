use super::*;
#[test]
fn native_annotation_reserves_other_sheets_and_uses_exact_prefix_and_position_order() {
    let (root, mut model, sheet, ids, _) = setup("annotation-reservations");
    for (symbol, reference) in [(ids[0], "U9"), (ids[1], "U8"), (ids[2], "R?")] {
        verb(
            &mut model,
            &root,
            "datum.schematic.set_symbol_reference",
            json!({"symbol":symbol,"instance_path":[],"reference":reference}),
        );
    }
    let other = Uuid::new_v4();
    verb(
        &mut model,
        &root,
        "datum.schematic.create_sheet",
        json!({"sheet":other,"name":"Reserved"}),
    );
    let external = verb(
        &mut model,
        &root,
        "datum.schematic.place_symbol",
        json!({"sheet":other,"library_symbol":"30000000-0000-4000-8000-000000000001","reference":"U1","value":"OPAMP","x_nm":0,"y_nm":0}),
    );
    let untouched = raw(&model, external);
    let plan = preview(&model, sheet, &[]).unwrap();
    assert_eq!(plan, preview(&model, sheet, &[]).unwrap());
    for (symbol, reference) in [(ids[0], "U3"), (ids[1], "U2"), (ids[2], "R1")] {
        assert!(
            plan.changes
                .iter()
                .any(|c| c.symbol == symbol && c.reference == reference)
        );
    }
    annotate(&mut model, &root, sheet);
    assert_eq!(raw(&model, external), untouched);
    assert!(preview(&model, sheet, &[]).unwrap().changes.is_empty());
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn native_annotation_equal_coordinates_use_uuid_tie_break_and_repeated_sources_refuse() {
    use crate::api::native_write::schematic_sheets::{
        build_create_schematic_definition, build_create_schematic_sheet_instance,
    };
    use crate::ir::geometry::Point;
    use crate::schematic::{SheetDefinition, SheetInstance};
    let (root, mut model, sheet, ids, _) = setup("annotation-tie-occurrences");
    let snapshot = ElectricalSelectionSnapshot::capture(&model).unwrap();
    let source = snapshot.schematic_source().unwrap();
    let schematic = source.uuid;
    let originals: Vec<_> = ids
        .iter()
        .map(|id| source.sheets[&sheet].symbols[id].clone())
        .collect();
    let mut ops = vec![];
    for symbol in &originals {
        let mut value = raw(&model, symbol.uuid)["symbols"][symbol.uuid.to_string()].clone();
        value["position"]["x"] = json!(0);
        value["position"]["y"] = json!(0);
        ops.push(Operation::SetSchematicSymbol {
            sheet_id: sheet,
            symbol_id: symbol.uuid,
            symbol: value,
        });
    }
    ops.extend(repin_symbols(&model, &originals).unwrap());
    let write = BatchComposer::compose(&model, provenance())
        .push_ops(ops)
        .finish()
        .unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    let plan = preview(&model, sheet, &[]).unwrap();
    let mut ordered = ids.clone();
    ordered.sort();
    for (index, id) in ordered.iter().enumerate() {
        let actual = plan
            .changes
            .iter()
            .find(|c| c.symbol == *id)
            .map(|c| c.reference.clone())
            .unwrap_or_else(|| {
                raw(&model, *id)["symbols"][id.to_string()]["reference"]
                    .as_str()
                    .unwrap()
                    .into()
            });
        assert_eq!(actual, format!("U{}", index + 1));
    }
    let definition = SheetDefinition {
        uuid: Uuid::new_v4(),
        name: "Repeated".into(),
        root_sheet: sheet,
    };
    let write = build_create_schematic_definition(
        &model,
        provenance(),
        schematic,
        definition.uuid,
        "definitions/repeated.json",
        serde_json::to_value(&definition).unwrap(),
    )
    .unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    for i in 0..2 {
        let instance = SheetInstance {
            uuid: Uuid::new_v4(),
            definition: definition.uuid,
            parent_sheet: None,
            position: Point::new(i, 0),
            name: format!("I{i}"),
            ports: vec![],
        };
        let write = build_create_schematic_sheet_instance(
            &model,
            provenance(),
            schematic,
            instance.uuid,
            serde_json::to_value(instance).unwrap(),
        )
        .unwrap();
        commit_prepared(&mut model, &root, write).unwrap();
    }
    let before = raw(&model, ids[0]);
    assert!(preview(&model, sheet, &[]).is_err());
    assert_eq!(raw(&model, ids[0]), before);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn native_annotation_repairs_repeated_placeholders_and_preserves_reserved_references() {
    let (root, mut model, sheet, ids, _) = setup("annotation-placeholders");
    let snapshot = ElectricalSelectionSnapshot::capture(&model).unwrap();
    let originals: Vec<_> = ids
        .iter()
        .map(|id| snapshot.schematic_source().unwrap().sheets[&sheet].symbols[id].clone())
        .collect();
    let mut ops = vec![];
    for symbol in &originals {
        let mut payload = raw(&model, symbol.uuid)["symbols"][symbol.uuid.to_string()].clone();
        payload["reference"] = json!("R?");
        ops.push(Operation::SetSchematicSymbol {
            sheet_id: sheet,
            symbol_id: symbol.uuid,
            symbol: payload,
        });
    }
    ops.extend(repin_symbols(&model, &originals).unwrap());
    let write = BatchComposer::compose(&model, provenance())
        .push_ops(ops)
        .finish()
        .unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    let other = Uuid::new_v4();
    verb(
        &mut model,
        &root,
        "datum.schematic.create_sheet",
        json!({"sheet":other,"name":"Reserved"}),
    );
    let external = verb(
        &mut model,
        &root,
        "datum.schematic.place_symbol",
        json!({"sheet":other,"library_symbol":"30000000-0000-4000-8000-000000000001","reference":"R1","value":"OPAMP","x_nm":0,"y_nm":0}),
    );
    let untouched = raw(&model, external);
    let summary = preview(&model, sheet, &[]).unwrap();
    assert_eq!(summary.changes.len(), 3);
    for (symbol, reference) in [(ids[1], "R2"), (ids[0], "R3"), (ids[2], "R4")] {
        assert!(summary.changes.iter().any(|c| c.symbol == symbol
            && c.previous_reference == "R?"
            && c.reference == reference));
    }
    annotate(&mut model, &root, sheet);
    assert_eq!(raw(&model, external), untouched);
    assert!(preview(&model, sheet, &[]).unwrap().changes.is_empty());
    let duplicate = verb(
        &mut model,
        &root,
        "datum.schematic.place_symbol",
        json!({"sheet":other,"library_symbol":"30000000-0000-4000-8000-000000000001","reference":"R9","value":"OPAMP","x_nm":40_000_000,"y_nm":0}),
    );
    let mut payload = raw(&model, duplicate)["symbols"][duplicate.to_string()].clone();
    payload["reference"] = json!("R1");
    let write = BatchComposer::compose(&model, provenance())
        .push_op(Operation::SetSchematicSymbol {
            sheet_id: other,
            symbol_id: duplicate,
            symbol: payload,
        })
        .finish()
        .unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    let count = model.journal.len();
    assert!(
        preview(&model, sheet, &[]).is_err(),
        "Annotation cannot repair duplicates wholly outside its sheet"
    );
    assert_eq!(model.journal.len(), count);
    std::fs::remove_dir_all(root).unwrap();
}
