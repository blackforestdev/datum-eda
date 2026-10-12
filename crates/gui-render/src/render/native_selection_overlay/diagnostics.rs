//! Topmost semantic finding markers; target navigation never acquires its object.
use super::*;
use datum_gui_protocol::{SelectionSubject, selection_artifacts::diagnostic_markers as marker};

pub(super) fn push(
    out: &mut Vec<Quad>,
    hits: &mut Vec<HitRegion>,
    scene: &BoardReviewSceneV1,
    state: &ReviewWorkspaceState,
    viewport: RectPx,
    camera: CameraState,
    surface: datum_gui_protocol::PaneContent,
    plan: &mut super::cue_plan::PanePlan,
) {
    let projection = Projection::new(
        inset_rect(viewport, 10.0, 10.0, 10.0, 10.0),
        &scene.bounds,
        camera,
    );
    let start = out.len();
    let hit_start = hits.len();
    let anchors = marker::MarkerProjectionIndex::capture(scene, &state.checks, surface, |l| {
        layer_visible(state, l)
    });
    for finding in &state.checks.findings {
        let Ok(point) = anchors.anchor(finding) else {
            continue;
        };
        let (x, y) = projection.project_point(point);
        let rect = RectPx {
            x: x - 13.0,
            y: y - 13.0,
            width: 26.0,
            height: 24.0,
        };
        if rect.intersect(viewport).is_none() {
            continue;
        }
        let selected = matches!(&state.selection, SelectionTarget::Native(v) if !v.stale && matches!(&v.subject,SelectionSubject::Diagnostic(id) if *id==finding.fingerprint));
        // Exact Selection Study section 4 triangle/exclamation construction.
        // The selection border surrounds, never overwrites, the severity core.
        if selected {
            let detail_start = out.len();
            let vertices = [
                (x, y - 13.0),
                (x + 13.0, y + 10.0),
                (x - 13.0, y + 10.0),
                (x, y - 13.0),
            ];
            for pair in vertices.windows(2) {
                push_screen_line(out, pair[0], pair[1], TEXT_ACCENT, 2.0);
            }
            plan.finish_detail(out, detail_start);
        }
        let severity = match finding.severity.as_str() {
            "error" | "fatal" => crate::design_tokens::chrome::STATUS_ERROR,
            "warning" | "warn" => [216.0 / 255.0, 160.0 / 255.0, 62.0 / 255.0],
            _ => crate::design_tokens::chrome::STATUS_INFO,
        };
        let mut base = plan.base_output(out, selected);
        base.push(Quad {
            points: [
                (x, y - 11.0),
                (x + 11.0, y + 8.0),
                (x - 11.0, y + 8.0),
                (x - 11.0, y + 8.0),
            ],
            color: severity,
        });
        drop(base);
        out.push(Quad::from_rect(
            RectPx {
                x: x - 1.2,
                y: y - 4.0,
                width: 2.4,
                height: 7.0,
            },
            [11.0 / 255.0, 12.0 / 255.0, 14.0 / 255.0],
        ));
        let dot: Vec<_> = (0..=12)
            .map(|n| {
                let a = n as f32 * std::f32::consts::TAU / 12.0;
                (x + a.cos() * 1.3, y + 5.4 + a.sin() * 1.3)
            })
            .collect();
        for pair in dot.windows(2) {
            out.push(Quad {
                points: [(x, y + 5.4), pair[0], pair[1], pair[1]],
                color: [11.0 / 255.0, 12.0 / 255.0, 14.0 / 255.0],
            });
        }
        hits.push(HitRegion {
            rect,
            target: HitTarget::CheckFinding(finding.fingerprint.clone()),
        });
    }
    crate::hit_clipping::clip_content(out, &mut Vec::new(), hits, start, 0, hit_start, viewport);
}

#[cfg(test)]
#[path = "diagnostics/tests.rs"]
mod tests;
