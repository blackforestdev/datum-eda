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
