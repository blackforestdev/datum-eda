use datum_gui_protocol::SourceShardStatusSummary;

use super::{RectPx, TEXT_ACCENT, TextFace, TextRun, draw_text, truncate_text};

pub(super) fn source_shard_health_label(summary: &SourceShardStatusSummary) -> String {
    if summary.attention_count() == 0 {
        format!("SOURCE SHARDS CLEAN {}/{}", summary.clean, summary.total)
    } else {
        format!(
            "SOURCE SHARDS D{} M{} U{}",
            summary.dirty, summary.missing, summary.unknown
        )
    }
}

pub(super) fn render_source_shard_attention_rows(
    summary: &SourceShardStatusSummary,
    project_rect: RectPx,
    start_y: f32,
    text_runs: &mut Vec<TextRun>,
) -> f32 {
    let mut y = start_y;
    for item in summary.attention.iter().take(2) {
        draw_text(
            &truncate_text(
                &format!("{} {}", item.dirty_state.to_uppercase(), item.relative_path),
                34,
            ),
            project_rect.x + 12.0,
            y,
            10.0,
            TEXT_ACCENT,
            TextFace::Mono,
            text_runs,
        );
        y += 14.0;
    }
    y + 4.0
}
