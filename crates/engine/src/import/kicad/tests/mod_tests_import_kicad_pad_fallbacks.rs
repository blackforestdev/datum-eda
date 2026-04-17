#[test]
fn parse_pad_shape_rejects_unknown_shape_in_pad_head() {
    let block = r#"(pad "1" smd chamfered_rect (at 0 0) (size 1 1) (layers "F.Cu"))"#;
    assert_eq!(crate::import::kicad::skeleton::parse_pad_shape_anywhere(block), None);
}

#[test]
fn parse_pad_kind_recognizes_supported_head_kinds() {
    let block = r#"(pad "1" thru_hole circle (at 0 0) (size 1.5 1.5) (drill 0.8) (layers "*.Cu" "*.Mask"))"#;
    let kind = crate::import::kicad::skeleton::parse_pad_kind_anywhere(block);
    assert!(matches!(kind, Some(crate::import::kicad::skeleton::KiCadPadKind::ThruHole)));
}

#[test]
fn parse_pad_drill_requires_drill_for_through_hole_pad() {
    let block = r#"(pad "1" thru_hole circle (at 0 0) (size 1.5 1.5) (layers "*.Cu" "*.Mask"))"#;
    let drill = crate::import::kicad::skeleton::parse_pad_drill_anywhere(
        block,
        crate::import::kicad::skeleton::KiCadPadKind::ThruHole,
    );
    assert_eq!(drill, None);
}

#[test]
fn parse_pad_drill_defaults_smd_to_zero_without_silent_hole_guess() {
    let block = r#"(pad "1" smd roundrect (at 0 0) (size 1.5 0.8) (layers "F.Cu" "F.Mask" "F.Paste"))"#;
    let drill = crate::import::kicad::skeleton::parse_pad_drill_anywhere(
        block,
        crate::import::kicad::skeleton::KiCadPadKind::Smd,
    );
    assert_eq!(drill, Some(0));
}
