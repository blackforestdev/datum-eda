use super::*;
use crate::ir::geometry::Point;
use crate::pool::{PinDirection, Primitive, SymbolPinOrientation};

// First-party parser inputs exercise refusal/identity semantics. Actual installed
// library coverage below reads the user's local source without copying it here.
fn source(extra: &str, pin: &str) -> String {
    format!(
        r#"(kicad_symbol_lib (version 20241209)
      (symbol "Base" (property "Reference" "U" (at 0 1 0))
        (symbol "Base_0_1" (rectangle (start -1 -2) (end 1 2)
          (stroke (width 0.2) (type default)) (fill (type none))))
        (symbol "Base_1_1" {pin})) {extra})"#
    )
}

const PIN: &str = r#"(pin input line (at -3 1 0) (length 2)
    (name "IN+") (number "3"))"#;

fn converted(source: &str, name: &str) -> Result<ConvertedSymbol> {
    let library = Library::parse(PathBuf::from("/local/test.kicad_sym"), source)?;
    convert::convert(&library, name)
}

#[test]
fn inheritance_keeps_geometry_and_distinct_designator_with_stable_identity() {
    let input = source(
        r#"(symbol "Derived" (extends "Base")
        (property "Reference" "X" (at 0 2 0)))"#,
        PIN,
    );
    let out = converted(&input, "Derived").unwrap();
    assert_eq!(out.symbol.default_refdes_prefix.as_deref(), Some("X"));
    assert_eq!(out.symbol.drawings.len(), 1);
    let pin = out.unit.pins.values().next().unwrap();
    assert_eq!(
        (pin.name.as_str(), &pin.direction),
        ("IN+", &PinDirection::Input)
    );
    let anchor = &out.symbol.pin_anchors[0];
    assert_eq!(anchor.number.as_deref(), Some("3"));
    assert_eq!(anchor.pin, pin.uuid);
    assert_eq!(anchor.position, Point::new(-3_000_000, -1_000_000));
    assert_eq!(anchor.style.orientation, SymbolPinOrientation::Right);
    assert_eq!(anchor.style.length_nm, Some(2_000_000));
    let edited = converted(&input.replace("IN+", "INPUT"), "Derived").unwrap();
    assert_eq!(out.unit.uuid, edited.unit.uuid);
    assert_eq!(out.symbol.uuid, edited.symbol.uuid);
    assert_eq!(anchor.pin, edited.symbol.pin_anchors[0].pin);
    assert_ne!(out.source_hash, edited.source_hash);
    assert!(out.symbol.check_state.is_none());
    assert!(
        out.symbol
            .provenance
            .as_ref()
            .unwrap()
            .reviewed_by
            .is_none()
    );
}

#[test]
fn missing_cyclic_multiunit_and_implicit_power_are_refused() {
    for (input, name, reason) in [
        (
            source(r#"(symbol "Missing" (extends "Absent"))"#, PIN),
            "Missing",
            "missing symbol",
        ),
        (
            source(
                r#"(symbol "A" (extends "B")) (symbol "B" (extends "A"))"#,
                PIN,
            ),
            "A",
            "cycle",
        ),
        (
            source("", PIN).replace("Base_1_1", "Base_2_1"),
            "Base",
            "multiunit",
        ),
        (
            source("", PIN).replace("(symbol \"Base\"", "(symbol \"Base\" (power)"),
            "Base",
            "power symbols",
        ),
        (
            source("", &PIN.replace("input line", "power_in line hide")),
            "Base",
            "hidden power",
        ),
        (
            source("", &PIN.replace("input line", "input inverted")),
            "Base",
            "decorated",
        ),
    ] {
        let error = converted(&input, name).unwrap_err().to_string();
        assert!(error.contains(reason), "{reason}: {error}");
    }
}

#[test]
fn parser_handles_quoted_delimiters_and_refuses_malformed_or_duplicate_sources() {
    let input = source("", &PIN.replace("IN+", "IN(+)\\\" quoted"));
    let out = converted(&input, "Base").unwrap();
    assert_eq!(
        out.unit.pins.values().next().unwrap().name,
        "IN(+)\" quoted"
    );
    for input in [
        "(kicad_symbol_lib",
        "(kicad_symbol_lib) trailing",
        "(kicad_symbol_lib (symbol \"A\") (symbol \"A\"))",
    ] {
        assert!(Library::parse(PathBuf::from("test"), input).is_err());
    }
    assert!(
        converted(&source("", &format!("{PIN} {PIN}")), "Base")
            .unwrap_err()
            .to_string()
            .contains("duplicate pin")
    );
}

#[test]
fn exact_coordinates_and_all_cardinal_stub_directions_are_preserved() {
    for (degrees, orientation) in [
        ("0", SymbolPinOrientation::Right),
        ("90", SymbolPinOrientation::Up),
        ("180", SymbolPinOrientation::Left),
        ("270", SymbolPinOrientation::Down),
    ] {
        let input = source(
            "",
            &PIN.replace("-3 1 0", &format!("-3.000001 1 {degrees}")),
        );
        let out = converted(&input, "Base").unwrap();
        let anchor = &out.symbol.pin_anchors[0];
        assert_eq!(anchor.position.x, -3_000_001);
        assert_eq!(anchor.style.orientation, orientation);
    }
    assert!(converted(&source("", &PIN.replace("-3 1 0", "0.0000001 1 0")), "Base").is_err());
    assert!(converted(&source("", &PIN.replace("-3 1 0", "-3 1 45")), "Base").is_err());
}

#[test]
fn arc_preserves_middle_side_after_y_axis_conversion() {
    let node = sexpr::parse("(arc (start -1 0) (mid 0 1) (end 1 0) (stroke (width 0.1) (type default)) (fill (type none)))").unwrap();
    let primitives = geometry::drawing(&node, &mut Vec::new()).unwrap();
    let Primitive::Arc { arc, .. } = &primitives[0] else {
        panic!("arc expected")
    };
    assert_eq!(arc.center, Point::zero());
    assert_eq!(arc.radius, 1_000_000);
    assert_eq!((arc.start_angle, arc.end_angle), (1800, 0));
}

#[test]
fn circle_and_arc_refuse_overflowing_extents_even_when_source_scalars_fit() {
    for geometry in [
        "(circle (center 9223372036854.775806 0) (radius 0.000002)",
        "(circle (center -9223372036854.775807 0) (radius 0.000002)",
        "(circle (center 0 9223372036854.775807) (radius 0.000002)",
        "(circle (center 0 -9223372036854.775806) (radius 0.000002)",
        "(arc (start 5000000000000 5000000000000) (mid 0 0) (end 5000000000000 -5000000000000)",
    ] {
        let node = sexpr::parse(&format!(
            "{geometry} (stroke (width 0.1) (type default)) (fill (type none)))"
        ))
        .unwrap();
        let error = geometry::drawing(&node, &mut Vec::new()).unwrap_err();
        assert!(error.to_string().contains("extents"), "{geometry}: {error}");
    }
    let node = sexpr::parse("(circle (center 9223372036854.775805 0) (radius 0.000002) (stroke (width 0.1) (type default)) (fill (type none)))").unwrap();
    assert!(
        geometry::drawing(&node, &mut Vec::new()).is_ok(),
        "exactly representable boundary remains valid"
    );
}

#[test]
#[ignore = "requires installed local KiCad libraries; explicitly run for local-source proof"]
fn installed_real_resistor_capacitor_connector_and_lm741() {
    for (library, name, count, prefix) in [
        ("Device", "R", 2, "R"),
        ("Device", "C", 2, "C"),
        ("Connector_Generic", "Conn_01x02", 2, "J"),
        ("Amplifier_Operational", "LM741", 8, "U"),
    ] {
        let path = PathBuf::from(format!("/usr/share/kicad/symbols/{library}.kicad_sym"));
        assert!(catalog(&path).unwrap().iter().any(|s| s.name == name));
        let before = std::fs::read(&path).unwrap();
        let out = convert(&path, name).unwrap();
        assert_eq!(out.unit.pins.len(), count, "{name}");
        assert_eq!(out.symbol.pin_anchors.len(), count);
        assert_eq!(out.symbol.default_refdes_prefix.as_deref(), Some(prefix));
        assert!(!out.symbol.drawings.is_empty());
        assert_eq!(std::fs::read(&path).unwrap(), before);
        if name == "LM741" {
            let anchor = out
                .symbol
                .pin_anchors
                .iter()
                .find(|a| a.number.as_deref() == Some("3"))
                .unwrap();
            assert_eq!(out.unit.pins[&anchor.pin].name, "+");
            assert_eq!(anchor.position, Point::new(-7_620_000, -2_540_000));
            assert!(out.warnings.iter().any(|w| w.contains("outlines")));
        }
    }
}
