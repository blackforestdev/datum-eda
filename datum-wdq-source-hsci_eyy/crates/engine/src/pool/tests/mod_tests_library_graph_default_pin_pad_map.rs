use super::*;

#[test]
fn library_graph_default_pin_pad_map_without_footprint_uses_part_default_footprint_pads() {
    let unit_id = Uuid::from_u128(900);
    let pin_id = Uuid::from_u128(901);
    let symbol_id = Uuid::from_u128(902);
    let entity_id = Uuid::from_u128(903);
    let gate_id = Uuid::from_u128(904);
    let package_id = Uuid::from_u128(905);
    let footprint_id = Uuid::from_u128(906);
    let package_pad_id = Uuid::from_u128(907);
    let footprint_pad_id = Uuid::from_u128(908);
    let padstack_id = Uuid::from_u128(909);
    let part_id = Uuid::from_u128(910);
    let map_id = Uuid::from_u128(911);
    let mut graph = default_pin_pad_map_default_footprint_graph(
        unit_id,
        pin_id,
        symbol_id,
        entity_id,
        gate_id,
        package_id,
        footprint_id,
        footprint_pad_id,
        padstack_id,
        part_id,
        map_id,
    );
    graph.packages.insert(
        package_id,
        serde_json::json!({
            "uuid": package_id,
            "pads": {
                package_pad_id: {
                    "uuid": package_pad_id,
                    "name": "P1",
                    "padstack": padstack_id
                }
            }
        }),
    );
    graph.pin_pad_maps.insert(
        map_id,
        serde_json::json!({
            "uuid": map_id,
            "part": part_id,
            "mappings": {
                package_pad_id: {
                    "gate": gate_id,
                    "pin": pin_id
                }
            }
        }),
    );

    let diagnostics = graph.dependency_diagnostics();

    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic.code == "dangling_reference"
            && diagnostic
                .subject
                .ends_with(&format!("#mappings/{package_pad_id}"))
            && diagnostic
                .message
                .contains("references missing footprint pad")
    }));
}

#[test]
fn library_graph_default_pin_pad_map_without_footprint_accepts_default_footprint_pad() {
    let unit_id = Uuid::from_u128(920);
    let pin_id = Uuid::from_u128(921);
    let symbol_id = Uuid::from_u128(922);
    let entity_id = Uuid::from_u128(923);
    let gate_id = Uuid::from_u128(924);
    let package_id = Uuid::from_u128(925);
    let footprint_id = Uuid::from_u128(926);
    let footprint_pad_id = Uuid::from_u128(927);
    let padstack_id = Uuid::from_u128(928);
    let part_id = Uuid::from_u128(929);
    let map_id = Uuid::from_u128(930);
    let mut graph = default_pin_pad_map_default_footprint_graph(
        unit_id,
        pin_id,
        symbol_id,
        entity_id,
        gate_id,
        package_id,
        footprint_id,
        footprint_pad_id,
        padstack_id,
        part_id,
        map_id,
    );
    graph.pin_pad_maps.insert(
        map_id,
        serde_json::json!({
            "uuid": map_id,
            "part": part_id,
            "mappings": {
                footprint_pad_id: {
                    "gate": gate_id,
                    "pin": pin_id
                }
            }
        }),
    );

    let diagnostics = graph.dependency_diagnostics();

    assert!(
        diagnostics.is_empty(),
        "unexpected diagnostics: {diagnostics:?}"
    );
}

#[allow(clippy::too_many_arguments)]
fn default_pin_pad_map_default_footprint_graph(
    unit_id: Uuid,
    pin_id: Uuid,
    symbol_id: Uuid,
    entity_id: Uuid,
    gate_id: Uuid,
    package_id: Uuid,
    footprint_id: Uuid,
    footprint_pad_id: Uuid,
    padstack_id: Uuid,
    part_id: Uuid,
    map_id: Uuid,
) -> LibraryGraph {
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
            "gates": {
                gate_id: {
                    "uuid": gate_id,
                    "unit": unit_id,
                    "symbol": symbol_id
                }
            }
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
    graph
        .padstacks
        .insert(padstack_id, serde_json::json!({ "uuid": padstack_id }));
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
