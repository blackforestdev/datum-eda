use super::*;
use crate::schematic::{LabelKind, PortDirection};
use uuid::Uuid;

#[test]
fn imports_kicad_schematic_header_and_skeleton_counts() {
    let report = import_schematic_file(&fixture_path("simple-demo.kicad_sch"))
        .expect("fixture should parse");

    assert_eq!(report.kind, ImportKind::KiCadSchematic);
    assert!(report.counts.is_empty());
    assert_eq!(
        report.metadata.get("kicad_version").map(String::as_str),
        Some("20230121")
    );
    assert_eq!(
        report.metadata.get("symbol_count").map(String::as_str),
        Some("1")
    );
    assert_eq!(
        report.metadata.get("wire_count").map(String::as_str),
        Some("1")
    );
    assert_eq!(
        report.metadata.get("junction_count").map(String::as_str),
        Some("1")
    );
    assert_eq!(
        report.metadata.get("label_count").map(String::as_str),
        Some("1")
    );
    assert_eq!(
        report
            .metadata
            .get("global_label_count")
            .map(String::as_str),
        Some("1")
    );
    assert_eq!(
        report
            .metadata
            .get("hierarchical_label_count")
            .map(String::as_str),
        Some("1")
    );
    assert_eq!(
        report.metadata.get("bus_count").map(String::as_str),
        Some("1")
    );
    assert_eq!(
        report.metadata.get("sheet_count").map(String::as_str),
        Some("1")
    );
    assert_eq!(
        report.metadata.get("no_connect_count").map(String::as_str),
        Some("1")
    );
    assert_eq!(report.warnings.len(), 1);
}

#[test]
fn imports_kicad_schematic_into_canonical_objects() {
    let (schematic, report) = import_schematic_document(&fixture_path("simple-demo.kicad_sch"))
        .expect("fixture should parse");

    assert_eq!(report.kind, ImportKind::KiCadSchematic);
    assert_eq!(schematic.sheets.len(), 2);

    let root = schematic
        .sheets
        .values()
        .find(|sheet| sheet.name == "Root")
        .expect("root sheet should exist");
    assert_eq!(root.symbols.len(), 1);
    assert_eq!(root.wires.len(), 1);
    assert_eq!(root.junctions.len(), 1);
    assert_eq!(root.labels.len(), 3);
    assert_eq!(root.buses.len(), 1);
    assert_eq!(root.ports.len(), 1);
    assert_eq!(root.noconnects.len(), 1);
    assert_eq!(schematic.sheet_instances.len(), 1);
    let child = schematic
        .sheets
        .values()
        .find(|sheet| sheet.name == "Sub")
        .expect("child sheet should exist");
    assert_eq!(child.symbols.len(), 1);
    assert_eq!(child.labels.len(), 1);
    assert!(child.ports.is_empty());

    let symbol = root
        .symbols
        .values()
        .find(|symbol| symbol.reference == "R1")
        .expect("R1 symbol should exist");
    assert_eq!(symbol.value, "10k");
    assert_eq!(symbol.lib_id.as_deref(), Some("Device:R"));
    assert_eq!(symbol.position, Point::new(25_000_000, 20_000_000));
    assert_eq!(symbol.pins.len(), 2);
    assert!(symbol.pins.iter().any(|pin| {
        pin.number == "1"
            && pin.position == Point::new(20_000_000, 20_000_000)
            && pin.electrical_type == PinElectricalType::Passive
    }));

    let local = root
        .labels
        .values()
        .find(|label| label.kind == LabelKind::Local)
        .expect("local label should exist");
    assert_eq!(local.name, "SCL");
    assert_eq!(local.position, Point::new(20_000_000, 20_000_000));

    let hier = root
        .labels
        .values()
        .find(|label| label.kind == LabelKind::Hierarchical)
        .expect("hierarchical label should exist");
    assert_eq!(hier.name, "SUB_IN");
    assert_eq!(hier.position, Point::new(80_000_000, 15_000_000));

    let port = root
        .ports
        .values()
        .find(|port| port.name == "SUB_IN")
        .expect("sheet pin should exist");
    assert_eq!(port.direction, PortDirection::Input);
    assert_eq!(port.position, Point::new(60_000_000, 15_000_000));
}

#[test]
fn imports_kicad_noconnect_with_pin_binding_when_marker_overlaps_pin() {
    let (schematic, _report) =
        import_schematic_document(&fixture_path("erc-coverage-demo.kicad_sch"))
            .expect("fixture should parse");
    let root = schematic
        .sheets
        .values()
        .next()
        .expect("root sheet should exist");
    let marker = root
        .noconnects
        .values()
        .next()
        .expect("fixture should include one no_connect marker");
    assert_ne!(marker.symbol, Uuid::nil());
    assert_ne!(marker.pin, Uuid::nil());
}

#[test]
fn imports_kicad_bus_members_and_entries_for_supported_subset() {
    let (schematic, _report) = import_schematic_document(&fixture_path("bus-demo.kicad_sch"))
        .expect("fixture should parse");
    let root = schematic
        .sheets
        .values()
        .find(|sheet| sheet.name == "Root")
        .expect("root sheet should exist");

    assert_eq!(root.buses.len(), 1);
    let bus = root.buses.values().next().expect("bus should exist");
    assert_eq!(bus.name, "DATA");
    assert_eq!(bus.members, vec!["DATA0".to_string(), "DATA1".to_string()]);

    assert_eq!(root.bus_entries.len(), 2);
    assert!(
        root.bus_entries
            .values()
            .all(|entry| entry.bus == bus.uuid && entry.wire.is_some())
    );
}
