//! Read-only supervision-reflection panel (Decision-013 level-1).
//!
//! Renders the R12 journal / activity ledger and the R13 resolver-status /
//! recovery projection from the read-only `SupervisionReflectionState` carried on
//! the workspace. This surface DISPLAYS committed engine state only: it reuses
//! the existing dock/panel + text-layout machinery and introduces NO control that
//! emits an `OperationBatch` or a journaled write. The only hit region it pushes
//! is `SupervisionJournalEntry`, which selects a row (consumer state) for
//! cross-highlight — it carries no commit-bearing payload.

use datum_gui_protocol::{
    ReviewWorkspaceState, SelectionTarget, SupervisionJournalEntry, SupervisionResolverStatusV1,
    SUPERVISION_DIAGNOSTIC_SEVERITY_ERROR, SUPERVISION_RESOLVER_MODE_RECOVERY,
};

use super::{
    HitRegion, HitTarget, Quad, RectPx, TextFace, TextRun, draw_text, push_rect_border,
    truncate_text, PANEL_CARD_BORDER, TEXT_MUTED, TEXT_PANEL_VALUE, TEXT_PRIMARY, TEXT_SECONDARY,
};

// Supervision-local accents. Human sources read neutral; agent sources read with
// a cool (`tool`) or warm (`assistant`) accent; `test` reads muted — the §4.7
// provenance ledger classes. Recovery promotes an alert accent (§4.8).
const SUPERVISION_ACCENT_ASSISTANT: [f32; 3] = [0.96, 0.66, 0.36]; // warm AI accent
const SUPERVISION_ACCENT_TOOL: [f32; 3] = [0.45, 0.74, 0.92]; // cool agent accent
const SUPERVISION_TEXT_TEST: [f32; 3] = [0.50, 0.54, 0.60]; // muted synthetic
const SUPERVISION_ALERT: [f32; 3] = [0.95, 0.42, 0.40]; // recovery / error
const SUPERVISION_WARN: [f32; 3] = [0.93, 0.78, 0.36]; // warning diagnostics
const SUPERVISION_OK: [f32; 3] = [0.45, 0.82, 0.55]; // resolved-clean
const SUPERVISION_ROW_SELECTED: [f32; 3] = [0.20, 0.23, 0.28];
const SUPERVISION_BANNER_BG: [f32; 3] = [0.13, 0.14, 0.17];

/// Render the supervision dock content into the already-framed `content_rect`.
/// Read-only: reads `state.supervision`, mutates nothing.
pub(super) fn render_supervision_lane(
    state: &ReviewWorkspaceState,
    rect: RectPx,
    panel_quads: &mut Vec<Quad>,
    text_runs: &mut Vec<TextRun>,
    hit_regions: &mut Vec<HitRegion>,
) {
    draw_text(
        "SUPERVISION  (READ-ONLY)",
        rect.x + 12.0,
        rect.y + 12.0,
        12.0,
        TEXT_SECONDARY,
        TextFace::Ui,
        text_runs,
    );

    // R13 — resolver-status banner. In recovery this is the primary surface.
    let banner_bottom = render_resolver_banner(
        state.supervision.resolver_status.as_ref(),
        rect,
        rect.y + 32.0,
        panel_quads,
        text_runs,
    );

    if state.supervision.is_recovery() {
        // Recovery layout (§4.8 / QG-RESOLVER-RECOVERY): diagnostics are primary,
        // no journal ledger is promoted as accepted history.
        return;
    }

    // R12 — journal / activity ledger.
    render_journal_ledger(state, rect, banner_bottom + 8.0, panel_quads, text_runs, hit_regions);
}

fn render_resolver_banner(
    status: Option<&SupervisionResolverStatusV1>,
    rect: RectPx,
    top_y: f32,
    panel_quads: &mut Vec<Quad>,
    text_runs: &mut Vec<TextRun>,
) -> f32 {
    let Some(status) = status else {
        draw_text(
            "RESOLVER  no model projected",
            rect.x + 12.0,
            top_y,
            10.5,
            TEXT_MUTED,
            TextFace::Mono,
            text_runs,
        );
        return top_y + 16.0;
    };

    let recovery = status.mode == SUPERVISION_RESOLVER_MODE_RECOVERY;
    let banner_height = if recovery {
        (rect.y + rect.height - top_y - 10.0).max(40.0)
    } else {
        38.0
    };
    let banner_rect = RectPx {
        x: rect.x + 8.0,
        y: top_y,
        width: rect.width - 16.0,
        height: banner_height,
    };
    panel_quads.push(Quad::from_rect(banner_rect, SUPERVISION_BANNER_BG));
    push_rect_border(panel_quads, banner_rect, PANEL_CARD_BORDER, 1.0);

    let (mode_label, mode_color) = if recovery {
        ("RECOVERY", SUPERVISION_ALERT)
    } else {
        ("RESOLVED", SUPERVISION_OK)
    };
    draw_text(
        &format!("MODE {mode_label}"),
        banner_rect.x + 10.0,
        banner_rect.y + 10.0,
        11.0,
        mode_color,
        TextFace::Mono,
        text_runs,
    );
    let revision = status
        .model_revision
        .clone()
        .unwrap_or_else(|| "—".to_string());
    draw_text(
        &format!(
            "REV {}  SHARDS {}  COHERENT {}",
            truncate_text(&revision, 28),
            status.shard_count,
            if status.coherent { "yes" } else { "no" }
        ),
        banner_rect.x + 130.0,
        banner_rect.y + 10.0,
        10.5,
        TEXT_SECONDARY,
        TextFace::Mono,
        text_runs,
    );

    // Diagnostics list (primary in recovery; advisory chips otherwise).
    let mut y = banner_rect.y + 28.0;
    if status.diagnostics.is_empty() {
        if !recovery {
            draw_text(
                "DIAGNOSTICS none",
                banner_rect.x + 10.0,
                y,
                10.0,
                TEXT_MUTED,
                TextFace::Mono,
                text_runs,
            );
        }
    } else {
        for diagnostic in &status.diagnostics {
            if y > banner_rect.y + banner_rect.height - 12.0 {
                break;
            }
            let color = if diagnostic.severity == SUPERVISION_DIAGNOSTIC_SEVERITY_ERROR {
                SUPERVISION_ALERT
            } else {
                SUPERVISION_WARN
            };
            let path = diagnostic
                .path
                .as_deref()
                .map(|p| format!("  {p}"))
                .unwrap_or_default();
            draw_text(
                &truncate_text(
                    &format!(
                        "[{}] {} {}{}",
                        diagnostic.severity.to_uppercase(),
                        diagnostic.code,
                        diagnostic.message,
                        path
                    ),
                    160,
                ),
                banner_rect.x + 10.0,
                y,
                10.0,
                color,
                TextFace::Mono,
                text_runs,
            );
            y += 15.0;
        }
    }
    banner_rect.y + banner_rect.height
}

fn render_journal_ledger(
    state: &ReviewWorkspaceState,
    rect: RectPx,
    top_y: f32,
    panel_quads: &mut Vec<Quad>,
    text_runs: &mut Vec<TextRun>,
    hit_regions: &mut Vec<HitRegion>,
) {
    let Some(journal) = state.supervision.journal.as_ref() else {
        draw_text(
            "ACTIVITY LEDGER  no journal projected",
            rect.x + 12.0,
            top_y,
            10.5,
            TEXT_MUTED,
            TextFace::Mono,
            text_runs,
        );
        return;
    };

    draw_text(
        &format!(
            "ACTIVITY LEDGER  {} TXN  APPLIED {}",
            journal.entries.len(),
            journal.applied_transaction_count
        ),
        rect.x + 12.0,
        top_y,
        10.5,
        TEXT_MUTED,
        TextFace::Mono,
        text_runs,
    );

    let selected_txid = match &state.selection {
        SelectionTarget::JournalEntry(id) => Some(id.as_str()),
        _ => None,
    };

    let mut y = top_y + 18.0;
    let row_height = 16.0;
    // Newest-last in journal order; show most-recent first for the ledger.
    for entry in journal.entries.iter().rev() {
        if y > rect.y + rect.height - row_height {
            break;
        }
        let selected = selected_txid == Some(entry.transaction_id.as_str());
        if selected {
            panel_quads.push(Quad::from_rect(
                RectPx {
                    x: rect.x + 8.0,
                    y: y - 2.0,
                    width: rect.width - 16.0,
                    height: row_height,
                },
                SUPERVISION_ROW_SELECTED,
            ));
        }
        draw_journal_row(entry, rect, y, text_runs);
        hit_regions.push(HitRegion {
            target: HitTarget::SupervisionJournalEntry(entry.transaction_id.clone()),
            rect: RectPx {
                x: rect.x + 8.0,
                y: y - 2.0,
                width: rect.width - 16.0,
                height: row_height,
            },
        });
        y += row_height;
    }
}

fn draw_journal_row(
    entry: &SupervisionJournalEntry,
    rect: RectPx,
    y: f32,
    text_runs: &mut Vec<TextRun>,
) {
    // Source-class accent — the §4.7 human-vs-agent ledger (total over CommitSource).
    let (source_glyph, source_color) = source_class(&entry.source);
    let kind_chevron = match entry.transaction_kind.as_str() {
        "undo" => "↶ ",
        "redo" => "↷ ",
        _ => "",
    };
    let diff_counts = format!(
        "+{} ~{} -{}",
        entry.created_object_ids.len(),
        entry.modified_object_ids.len(),
        entry.deleted_object_ids.len()
    );
    let applied_mark = if entry.applied { "" } else { " (beyond-cursor)" };

    // Source badge.
    draw_text(
        source_glyph,
        rect.x + 12.0,
        y,
        10.5,
        source_color,
        TextFace::Mono,
        text_runs,
    );
    // Actor + reason.
    draw_text(
        &truncate_text(
            &format!(
                "{kind_chevron}{} · {}{}",
                entry.actor, entry.reason, applied_mark
            ),
            96,
        ),
        rect.x + 78.0,
        y,
        10.5,
        if entry.applied {
            TEXT_PANEL_VALUE
        } else {
            TEXT_MUTED
        },
        TextFace::Mono,
        text_runs,
    );
    // Revision delta + diff counts (right-aligned cluster).
    draw_text(
        &truncate_text(
            &format!(
                "{}→{}  {}",
                entry.before_model_revision, entry.after_model_revision, diff_counts
            ),
            48,
        ),
        rect.x + rect.width * 0.62,
        y,
        10.0,
        TEXT_SECONDARY,
        TextFace::Mono,
        text_runs,
    );
}

/// Source → (badge glyph, accent color), total over the §4.7 `CommitSource`
/// classes. A new engine source string falls into the neutral human bucket here;
/// the exhaustiveness guarantee lives in `gui-protocol`'s `commit_source_str`.
fn source_class(source: &str) -> (&'static str, [f32; 3]) {
    match source {
        "assistant" => ("AI  assistant", SUPERVISION_ACCENT_ASSISTANT),
        "tool" => ("AGT tool", SUPERVISION_ACCENT_TOOL),
        "test" => ("·   test", SUPERVISION_TEXT_TEST),
        "cli" => (">_  cli", TEXT_PRIMARY),
        _ => ("H   manual", TEXT_PRIMARY),
    }
}
