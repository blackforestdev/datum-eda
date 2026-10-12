use super::*;
use crate::{Affine2DFixedPrimitive, MeshRectEmPrimitive};
use uuid::Uuid;

fn text(render: &str, source: Option<&str>, object: &str) -> BoardTextPrimitive {
    serde_json::from_value(serde_json::json!({
        "object_id": object, "object_kind": "schematic_text", "text_uuid": render,
        "source_object_uuid": source, "text": "I", "layer_id": "L214",
        "position": {"x": 0, "y": 0}, "rotation_degrees": 0,
        "height_nm": 640000, "stroke_width_nm": 120000,
        "render_intent": "annotation", "family": "ibm_plex_sans_condensed",
        "style": "regular", "h_align": "left", "v_align": "center",
        "mirrored": false, "keep_upright": true, "line_spacing_ratio_ppm": 1000000,
        "bold": false, "italic": false
    }))
    .unwrap()
}
fn geometry(object: &str, x: i64) -> BoardTextGeometryPrimitive {
    BoardTextGeometryPrimitive {
        object_id: object.into(),
        object_kind: "schematic_text".into(),
        text_uuid: object.into(),
        layer_id: "L214".into(),
        world_transform_nm: Some(Affine2DFixedPrimitive {
            m11_ppm: 640000,
            m12_ppm: 0,
            m21_ppm: 0,
            m22_ppm: 640000,
            tx_nm: x,
            ty_nm: 0,
        }),
        block_bbox_em_nm: Some(MeshRectEmPrimitive {
            min_x_em_nm: 53000,
            min_y_em_nm: -698000,
            max_x_em_nm: 309000,
            max_y_em_nm: 0,
        }),
        glyphs: vec![],
        fills: vec![],
        strokes: vec![],
    }
}

#[test]
fn raw_alias_collisions_preserve_first_vector_match_and_first_geometry() {
    let id = "50000000-0000-4000-8000-000000001abc";
    let texts = [
        text("other-render", Some(id), "source-first"),
        text(id, None, "render-later"),
        text("last-render", Some(id), "source-last"),
        text(&id.to_uppercase(), None, "uppercase"),
    ];
    let layouts = [
        geometry("source-first", 11),
        geometry("source-first", 99),
        geometry("render-later", 22),
        geometry("source-last", 33),
        geometry("uppercase", 44),
    ];
    let index = TextLayoutIndex::new(&texts, &layouts);
    assert_eq!(index.layout(id), layout_polygon(&layouts[0]));
    assert_eq!(
        index.layout(&id.to_uppercase()),
        layout_polygon(&layouts[4])
    );
    assert_eq!(index.layout("other-render"), index.layout(id));
    // Reversing source order changes the old OR predicate's first match too.
    let reversed = [texts[1].clone(), texts[0].clone()];
    assert_eq!(
        TextLayoutIndex::new(&reversed, &layouts).layout(id),
        layout_polygon(&layouts[2])
    );
    assert!(index.layout("unowned-id").is_none());
}

#[test]
fn missing_first_geometry_basis_never_falls_through_to_later_alias_or_layout() {
    let texts = [
        text("render", Some("source"), "first"),
        text("source", None, "later"),
    ];
    let mut missing = geometry("first", 0);
    missing.block_bbox_em_nm = None;
    let layouts = [missing, geometry("first", 99), geometry("later", 123)];
    let index = TextLayoutIndex::new(&texts, &layouts);
    assert!(index.layout("source").is_none());
    assert!(index.layout("render").is_none());
    assert!(
        TextLayoutIndex::new(&texts, &layouts[2..])
            .layout("source")
            .is_none()
    );
    let mut no_transform = geometry("first", 0);
    no_transform.world_transform_nm = None;
    assert!(
        TextLayoutIndex::new(&texts, &[no_transform])
            .layout("source")
            .is_none()
    );
}

#[test]
fn dense_index_preserves_all100000_exact_layouts_without_result_truncation() {
    let ids: Vec<_> = (0x1000..0x1000 + 100000)
        .map(|id| Uuid::from_u128(0x50000000000040008000000000000000 + id).to_string())
        .collect();
    let texts: Vec<_> = ids.iter().map(|id| text(id, None, id)).collect();
    let layouts: Vec<_> = ids
        .iter()
        .enumerate()
        .map(|(i, id)| geometry(id, (i % 400) as i64 * 2000000))
        .collect();
    let index = TextLayoutIndex::new(&texts, &layouts);
    for (id, layout) in ids.iter().zip(&layouts) {
        assert_eq!(index.layout(id), layout_polygon(layout));
    }
    assert_eq!(index.text_objects.len(), 100000);
    assert_eq!(index.geometries.len(), 100000);
    assert!(
        index
            .layout("50000000-0000-4000-8000-0000000196a0")
            .is_none()
    );
}
