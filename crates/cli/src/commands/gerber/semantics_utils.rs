pub(crate) fn render_mm_6(nm: i64) -> String {
    let sign = if nm < 0 { "-" } else { "" };
    let abs = nm.abs();
    let whole = abs / 1_000_000;
    let frac = abs % 1_000_000;
    format!("{sign}{whole}.{frac:06}")
}
