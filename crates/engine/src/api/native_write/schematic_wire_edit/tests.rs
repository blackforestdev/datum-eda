use super::*;
use crate::{
    ir::geometry::Point,
    schematic::{
        HiddenPowerBehavior, Junction, PinElectricalType, PlacedSymbol, SchematicWire, Sheet,
        SymbolDisplayMode, SymbolPin,
    },
};
use std::collections::BTreeSet;
use uuid::Uuid;
pub(super) fn id(n: u128) -> Uuid {
    Uuid::from_u128(n)
}
pub(super) fn p(x: i64, y: i64) -> Point {
    Point::new(x, y)
}
pub(super) fn blank() -> Sheet {
    serde_json::from_value(serde_json::json!({"uuid":id(1),"name":"Owner wire geometry","frame":null,"symbols":{},"wires":{},"junctions":{},"labels":{},"buses":{},"bus_entries":{},"ports":{},"noconnects":{},"texts":{},"drawings":{}})).unwrap()
}
pub(super) fn wire(sheet: &mut Sheet, n: u128, a: Point, b: Point) {
    sheet.wires.insert(
        id(n),
        SchematicWire {
            uuid: id(n),
            from: a,
            to: b,
        },
    );
}
pub(super) fn pin(sheet: &mut Sheet, n: u128, point: Point) {
    sheet.symbols.insert(
        id(n),
        PlacedSymbol {
            uuid: id(n),
            part: None,
            entity: None,
            gate: None,
            lib_id: None,
            reference: format!("P{n}"),
            value: String::new(),
            fields: vec![],
            pins: vec![SymbolPin {
                uuid: id(n + 1000),
                library_pin: None,
                number: "1".into(),
                name: "contact".into(),
                electrical_type: PinElectricalType::Passive,
                position: point,
            }],
            position: point,
            rotation: 0,
            mirrored: false,
            unit_selection: None,
            display_mode: SymbolDisplayMode::LibraryDefault,
            pin_overrides: vec![],
            hidden_power_behavior: HiddenPowerBehavior::PreservedAsImportedMetadata,
        },
    );
}
pub(super) fn owner() -> Sheet {
    let mut sheet = blank();
    wire(
        &mut sheet,
        10,
        p(-2_400_000, -3_290_000),
        p(-2_400_000, 2_690_000),
    );
    wire(
        &mut sheet,
        11,
        p(1_720_000, 700_000),
        p(-2_400_000, 1_200_000),
    );
    sheet.junctions.insert(
        id(20),
        Junction {
            uuid: id(20),
            position: p(-2_400_000, 1_200_000),
        },
    );
    pin(&mut sheet, 30, p(-2_400_000, -3_290_000));
    pin(&mut sheet, 31, p(-2_400_000, 2_690_000));
    pin(&mut sheet, 32, p(1_720_000, 700_000));
    sheet
}
fn geometry(sheet: &Sheet) -> BTreeSet<((i64, i64), (i64, i64))> {
    sheet
        .wires
        .values()
        .map(|w| {
            let a = (w.from.x, w.from.y);
            let b = (w.to.x, w.to.y);
            if a <= b { (a, b) } else { (b, a) }
        })
        .collect()
}

#[test]
fn unsplit_owner_contact_becomes_virtual_handle_without_mutating_or_splitting_source() {
    let sheet = owner();
    let before = sheet.clone();
    let basis = WireEditBasis::capture(&sheet, id(10), id(90)).unwrap();
    assert!(basis.handles().contains(&WireHandle::Vertex {
        point: p(-2_400_000, 1_200_000)
    }));
    assert!(basis.handles().contains(&WireHandle::Segment {
        from: p(-2_400_000, -3_290_000),
        to: p(-2_400_000, 1_200_000)
    }));
    assert_eq!(sheet, before);
    assert_eq!(basis.wire(), &before.wires[&id(10)]);
}

#[test]
fn unsplit_t_endpoint_reshape_refuses_new_physical_ends_at_retained_marker() {
    let sheet = owner();
    let before = sheet.clone();
    let basis = WireEditBasis::capture(&sheet, id(10), id(90)).unwrap();
    let error = basis
        .plan_move(
            &WireHandle::Endpoint {
                end: WireEndpoint::From,
            },
            p(-3_000_000, -3_290_000),
            WireRouting::Free,
        )
        .unwrap_err();
    assert!(error.to_string().contains("marker-gated"), "{error}");
    assert_eq!(sheet, before);
}

#[test]
fn ordinary_branch_endpoint_edit_keeps_through_wire_and_junction_fixed() {
    let sheet = owner();
    let basis = WireEditBasis::capture(&sheet, id(11), id(90)).unwrap();
    let plan = basis
        .plan_move(
            &WireHandle::Endpoint {
                end: WireEndpoint::From,
            },
            p(2_000_000, 700_000),
            WireRouting::Free,
        )
        .unwrap();
    let after = plan.apply_to_sheet(&sheet);
    assert_eq!(after.wires[&id(10)], sheet.wires[&id(10)]);
    assert_eq!(after.wires[&id(11)].from, p(2_000_000, 700_000));
    assert_eq!(after.wires[&id(11)].to, sheet.wires[&id(11)].to);
    assert_eq!(after.junctions, sheet.junctions);
    assert_eq!(after.symbols, sheet.symbols);
}

#[test]
fn owner_junction_axis_slide_keeps_through_wire_unsplit_and_branch_pin_fixed() {
    let sheet = owner();
    let basis = WireEditBasis::capture(&sheet, id(10), id(90)).unwrap();
    let plan = basis
        .plan_move(
            &WireHandle::Vertex {
                point: p(-2_400_000, 1_200_000),
            },
            p(-2_400_000, 700_000),
            WireRouting::Orthogonal,
        )
        .unwrap();
    let after = plan.apply_to_sheet(&sheet);
    assert_eq!(after.wires[&id(10)], sheet.wires[&id(10)]);
    assert_eq!(after.wires[&id(11)].from, p(1_720_000, 700_000));
    assert_eq!(after.wires[&id(11)].to, p(-2_400_000, 700_000));
    assert_eq!(after.junctions[&id(20)].position, p(-2_400_000, 700_000));
    assert_eq!(after.symbols, sheet.symbols);
    assert!(plan.new_wire_origins.is_empty());
}

#[test]
fn unsplit_and_presegmented_pin_contacts_offer_equivalent_endpoint_edit_geometry() {
    let mut unsplit = owner();
    unsplit.junctions.clear();
    pin(&mut unsplit, 33, p(-2_400_000, 1_200_000));
    let mut segmented = unsplit.clone();
    segmented.wires.get_mut(&id(10)).unwrap().to = p(-2_400_000, 1_200_000);
    wire(
        &mut segmented,
        12,
        p(-2_400_000, 1_200_000),
        p(-2_400_000, 2_690_000),
    );
    let handle = WireHandle::Endpoint {
        end: WireEndpoint::From,
    };
    let run = |sheet: &Sheet| {
        WireEditBasis::capture(sheet, id(10), id(90))
            .unwrap()
            .plan_move(&handle, p(-3_000_000, -3_290_000), WireRouting::Free)
            .unwrap()
            .apply_to_sheet(sheet)
    };
    assert_eq!(geometry(&run(&unsplit)), geometry(&run(&segmented)));
}

#[test]
fn endpoint_connected_bend_moves_all_incident_edges_with_outer_pins_fixed() {
    let mut sheet = blank();
    wire(&mut sheet, 10, p(0, 0), p(1000, 0));
    wire(&mut sheet, 11, p(1000, 0), p(1000, 1000));
    pin(&mut sheet, 30, p(0, 0));
    pin(&mut sheet, 31, p(1000, 1000));
    let basis = WireEditBasis::capture(&sheet, id(10), id(90)).unwrap();
    let after = basis
        .plan_move(
            &WireHandle::Vertex { point: p(1000, 0) },
            p(1500, 200),
            WireRouting::Free,
        )
        .unwrap()
        .apply_to_sheet(&sheet);
    assert_eq!(after.wires[&id(10)].from, p(0, 0));
    assert_eq!(after.wires[&id(10)].to, p(1500, 200));
    assert_eq!(after.wires[&id(11)].from, p(1500, 200));
    assert_eq!(after.wires[&id(11)].to, p(1000, 1000));
    assert_eq!(after.symbols, sheet.symbols);
}

#[test]
fn segment_handle_preserves_ends_and_orthogonal_corner_geometry() {
    let mut sheet = blank();
    wire(&mut sheet, 10, p(0, 0), p(1000, 0));
    pin(&mut sheet, 30, p(0, 0));
    pin(&mut sheet, 31, p(1000, 0));
    let basis = WireEditBasis::capture(&sheet, id(10), id(90)).unwrap();
    let handle = WireHandle::Segment {
        from: p(0, 0),
        to: p(1000, 0),
    };
    let after = basis
        .plan_move(&handle, p(777, 500), WireRouting::Orthogonal)
        .unwrap()
        .apply_to_sheet(&sheet);
    assert_eq!(
        geometry(&after),
        BTreeSet::from([
            ((0, 0), (0, 500)),
            ((0, 500), (1000, 500)),
            ((1000, 0), (1000, 500))
        ])
    );
    assert_eq!(after.symbols, sheet.symbols);
    assert!(
        basis
            .plan_move(&handle, p(800, 0), WireRouting::Free)
            .unwrap()
            .wires
            .is_empty()
    );
}

#[test]
fn disconnected_crossing_has_no_virtual_vertex_and_foreign_pin_short_refuses() {
    let mut sheet = blank();
    wire(&mut sheet, 10, p(0, 0), p(1000, 0));
    wire(&mut sheet, 11, p(500, -1000), p(500, 1000));
    let basis = WireEditBasis::capture(&sheet, id(10), id(90)).unwrap();
    assert!(
        !basis
            .handles()
            .contains(&WireHandle::Vertex { point: p(500, 0) })
    );
    let handle = WireHandle::Segment {
        from: p(0, 0),
        to: p(1000, 0),
    };
    assert!(
        basis
            .plan_move(&handle, p(500, 500), WireRouting::Free)
            .is_ok()
    );
    pin(&mut sheet, 30, p(500, 500));
    let basis = WireEditBasis::capture(&sheet, id(10), id(90)).unwrap();
    assert!(
        basis
            .plan_move(&handle, p(500, 500), WireRouting::Free)
            .is_err()
    );
}

#[test]
fn explicit_endpoint_reconnection_is_allowed_only_at_destination() {
    let mut sheet = blank();
    wire(&mut sheet, 10, p(0, 0), p(1000, 0));
    pin(&mut sheet, 30, p(1000, 1000));
    let handle = WireHandle::Endpoint {
        end: WireEndpoint::To,
    };
    let basis = WireEditBasis::capture(&sheet, id(10), id(90)).unwrap();
    assert!(
        basis
            .plan_move(&handle, p(1000, 1000), WireRouting::Free)
            .is_ok()
    );
    pin(&mut sheet, 31, p(500, 500));
    let basis = WireEditBasis::capture(&sheet, id(10), id(90)).unwrap();
    assert!(
        basis
            .plan_move(&handle, p(1000, 1000), WireRouting::Free)
            .is_err()
    );
}

#[test]
fn insert_vertex_is_explicit_and_fixed_pin_vertex_cannot_silently_move() {
    let mut sheet = blank();
    wire(&mut sheet, 10, p(0, 0), p(1000, 0));
    let basis = WireEditBasis::capture(&sheet, id(10), id(90)).unwrap();
    let plan = basis
        .plan_move(
            &WireHandle::InsertVertex {
                from: p(0, 0),
                to: p(1000, 0),
            },
            p(500, 250),
            WireRouting::Free,
        )
        .unwrap();
    assert_eq!(plan.wires.len(), 2);
    assert_eq!(plan.new_wire_origins.len(), 1);
    assert_eq!(sheet.wires.len(), 1);
    pin(&mut sheet, 30, p(500, 0));
    let basis = WireEditBasis::capture(&sheet, id(10), id(90)).unwrap();
    assert!(
        basis
            .plan_move(
                &WireHandle::Vertex { point: p(500, 0) },
                p(500, 250),
                WireRouting::Free
            )
            .is_err()
    );
}

#[test]
fn plan_is_order_independent_and_seed_qualifies_only_new_physical_edges() {
    let mut a = owner();
    a.junctions.clear();
    pin(&mut a, 33, p(-2_400_000, 1_200_000));
    let mut b = a.clone();
    b.wires.clear();
    let mut entries: Vec<_> = a.wires.iter().collect();
    entries.sort_by_key(|(key, _)| **key);
    for (key, value) in entries.into_iter().rev() {
        b.wires.insert(*key, value.clone());
    }
    let handle = WireHandle::Endpoint {
        end: WireEndpoint::From,
    };
    let run = |sheet: &Sheet, seed| {
        WireEditBasis::capture(sheet, id(10), seed)
            .unwrap()
            .plan_move(&handle, p(-3_000_000, -3_290_000), WireRouting::Free)
            .unwrap()
    };
    assert_eq!(run(&a, id(90)), run(&b, id(90)));
    let next = run(&a, id(91));
    assert_ne!(run(&a, id(90)).new_wire_origins, next.new_wire_origins);
    assert!(next.wires.contains_key(&id(10)));
}

#[test]
fn invalid_handles_and_full_domain_translation_refuse_without_source_changes() {
    let mut sheet = blank();
    wire(&mut sheet, 10, p(i64::MAX - 10, 0), p(i64::MAX, 10));
    let before = sheet.clone();
    let basis = WireEditBasis::capture(&sheet, id(10), id(90)).unwrap();
    assert!(
        basis
            .plan_move(
                &WireHandle::Segment {
                    from: p(i64::MAX - 10, 0),
                    to: p(i64::MAX, 10)
                },
                p(i64::MAX, 5),
                WireRouting::Free
            )
            .is_err()
    );
    assert!(
        basis
            .anchor(&WireHandle::Vertex { point: p(0, 0) })
            .is_err()
    );
    assert_eq!(sheet, before);
    assert_eq!(
        serde_json::to_value(WireRouting::default()).unwrap(),
        "free"
    );
    let handle = WireHandle::Endpoint {
        end: WireEndpoint::To,
    };
    assert_eq!(
        serde_json::from_value::<WireHandle>(serde_json::to_value(&handle).unwrap()).unwrap(),
        handle
    );
}

#[test]
fn initial_preview_is_empty_for_every_admitted_handle_and_never_normalizes_source() {
    let sheet = owner();
    let basis = WireEditBasis::capture(&sheet, id(10), id(90)).unwrap();
    for handle in basis.handles() {
        let plan = basis
            .plan_move(handle, basis.anchor(handle).unwrap(), WireRouting::Free)
            .unwrap();
        assert_eq!(plan, WireEditPlan::default());
        assert_eq!(plan.apply_to_sheet(&sheet), sheet);
    }
    assert_eq!(available_handles(&sheet, id(10)).unwrap(), basis.handles());
}

#[test]
fn endpoint_to_unsplit_foreign_wire_interior_creates_explicit_stable_t_contact() {
    let mut sheet = blank();
    wire(&mut sheet, 10, p(0, 0), p(1000, 0));
    wire(&mut sheet, 11, p(2000, -1000), p(2000, 1000));
    let basis = WireEditBasis::capture(&sheet, id(10), id(90)).unwrap();
    let handle = WireHandle::Endpoint {
        end: WireEndpoint::To,
    };
    let plan = basis
        .plan_move(&handle, p(2000, 0), WireRouting::Free)
        .unwrap();
    assert_eq!(plan.junctions.len(), 1);
    let after = plan.apply_to_sheet(&sheet);
    assert_eq!(after.wires[&id(11)], sheet.wires[&id(11)]);
    assert_eq!(after.wires[&id(10)].from, p(0, 0));
    assert_eq!(after.wires[&id(10)].to, p(2000, 0));
    assert_eq!(plan.junctions.values().next().unwrap().position, p(2000, 0));
    let next = basis
        .plan_move(&handle, p(2000, 500), WireRouting::Free)
        .unwrap();
    assert_eq!(
        plan.junctions.keys().collect::<Vec<_>>(),
        next.junctions.keys().collect::<Vec<_>>()
    );
    assert!(sheet.junctions.is_empty());
}

#[test]
fn junction_capture_chooses_incident_wire_deterministically_and_admits_endpoint_marker() {
    let mut sheet = owner();
    let basis = WireEditBasis::capture_junction(&sheet, id(20), id(90)).unwrap();
    assert_eq!(basis.wire().uuid, id(10));
    assert!(basis.handles().contains(&WireHandle::Vertex {
        point: p(-2_400_000, 1_200_000)
    }));
    sheet.junctions.clear();
    let lone = Junction {
        uuid: id(21),
        position: sheet.wires[&id(10)].from,
    };
    sheet.junctions.insert(lone.uuid, lone.clone());
    let basis = WireEditBasis::capture_junction(&sheet, lone.uuid, id(90)).unwrap();
    assert!(basis.handles().contains(&WireHandle::Vertex {
        point: lone.position
    }));
    sheet.junctions.get_mut(&lone.uuid).unwrap().position = p(99, 99);
    assert!(WireEditBasis::capture_junction(&sheet, lone.uuid, id(90)).is_err());
}

#[test]
fn preserving_segment_refuses_new_foreign_interior_t_but_permits_same_group_contact() {
    let mut sheet = blank();
    wire(&mut sheet, 10, p(0, 0), p(1000, 0));
    wire(&mut sheet, 11, p(1000, 200), p(1000, 800));
    let handle = WireHandle::Segment {
        from: p(0, 0),
        to: p(1000, 0),
    };
    let basis = WireEditBasis::capture(&sheet, id(10), id(90)).unwrap();
    let error = basis
        .plan_move(&handle, p(500, 500), WireRouting::Free)
        .unwrap_err();
    assert!(
        error.to_string().contains("foreign wire interior"),
        "{error}"
    );
    // Connect the same stationary wire by an existing alternate path. The same
    // geometric T now belongs to one original electrical group, so it is safe.
    wire(&mut sheet, 12, p(0, 0), p(-1000, 0));
    wire(&mut sheet, 13, p(-1000, 0), p(1000, 200));
    let basis = WireEditBasis::capture(&sheet, id(10), id(90)).unwrap();
    let after = basis
        .plan_move(&handle, p(500, 500), WireRouting::Free)
        .unwrap()
        .apply_to_sheet(&sheet);
    assert_eq!(after.wires[&id(11)], sheet.wires[&id(11)]);
    assert!(
        after
            .wires
            .values()
            .any(|w| w.from == p(0, 500) && w.to == p(1000, 500))
    );
}

#[test]
fn preserving_vertex_refuses_new_foreign_wire_interior_t_without_manufacturing_junction() {
    let mut sheet = blank();
    wire(&mut sheet, 10, p(0, 0), p(1000, 0));
    wire(&mut sheet, 12, p(1000, 0), p(1000, 1000));
    wire(&mut sheet, 11, p(1500, -500), p(1500, 500));
    let before = sheet.clone();
    let basis = WireEditBasis::capture(&sheet, id(10), id(90)).unwrap();
    let error = basis
        .plan_move(
            &WireHandle::Vertex { point: p(1000, 0) },
            p(1500, 0),
            WireRouting::Free,
        )
        .unwrap_err();
    assert!(
        error.to_string().contains("foreign wire interior"),
        "{error}"
    );
    assert_eq!(sheet, before);
    assert!(sheet.junctions.is_empty());
}

#[test]
fn off_through_crossing_reshape_refuses_and_original_junction_delete_still_disconnects() {
    let mut sheet = blank();
    wire(&mut sheet, 10, p(-1000, 0), p(1000, 0));
    wire(&mut sheet, 11, p(0, -1000), p(0, 1000));
    sheet.junctions.insert(
        id(20),
        Junction {
            uuid: id(20),
            position: p(0, 0),
        },
    );
    let before = sheet.clone();
    let basis = WireEditBasis::capture_junction(&sheet, id(20), id(90)).unwrap();
    let error = basis
        .plan_move(
            &WireHandle::Vertex { point: p(0, 0) },
            p(100, 100),
            WireRouting::Free,
        )
        .unwrap_err();
    assert!(error.to_string().contains("marker-gated"), "{error}");
    assert_eq!(sheet, before);
    let groups = |s: &Sheet| {
        crate::connectivity::physical_partitions(&crate::schematic::Schematic {
            uuid: s.uuid,
            sheets: std::collections::HashMap::from([(s.uuid, s.clone())]),
            sheet_definitions: Default::default(),
            sheet_instances: Default::default(),
            variants: Default::default(),
            waivers: vec![],
        })
        .unwrap()
    };
    assert_eq!(groups(&sheet).len(), 1);
    sheet.junctions.remove(&id(20));
    assert_eq!(
        groups(&sheet).len(),
        2,
        "removing the original marker still disconnects crossing wires"
    );
}
