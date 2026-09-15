use super::*;

pub(super) fn padded_rect_bounds(rect: RectNm, padding_nm: i64) -> SceneBounds {
    SceneBounds {
        min_x: rect.min_x.saturating_sub(padding_nm),
        min_y: rect.min_y.saturating_sub(padding_nm),
        max_x: rect.max_x.saturating_add(padding_nm),
        max_y: rect.max_y.saturating_add(padding_nm),
    }
}

pub(super) fn bounds_from_points(
    points: impl IntoIterator<Item = PointNm>,
    padding_nm: i64,
) -> Option<SceneBounds> {
    let mut iter = points.into_iter();
    let first = iter.next()?;
    let mut min_x = first.x;
    let mut max_x = first.x;
    let mut min_y = first.y;
    let mut max_y = first.y;
    for point in iter {
        min_x = min_x.min(point.x);
        max_x = max_x.max(point.x);
        min_y = min_y.min(point.y);
        max_y = max_y.max(point.y);
    }
    Some(SceneBounds {
        min_x: min_x.saturating_sub(padding_nm),
        min_y: min_y.saturating_sub(padding_nm),
        max_x: max_x.saturating_add(padding_nm),
        max_y: max_y.saturating_add(padding_nm),
    })
}

pub(super) fn marking_menu_key_for_target(target_object_id: Option<&str>) -> String {
    let key = match target_object_id {
        Some(id) if id.starts_with("component:") => "pcb.component",
        Some(id) if id.starts_with("pad:") => "pcb.pad",
        Some(id) if id.starts_with("track:") => "pcb.track",
        Some(id) if id.starts_with("via:") => "pcb.via",
        Some(id) if id.starts_with("zone:") => "pcb.zone",
        Some(id) if id.starts_with("net:") => "pcb.net",
        _ => "pcb.empty",
    };
    key.to_string()
}

pub(super) fn marking_slot_for_delta(dx: i32, dy: i32) -> Option<String> {
    let dx = dx as f32;
    let dy = dy as f32;
    if (dx * dx + dy * dy).sqrt() < 18.0 {
        return None;
    }
    let angle = dy.atan2(dx).to_degrees();
    let slot = if (-22.5..22.5).contains(&angle) {
        "E"
    } else if (22.5..67.5).contains(&angle) {
        "SE"
    } else if (67.5..112.5).contains(&angle) {
        "S"
    } else if (112.5..157.5).contains(&angle) {
        "SW"
    } else if !(-157.5..157.5).contains(&angle) {
        "W"
    } else if (-157.5..-112.5).contains(&angle) {
        "NW"
    } else if (-112.5..-67.5).contains(&angle) {
        "N"
    } else {
        "NE"
    };
    Some(slot.to_string())
}
