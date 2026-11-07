use super::*;
use eda_engine::ir::units::{LengthUnit, UnitsProfile};

fn p(x: i64, y: i64) -> PointNm {
    PointNm { x, y }
}
fn units() -> ResolvedUnitsProfile {
    UnitsProfile::default().resolve().unwrap()
}
fn entry(reference: PointNm, current: PointNm, mode: CoordinateEntryMode) -> CoordinateEntry {
    CoordinateEntry::new(
        Uuid::from_u128(1),
        LengthQuantity::SchematicGeometry,
        units(),
        reference,
        current,
        mode,
    )
    .unwrap()
}

#[test]
fn exact_offgrid_initial_fields_and_mode_roundtrip_do_not_round_geometry() {
    let reference = p(10_000_001, -4_000_003);
    let current = p(10_001_234, -3_987_655);
    let mut form = entry(reference, current, CoordinateEntryMode::Absolute);
    assert_eq!(form.destination().unwrap(), current);
    assert_eq!(form.project(), Uuid::from_u128(1));
    assert_eq!(form.reference(), reference);
    assert_eq!(form.quantity(), LengthQuantity::SchematicGeometry);
    form.set_mode(CoordinateEntryMode::Delta).unwrap();
    assert_eq!(form.destination().unwrap(), current);
    assert_eq!(
        (form.x.as_str(), form.y.as_str()),
        ("0.001233mm", "0.012348mm")
    );
    form.set_mode(CoordinateEntryMode::Absolute).unwrap();
    assert_eq!(form.destination().unwrap(), current);
    assert_eq!(form.current(), current);
    assert_eq!(form.x_editor.caret, form.x.len());
    assert_eq!(form.y_editor.caret, form.y.len());
}

#[test]
fn bare_input_uses_captured_project_units_while_explicit_suffix_is_exact() {
    let mut profile = units();
    let mut form = CoordinateEntry::new(
        Uuid::from_u128(2),
        LengthQuantity::SchematicGeometry,
        profile,
        p(0, 0),
        p(0, 0),
        CoordinateEntryMode::Absolute,
    )
    .unwrap();
    profile.schematic.unit = LengthUnit::Mil;
    form.x = "1.5".into();
    form.y = "-2".into();
    assert_eq!(form.destination().unwrap(), p(1_500_000, -2_000_000));
    assert_ne!(
        form.units(),
        profile,
        "later preferences cannot reinterpret captured buffers"
    );
    form.x = "1mil".into();
    form.y = "0.000017mm".into();
    assert_eq!(form.destination().unwrap(), p(25_400, 17));
}

#[test]
fn delta_input_adds_exact_parsed_offsets_without_background_snap() {
    let reference = p(10_000_001, -4_000_003);
    let mut form = entry(reference, reference, CoordinateEntryMode::Delta);
    form.x = "1mil".into();
    form.y = "-0.5mm".into();
    assert_eq!(form.destination().unwrap(), p(10_025_401, -4_500_003));
    form.set_mode(CoordinateEntryMode::Absolute).unwrap();
    assert_eq!(form.destination().unwrap(), p(10_025_401, -4_500_003));
}

#[test]
fn invalid_axis_refuses_mode_conversion_without_changing_text_or_caret() {
    let mut form = entry(p(1, 2), p(17, 29), CoordinateEntryMode::Absolute);
    form.x = "2mm".into();
    form.y = "NaN".into();
    form.x_editor = SingleLineEditor {
        caret: 1,
        anchor: 0,
    };
    let before = form.clone();
    let error = form.destination().unwrap_err();
    assert_eq!(error, "Invalid Y coordinate: Unsupported unit suffix 'NaN'");
    assert!(form.set_mode(CoordinateEntryMode::Delta).is_err());
    assert_eq!(form, before);
}

#[test]
fn mode_and_destination_updates_refuse_unrepresentable_delta_atomically() {
    let mut form = entry(
        p(i64::MIN, 0),
        p(i64::MAX, 17),
        CoordinateEntryMode::Absolute,
    );
    let before = form.clone();
    assert!(form.set_mode(CoordinateEntryMode::Delta).is_err());
    assert_eq!(form, before);
    let mut delta = entry(
        p(i64::MIN, 0),
        p(i64::MIN + 1, 17),
        CoordinateEntryMode::Delta,
    );
    let before = delta.clone();
    assert!(delta.set_destination(p(i64::MAX, 29)).is_err());
    assert_eq!(delta, before);
}

#[test]
fn parsed_delta_overflow_never_clamps_or_changes_the_reference() {
    let mut form = entry(p(i64::MAX, 0), p(i64::MAX, 0), CoordinateEntryMode::Delta);
    form.x = "1nm".into();
    form.y = "0nm".into();
    let before = form.clone();
    assert!(form.destination().is_err());
    assert!(form.set_mode(CoordinateEntryMode::Absolute).is_err());
    assert_eq!(form, before);
    form.x = "-1nm".into();
    assert_eq!(form.destination().unwrap(), p(i64::MAX - 1, 0));
}

#[test]
fn shared_form_can_use_board_quantity_and_preserves_caller_editor_operations() {
    let mut profile = units();
    profile.board.unit = LengthUnit::Inch;
    let mut form = CoordinateEntry::new(
        Uuid::from_u128(3),
        LengthQuantity::BoardLayout,
        profile,
        p(0, 0),
        p(0, 0),
        CoordinateEntryMode::Absolute,
    )
    .unwrap();
    form.x = "1".into();
    form.y = "2nm".into();
    assert_eq!(form.destination().unwrap(), p(25_400_000, 2));
    form.x_editor.select_all(&form.x);
    form.x_editor.replace(&mut form.x, "3nm").unwrap();
    assert_eq!(form.destination().unwrap(), p(3, 2));
    form.set_destination(p(17, -29)).unwrap();
    assert_eq!(form.destination().unwrap(), p(17, -29));
}
