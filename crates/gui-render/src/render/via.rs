use super::{
    dim_authored_color, dim_structural_color, inset_rect, project_point, push_projected_ellipse,
    push_world_ellipse_nm, resolve_layer_appearance, world_inset_rect, world_length_to_px,
    Projection, Quad, RectPx, AUTHOR_SELECTED,
};

#[allow(dead_code)]
fn push_via_primitive(
    out: &mut Vec<Quad>,
    via: &datum_gui_protocol::ViaPrimitive,
    projection: &Projection,
    selected: bool,
    dimmed: bool,
) -> RectPx {
    let outer_size = world_length_to_px(via.diameter_nm, projection).clamp(7.0, 18.0);
    let (x, y) = project_point(via.position, projection);
    let rect = RectPx {
        x: x - outer_size * 0.5,
        y: y - outer_size * 0.5,
        width: outer_size,
        height: outer_size,
    };
    push_projected_ellipse(
        out,
        rect,
        dim_authored_color(
            if selected {
                AUTHOR_SELECTED
            } else {
                resolve_layer_appearance(Some(&via.start_layer_id)).pad_copper
            },
            dimmed,
        ),
        128,
    );
    let ring = outer_size * 0.14;
    let copper = inset_rect(rect, ring, ring, ring, ring);
    push_projected_ellipse(
        out,
        copper,
        dim_authored_color(
            if selected {
                [0.72, 0.86, 0.93]
            } else {
                resolve_layer_appearance(Some(&via.start_layer_id)).pad_copper
            },
            dimmed,
        ),
        128,
    );
    let drill_px =
        world_length_to_px(via.drill_nm, projection).clamp(3.2, (outer_size - ring * 2.0).max(3.2));
    let drill = RectPx {
        x: x - drill_px * 0.5,
        y: y - drill_px * 0.5,
        width: drill_px,
        height: drill_px,
    };
    push_projected_ellipse(
        out,
        drill,
        dim_structural_color([0.13, 0.14, 0.16], dimmed),
        18,
    );
    rect
}

pub(crate) fn push_via_primitive_world(
    out: &mut Vec<Quad>,
    via: &datum_gui_protocol::ViaPrimitive,
    copper_color: [f32; 3],
    selected: bool,
    dimmed: bool,
    _reference_projection: &Projection,
) {
    let half = via.diameter_nm as f32 * 0.5;
    let rect = datum_gui_protocol::RectNm {
        min_x: (via.position.x as f32 - half).round() as i64,
        min_y: (via.position.y as f32 - half).round() as i64,
        max_x: (via.position.x as f32 + half).round() as i64,
        max_y: (via.position.y as f32 + half).round() as i64,
    };
    push_world_ellipse_nm(
        out,
        rect,
        dim_authored_color(
            if selected {
                AUTHOR_SELECTED
            } else {
                copper_color
            },
            dimmed,
        ),
        128,
    );
    let ring = via.diameter_nm as f32 * 0.14;
    let copper = world_inset_rect(rect, ring);
    push_world_ellipse_nm(
        out,
        copper,
        dim_authored_color(
            if selected {
                [0.72, 0.86, 0.93]
            } else {
                copper_color
            },
            dimmed,
        ),
        128,
    );
    let drill_half = via.drill_nm as f32 * 0.5;
    push_world_ellipse_nm(
        out,
        datum_gui_protocol::RectNm {
            min_x: (via.position.x as f32 - drill_half).round() as i64,
            min_y: (via.position.y as f32 - drill_half).round() as i64,
            max_x: (via.position.x as f32 + drill_half).round() as i64,
            max_y: (via.position.y as f32 + drill_half).round() as i64,
        },
        dim_structural_color([0.13, 0.14, 0.16], dimmed),
        128,
    );
}
