//! Bounded diagnostic of the current engine query boundary, not S5A proof.
use eda_engine::connectivity::schematic_net_info;
use eda_engine::ir::geometry::Point;
use eda_engine::schematic::{LabelKind, NetLabel, Schematic, SchematicWire, Sheet};
use std::collections::HashMap;
use uuid::Uuid;

#[test]
fn schematic_summary_identity_changes_on_label_only_rename() {
    let sheet_id = Uuid::from_u128(1);
    let wire_id = Uuid::from_u128(2);
    let label_id = Uuid::from_u128(3);
    let mut schematic = Schematic {
        uuid: Uuid::from_u128(4),
        sheets: HashMap::from([(
            sheet_id,
            Sheet {
                uuid: sheet_id,
                name: "Root".into(),
                frame: None,
                symbols: HashMap::new(),
                wires: HashMap::from([(
                    wire_id,
                    SchematicWire {
                        uuid: wire_id,
                        from: Point::new(0, 0),
                        to: Point::new(100, 0),
                    },
                )]),
                junctions: HashMap::new(),
                labels: HashMap::from([(
                    label_id,
                    NetLabel {
                        uuid: label_id,
                        kind: LabelKind::Global,
                        name: "BEFORE".into(),
                        position: Point::new(0, 0),
                    },
                )]),
                buses: HashMap::new(),
                bus_entries: HashMap::new(),
                ports: HashMap::new(),
                noconnects: HashMap::new(),
                texts: HashMap::new(),
                drawings: HashMap::new(),
            },
        )]),
        sheet_definitions: HashMap::new(),
        sheet_instances: HashMap::new(),
        variants: HashMap::new(),
        waivers: Vec::new(),
    };
    let before = schematic_net_info(&schematic);
    schematic
        .sheets
        .get_mut(&sheet_id)
        .unwrap()
        .labels
        .get_mut(&label_id)
        .unwrap()
        .name = "AFTER".into();
    let after = schematic_net_info(&schematic);
    assert_eq!(before.len(), 1);
    assert_eq!(after.len(), 1);
    assert_eq!(before[0].labels, after[0].labels);
    assert_eq!(before[0].sheets, after[0].sheets);
    assert_ne!(before[0].uuid, after[0].uuid);
    let sheet = &schematic.sheets[&sheet_id];
    assert_eq!(sheet.wires[&wire_id].uuid, wire_id);
    assert_eq!(sheet.labels[&label_id].uuid, label_id);
}
