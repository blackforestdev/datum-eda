//! Real connected Label/Power source metadata survives ordinary Move and history.
use super::*;
use serde_json::{Value, json};

fn sheet_payload(model: &DesignModel, sheet: Uuid, label: Uuid) -> Value {
    source_sheet_payload(model, sheet, label).unwrap()
}

#[test]
fn label_move_preserves_raw_connected_sources_and_exact_history() {
    for kind in [LabelKind::Local, LabelKind::Power] {
        let mut f = fixture("label_move_raw_history");
        let moving = label(&mut f.model, &f.root, f.sheet, Point::new(1000, 0), kind);
        let a = wire(
            &mut f.model,
            &f.root,
            f.sheet,
            Point::zero(),
            moving.position,
        );
        let b = wire(
            &mut f.model,
            &f.root,
            f.sheet,
            moving.position,
            Point::new(1000, -1000),
        );
        let original = sheet_payload(&f.model, f.sheet, moving.uuid);
        let key = moving.uuid.to_string();
        let mut label_raw = original["labels"][&key].clone();
        label_raw["source_extension"] = json!({"text":"keep literal", "unknown":[false,7]});
        label_raw["position"]["source_coordinate"] = json!({"units":"nm", "literal":"1000,0"});
        let mut operations = vec![Operation::SetSchematicLabel {
            sheet_id: f.sheet,
            label_id: moving.uuid,
            label: label_raw,
        }];
        for (index, w) in [a.clone(), b.clone()].iter().enumerate() {
            let old = original["wires"][w.uuid.to_string()].clone();
            let mut raw = old.clone();
            raw["source_extension"] = json!({"wire":index, "future":[1,2,3]});
            raw["from"]["source_coordinate"] = json!(["from", index]);
            raw["to"]["source_coordinate"] = json!(["to", index]);
            operations.push(Operation::DeleteSchematicWire {
                sheet_id: f.sheet,
                wire_id: w.uuid,
                wire: old,
            });
            operations.push(Operation::CreateSchematicWire {
                sheet_id: f.sheet,
                wire_id: w.uuid,
                wire: raw,
            });
        }
        let write = BatchComposer::compose(&f.model, provenance())
            .push_ops(operations)
            .finish()
            .unwrap();
        commit_prepared(&mut f.model, &f.root, write).unwrap();
        f.model = ProjectResolver::new(&f.root).resolve().unwrap();
        let before = sheet_payload(&f.model, f.sheet, moving.uuid);
        let partitions = electrical_test_partitions(&f.model, &[]).unwrap();
        let identities = f.model.electrical_identities.clone();
        let count = f.model.journal.len();
        assert!(build(&f.model, &f.root, moving.uuid, moving.position).is_err());
        let snapshot = ElectricalSelectionSnapshot::capture(&f.model).unwrap();
        assert!(LabelMoveBasis::capture(&snapshot, moving.uuid, &[Uuid::new_v4()]).is_err());
        let basis = LabelMoveBasis::capture(&snapshot, moving.uuid, &[]).unwrap();
        let destination = Point::new(1200, 500);
        basis.destination_edit(destination).unwrap();
        assert_eq!(
            f.model.journal.len(),
            count,
            "preview/cancel and refused edits do not write"
        );
        assert_eq!(
            sheet_payload(
                &ProjectResolver::new(&f.root).resolve().unwrap(),
                f.sheet,
                moving.uuid
            ),
            before
        );

        let mut expected = before.clone();
        expected["labels"][&key]["position"]["x"] = json!(1200);
        expected["labels"][&key]["position"]["y"] = json!(500);
        expected["wires"][a.uuid.to_string()]["to"]["x"] = json!(1200);
        expected["wires"][a.uuid.to_string()]["to"]["y"] = json!(500);
        expected["wires"][b.uuid.to_string()]["from"]["x"] = json!(1200);
        expected["wires"][b.uuid.to_string()]["from"]["y"] = json!(500);
        let write = build(&f.model, &f.root, moving.uuid, destination).unwrap();
        commit_prepared(&mut f.model, &f.root, write).unwrap();
        assert_eq!(f.model.journal.len(), count + 1);
        f.model = ProjectResolver::new(&f.root).resolve().unwrap();
        assert_eq!(sheet_payload(&f.model, f.sheet, moving.uuid), expected);
        assert_eq!(
            electrical_test_partitions(&f.model, &[]).unwrap(),
            partitions
        );
        assert_eq!(f.model.electrical_identities, identities);
        f.model
            .commit_journal_undo(&f.root, provenance().into())
            .unwrap();
        f.model = ProjectResolver::new(&f.root).resolve().unwrap();
        assert_eq!(sheet_payload(&f.model, f.sheet, moving.uuid), before);
        assert_eq!(
            electrical_test_partitions(&f.model, &[]).unwrap(),
            partitions
        );
        f.model
            .commit_journal_redo(&f.root, provenance().into())
            .unwrap();
        f.model = ProjectResolver::new(&f.root).resolve().unwrap();
        assert_eq!(sheet_payload(&f.model, f.sheet, moving.uuid), expected);
        assert_eq!(
            electrical_test_partitions(&f.model, &[]).unwrap(),
            partitions
        );
        assert_eq!(f.model.electrical_identities, identities);
        std::fs::remove_dir_all(f.root).unwrap();
    }
}

#[test]
fn interior_label_move_keeps_wire_payloads_membership_and_cold_history() {
    for kind in [
        LabelKind::Local,
        LabelKind::Global,
        LabelKind::Hierarchical,
        LabelKind::Power,
    ] {
        let mut f = fixture("interior_label_move_history");
        let moving = label(&mut f.model, &f.root, f.sheet, Point::new(500, 0), kind);
        let fixed = wire(
            &mut f.model,
            &f.root,
            f.sheet,
            Point::zero(),
            Point::new(1000, 0),
        );
        wire(
            &mut f.model,
            &f.root,
            f.sheet,
            Point::new(0, 500),
            Point::new(1000, 500),
        );
        let original = sheet_payload(&f.model, f.sheet, moving.uuid);
        let key = moving.uuid.to_string();
        let mut raw_label = original["labels"][&key].clone();
        raw_label["source_extension"] = json!({"literal":"${LABEL}", "future":[false,7]});
        raw_label["position"]["source_coordinate"] = json!("500,0");
        let old_wire = original["wires"][fixed.uuid.to_string()].clone();
        let mut raw_wire = old_wire.clone();
        raw_wire["source_extension"] = json!({"literal":"keep wire"});
        raw_wire["from"]["source_coordinate"] = json!("0,0");
        let write = BatchComposer::compose(&f.model, provenance())
            .push_ops(vec![
                Operation::SetSchematicLabel {
                    sheet_id: f.sheet,
                    label_id: moving.uuid,
                    label: raw_label,
                },
                Operation::DeleteSchematicWire {
                    sheet_id: f.sheet,
                    wire_id: fixed.uuid,
                    wire: old_wire,
                },
                Operation::CreateSchematicWire {
                    sheet_id: f.sheet,
                    wire_id: fixed.uuid,
                    wire: raw_wire,
                },
            ])
            .finish()
            .unwrap();
        commit_prepared(&mut f.model, &f.root, write).unwrap();
        f.model = ProjectResolver::new(&f.root).resolve().unwrap();
        let before = sheet_payload(&f.model, f.sheet, moving.uuid);
        let partitions = electrical_test_partitions(&f.model, &[]).unwrap();
        let identities = f.model.electrical_identities.clone();
        let count = f.model.journal.len();
        let snapshot = ElectricalSelectionSnapshot::capture(&f.model).unwrap();
        let basis = LabelMoveBasis::capture(&snapshot, moving.uuid, &[]).unwrap();
        assert!(basis.destination_edit(Point::new(750, 500)).is_err());
        assert!(basis.destination_edit(Point::new(750, 100)).is_err());
        let at = Point::new(750, 0);
        assert!(basis.destination_edit(at).unwrap().wires.is_empty());
        assert_eq!(f.model.journal.len(), count, "preview/refusals never write");
        assert_eq!(sheet_payload(&f.model, f.sheet, moving.uuid), before);
        let write = build(&f.model, &f.root, moving.uuid, at).unwrap();
        commit_prepared(&mut f.model, &f.root, write).unwrap();
        f.model = ProjectResolver::new(&f.root).resolve().unwrap();
        let mut expected = before.clone();
        expected["labels"][&key]["position"]["x"] = json!(750);
        assert_eq!(sheet_payload(&f.model, f.sheet, moving.uuid), expected);
        assert_eq!(
            electrical_test_partitions(&f.model, &[]).unwrap(),
            partitions
        );
        assert_eq!(f.model.electrical_identities, identities);
        f.model
            .commit_journal_undo(&f.root, provenance().into())
            .unwrap();
        f.model = ProjectResolver::new(&f.root).resolve().unwrap();
        assert_eq!(sheet_payload(&f.model, f.sheet, moving.uuid), before);
        f.model
            .commit_journal_redo(&f.root, provenance().into())
            .unwrap();
        f.model = ProjectResolver::new(&f.root).resolve().unwrap();
        assert_eq!(sheet_payload(&f.model, f.sheet, moving.uuid), expected);
        assert_eq!(
            electrical_test_partitions(&f.model, &[]).unwrap(),
            partitions
        );
        assert_eq!(f.model.electrical_identities, identities);
        std::fs::remove_dir_all(f.root).unwrap();
    }
}
