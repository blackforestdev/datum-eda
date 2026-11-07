use super::{
    tests::{blank, id, p, pin, wire},
    *,
};
use crate::{
    api::native_write::design_clipboard::{ClipboardClass, ClipboardPlacement, ClipboardTarget},
    schematic::{Junction, Sheet},
};

fn target(n: u128, class: ClipboardClass) -> ClipboardTarget {
    ClipboardTarget { class, id: id(n) }
}
fn pose(
    position: crate::ir::geometry::Point,
    rotation_deg: i32,
    mirrored: bool,
) -> ClipboardPlacement {
    ClipboardPlacement {
        position,
        rotation_deg,
        mirrored,
    }
}
fn joint(sheet: &mut Sheet, n: u128, point: crate::ir::geometry::Point) {
    sheet.junctions.insert(
        id(n),
        Junction {
            uuid: id(n),
            position: point,
        },
    );
}
fn partitions(
    sheet: &Sheet,
) -> Vec<std::collections::BTreeSet<crate::substrate::ElectricalOccurrence>> {
    let mut groups = crate::connectivity::physical_partitions(&crate::schematic::Schematic {
        uuid: sheet.uuid,
        sheets: std::collections::HashMap::from([(sheet.uuid, sheet.clone())]),
        sheet_definitions: Default::default(),
        sheet_instances: Default::default(),
        variants: Default::default(),
        waivers: vec![],
    })
    .unwrap();
    groups.sort();
    groups
}

#[test]
fn existing_wire_cardinal_pose_retains_identity_and_immutable_source() {
    let mut sheet = blank();
    wire(&mut sheet, 10, p(10, 20), p(110, 20));
    wire(&mut sheet, 11, p(5000, 5000), p(5100, 5000));
    let before = sheet.clone();
    let basis = WiringTransformBasis::capture(&sheet, &[target(10, ClipboardClass::Wire)]).unwrap();
    for (rotation, mirrored, end) in [
        (0, false, p(1100, 2000)),
        (90, false, p(1000, 2100)),
        (180, false, p(900, 2000)),
        (270, false, p(1000, 1900)),
        (90, true, p(1000, 1900)),
    ] {
        let plan = basis
            .plan(p(10, 20), pose(p(1000, 2000), rotation, mirrored))
            .unwrap();
        let after = plan.apply_to_sheet(&sheet);
        assert_eq!(after.wires[&id(10)].from, p(1000, 2000));
        assert_eq!(after.wires[&id(10)].to, end);
        assert_eq!(after.wires[&id(11)], sheet.wires[&id(11)]);
        assert_eq!(
            after
                .wires
                .keys()
                .collect::<std::collections::BTreeSet<_>>(),
            sheet
                .wires
                .keys()
                .collect::<std::collections::BTreeSet<_>>()
        );
        assert!(plan.new_wire_origins.is_empty() && plan.removed_wire_ids.is_empty());
        assert_eq!(
            plan,
            basis
                .plan(p(10, 20), pose(p(1000, 2000), rotation, mirrored))
                .unwrap()
        );
    }
    assert_eq!(sheet, before);
    assert_eq!(
        basis.plan(p(10, 20), pose(p(10, 20), 0, false)).unwrap(),
        WireEditPlan::default()
    );
}

#[test]
fn complete_t_group_pose_is_order_independent_and_preserves_marker_deletion_gate() {
    let mut sheet = blank();
    wire(&mut sheet, 10, p(-100, 0), p(100, 0));
    wire(&mut sheet, 11, p(0, 100), p(0, 0));
    joint(&mut sheet, 20, p(0, 0));
    let targets = vec![
        target(10, ClipboardClass::Wire),
        target(11, ClipboardClass::Wire),
        target(20, ClipboardClass::Junction),
    ];
    let basis = WiringTransformBasis::capture(&sheet, &targets).unwrap();
    let plan = basis.plan(p(0, 0), pose(p(1000, 2000), 90, false)).unwrap();
    let mut reversed = targets.clone();
    reversed.reverse();
    assert_eq!(
        plan,
        WiringTransformBasis::capture(&sheet, &reversed)
            .unwrap()
            .plan(p(0, 0), pose(p(1000, 2000), 90, false))
            .unwrap()
    );
    let mut after = plan.apply_to_sheet(&sheet);
    assert_eq!(after.wires[&id(10)].from, p(1000, 1900));
    assert_eq!(after.wires[&id(10)].to, p(1000, 2100));
    assert_eq!(after.wires[&id(11)].from, p(900, 2000));
    assert_eq!(after.junctions[&id(20)].position, p(1000, 2000));
    assert_eq!(partitions(&after), partitions(&sheet));
    sheet.junctions.remove(&id(20));
    after.junctions.remove(&id(20));
    assert_eq!(partitions(&after), partitions(&sheet));
    assert_eq!(partitions(&after).len(), 2);
    assert!(plan.new_wire_origins.is_empty() && plan.removed_junction_ids.is_empty());
}

#[test]
fn singleton_junction_is_supported_but_cannot_join_a_foreign_wire() {
    let mut sheet = blank();
    joint(&mut sheet, 20, p(10, 20));
    wire(&mut sheet, 10, p(1000, 1000), p(2000, 1000));
    let basis =
        WiringTransformBasis::capture(&sheet, &[target(20, ClipboardClass::Junction)]).unwrap();
    let plan = basis.plan(p(10, 20), pose(p(50, 60), 90, true)).unwrap();
    assert!(plan.wires.is_empty());
    assert_eq!(plan.junctions[&id(20)].position, p(50, 60));
    assert!(
        basis
            .plan(p(10, 20), pose(p(1500, 1000), 0, false))
            .is_err()
    );
}

#[test]
fn fixed_pin_at_pivot_survives_but_redundant_net_path_does_not_authorize_contact_loss() {
    let mut sheet = blank();
    wire(&mut sheet, 10, p(0, 0), p(100, 0));
    pin(&mut sheet, 30, p(0, 0));
    let basis = WiringTransformBasis::capture(&sheet, &[target(10, ClipboardClass::Wire)]).unwrap();
    assert!(basis.plan(p(0, 0), pose(p(0, 0), 90, false)).is_ok());
    pin(&mut sheet, 31, p(100, 0));
    wire(&mut sheet, 11, p(0, 0), p(0, -100));
    wire(&mut sheet, 12, p(0, -100), p(100, -100));
    wire(&mut sheet, 13, p(100, -100), p(100, 0));
    let basis = WiringTransformBasis::capture(&sheet, &[target(10, ClipboardClass::Wire)]).unwrap();
    let error = basis.plan(p(0, 0), pose(p(0, 0), 90, false)).unwrap_err();
    assert!(error.to_string().contains("fixed contact"), "{error}");
}

#[test]
fn preserving_transform_refuses_foreign_endpoint_t_but_allows_unconnected_crossing() {
    let mut sheet = blank();
    wire(&mut sheet, 10, p(0, 0), p(100, 0));
    wire(&mut sheet, 11, p(-100, 100), p(100, 100));
    let basis = WiringTransformBasis::capture(&sheet, &[target(10, ClipboardClass::Wire)]).unwrap();
    let error = basis.plan(p(0, 0), pose(p(0, 0), 90, false)).unwrap_err();
    assert!(
        error.to_string().contains("foreign wire interior"),
        "{error}"
    );
    sheet.wires.get_mut(&id(10)).unwrap().from = p(-200, 0);
    sheet.wires.get_mut(&id(10)).unwrap().to = p(200, 0);
    let basis = WiringTransformBasis::capture(&sheet, &[target(10, ClipboardClass::Wire)]).unwrap();
    let after = basis
        .plan(p(0, 0), pose(p(0, 0), 90, false))
        .unwrap()
        .apply_to_sheet(&sheet);
    assert_eq!(partitions(&after), partitions(&sheet));
    assert!(after.junctions.is_empty());
}

#[test]
fn moving_marker_to_fixed_through_endpoint_cannot_erase_its_deletion_gate() {
    let mut sheet = blank();
    wire(&mut sheet, 10, p(0, 0), p(100, 0));
    wire(&mut sheet, 11, p(50, 100), p(50, 0));
    joint(&mut sheet, 20, p(50, 0));
    let basis = WiringTransformBasis::capture(
        &sheet,
        &[
            target(11, ClipboardClass::Wire),
            target(20, ClipboardClass::Junction),
        ],
    )
    .unwrap();
    let error = basis.plan(p(0, 0), pose(p(-50, 0), 0, false)).unwrap_err();
    assert!(error.to_string().contains("marker-gated"), "{error}");
}

#[test]
fn transform_refuses_bus_dependencies_duplicate_targets_and_missing_members() {
    let mut sheet = blank();
    wire(&mut sheet, 10, p(0, 0), p(100, 0));
    let t = target(10, ClipboardClass::Wire);
    assert!(WiringTransformBasis::capture(&sheet, &[]).is_err());
    assert!(WiringTransformBasis::capture(&sheet, &[t.clone(), t.clone()]).is_err());
    assert!(WiringTransformBasis::capture(&sheet, &[target(99, ClipboardClass::Wire)]).is_err());
    sheet.bus_entries.insert(
        id(70),
        crate::schematic::BusEntry {
            uuid: id(70),
            bus: id(80),
            wire: Some(id(10)),
            member_declaration: None,
            position: p(0, 0),
            size: p(100, 100),
        },
    );
    let basis = WiringTransformBasis::capture(&sheet, &[t]).unwrap();
    let error = basis.plan(p(0, 0), pose(p(0, 0), 90, false)).unwrap_err();
    assert!(error.to_string().contains("bus entry"), "{error}");
}

#[test]
fn cardinal_pose_and_handle_displacement_check_full_coordinate_domain() {
    let mut sheet = blank();
    wire(&mut sheet, 10, p(i64::MAX - 10, 0), p(i64::MAX - 5, 0));
    let basis = WiringTransformBasis::capture(&sheet, &[target(10, ClipboardClass::Wire)]).unwrap();
    assert!(basis.plan(p(0, 0), pose(p(100, 0), 0, false)).is_err());
    assert!(basis.plan(p(0, 0), pose(p(0, 0), 45, false)).is_err());
    let handle = WireHandle::Endpoint {
        end: WireEndpoint::To,
    };
    let wire_basis = WireEditBasis::capture(&sheet, id(10), id(90)).unwrap();
    assert!(
        wire_basis
            .plan_displacement(&handle, p(10, 0), WireRouting::Free)
            .is_err()
    );
    let plan = wire_basis
        .plan_displacement(&handle, p(-2, 0), WireRouting::Free)
        .unwrap();
    let after = plan.apply_to_sheet(&sheet);
    assert_eq!(after.wires[&id(10)].from, sheet.wires[&id(10)].from);
    assert_eq!(after.wires[&id(10)].to, p(i64::MAX - 7, 0));
    assert!(
        wire_basis
            .plan_displacement(
                &WireHandle::Vertex { point: p(0, 0) },
                p(1, 0),
                WireRouting::Free
            )
            .is_err()
    );
}

#[test]
fn selected_interior_marker_cannot_carry_off_unselected_branch_despite_redundant_path() {
    let mut sheet = blank();
    wire(&mut sheet, 10, p(-100, 0), p(100, 0));
    wire(&mut sheet, 11, p(0, 0), p(0, 100));
    wire(&mut sheet, 12, p(0, 100), p(-100, 100));
    wire(&mut sheet, 13, p(-100, 100), p(-100, 0));
    joint(&mut sheet, 20, p(0, 0));
    let mut disconnected = sheet.clone();
    disconnected.wires.get_mut(&id(10)).unwrap().to = p(-100, 200);
    disconnected.junctions.get_mut(&id(20)).unwrap().position = p(-100, 100);
    assert_eq!(
        partitions(&disconnected),
        partitions(&sheet),
        "the redundant route masks loss of the actual branch incidence"
    );
    let basis = WiringTransformBasis::capture(
        &sheet,
        &[
            target(10, ClipboardClass::Wire),
            target(20, ClipboardClass::Junction),
        ],
    )
    .unwrap();
    let error = basis
        .plan(p(-100, 0), pose(p(-100, 0), 90, false))
        .unwrap_err();
    assert!(error.to_string().contains("fixed contact"), "{error}");
    assert_eq!(sheet.wires[&id(11)].from, p(0, 0));
}

#[test]
fn fixed_interior_wire_contact_keeps_its_gate_but_markerless_t_and_direct_pin_differ() {
    let mut sheet = blank();
    wire(&mut sheet, 10, p(-100, 0), p(100, 0));
    wire(&mut sheet, 11, p(0, 0), p(0, 100));
    wire(&mut sheet, 12, p(0, 100), p(50, 100));
    wire(&mut sheet, 13, p(50, 100), p(50, 0));
    joint(&mut sheet, 20, p(0, 0));
    joint(&mut sheet, 21, p(50, 0));
    let mut lost_gate = sheet.clone();
    lost_gate.wires.get_mut(&id(10)).unwrap().from = p(-110, 0);
    lost_gate.wires.get_mut(&id(10)).unwrap().to = p(90, 0);
    lost_gate.junctions.get_mut(&id(20)).unwrap().position = p(-10, 0);
    assert_eq!(partitions(&lost_gate), partitions(&sheet));
    let targets = [
        target(10, ClipboardClass::Wire),
        target(20, ClipboardClass::Junction),
    ];
    let basis = WiringTransformBasis::capture(&sheet, &targets).unwrap();
    let error = basis.plan(p(0, 0), pose(p(-10, 0), 0, false)).unwrap_err();
    assert!(
        error.to_string().contains("fixed marker-gated contact"),
        "{error}"
    );
    let marker_only = [target(20, ClipboardClass::Junction)];
    let mut marker_lost_gate = sheet.clone();
    marker_lost_gate
        .junctions
        .get_mut(&id(20))
        .unwrap()
        .position = p(-10, 0);
    assert_eq!(partitions(&marker_lost_gate), partitions(&sheet));
    let basis = WiringTransformBasis::capture(&sheet, &marker_only).unwrap();
    let error = basis.plan(p(0, 0), pose(p(-10, 0), 0, false)).unwrap_err();
    assert!(
        error.to_string().contains("fixed marker-gated contact"),
        "{error}"
    );
    // An independent fixed pin joins wire interiors without relying on the dot.
    pin(&mut sheet, 30, p(0, 0));
    assert!(
        WiringTransformBasis::capture(&sheet, &targets)
            .unwrap()
            .plan(p(0, 0), pose(p(-10, 0), 0, false))
            .is_ok()
    );
    assert!(
        WiringTransformBasis::capture(&sheet, &marker_only)
            .unwrap()
            .plan(p(0, 0), pose(p(-10, 0), 0, false))
            .is_ok()
    );
    sheet.symbols.remove(&id(30));
    sheet.junctions.remove(&id(20));
    let basis = WiringTransformBasis::capture(&sheet, &[targets[0].clone()]).unwrap();
    let plan = basis.plan(p(0, 0), pose(p(-10, 0), 0, false)).unwrap();
    assert_eq!(partitions(&plan.apply_to_sheet(&sheet)), partitions(&sheet));
    assert_eq!(sheet.wires[&id(11)].from, p(0, 0));
}

#[test]
fn marker_only_transform_preserves_each_fixed_through_crossing_incidence() {
    let mut sheet = blank();
    wire(&mut sheet, 10, p(-100, 0), p(100, 0));
    wire(&mut sheet, 11, p(0, -100), p(0, 100));
    wire(&mut sheet, 12, p(100, 0), p(100, 100));
    wire(&mut sheet, 13, p(100, 100), p(0, 100));
    joint(&mut sheet, 20, p(0, 0));
    let mut lost_crossing = sheet.clone();
    lost_crossing.junctions.get_mut(&id(20)).unwrap().position = p(-10, 0);
    assert_eq!(partitions(&lost_crossing), partitions(&sheet));
    let basis =
        WiringTransformBasis::capture(&sheet, &[target(20, ClipboardClass::Junction)]).unwrap();
    let error = basis.plan(p(0, 0), pose(p(-10, 0), 0, false)).unwrap_err();
    assert!(
        error.to_string().contains("fixed marker-gated contact"),
        "{error}"
    );
}

#[test]
fn direct_fixed_endpoint_cannot_become_wire_interior_even_with_redundant_route_or_dot() {
    for dot in [false, true] {
        let mut sheet = blank();
        wire(&mut sheet, 10, p(0, 0), p(100, 0));
        wire(&mut sheet, 11, p(0, 0), p(0, 100));
        wire(&mut sheet, 12, p(0, 100), p(50, 100));
        wire(&mut sheet, 13, p(50, 100), p(50, 0));
        joint(&mut sheet, 21, p(50, 0));
        if dot {
            joint(&mut sheet, 20, p(0, 0));
        }
        let mut lost_endpoint = sheet.clone();
        lost_endpoint.wires.get_mut(&id(10)).unwrap().from = p(-10, 0);
        lost_endpoint.wires.get_mut(&id(10)).unwrap().to = p(90, 0);
        assert_eq!(partitions(&lost_endpoint), partitions(&sheet));
        let targets = [target(10, ClipboardClass::Wire)];
        let error = WiringTransformBasis::capture(&sheet, &targets)
            .unwrap()
            .plan(p(0, 0), pose(p(-10, 0), 0, false))
            .unwrap_err();
        assert!(
            error.to_string().contains("fixed direct endpoint contact"),
            "{error}"
        );
        pin(&mut sheet, 30, p(0, 0));
        assert!(
            WiringTransformBasis::capture(&sheet, &targets)
                .unwrap()
                .plan(p(0, 0), pose(p(-10, 0), 0, false))
                .is_ok()
        );
    }
}
