use super::*;

#[test]
fn library_graph_reports_reference_diagnostics_in_engine() {
    let unit_id = Uuid::from_u128(101);
    let entity_id = Uuid::from_u128(102);
    let part_id = Uuid::from_u128(103);
    let other_part_id = Uuid::from_u128(104);
    let package_id = Uuid::from_u128(105);
    let other_package_id = Uuid::from_u128(106);
    let footprint_id = Uuid::from_u128(107);
    let map_id = Uuid::from_u128(108);
    let symbol_id = Uuid::from_u128(109);
    let other_footprint_id = Uuid::from_u128(110);

    let mut graph = LibraryGraph::default();
    graph
        .units
        .insert(unit_id, serde_json::json!({ "uuid": unit_id, "pins": {} }));
    graph.entities.insert(
        entity_id,
        serde_json::json!({ "uuid": entity_id, "gates": {} }),
    );
    graph.packages.insert(
        package_id,
        serde_json::json!({ "uuid": package_id, "pads": {} }),
    );
    graph.packages.insert(
        other_package_id,
        serde_json::json!({ "uuid": other_package_id, "pads": {} }),
    );
    graph.footprints.insert(
        footprint_id,
        serde_json::json!({
            "uuid": footprint_id,
            "package": other_package_id,
            "pads": {}
        }),
    );
    graph.footprints.insert(
        other_footprint_id,
        serde_json::json!({
            "uuid": other_footprint_id,
            "package": other_package_id,
            "pads": {}
        }),
    );
    graph.parts.insert(
        part_id,
        serde_json::json!({
            "uuid": part_id,
            "entity": entity_id,
            "package": package_id,
            "default_footprint": footprint_id,
            "default_pin_pad_map": map_id
        }),
    );
    graph.parts.insert(
        other_part_id,
        serde_json::json!({
            "uuid": other_part_id,
            "entity": entity_id,
            "package": package_id
        }),
    );
    graph.pin_pad_maps.insert(
        map_id,
        serde_json::json!({
            "uuid": map_id,
            "part": other_part_id,
            "footprint": other_footprint_id,
            "mappings": {}
        }),
    );
    graph.symbols.insert(
        symbol_id,
        serde_json::json!({
            "uuid": symbol_id,
            "unit": Uuid::from_u128(999)
        }),
    );
    for id in [
        unit_id,
        entity_id,
        part_id,
        other_part_id,
        package_id,
        other_package_id,
        footprint_id,
        other_footprint_id,
        map_id,
        symbol_id,
    ] {
        graph.subjects.insert(id, format!("fixture/{id}.json"));
    }

    let codes = graph
        .dependency_diagnostics()
        .into_iter()
        .map(|diagnostic| diagnostic.code)
        .collect::<std::collections::BTreeSet<_>>();

    assert!(codes.contains("dangling_reference"));
    assert!(codes.contains("package_mismatch"));
    assert!(codes.contains("part_mismatch"));
    assert!(codes.contains("footprint_mismatch"));
}

#[test]
fn library_graph_owns_pool_object_registration_policy() {
    let object_id = Uuid::from_u128(700);
    let mut graph = LibraryGraph::default();

    let diagnostics = graph.insert_pool_object(
        "symbols",
        object_id,
        "pool/symbols/700.json",
        serde_json::json!({
            "uuid": object_id,
            "unit": Uuid::from_u128(701)
        }),
    );

    assert!(diagnostics.is_empty());
    assert!(graph.symbols.contains_key(&object_id));
    assert_eq!(
        graph.subjects.get(&object_id).map(String::as_str),
        Some("pool/symbols/700.json")
    );

    let diagnostics = graph.insert_pool_object(
        "parts",
        object_id,
        "pool/parts/700.json",
        serde_json::json!({
            "uuid": object_id
        }),
    );

    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].code, "duplicate_uuid");
    assert_eq!(diagnostics[0].subject, "pool/parts/700.json");
    assert!(
        diagnostics[0]
            .message
            .contains("already appeared at pool/symbols/700.json")
    );
    assert_eq!(
        diagnostics[0].tier(),
        LibraryGraphValidationTier::Registration
    );
    assert!(
        diagnostics[0]
            .message
            .contains("shadows the earlier registration")
    );
}

#[test]
fn library_graph_reports_engine_owned_validation_summary() {
    let object_id = Uuid::from_u128(800);
    let missing_unit_id = Uuid::from_u128(801);
    let mut graph = LibraryGraph::default();

    assert!(
        graph
            .insert_pool_object(
                "symbols",
                object_id,
                "pool-a/symbols/800.json",
                serde_json::json!({
                    "uuid": object_id,
                    "unit": missing_unit_id
                }),
            )
            .is_empty()
    );
    graph.insert_pool_object(
        "symbols",
        object_id,
        "pool-b/symbols/800.json",
        serde_json::json!({
            "uuid": object_id,
            "unit": missing_unit_id
        }),
    );

    let report = graph.validation_report();
    let codes = report
        .diagnostics
        .iter()
        .map(|diagnostic| diagnostic.code)
        .collect::<std::collections::BTreeSet<_>>();

    assert!(!report.valid);
    assert_eq!(report.summary.diagnostics, 2);
    assert_eq!(report.summary.errors, 2);
    assert_eq!(report.summary.by_tier.get("registration").copied(), Some(1));
    assert_eq!(report.summary.by_tier.get("dependency").copied(), Some(1));
    assert_eq!(
        report.summary.by_code.get("duplicate_uuid").copied(),
        Some(1)
    );
    assert_eq!(
        report.summary.by_code.get("dangling_reference").copied(),
        Some(1)
    );
    assert!(codes.contains("duplicate_uuid"));
    assert!(codes.contains("dangling_reference"));
}

#[test]
fn library_graph_accepts_unambiguous_legacy_pin_pad_map_rows() {
    let unit_id = Uuid::from_u128(1);
    let pin_id = Uuid::from_u128(2);
    let symbol_id = Uuid::from_u128(3);
    let entity_id = Uuid::from_u128(4);
    let gate_id = Uuid::from_u128(5);
    let package_id = Uuid::from_u128(6);
    let footprint_id = Uuid::from_u128(7);
    let footprint_pad_id = Uuid::from_u128(8);
    let part_id = Uuid::from_u128(9);
    let map_id = Uuid::from_u128(10);
    let mut graph = legacy_pin_pad_map_graph(
        unit_id,
        pin_id,
        symbol_id,
        entity_id,
        &[(gate_id, unit_id, symbol_id)],
        package_id,
        footprint_id,
        footprint_pad_id,
        part_id,
        map_id,
    );
    graph.pin_pad_maps.insert(
        map_id,
        serde_json::json!({
            "uuid": map_id,
            "part": part_id,
            "footprint": footprint_id,
            "mappings": {
                pin_id: footprint_pad_id.to_string()
            }
        }),
    );

    let diagnostics = graph.dependency_diagnostics();

    assert!(
        diagnostics.is_empty(),
        "unexpected diagnostics: {diagnostics:?}"
    );
}

#[test]
fn library_graph_rejects_ambiguous_legacy_pin_pad_map_rows() {
    let unit_id = Uuid::from_u128(1);
    let pin_id = Uuid::from_u128(2);
    let symbol_id = Uuid::from_u128(3);
    let entity_id = Uuid::from_u128(4);
    let gate_a_id = Uuid::from_u128(5);
    let gate_b_id = Uuid::from_u128(6);
    let package_id = Uuid::from_u128(7);
    let footprint_id = Uuid::from_u128(8);
    let footprint_pad_id = Uuid::from_u128(9);
    let part_id = Uuid::from_u128(10);
    let map_id = Uuid::from_u128(11);
    let mut graph = legacy_pin_pad_map_graph(
        unit_id,
        pin_id,
        symbol_id,
        entity_id,
        &[
            (gate_a_id, unit_id, symbol_id),
            (gate_b_id, unit_id, symbol_id),
        ],
        package_id,
        footprint_id,
        footprint_pad_id,
        part_id,
        map_id,
    );
    graph.pin_pad_maps.insert(
        map_id,
        serde_json::json!({
            "uuid": map_id,
            "part": part_id,
            "footprint": footprint_id,
            "mappings": {
                pin_id: {
                    "pad": footprint_pad_id
                }
            }
        }),
    );

    let codes = graph
        .dependency_diagnostics()
        .into_iter()
        .map(|diagnostic| diagnostic.code)
        .collect::<std::collections::BTreeSet<_>>();

    assert!(codes.contains("ambiguous_legacy_pin_pad_map"));
}

#[allow(clippy::too_many_arguments)]
fn legacy_pin_pad_map_graph(
    unit_id: Uuid,
    pin_id: Uuid,
    symbol_id: Uuid,
    entity_id: Uuid,
    gates: &[(Uuid, Uuid, Uuid)],
    package_id: Uuid,
    footprint_id: Uuid,
    footprint_pad_id: Uuid,
    part_id: Uuid,
    map_id: Uuid,
) -> LibraryGraph {
    let padstack_id = Uuid::from_u128(99);
    let mut graph = LibraryGraph::default();
    graph.units.insert(
        unit_id,
        serde_json::json!({
            "uuid": unit_id,
            "pins": {
                pin_id: {
                    "uuid": pin_id,
                    "name": "A"
                }
            }
        }),
    );
    graph.symbols.insert(
        symbol_id,
        serde_json::json!({
            "uuid": symbol_id,
            "unit": unit_id
        }),
    );
    graph.entities.insert(
        entity_id,
        serde_json::json!({
            "uuid": entity_id,
            "gates": gates.iter().map(|(gate_id, gate_unit_id, gate_symbol_id)| {
                (
                    gate_id.to_string(),
                    serde_json::json!({
                        "uuid": gate_id,
                        "unit": gate_unit_id,
                        "symbol": gate_symbol_id
                    }),
                )
            }).collect::<serde_json::Map<_, _>>()
        }),
    );
    graph.packages.insert(
        package_id,
        serde_json::json!({
            "uuid": package_id,
            "pads": {}
        }),
    );
    graph.footprints.insert(
        footprint_id,
        serde_json::json!({
            "uuid": footprint_id,
            "package": package_id,
            "pads": {
                footprint_pad_id: {
                    "uuid": footprint_pad_id,
                    "name": "1",
                    "padstack": padstack_id
                }
            }
        }),
    );
    graph.padstacks.insert(
        padstack_id,
        serde_json::json!({
            "uuid": padstack_id
        }),
    );
    graph.parts.insert(
        part_id,
        serde_json::json!({
            "uuid": part_id,
            "entity": entity_id,
            "package": package_id
        }),
    );
    for id in [
        unit_id,
        symbol_id,
        entity_id,
        package_id,
        footprint_id,
        padstack_id,
        part_id,
        map_id,
    ] {
        graph.subjects.insert(id, format!("fixture/{id}.json"));
    }
    graph
}
