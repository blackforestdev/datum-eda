use super::{
    tests::{blank, id, owner, p, pin, wire},
    *,
};
use crate::schematic::Junction;

#[test]
fn explicit_split_keeps_source_identity_and_stable_children_without_read_normalization() {
    let mut sheet = blank();
    wire(&mut sheet, 10, p(0, 0), p(1000, 0));
    let before = sheet.clone();
    let basis = WireEditBasis::capture(&sheet, id(10), id(90)).unwrap();
    let plan = basis.plan_split(p(400, 0)).unwrap();
    let after = plan.apply_to_sheet(&sheet);
    assert_eq!(after.wires[&id(10)].from, p(0, 0));
    assert_eq!(after.wires[&id(10)].to, p(400, 0));
    assert!(
        after
            .wires
            .values()
            .any(|w| w.from == p(400, 0) && w.to == p(1000, 0))
    );
    assert_eq!(
        plan.new_wire_origins.values().copied().collect::<Vec<_>>(),
        vec![id(10)]
    );
    assert_eq!(plan, basis.plan_split(p(400, 0)).unwrap());
    assert_eq!(sheet, before);
    for point in [p(0, 0), p(1000, 0), p(400, 1)] {
        assert!(basis.plan_split(point).is_err());
    }
    assert_eq!(
        serde_json::to_value(WireEditAction::default()).unwrap(),
        "move"
    );
    assert_eq!(
        serde_json::to_value(WireEditAction::RemoveSpan).unwrap(),
        "remove_span"
    );
}

#[test]
fn split_at_preexisting_gated_t_refuses_instead_of_hardwiring_junction() {
    let sheet = owner();
    let basis = WireEditBasis::capture(&sheet, id(10), id(90)).unwrap();
    let error = basis.plan_split(p(-2_400_000, 1_200_000)).unwrap_err();
    assert!(error.to_string().contains("marker-gated"), "{error}");
}

#[test]
fn split_remove_vertex_inverse_preserves_interior_t_and_marker_deletion_law() {
    let original = owner();
    let point = p(-2_400_000, 0);
    let split = WireEditBasis::capture(&original, id(10), id(90))
        .unwrap()
        .plan_split(point)
        .unwrap();
    let divided = split.apply_to_sheet(&original);
    let child = *split.new_wire_origins.keys().next().unwrap();
    for selected in [id(10), child] {
        let plan = WireEditBasis::capture(&divided, selected, id(91))
            .unwrap()
            .plan_remove_vertex(point)
            .unwrap();
        let restored = plan.apply_to_sheet(&divided);
        assert_eq!(
            plan.coalesced_wire_origins[&selected],
            std::collections::BTreeSet::from([id(10), child])
        );
        assert_eq!(restored.wires.len(), original.wires.len());
        assert_eq!(restored.wires[&selected].from, original.wires[&id(10)].from);
        assert_eq!(restored.wires[&selected].to, original.wires[&id(10)].to);
        assert_eq!(restored.wires[&id(11)], original.wires[&id(11)]);
        assert_eq!(restored.junctions, original.junctions);
        assert_eq!(restored.symbols, original.symbols);
        if selected == id(10) {
            assert_eq!(
                restored, original,
                "split then coalesce is an exact inverse"
            );
        }
        let groups = |sheet: &crate::schematic::Sheet| {
            crate::connectivity::physical_partitions(&crate::schematic::Schematic {
                uuid: sheet.uuid,
                sheets: std::collections::HashMap::from([(sheet.uuid, sheet.clone())]),
                sheet_definitions: Default::default(),
                sheet_instances: Default::default(),
                variants: Default::default(),
                waivers: vec![],
            })
            .unwrap()
        };
        assert_eq!(groups(&restored).len(), 1);
        let mut without_marker = restored.clone();
        without_marker.junctions.remove(&id(20));
        assert_eq!(groups(&without_marker).len(), 2);
        if selected == id(10) {
            let mut original_without_marker = original.clone();
            original_without_marker.junctions.remove(&id(20));
            assert_eq!(groups(&without_marker), groups(&original_without_marker));
        }
    }
    assert_eq!(split.apply_to_sheet(&original), divided);
}

#[test]
fn split_at_unconnected_crossing_refuses_misleading_new_physical_endpoints() {
    let mut sheet = blank();
    wire(&mut sheet, 10, p(0, 0), p(1000, 0));
    wire(&mut sheet, 11, p(500, -1000), p(500, 1000));
    let basis = WireEditBasis::capture(&sheet, id(10), id(90)).unwrap();
    let error = basis.plan_split(p(500, 0)).unwrap_err();
    assert!(
        error.to_string().contains("foreign wire interior"),
        "{error}"
    );
    assert!(sheet.junctions.is_empty());
}

#[test]
fn remove_span_deletes_only_selected_contact_interval_and_keeps_pin_positions() {
    let mut sheet = blank();
    wire(&mut sheet, 10, p(0, 0), p(1000, 0));
    pin(&mut sheet, 30, p(250, 0));
    pin(&mut sheet, 31, p(750, 0));
    let basis = WireEditBasis::capture(&sheet, id(10), id(90)).unwrap();
    let plan = basis.plan_remove_span(p(250, 0), p(750, 0)).unwrap();
    let after = plan.apply_to_sheet(&sheet);
    assert_eq!(after.wires.len(), 2);
    assert_eq!(after.wires[&id(10)].from, p(0, 0));
    assert_eq!(after.wires[&id(10)].to, p(250, 0));
    assert!(
        after
            .wires
            .values()
            .any(|w| w.from == p(750, 0) && w.to == p(1000, 0))
    );
    assert_eq!(after.symbols, sheet.symbols);
    assert!(basis.plan_remove_span(p(100, 0), p(900, 0)).is_err());
}

#[test]
fn deleting_t_branch_cleans_redundant_dot_and_retains_unsplit_through_wire() {
    let sheet = owner();
    let branch = &sheet.wires[&id(11)];
    let basis = WireEditBasis::capture(&sheet, id(11), id(90)).unwrap();
    let plan = basis.plan_remove_span(branch.from, branch.to).unwrap();
    let after = plan.apply_to_sheet(&sheet);
    assert!(plan.removed_wire_ids.contains(&id(11)));
    assert!(plan.removed_junction_ids.contains(&id(20)));
    assert_eq!(after.wires[&id(10)], sheet.wires[&id(10)]);
    assert_eq!(after.symbols, sheet.symbols);
    assert_eq!(after.wires.len(), 1);
    assert!(after.junctions.is_empty());
}

#[test]
fn deleting_presegmented_t_branch_coalesces_collinear_survivors() {
    let mut sheet = owner();
    let point = p(-2_400_000, 1_200_000);
    sheet.wires.get_mut(&id(10)).unwrap().to = point;
    wire(&mut sheet, 12, point, p(-2_400_000, 2_690_000));
    let branch = &sheet.wires[&id(11)];
    let basis = WireEditBasis::capture(&sheet, id(11), id(90)).unwrap();
    let plan = basis.plan_remove_span(branch.from, branch.to).unwrap();
    let after = plan.apply_to_sheet(&sheet);
    assert_eq!(after.wires.len(), 1);
    assert_eq!(after.wires[&id(10)].from, p(-2_400_000, -3_290_000));
    assert_eq!(after.wires[&id(10)].to, p(-2_400_000, 2_690_000));
    assert!(after.junctions.is_empty());
    assert!(plan.removed_wire_ids.contains(&id(12)));
    assert_eq!(after.symbols, sheet.symbols);
}

#[test]
fn remove_bend_keeps_selected_wire_identity_and_outer_pin_contacts() {
    let mut sheet = blank();
    wire(&mut sheet, 10, p(0, 0), p(1000, 0));
    wire(&mut sheet, 11, p(1000, 0), p(1000, 1000));
    pin(&mut sheet, 30, p(0, 0));
    pin(&mut sheet, 31, p(1000, 1000));
    let basis = WireEditBasis::capture(&sheet, id(10), id(90)).unwrap();
    let plan = basis.plan_remove_vertex(p(1000, 0)).unwrap();
    let after = plan.apply_to_sheet(&sheet);
    assert_eq!(after.wires.len(), 1);
    assert_eq!(after.wires[&id(10)].from, p(0, 0));
    assert_eq!(after.wires[&id(10)].to, p(1000, 1000));
    assert!(plan.removed_wire_ids.contains(&id(11)));
    assert_eq!(after.symbols, sheet.symbols);
}

#[test]
fn redundant_degree_two_marker_removal_does_not_split_its_through_wire() {
    let mut sheet = blank();
    wire(&mut sheet, 10, p(0, 0), p(1000, 0));
    sheet.junctions.insert(
        id(20),
        Junction {
            uuid: id(20),
            position: p(500, 0),
        },
    );
    let basis = WireEditBasis::capture(&sheet, id(10), id(90)).unwrap();
    let plan = basis.plan_remove_vertex(p(500, 0)).unwrap();
    assert!(plan.wires.is_empty());
    assert!(plan.removed_junction_ids.contains(&id(20)));
    let after = plan.apply_to_sheet(&sheet);
    assert_eq!(after.wires, sheet.wires);
    assert!(after.junctions.is_empty());
}

#[test]
fn removing_real_t_vertex_or_fixed_pin_or_foreign_contact_refuses() {
    let sheet = owner();
    let basis = WireEditBasis::capture(&sheet, id(10), id(90)).unwrap();
    assert!(basis.plan_remove_vertex(p(-2_400_000, 1_200_000)).is_err());
    let mut sheet = blank();
    wire(&mut sheet, 10, p(0, 0), p(1000, 0));
    wire(&mut sheet, 11, p(1000, 0), p(1000, 1000));
    pin(&mut sheet, 30, p(500, 500));
    let basis = WireEditBasis::capture(&sheet, id(10), id(90)).unwrap();
    assert!(
        basis.plan_remove_vertex(p(1000, 0)).is_err(),
        "shortcut must not acquire the foreign interior pin"
    );
    pin(&mut sheet, 31, p(1000, 0));
    let basis = WireEditBasis::capture(&sheet, id(10), id(90)).unwrap();
    assert!(
        basis.plan_remove_vertex(p(1000, 0)).is_err(),
        "fixed vertex pin cannot be discarded"
    );
}

#[test]
fn coalesced_output_names_only_contributing_old_raw_origins() {
    let mut sheet = owner();
    let point = p(-2_400_000, 1_200_000);
    sheet.wires.get_mut(&id(10)).unwrap().to = point;
    wire(&mut sheet, 12, point, p(-2_400_000, 2_690_000));
    let branch = &sheet.wires[&id(11)];
    let basis = WireEditBasis::capture(&sheet, id(11), id(90)).unwrap();
    let plan = basis.plan_remove_span(branch.from, branch.to).unwrap();
    assert_eq!(
        plan.coalesced_wire_origins[&id(10)],
        std::collections::BTreeSet::from([id(10), id(12)])
    );
    assert!(
        !plan
            .coalesced_wire_origins
            .values()
            .any(|origins| origins.contains(&id(11)))
    );
}

#[test]
fn coalescence_refuses_duplicate_chord_and_lifecycle_retains_bus_ownership_guards() {
    let mut sheet = blank();
    wire(&mut sheet, 10, p(0, 0), p(1000, 0));
    wire(&mut sheet, 11, p(1000, 0), p(1000, 1000));
    wire(&mut sheet, 12, p(0, 0), p(1000, 1000));
    let basis = WireEditBasis::capture(&sheet, id(10), id(90)).unwrap();
    let error = basis.plan_remove_vertex(p(1000, 0)).unwrap_err();
    assert!(error.to_string().contains("duplicate"), "{error}");
    sheet.wires.remove(&id(12));
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
    let basis = WireEditBasis::capture(&sheet, id(10), id(90)).unwrap();
    for result in [
        basis.plan_split(p(500, 0)),
        basis.plan_remove_span(p(0, 0), p(1000, 0)),
        basis.plan_remove_vertex(p(1000, 0)),
    ] {
        let error = result.unwrap_err();
        assert!(error.to_string().contains("bus entry"), "{error}");
    }
}
