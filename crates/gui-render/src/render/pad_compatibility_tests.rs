//! Existing aperture proof follows the typed integer scene adapter unchanged.
use super::super::*;
#[test]
fn roundrect_pad_uses_richer_geometry_than_rect_pad() {
    let viewport = RectPx {
        x: 0.0,
        y: 0.0,
        width: 200.0,
        height: 120.0,
    };
    let bounds = datum_gui_protocol::SceneBounds {
        min_x: 0,
        min_y: 0,
        max_x: 2_000_000,
        max_y: 1_200_000,
    };
    let projection = Projection::new(viewport, &bounds, CameraState::fit_to_bounds(&bounds));
    let mut rect_out = Vec::new();
    let mut roundrect_out = Vec::new();
    let rect_pad = datum_gui_protocol::PadPrimitive {
        native_process: None,
        object_id: "pad:rect".to_string(),
        object_kind: "pad".to_string(),
        source_object_uuid: "rect".to_string(),
        pad_uuid: "rect".to_string(),
        component_uuid: "U1".to_string(),
        net_uuid: None,
        layer_id: "L1".to_string(),
        copper_layer_ids: vec!["L1".to_string()],
        center: PointNm {
            x: 1_000_000,
            y: 600_000,
        }
        .into(),
        bounds: datum_gui_protocol::RectNm {
            min_x: 700_000,
            min_y: 350_000,
            max_x: 1_300_000,
            max_y: 850_000,
        },
        shape_kind: "rect".to_string(),
        roundrect_rratio_ppm: 250_000,
        mask_layer_ids: vec![],
        paste_layer_ids: vec![],
        solder_mask_margin_nm: 0,
        solder_paste_margin_nm: 0,
        solder_paste_margin_ratio_ppm: 0,
        drill_nm: None,
        rotation_degrees: 0.0,
    };
    let mut roundrect_pad = rect_pad.clone();
    roundrect_pad.shape_kind = "roundrect".to_string();

    push_pad_primitive(
        &mut rect_out,
        &rect_pad,
        &projection,
        "L1",
        PAD_COPPER,
        None,
        false,
    );
    push_pad_primitive(
        &mut roundrect_out,
        &roundrect_pad,
        &projection,
        "L1",
        PAD_COPPER,
        None,
        false,
    );

    assert!(roundrect_out.len() > rect_out.len());
}
