use super::{HitRegion, HitTarget, RectPx, TEXT_MUTED, TextFace, TextRun, draw_text};

pub(crate) fn render_terminal_session_controls(
    rect: RectPx,
    y: f32,
    status: &str,
    application_shutdown_blocked: Option<&str>,
    text_runs: &mut Vec<TextRun>,
    hit_regions: &mut Vec<HitRegion>,
) -> f32 {
    let mut x = rect.x + 66.0;
    for (label, target) in [
        ("+NEW", HitTarget::TerminalSessionNew),
        ("RENAME", HitTarget::TerminalSessionRenameActive),
        ("RESTART", HitTarget::TerminalSessionRestartActive),
        ("CLOSE", HitTarget::TerminalSessionCloseActive),
    ] {
        x = push_control(label, target, x, y, text_runs, hit_regions);
    }
    if status.starts_with("close terminal?") {
        x = push_control(
            "TERMINATE",
            HitTarget::TerminalSessionTerminateActive,
            x,
            y,
            text_runs,
            hit_regions,
        );
        x = push_control(
            "CANCEL",
            HitTarget::TerminalShutdownCancel,
            x,
            y,
            text_runs,
            hit_regions,
        );
    }
    let contextual = if status.starts_with("termination failed") {
        Some(("RETRY", HitTarget::TerminalSessionRetryTermination))
    } else {
        None
    };
    if let Some((label, target)) = contextual {
        x = push_control(label, target, x, y, text_runs, hit_regions);
    }
    if application_shutdown_blocked.is_some() {
        x = push_control(
            "RETRY",
            HitTarget::TerminalSessionRetryTermination,
            x,
            y,
            text_runs,
            hit_regions,
        );
        x = push_control(
            "CANCEL SHUTDOWN",
            HitTarget::TerminalShutdownCancel,
            x,
            y,
            text_runs,
            hit_regions,
        );
    }
    if status.starts_with("termination failed")
        || status.contains("TERM grace")
        || status.contains("KILL verification")
    {
        x = push_control(
            "FORCE KILL",
            HitTarget::TerminalSessionForceKillActive,
            x,
            y,
            text_runs,
            hit_regions,
        );
    }
    x + 4.0
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
        render_terminal_session_controls(
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
        let armed = targets("close terminal? type yes + Enter");
        assert!(armed.contains(&HitTarget::TerminalSessionTerminateActive));
        assert!(armed.contains(&HitTarget::TerminalShutdownCancel));

        let stalled = targets("terminating (TERM grace)");
        assert!(stalled.contains(&HitTarget::TerminalSessionForceKillActive));

        let blocked = targets("shutdown blocked by terminal teardown");
        assert!(blocked.contains(&HitTarget::TerminalSessionRetryTermination));
        assert!(blocked.contains(&HitTarget::TerminalShutdownCancel));
    }
}
