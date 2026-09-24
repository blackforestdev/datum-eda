use super::*;

pub(super) fn push_board_text_geometry_world(
    out: &mut impl Output<Quad>,
    text_geometry: &BoardTextGeometryPrimitive,
    glyph_mesh_assets: &impl GlyphMeshLookup,
    color: [f32; 3],
    _reference_projection: &Projection,
) {
    if let Some(transform) = text_geometry.world_transform_nm
        && !text_geometry.glyphs.is_empty()
    {
        push_board_text_mesh_world(out, text_geometry, glyph_mesh_assets, transform, color);
        return;
    }
    for fill in &text_geometry.fills {
        push_world_polygon_fill_contours(out, &fill.outer, &fill.holes, color);
    }
    for stroke in &text_geometry.strokes {
        push_world_polyline_segments(
            out,
            &[stroke.from, stroke.to],
            stroke.width_nm.max(1) as f32,
            color,
        );
    }
}

fn push_board_text_mesh_world(
    out: &mut impl Output<Quad>,
    text_geometry: &BoardTextGeometryPrimitive,
    glyph_mesh_assets: &impl GlyphMeshLookup,
    transform: Affine2DFixedPrimitive,
    color: [f32; 3],
) {
    for glyph in &text_geometry.glyphs {
        let Some(asset) = glyph_mesh_assets.mesh(&glyph.glyph_handle) else {
            trace_text_mesh_skip(format!(
                "{} missing glyph mesh asset font={} glyph={} tolerance={} epoch={}",
                text_geometry.object_id,
                glyph.glyph_handle.font_id,
                glyph.glyph_handle.glyph_id,
                glyph.glyph_handle.tolerance_class,
                glyph.glyph_handle.epoch,
            ));
            continue;
        };
        for triangle in asset.indices.as_chunks::<3>().0.iter() {
            let Some(a) = asset.vertices.get(triangle[0] as usize) else {
                trace_text_mesh_skip(format!(
                    "{} glyph={} triangle references missing vertex {}",
                    text_geometry.object_id, glyph.glyph_handle.glyph_id, triangle[0],
                ));
                continue;
            };
            let Some(b) = asset.vertices.get(triangle[1] as usize) else {
                trace_text_mesh_skip(format!(
                    "{} glyph={} triangle references missing vertex {}",
                    text_geometry.object_id, glyph.glyph_handle.glyph_id, triangle[1],
                ));
                continue;
            };
            let Some(c) = asset.vertices.get(triangle[2] as usize) else {
                trace_text_mesh_skip(format!(
                    "{} glyph={} triangle references missing vertex {}",
                    text_geometry.object_id, glyph.glyph_handle.glyph_id, triangle[2],
                ));
                continue;
            };
            let a = transform_text_mesh_point(
                transform,
                glyph.origin_em_nm_x + a.x_em_nm,
                glyph.origin_em_nm_y + a.y_em_nm,
            );
            let b = transform_text_mesh_point(
                transform,
                glyph.origin_em_nm_x + b.x_em_nm,
                glyph.origin_em_nm_y + b.y_em_nm,
            );
            let c = transform_text_mesh_point(
                transform,
                glyph.origin_em_nm_x + c.x_em_nm,
                glyph.origin_em_nm_y + c.y_em_nm,
            );
            push_world_triangle(out, a, b, c, color);
        }
    }
}

fn trace_text_mesh_skip(message: String) {
    if std::env::var_os("DATUM_TRACE_GRAPHICS").is_some() {
        eprintln!("[datum-text-mesh] {message}");
    }
}

fn transform_text_mesh_point(
    transform: Affine2DFixedPrimitive,
    x_em_nm: i64,
    y_em_nm: i64,
) -> (f32, f32) {
    const EM_NM: i128 = 1_000_000;
    let x = (i128::from(transform.m11_ppm) * i128::from(x_em_nm)
        + i128::from(transform.m12_ppm) * i128::from(y_em_nm))
        / EM_NM
        + i128::from(transform.tx_nm);
    let y = (i128::from(transform.m21_ppm) * i128::from(x_em_nm)
        + i128::from(transform.m22_ppm) * i128::from(y_em_nm))
        / EM_NM
        + i128::from(transform.ty_nm);
    (x as f32, y as f32)
}
