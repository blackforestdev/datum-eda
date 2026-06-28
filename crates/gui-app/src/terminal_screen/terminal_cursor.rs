pub(super) fn clamp_column(column: usize, columns: Option<usize>) -> usize {
    columns
        .map(|columns| column.min(columns.saturating_sub(1)))
        .unwrap_or(column)
}

pub(super) fn clamp_row(row: usize, rows: Option<usize>) -> usize {
    rows.map(|rows| row.min(rows.saturating_sub(1)))
        .unwrap_or(row)
}

pub(super) fn addressed_row(
    one_based_row: usize,
    scroll_region: &Option<(usize, usize)>,
    origin_mode: bool,
    rows: Option<usize>,
) -> usize {
    let row = one_based_row.saturating_sub(1);
    if origin_mode && let Some((top, bottom)) = scroll_region {
        return (top + row).min(*bottom);
    }
    clamp_row(row, rows)
}
