use super::{HitRegion, HitTarget, RectPx, TEXT_SECONDARY, TextFace, TextRun, draw_text};

pub(super) fn push_artifact_preview_controls(hit_regions: &mut Vec<HitRegion>, rect: RectPx) {
    hit_regions.push(HitRegion {
        target: HitTarget::ArtifactPreviewViewport,
        rect,
    });
    for (target, x) in [
        (
            HitTarget::ArtifactPreviewZoomOut,
            rect.x + rect.width - 118.0,
        ),
        (HitTarget::ArtifactPreviewReset, rect.x + rect.width - 92.0),
        (HitTarget::ArtifactPreviewZoomIn, rect.x + rect.width - 48.0),
        (
            HitTarget::ToggleArtifactPreviewGeometry,
            rect.x + rect.width - 178.0,
        ),
        (
            HitTarget::ToggleArtifactPreviewDrills,
            rect.x + rect.width - 148.0,
        ),
    ] {
        hit_regions.push(HitRegion {
            target,
            rect: RectPx {
                x,
                y: rect.y + 5.0,
                width: 24.0,
                height: 18.0,
            },
        });
    }
}

pub(super) fn draw_artifact_preview_controls(
    preview: &datum_gui_protocol::ArtifactPreviewViewportState,
    rect: RectPx,
    text_runs: &mut Vec<TextRun>,
) {
    for (label, x) in [
        ("-", rect.x + rect.width - 111.0),
        ("RESET", rect.x + rect.width - 88.0),
        ("+", rect.x + rect.width - 41.0),
        (
            if preview.show_geometry { "G" } else { "g" },
            rect.x + rect.width - 171.0,
        ),
        (
            if preview.show_drills { "D" } else { "d" },
            rect.x + rect.width - 141.0,
        ),
    ] {
        draw_text(
            label,
            x,
            rect.y + 9.0,
            9.0,
            TEXT_SECONDARY,
            TextFace::Mono,
            text_runs,
        );
    }
}
