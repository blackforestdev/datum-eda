use datum_gui_protocol::{CheckFindingSummary, ReviewWorkspaceState};

use super::{
    RectPx, TEXT_MUTED, TEXT_PRIMARY, TEXT_SECONDARY, TextFace, TextRun, draw_text, suffix_id,
    truncate_text,
};

pub(super) fn render_check_finding_inspector(
    state: &ReviewWorkspaceState,
    fingerprint: &str,
    inspector_rect: RectPx,
    text_runs: &mut Vec<TextRun>,
) {
    let finding = state
        .checks
        .findings
        .iter()
        .find(|finding| finding.fingerprint == fingerprint);
    draw_text(
        &format!("FINDING {}", suffix_id(fingerprint).to_uppercase()),
        inspector_rect.x + 12.0,
        inspector_rect.y + 54.0,
        15.0,
        TEXT_PRIMARY,
        TextFace::Mono,
        text_runs,
    );
    if let Some(finding) = finding {
        draw_text(
            &format!(
                "{} / {}",
                finding.severity.to_uppercase(),
                truncate_text(&finding.rule_id.to_uppercase(), 24)
            ),
            inspector_rect.x + 12.0,
            inspector_rect.y + 74.0,
            11.0,
            TEXT_SECONDARY,
            TextFace::Mono,
            text_runs,
        );
        draw_text(
            &truncate_text(&finding.message, 36),
            inspector_rect.x + 12.0,
            inspector_rect.y + 92.0,
            10.0,
            TEXT_MUTED,
            TextFace::Ui,
            text_runs,
        );
        render_check_finding_inspector_details(finding, inspector_rect, text_runs);
    }
}

fn render_check_finding_inspector_details(
    finding: &CheckFindingSummary,
    inspector_rect: RectPx,
    text_runs: &mut Vec<TextRun>,
) {
    let mut y = inspector_rect.y + 110.0;
    if !finding.status.is_empty() {
        push_detail(
            "STATUS",
            &finding.status.to_uppercase(),
            &mut y,
            inspector_rect,
            text_runs,
        );
    }
    if let Some(target) = finding.target_label() {
        push_detail("TARGET", &target, &mut y, inspector_rect, text_runs);
    }
    if let Some(basis) = finding.standards_basis_label() {
        push_detail("BASIS", &basis, &mut y, inspector_rect, text_runs);
    }
    if let Some(action) = &finding.suggested_next_action {
        push_detail("NEXT", action, &mut y, inspector_rect, text_runs);
    }
}

fn push_detail(
    label: &str,
    value: &str,
    y: &mut f32,
    inspector_rect: RectPx,
    text_runs: &mut Vec<TextRun>,
) {
    draw_text(
        &format!("{label} {}", truncate_text(value, 31)),
        inspector_rect.x + 12.0,
        *y,
        10.0,
        TEXT_MUTED,
        TextFace::Mono,
        text_runs,
    );
    *y += 16.0;
}
