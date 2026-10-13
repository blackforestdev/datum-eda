use super::*;
use crate::ir::geometry::Point;
use crate::text::{
    FAMILY_IBM_PLEX_SANS_CONDENSED, TextFamilyId, TextFamilySource, TextHAlign, TextRenderIntent,
    TextStyleId, TextVAlign,
};

fn text(content: &str) -> BoardText {
    BoardText {
        uuid: uuid::Uuid::from_u128(1080),
        text: content.into(),
        position: Point {
            x: 256_000_000,
            y: 0,
        },
        rotation: 0,
        layer: 214,
        render_intent: TextRenderIntent::Annotation,
        family: TextFamilyId(FAMILY_IBM_PLEX_SANS_CONDENSED.into()),
        family_source: TextFamilySource::Explicit,
        style: TextStyleId::default(),
        height_nm: 640_000,
        stroke_width_nm: 120_000,
        h_align: TextHAlign::Left,
        v_align: TextVAlign::Center,
        mirrored: false,
        keep_upright: false,
        line_spacing_ratio_ppm: 1_000_000,
        italic: false,
        bold: false,
        style_class: None,
    }
}

// Independent painted envelope: combine actual asset bounds with the emitted
// glyph origins, rather than deriving expectations from the batch layout box.
fn painted_envelope(scene: &TextMeshScene) -> MeshRectEm {
    let rectangles: Vec<_> = scene
        .batch
        .glyphs
        .iter()
        .map(|glyph| {
            let asset = scene
                .glyph_mesh_assets
                .iter()
                .find(|asset| asset.handle == glyph.glyph_handle)
                .unwrap();
            MeshRectEm {
                min_x_em_nm: asset.bbox_em_nm.min_x_em_nm + glyph.origin_em_nm_x,
                max_x_em_nm: asset.bbox_em_nm.max_x_em_nm + glyph.origin_em_nm_x,
                min_y_em_nm: asset.bbox_em_nm.min_y_em_nm + glyph.origin_em_nm_y,
                max_y_em_nm: asset.bbox_em_nm.max_y_em_nm + glyph.origin_em_nm_y,
            }
        })
        .collect();
    MeshRectEm {
        min_x_em_nm: rectangles.iter().map(|r| r.min_x_em_nm).min().unwrap(),
        max_x_em_nm: rectangles.iter().map(|r| r.max_x_em_nm).max().unwrap(),
        min_y_em_nm: rectangles.iter().map(|r| r.min_y_em_nm).min().unwrap(),
        max_y_em_nm: rectangles.iter().map(|r| r.max_y_em_nm).max().unwrap(),
    }
}

#[test]
fn aligned_plex_layout_matches_painted_glyph_envelope() {
    for content in ["I", "I\nI"] {
        for h in [TextHAlign::Left, TextHAlign::Center, TextHAlign::Right] {
            for v in [TextVAlign::Top, TextVAlign::Center, TextVAlign::Bottom] {
                for (rotation, mirrored, upright) in [
                    (0, false, false),
                    (90, false, false),
                    (180, true, false),
                    (180, false, true),
                    (270, true, true),
                ] {
                    let mut source = text(content);
                    source.h_align = h;
                    source.v_align = v;
                    source.rotation = rotation;
                    source.mirrored = mirrored;
                    source.keep_upright = upright;
                    let before = serde_json::to_value(&source).unwrap();
                    let scene = layout_text_mesh_from_board_text(&source).unwrap();
                    let envelope = painted_envelope(&scene);
                    assert_eq!(
                        scene.batch.block_bbox_em_nm, envelope,
                        "layout must follow actual painted glyph origins"
                    );
                    let normalized =
                        normalize_text_attributes(TextAttributes::from_board_text(&source));
                    let x_pair = (envelope.min_x_em_nm, envelope.max_x_em_nm);
                    let y_pair = (envelope.min_y_em_nm, envelope.max_y_em_nm);
                    match normalized.h_align {
                        TextHAlign::Left => assert_eq!(x_pair.0, 0),
                        TextHAlign::Center => {
                            assert!(x_pair.0 + x_pair.1 <= 1 && x_pair.0 + x_pair.1 >= -1)
                        }
                        TextHAlign::Right => assert_eq!(x_pair.1, 0),
                    }
                    match normalized.v_align {
                        TextVAlign::Top => assert_eq!(y_pair.0, 0),
                        TextVAlign::Center => {
                            assert!(y_pair.0 + y_pair.1 <= 1 && y_pair.0 + y_pair.1 >= -1)
                        }
                        TextVAlign::Bottom => assert_eq!(y_pair.1, 0),
                    }
                    assert_eq!(
                        scene.batch.world_transform,
                        text_world_transform(&normalized)
                    );
                    assert_eq!(serde_json::to_value(&source).unwrap(), before);
                }
            }
        }
    }
}

#[test]
fn bearing_alignment_keeps_existing_paint_inputs_and_source_identity() {
    let source = text("I");
    let scene = layout_text_mesh_from_board_text(&source).unwrap();
    // Independently known vendored Plex glyph I: bearings53..309, y-698..0.
    assert_eq!(
        scene.batch.block_bbox_em_nm,
        MeshRectEm {
            min_x_em_nm: 0,
            max_x_em_nm: 256_000,
            min_y_em_nm: -349_000,
            max_y_em_nm: 349_000
        }
    );
    assert_eq!(scene.batch.glyphs[0].origin_em_nm_x, -53_000);
    assert_eq!(scene.batch.glyphs[0].origin_em_nm_y, 349_000);
    assert_eq!(
        scene.glyph_mesh_assets[0].bbox_em_nm,
        MeshRectEm {
            min_x_em_nm: 53_000,
            max_x_em_nm: 309_000,
            min_y_em_nm: -698_000,
            max_y_em_nm: 0
        }
    );
    assert_eq!(scene.batch.text_uuid, source.uuid);
    assert_eq!(
        scene.batch.world_transform,
        Affine2DFixed {
            m11_ppm: 640_000,
            m12_ppm: 0,
            m21_ppm: 0,
            m22_ppm: 640_000,
            tx_nm: 256_000_000,
            ty_nm: 0
        }
    );
    let again = layout_text_mesh_from_board_text(&source).unwrap();
    assert_eq!(scene.batch.glyphs, again.batch.glyphs);
    assert_eq!(scene.glyph_mesh_assets, again.glyph_mesh_assets);
}
