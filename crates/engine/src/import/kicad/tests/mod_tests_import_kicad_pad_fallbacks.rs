#[test]
fn parse_pad_shape_rejects_unknown_shape_in_pad_head() {
    let block = r#"(pad "1" smd chamfered_rect (at 0 0) (size 1 1) (layers "F.Cu"))"#;
    assert_eq!(
        crate::import::kicad::skeleton::parse_pad_shape_anywhere(block),
        None
    );
}

#[test]
fn parse_pad_kind_recognizes_supported_head_kinds() {
    let block = r#"(pad "1" thru_hole circle (at 0 0) (size 1.5 1.5) (drill 0.8) (layers "*.Cu" "*.Mask"))"#;
    let kind = crate::import::kicad::skeleton::parse_pad_kind_anywhere(block);
    assert!(matches!(
        kind,
        Some(crate::import::kicad::skeleton::KiCadPadKind::ThruHole)
    ));
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
    let block =
        r#"(pad "1" smd roundrect (at 0 0) (size 1.5 0.8) (layers "F.Cu" "F.Mask" "F.Paste"))"#;
    let drill = crate::import::kicad::skeleton::parse_pad_drill_anywhere(
        block,
        crate::import::kicad::skeleton::KiCadPadKind::Smd,
    );
    assert_eq!(drill, Some(0));
}

#[test]
fn board_import_retains_declared_pad_process_without_drill_inference() {
    use crate::board::PadLayerConnection;
    let source = r#"(kicad_pcb (version 20240108)
  (layers
    (0 "F.Cu" signal)
    (31 "B.Cu" signal)
  )
  (footprint "proof"
    (layer "F.Cu")
    (at 0 0)
    (uuid 00000000-0000-0000-0000-000000000001)
    (pad "plated" thru_hole circle
      (at 0 0)
      (size 2 2)
      (drill 1)
      (layers "*.Cu")
    )
    (pad "nonplated" np_thru_hole circle
      (at 3 0)
      (size 2 2)
      (drill 1)
      (layers "*.Cu")
    )
    (pad "surface" smd circle
      (at 6 0)
      (size 2 2)
      (layers "F.Cu")
    )
  )
)"#;
    let (board, _) = crate::import::kicad::skeleton::parse_board_skeleton(
        std::path::Path::new("process.kicad_pcb"),
        source,
        None,
        None,
    )
    .unwrap();
    let by_name = |name: &str| board.pads.values().find(|pad| pad.name == name).unwrap();
    assert_eq!(
        by_name("plated").layer_connection,
        PadLayerConnection::PlatedThrough
    );
    assert_eq!(
        by_name("nonplated").layer_connection,
        PadLayerConnection::Separate
    );
    assert_eq!(
        by_name("surface").layer_connection,
        PadLayerConnection::Separate
    );
    assert_eq!(by_name("plated").drill, by_name("nonplated").drill);
    assert_eq!(
        by_name("plated").copper_layers,
        by_name("nonplated").copper_layers
    );
}
