pub(super) fn transform_board_local_point(
    origin: crate::ir::geometry::Point,
    rotation_deg: i32,
    local: crate::ir::geometry::Point,
) -> crate::ir::geometry::Point {
    let rotated = match rotation_deg.rem_euclid(360) {
        90 => crate::ir::geometry::Point::new(-local.y, local.x),
        180 => crate::ir::geometry::Point::new(-local.x, -local.y),
        270 => crate::ir::geometry::Point::new(local.y, -local.x),
        _ => local,
    };
    crate::ir::geometry::Point::new(origin.x + rotated.x, origin.y + rotated.y)
}

pub(super) fn inverse_transform_board_local_point(
    origin: crate::ir::geometry::Point,
    rotation_deg: i32,
    absolute: crate::ir::geometry::Point,
) -> crate::ir::geometry::Point {
    let translated = crate::ir::geometry::Point::new(absolute.x - origin.x, absolute.y - origin.y);
    match rotation_deg.rem_euclid(360) {
        90 => crate::ir::geometry::Point::new(translated.y, -translated.x),
        180 => crate::ir::geometry::Point::new(-translated.x, -translated.y),
        270 => crate::ir::geometry::Point::new(-translated.y, translated.x),
        _ => translated,
    }
}
