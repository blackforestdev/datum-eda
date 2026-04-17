use super::*;

#[test]
fn imports_real_doa2526_board_with_copper_geometry() {
    let Some(path) = optional_doa2526_board_path() else {
        return;
    };

    let (board, report) = import_board_document(&path).expect("DOA2526 board should parse");

    assert_eq!(report.kind, ImportKind::KiCadBoard);
    assert!(!board.packages.is_empty());
    assert!(!board.pads.is_empty());
    assert!(
        !board.tracks.is_empty(),
        "real DOA2526 board should import tracks"
    );
    assert!(
        !board.vias.is_empty(),
        "real DOA2526 board should import vias"
    );
    assert!(
        !board.zones.is_empty(),
        "real DOA2526 board should import zones"
    );

    let diagnostics = board.diagnostics();
    assert!(
        diagnostics.iter().any(|d| d.kind != "net_without_copper"),
        "real DOA2526 board should not collapse to only empty-copper diagnostics"
    );

    let net_info = board.net_info();
    assert!(
        net_info
            .iter()
            .any(|net| net.tracks > 0 || net.vias > 0 || net.zones > 0),
        "real DOA2526 board should report imported copper on at least one net"
    );
}

#[test]
fn debug_real_doa2526_remaining_airwires() {
    let Some(path) = optional_doa2526_board_path() else {
        return;
    };

    let (board, _report) = import_board_document(&path).expect("DOA2526 board should parse");
    let airwires = board.unrouted();
    eprintln!("remaining airwires: {}", airwires.len());
    for airwire in &airwires {
        eprintln!(
            "net={} from={}:{} ({}, {}) to={}:{} ({}, {}) distance={}",
            airwire.net_name,
            airwire.from.component,
            airwire.from.pin,
            airwire.from_position.x,
            airwire.from_position.y,
            airwire.to.component,
            airwire.to.pin,
            airwire.to_position.x,
            airwire.to_position.y,
            airwire.distance_nm
        );
    }
}

#[test]
fn debug_real_doa2526_remaining_airwire_pad_zone_state() {
    let Some(path) = optional_doa2526_board_path() else {
        return;
    };

    let (board, _report) = import_board_document(&path).expect("DOA2526 board should parse");
    for (reference, pin, net_name) in [
        ("R6", "1", "/VCC"),
        ("R8", "1", "/VCC"),
        ("R15", "2", "/VEE"),
        ("R1", "2", "/VEE"),
    ] {
        let (package_uuid, package) = board
            .packages
            .iter()
            .find(|(_, pkg)| pkg.reference == reference)
            .expect("package should exist");
        let pad = board
            .pads
            .values()
            .find(|pad| pad.package == *package_uuid && pad.name == pin)
            .expect("pad should exist");
        let net_uuid = board
            .nets
            .values()
            .find(|net| net.name == net_name)
            .expect("net should exist")
            .uuid;
        eprintln!(
            "{reference}:{pin} pos=({}, {}) layer={} copper_layers={:?} rot={} size={}x{} dia={} net={}",
            pad.position.x,
            pad.position.y,
            pad.layer,
            pad.copper_layers,
            pad.rotation,
            pad.width,
            pad.height,
            pad.diameter,
            net_name
        );
        for zone in board.zones.values().filter(|zone| zone.net == net_uuid) {
            let center_in = debug_point_in_polygon(pad.position, &zone.polygon);
            eprintln!(
                "  zone layer={} center_in={} thermal={} gap={} spoke={}",
                zone.layer, center_in, zone.thermal_relief, zone.thermal_gap, zone.thermal_spoke_width
            );
        }
        let connected_tracks: Vec<_> = board
            .tracks
            .values()
            .filter(|track| track.net == net_uuid)
            .filter(|track| {
                track.from.distance_sq(&pad.position) < 10_000_000_000
                    || track.to.distance_sq(&pad.position) < 10_000_000_000
            })
            .collect();
        eprintln!("  nearby_tracks={}", connected_tracks.len());
        for track in connected_tracks {
            eprintln!(
                "    track layer={} from=({}, {}) to=({}, {})",
                track.layer, track.from.x, track.from.y, track.to.x, track.to.y
            );
        }
        let nearby_vias: Vec<_> = board
            .vias
            .values()
            .filter(|via| via.net == net_uuid)
            .filter(|via| via.position.distance_sq(&pad.position) < 10_000_000_000)
            .collect();
        eprintln!("  nearby_vias={}", nearby_vias.len());
        for via in nearby_vias {
            eprintln!(
                "    via at=({}, {}) {}->{}",
                via.position.x, via.position.y, via.from_layer, via.to_layer
            );
        }
        let _ = package;
    }
}

fn debug_point_in_polygon(point: crate::ir::geometry::Point, polygon: &crate::ir::geometry::Polygon) -> bool {
    let Some(bounds) = polygon.bounding_box() else {
        return false;
    };
    if !bounds.contains(&point) || polygon.vertices.len() < 3 {
        return false;
    }
    let mut inside = false;
    let mut j = polygon.vertices.len() - 1;
    for i in 0..polygon.vertices.len() {
        let xi = polygon.vertices[i].x as f64;
        let yi = polygon.vertices[i].y as f64;
        let xj = polygon.vertices[j].x as f64;
        let yj = polygon.vertices[j].y as f64;
        let px = point.x as f64;
        let py = point.y as f64;
        let intersects =
            ((yi > py) != (yj > py)) && (px < (xj - xi) * (py - yi) / ((yj - yi).max(1.0)) + xi);
        if intersects {
            inside = !inside;
        }
        j = i;
    }
    inside
}

#[test]
fn imports_real_doa2526_schematic_without_collapsing_anonymous_nets() {
    let Some(path) = optional_doa2526_schematic_path() else {
        return;
    };

    let (schematic, report) =
        import_schematic_document(&path).expect("DOA2526 schematic should parse");

    assert_eq!(report.kind, ImportKind::KiCadSchematic);

    let nets = crate::connectivity::schematic_net_info(&schematic);
    assert!(!nets.is_empty(), "real DOA2526 schematic should yield nets");

    let unique_net_ids: std::collections::HashSet<_> = nets.iter().map(|net| net.uuid).collect();
    assert_eq!(
        unique_net_ids.len(),
        nets.len(),
        "real DOA2526 schematic should not reuse the same net UUID across distinct aggregates"
    );

    let anonymous_names: std::collections::HashSet<_> = nets
        .iter()
        .filter(|net| net.name.starts_with("N$"))
        .map(|net| net.name.clone())
        .collect();
    assert!(
        anonymous_names.len() > 1,
        "real DOA2526 schematic should produce distinct anonymous net identities"
    );
}

#[test]
fn imports_real_doa2526_plusin_pin_at_expected_position() {
    let Some(path) = optional_doa2526_schematic_path() else {
        return;
    };

    let (schematic, _report) =
        import_schematic_document(&path).expect("DOA2526 schematic should parse");
    let root = schematic
        .sheets
        .values()
        .find(|sheet| sheet.name == "Root")
        .expect("root sheet should exist");
    let plus_in = root
        .symbols
        .values()
        .find(|symbol| symbol.reference == "+In1")
        .expect("+In1 should exist");
    let pin = plus_in
        .pins
        .iter()
        .find(|pin| pin.number == "1")
        .expect("+In1 pin 1 should exist");

    assert_eq!(pin.position.x, 39_370_000);
    assert_eq!(pin.position.y, 105_410_000);
}

#[test]
fn imports_real_doa2526_pad_level_mask_and_paste_margins() {
    let Some(path) = optional_doa2526_board_path() else {
        return;
    };

    let (board, _report) = import_board_document(&path).expect("DOA2526 board should parse");
    let q9_uuid = board
        .packages
        .iter()
        .find(|(_, pkg)| pkg.reference == "Q9")
        .map(|(uuid, _)| *uuid)
        .expect("Q9 should exist");
    let pad = board
        .pads
        .values()
        .find(|pad| pad.package == q9_uuid && pad.name == "1")
        .expect("Q9 pad 1 should exist");
    assert_eq!(pad.solder_mask_margin_nm, 50_000);
    assert_eq!(pad.solder_paste_margin_nm, -50_000);
}

#[test]
fn real_doa2526_named_nets_attach_expected_pins() {
    let Some(path) = optional_doa2526_schematic_path() else {
        return;
    };

    let (schematic, _report) =
        import_schematic_document(&path).expect("DOA2526 schematic should parse");
    let nets = crate::connectivity::schematic_net_info(&schematic);

    let in_p = nets
        .iter()
        .find(|net| net.name == "IN_P")
        .expect("IN_P net should exist");
    assert!(
        in_p.pins
            .iter()
            .any(|pin| pin.component == "+In1" && pin.pin == "1")
    );
    assert!(
        in_p.pins
            .iter()
            .any(|pin| pin.component == "Q1" && pin.pin == "1")
    );

    let vcc = nets
        .iter()
        .find(|net| net.name == "VCC")
        .expect("VCC net should exist");
    assert!(
        vcc.pins
            .iter()
            .any(|pin| pin.component == "+Supply1" && pin.pin == "1")
    );
    assert!(
        vcc.pins
            .iter()
            .any(|pin| pin.component == "#FLG01" && pin.pin == "1")
    );
    assert!(
        vcc.pins.len() >= 4,
        "VCC should attach multiple component pins on DOA2526"
    );

    let q2_col = nets
        .iter()
        .find(|net| net.name == "IN_Q2_COL")
        .expect("IN_Q2_COL net should exist");
    assert!(
        q2_col
            .pins
            .iter()
            .any(|pin| pin.component == "Q2" && pin.pin == "2"),
        "Q2 emitter should land on IN_Q2_COL after mirrored pin transform"
    );

    let ff_q4_c = nets
        .iter()
        .find(|net| net.name == "FF_Q4_C")
        .expect("FF_Q4_C net should exist");
    assert!(
        ff_q4_c
            .pins
            .iter()
            .any(|pin| pin.component == "Q4" && pin.pin == "2"),
        "Q4 emitter should land on FF_Q4_C after mirrored pin transform"
    );

    let in_n = nets
        .iter()
        .find(|net| net.name == "IN_N")
        .expect("IN_N net should exist");
    assert!(
        in_n.pins
            .iter()
            .any(|pin| pin.component == "Q2" && pin.pin == "1"),
        "Q2 base should land on IN_N after mirrored pin transform"
    );

    let in_q1_col = nets
        .iter()
        .find(|net| net.name == "IN_Q1_COL")
        .expect("IN_Q1_COL net should exist");
    assert!(
        in_q1_col
            .pins
            .iter()
            .any(|pin| pin.component == "Q4" && pin.pin == "1"),
        "Q4 base should land on IN_Q1_COL after mirrored pin transform"
    );
}

#[test]
fn real_doa2526_mirrored_transistors_are_not_reported_as_unconnected() {
    let Some(path) = optional_doa2526_schematic_path() else {
        return;
    };

    let (schematic, _report) =
        import_schematic_document(&path).expect("DOA2526 schematic should parse");
    let findings = crate::erc::run_prechecks(&schematic);

    for pin in ["Q2.1", "Q2.2", "Q2.3", "Q4.1", "Q4.2", "Q4.3"] {
        assert!(
            !findings.iter().any(|finding| {
                finding.code == "unconnected_component_pin"
                    && finding
                        .objects
                        .iter()
                        .any(|object| object.kind == "pin" && object.key == pin)
            }),
            "{pin} should no longer be reported as an unconnected component pin on DOA2526"
        );
    }
}
