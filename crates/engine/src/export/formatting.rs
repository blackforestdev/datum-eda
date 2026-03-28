pub(super) fn format_coord(nm: i64) -> String {
    nm.to_string()
}

pub(super) fn format_mm_6(nm: i64) -> String {
    let sign = if nm < 0 { "-" } else { "" };
    let abs_nm = nm.abs();
    let mm = abs_nm / 1_000_000;
    let frac = abs_nm % 1_000_000;
    format!("{sign}{mm}.{frac:06}")
}

pub(super) fn render_polygon_points(points: &[crate::ir::geometry::Point]) -> String {
    points
        .iter()
        .map(|point| format!("({}, {})", point.x, point.y))
        .collect::<Vec<_>>()
        .join(" -> ")
}

pub(super) fn parse_mm_6_to_nm(value: &str) -> Option<i64> {
    let mut parts = value.split('.');
    let whole = parts.next()?.parse::<i64>().ok()?;
    let frac_raw = parts.next().unwrap_or("0");
    if parts.next().is_some() || frac_raw.len() > 6 {
        return None;
    }
    let frac = format!("{frac_raw:0<6}").parse::<i64>().ok()?;
    Some(whole * 1_000_000 + frac)
}
