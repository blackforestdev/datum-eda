use super::{HitRegion, HitTarget, RectPx, TEXT_MUTED, TextFace, TextRun, draw_text};

fn lifecycle_actions(
    status: &str,
    application_shutdown_blocked: Option<&str>,
) -> Vec<(&'static str, HitTarget)> {
    let mut actions = Vec::new();
    if status.starts_with("close terminal?") {
        actions.push(("TERMINATE", HitTarget::TerminalSessionTerminateActive));
        actions.push(("CANCEL", HitTarget::TerminalShutdownCancel));
    }
    if status.starts_with("termination failed") {
        actions.push(("RETRY", HitTarget::TerminalSessionRetryTermination));
    }
    if application_shutdown_blocked.is_some() {
        if !actions
            .iter()
            .any(|(_, target)| *target == HitTarget::TerminalSessionRetryTermination)
        {
            actions.push(("RETRY", HitTarget::TerminalSessionRetryTermination));
        }
        actions.push(("CANCEL SHUTDOWN", HitTarget::TerminalShutdownCancel));
    }
    if status.starts_with("termination failed")
        || status.contains("TERM grace")
        || status.contains("KILL verification")
    {
        actions.push(("FORCE KILL", HitTarget::TerminalSessionForceKillActive));
    }
    actions
}

pub(crate) fn terminal_lifecycle_controls_width(
    status: &str,
    application_shutdown_blocked: Option<&str>,
) -> f32 {
    lifecycle_actions(status, application_shutdown_blocked)
        .iter()
        .map(|(label, _)| label.len() as f32 * 7.0 + 16.0)
        .sum()
}

pub(crate) fn render_terminal_lifecycle_controls(
    rect: RectPx,
    y: f32,
    status: &str,
    application_shutdown_blocked: Option<&str>,
    text_runs: &mut Vec<TextRun>,
    hit_regions: &mut Vec<HitRegion>,
) -> bool {
    let actions = lifecycle_actions(status, application_shutdown_blocked);
    if actions.is_empty() {
        return false;
    }
    let width = terminal_lifecycle_controls_width(status, application_shutdown_blocked);
    let mut x = (rect.x + rect.width - width).max(rect.x);
    for (label, target) in actions {
        x = push_control(label, target, x, y, text_runs, hit_regions);
    }
    true
}

pub(crate) const fn terminal_link_controls_width() -> f32 {
    // OPEN + CANCEL, using the same fixed mono-control measurement as the
    // lifecycle actions below.
    (4.0 * 7.0 + 16.0) + (6.0 * 7.0 + 16.0)
}

pub(crate) fn render_terminal_link_controls(
    rect: RectPx,
    y: f32,
    text_runs: &mut Vec<TextRun>,
    hit_regions: &mut Vec<HitRegion>,
) {
    let mut x = (rect.x + rect.width - terminal_link_controls_width()).max(rect.x);
    x = push_control(
        "OPEN",
        HitTarget::TerminalLinkConfirmOpen,
        x,
        y,
        text_runs,
        hit_regions,
    );
    push_control(
        "CANCEL",
        HitTarget::TerminalLinkCancel,
        x,
        y,
        text_runs,
        hit_regions,
    );
}

pub(crate) const fn terminal_clipboard_write_controls_width() -> f32 {
    (4.0 * 7.0 + 16.0) + (6.0 * 7.0 + 16.0)
}

pub(crate) fn render_terminal_clipboard_write_controls(
    rect: RectPx,
    y: f32,
    text_runs: &mut Vec<TextRun>,
    hit_regions: &mut Vec<HitRegion>,
) {
    let mut x = (rect.x + rect.width - terminal_clipboard_write_controls_width()).max(rect.x);
    x = push_control(
        "COPY",
        HitTarget::TerminalClipboardConfirmWrite,
        x,
        y,
        text_runs,
        hit_regions,
    );
    push_control(
        "CANCEL",
        HitTarget::TerminalClipboardCancelWrite,
        x,
        y,
        text_runs,
        hit_regions,
    );
}

fn push_control(
    label: &str,
    target: HitTarget,
    x: f32,
    y: f32,
    text_runs: &mut Vec<TextRun>,
    hit_regions: &mut Vec<HitRegion>,
) -> f32 {
    draw_text(label, x, y, 10.5, TEXT_MUTED, TextFace::Mono, text_runs);
    hit_regions.push(HitRegion {
        target,
        rect: RectPx {
            x: x - 4.0,
            y: y - 2.0,
            width: (label.len() as f32 * 7.0 + 8.0).max(24.0),
            height: 14.0,
        },
    });
    x + label.len() as f32 * 7.0 + 16.0
}

#[cfg(test)]
mod tests {
    use super::*;

    fn targets(status: &str) -> Vec<HitTarget> {
        let mut text = Vec::new();
        let mut hits = Vec::new();
        render_terminal_lifecycle_controls(
            RectPx {
                x: 0.0,
                y: 0.0,
                width: 900.0,
                height: 30.0,
            },
            10.0,
            status,
            status.starts_with("shutdown blocked").then_some(status),
            &mut text,
            &mut hits,
        );
        hits.into_iter().map(|hit| hit.target).collect()
    }

    #[test]
    fn close_and_shutdown_states_expose_only_the_ratified_actions() {
        assert!(targets("running").is_empty());

        let armed = targets("close terminal? Enter confirms; Escape cancels");
        assert!(armed.contains(&HitTarget::TerminalSessionTerminateActive));
        assert!(armed.contains(&HitTarget::TerminalShutdownCancel));

        let stalled = targets("terminating (TERM grace)");
        assert!(stalled.contains(&HitTarget::TerminalSessionForceKillActive));

        let blocked = targets("shutdown blocked by terminal teardown");
        assert!(blocked.contains(&HitTarget::TerminalSessionRetryTermination));
        assert!(blocked.contains(&HitTarget::TerminalShutdownCancel));
    }
}
