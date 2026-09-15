use super::kicad_scene_import::*;
use super::*;

#[test]
fn kicad_text_justify_defaults_to_center_center() {
    let justify = kicad_parse_text_justify(
        r#"(property "Reference" "R2"
  (effects
    (font (size 1 1) (thickness 0.15))
  )
)"#,
    );
    assert_eq!(justify.h, KicadTextHJustify::Center);
    assert_eq!(justify.v, KicadTextVJustify::Center);
    assert!(!justify.mirrored);
    assert!(!justify.keep_upright);
}

#[test]
fn kicad_text_justify_parses_left_and_bottom_tokens() {
    let justify = kicad_parse_text_justify(
        r#"(fp_text user "OUT"
  (effects
    (font (size 0.8 0.8) (thickness 0.12))
    (justify left bottom)
  )
)"#,
    );
    assert_eq!(justify.h, KicadTextHJustify::Left);
    assert_eq!(justify.v, KicadTextVJustify::Bottom);
}

#[test]
fn kicad_text_justify_parses_mirror_token() {
    let justify = kicad_parse_text_justify(
        r#"(fp_text user "OUT"
  (effects
    (font (size 0.8 0.8) (thickness 0.12))
    (justify left mirror)
  )
)"#,
    );
    assert_eq!(justify.h, KicadTextHJustify::Left);
    assert!(justify.mirrored);
}

#[test]
fn kicad_text_attributes_maps_justify_into_engine_alignments() {
    let left = kicad_text_attributes(
        PointNm { x: 0, y: 0 },
        0,
        800_000,
        None,
        KicadTextJustify {
            h: KicadTextHJustify::Left,
            v: KicadTextVJustify::Center,
            mirrored: false,
            keep_upright: false,
        },
    );
    let center = kicad_text_attributes(
        PointNm { x: 0, y: 0 },
        0,
        800_000,
        None,
        KicadTextJustify {
            h: KicadTextHJustify::Center,
            v: KicadTextVJustify::Center,
            mirrored: false,
            keep_upright: false,
        },
    );
    let right = kicad_text_attributes(
        PointNm { x: 0, y: 0 },
        0,
        800_000,
        None,
        KicadTextJustify {
            h: KicadTextHJustify::Right,
            v: KicadTextVJustify::Center,
            mirrored: false,
            keep_upright: false,
        },
    );
    assert_eq!(left.h_align, TextHAlign::Left);
    assert_eq!(center.h_align, TextHAlign::Center);
    assert_eq!(right.h_align, TextHAlign::Right);
    assert_eq!(left.v_align, TextVAlign::Center);
    assert!(!left.keep_upright);
}

#[test]
fn kicad_text_attributes_preserves_mirror_and_keep_upright() {
    let attrs = kicad_text_attributes(
        PointNm { x: 0, y: 0 },
        180,
        800_000,
        None,
        KicadTextJustify {
            h: KicadTextHJustify::Center,
            v: KicadTextVJustify::Center,
            mirrored: true,
            keep_upright: true,
        },
    );
    assert!(attrs.mirrored);
    assert!(attrs.keep_upright);
}

#[test]
fn kicad_text_rotation_is_converted_into_engine_sign_convention() {
    assert_eq!(kicad_text_rotation_degrees(0), 0);
    assert_eq!(kicad_text_rotation_degrees(90), -90);
    assert_eq!(kicad_text_rotation_degrees(-90), 90);
    assert_eq!(kicad_text_rotation_degrees(180), -180);
}

#[test]
fn kicad_layer_parser_canonicalizes_uppercase_silkscreen_name() {
    let block = r#"(property "Reference" "U1" (at 0 0 0) (layer "F.SILKS"))"#;
    assert_eq!(
        kicad_parse_layer_anywhere(block).as_deref(),
        Some("F.SilkS")
    );
    assert_eq!(kicad_render_role("F.SILKS"), Some("component_silkscreen"));
    assert_eq!(
        kicad_resolve_layer_id("F.SILKS", &std::collections::HashMap::new()),
        37
    );
}

#[test]
fn imported_kicad_component_text_ignores_render_cache_for_final_geometry() {
    let board = r#"(kicad_pcb
  (footprint "Demo:Part"
    (layer "F.Cu")
    (uuid "00000000-0000-0000-0000-00000000cafe")
    (at 10 20 0)
    (property "Reference" "U1"
      (at 1 2 90)
      (layer "F.SilkS")
      (effects
        (font (size 1 1) (thickness 0.15))
      )
      (render_cache "U1" 90
        (polygon
          (pts
            (xy 1 2)
            (xy 3 2)
            (xy 3 4)
            (xy 1 4)
          )
        )
      )
    )
  )
)"#;
    let components = vec![BoardComponentPayload {
        uuid: "00000000-0000-0000-0000-00000000cafe".to_string(),
        reference: "U1".to_string(),
        value: "Demo".to_string(),
        position: PointNm {
            x: 10_000_000,
            y: 20_000_000,
        },
        rotation: 0,
        layer: 0,
        locked: false,
    }];
    let (graphics, texts, board_texts, board_text_geometries, glyph_mesh_assets) =
        extract_kicad_footprint_graphics(board, &components, &std::collections::HashMap::new());
    assert!(
        texts.is_empty(),
        "imported KiCad text should materialize through geometry only, not the overlay text path"
    );
    assert!(
        board_texts.iter().any(|entry| entry.text == "U1"),
        "imported KiCad property text should materialize through the structured board-text path"
    );
    assert!(
        board_text_geometries
            .iter()
            .any(|entry| entry.layer_id == "L37" && !entry.glyphs.is_empty()),
        "imported KiCad property text should produce mesh-backed board text geometry"
    );
    assert!(
        !glyph_mesh_assets.is_empty(),
        "imported KiCad property text should contribute glyph mesh assets"
    );
    assert!(
        graphics
            .iter()
            .all(|entry| !entry.graphic_id.contains(":prop-stroke:")
                && !entry.graphic_id.contains(":prop-cache:")
                && !entry.graphic_id.contains(":kicad-text-cache:")),
        "cache-derived and stroke-derived imported text ids should disappear once Datum owns final text geometry"
    );
}

#[test]
fn imported_kicad_gr_text_uses_structured_text_geometry_path() {
    let board = r#"(kicad_pcb
  (layers
    (37 "F.SilkS" user "F.SilkS")
  )
  (gr_text "Demo"
    (at 5 5 0)
    (layer "F.SilkS")
    (effects
      (font (size 1 1) (thickness 0.15))
    )
    (uuid "66666666-7777-8888-9999-000000000000"))
)"#;
    let layer_table = kicad_parse_layer_table(board);
    let (texts, geometries, glyph_mesh_assets) = extract_kicad_board_texts(board, &layer_table);

    assert_eq!(texts.len(), 1);
    assert_eq!(texts[0].text, "Demo");
    assert_eq!(texts[0].layer_id, "L37");
    assert_eq!(geometries.len(), 1);
    assert_eq!(geometries[0].text_uuid, texts[0].text_uuid);
    assert!(
        !geometries[0].glyphs.is_empty(),
        "imported gr_text should produce mesh-backed text geometry"
    );
    assert!(
        geometries[0].strokes.is_empty(),
        "imported gr_text must not route through Newstroke stroke geometry"
    );
    assert!(
        !glyph_mesh_assets.is_empty(),
        "imported gr_text should contribute glyph mesh assets"
    );
}
