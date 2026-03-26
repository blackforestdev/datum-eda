use super::*;
use crate::import::kicad::symbol_helpers::transform_symbol_pin;

#[test]
fn imports_kicad_project_metadata() {
    let report = import_project_file(&fixture_path("simple-demo.kicad_pro")).expect("fixture should parse");

    assert_eq!(report.kind, ImportKind::KiCadProject);
    assert!(report.counts.is_empty());
    assert_eq!(
        report.metadata.get("project_name").map(String::as_str),
        Some("simple-demo")
    );
    assert_eq!(
        report.metadata.get("project_version").map(String::as_str),
        Some("1")
    );
    assert_eq!(report.warnings.len(), 1);
}

#[test]
fn imports_kicad_board_header_and_skeleton_counts() {
    let (board, report) =
        import_board_document(&fixture_path("simple-demo.kicad_pcb")).expect("fixture should parse");

    assert_eq!(report.kind, ImportKind::KiCadBoard);
    assert!(report.counts.is_empty());
    assert_eq!(
        report.metadata.get("kicad_version").map(String::as_str),
        Some("20221018")
    );
    assert_eq!(
        report.metadata.get("footprint_count").map(String::as_str),
        Some("1")
    );
    assert_eq!(
        report.metadata.get("segment_count").map(String::as_str),
        Some("1")
    );
    assert_eq!(
        report.metadata.get("via_count").map(String::as_str),
        Some("1")
    );
    assert_eq!(
        report.metadata.get("zone_count").map(String::as_str),
        Some("1")
    );
    assert_eq!(
        report.metadata.get("net_count").map(String::as_str),
        Some("2")
    );
    assert_eq!(
        report.metadata.get("gr_line_count").map(String::as_str),
        Some("1")
    );
    assert_eq!(
        report.metadata.get("gr_arc_count").map(String::as_str),
        Some("1")
    );
    assert_eq!(
        report.metadata.get("dimension_count").map(String::as_str),
        Some("1")
    );
    assert_eq!(
        report.metadata.get("gr_text_count").map(String::as_str),
        Some("1")
    );
    assert_eq!(report.warnings.len(), 1);
    assert_eq!(board.name, "simple-demo");
    assert_eq!(board.packages.len(), 1);
    assert_eq!(board.tracks.len(), 1);
    assert_eq!(board.vias.len(), 1);
    assert_eq!(board.zones.len(), 1);
    assert_eq!(board.nets.len(), 2);
    assert_eq!(board.texts.len(), 1);
    assert_eq!(board.texts[0].text, "Demo");
    assert_eq!(board.texts[0].layer, 37);
    assert_eq!(board.stackup.layers.len(), 3);
}

#[test]
fn imports_kicad_board_pads_for_unrouted_computation() {
    let (board, report) =
        import_board_document(&fixture_path("airwire-demo.kicad_pcb")).expect("fixture should parse");

    assert_eq!(report.kind, ImportKind::KiCadBoard);
    assert_eq!(board.packages.len(), 2);
    assert_eq!(board.pads.len(), 2);
    assert_eq!(board.tracks.len(), 0);
    assert_eq!(board.vias.len(), 0);
    assert_eq!(board.zones.len(), 0);

    let nets = board.net_info();
    assert_eq!(nets.len(), 2);
    let sig = nets
        .iter()
        .find(|net| net.name == "SIG")
        .expect("SIG net should exist");
    assert_eq!(sig.pins.len(), 2);
    assert_eq!(sig.tracks, 0);

    let airwires = board.unrouted();
    assert_eq!(airwires.len(), 1);
    assert_eq!(airwires[0].net_name, "SIG");
    assert_eq!(airwires[0].from.component, "R1");
    assert_eq!(airwires[0].to.component, "R2");
}

#[test]
fn imports_kicad_board_partial_route_still_reports_airwire() {
    let (board, report) = import_board_document(&fixture_path("partial-route-demo.kicad_pcb"))
        .expect("fixture should parse");

    assert_eq!(report.kind, ImportKind::KiCadBoard);
    assert_eq!(board.packages.len(), 2);
    assert_eq!(board.pads.len(), 2);
    assert_eq!(board.tracks.len(), 1);

    let nets = board.net_info();
    let sig = nets
        .iter()
        .find(|net| net.name == "SIG")
        .expect("SIG net should exist");
    assert_eq!(sig.pins.len(), 2);
    assert_eq!(sig.tracks, 1);

    let airwires = board.unrouted();
    assert_eq!(airwires.len(), 1);
    assert_eq!(airwires[0].net_name, "SIG");
    assert_eq!(airwires[0].from.component, "R1");
    assert_eq!(airwires[0].to.component, "R2");
}

#[test]
fn mirrored_symbol_pin_transform_reflects_local_x_before_rotation() {
    let origin = Point::new(73_660_000, 105_410_000);

    let base = transform_symbol_pin(origin, 0, true, Point::new(-5_080_000, 0));
    let collector = transform_symbol_pin(origin, 0, true, Point::new(2_540_000, 5_080_000));
    let emitter = transform_symbol_pin(origin, 0, true, Point::new(2_540_000, -5_080_000));

    assert_eq!(base, Point::new(78_740_000, 105_410_000));
    assert_eq!(collector, Point::new(71_120_000, 110_490_000));
    assert_eq!(emitter, Point::new(71_120_000, 100_330_000));
}
