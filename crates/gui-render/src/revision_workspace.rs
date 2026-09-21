use super::*;
use datum_gui_protocol::{PaneContent, RevisionPane, RevisionSurface};

pub(super) fn render_evidence_inspector(
    state: &ReviewWorkspaceState,
    rect: RectPx,
    quads: &mut Vec<Quad>,
    text: &mut Vec<TextRun>,
) -> bool {
    if state.ui.layout.focused_content()
        != PaneContent::Revision(RevisionPane::Surface(RevisionSurface::Evidence))
    {
        return false;
    }
    draw_text(
        "Immutable manifest",
        rect.x + 12.0,
        rect.y + 46.0,
        16.0,
        TEXT_PRIMARY,
        TextFace::UiStrong,
        text,
    );
    draw_text(
        "RLS-0007 evidence",
        rect.x + 12.0,
        rect.y + 64.0,
        10.0,
        TEXT_MUTED,
        TextFace::Mono,
        text,
    );
    let mut y = rect.y + 92.0;
    for (label, value) in [
        ("Configuration", "BL-2026-08-24-01 · LOCKED"),
        ("Sources", "4 subjects · digests recorded"),
        ("Producer", "datum-eda 1.4.2 · build 9917"),
        ("Invocation", "policy P-FAB-1"),
        ("Environment", "digest e77a… · fonts pinned"),
        ("Outputs", "12 files · per-file sha256"),
        ("Attempt", "independent executor ci-2"),
    ] {
        draw_text(
            label,
            rect.x + 12.0,
            y,
            10.0,
            TEXT_MUTED,
            TextFace::Ui,
            text,
        );
        draw_text_clipped(
            value,
            rect.x + 112.0,
            y,
            11.0,
            TEXT_PRIMARY,
            TextFace::Mono,
            RectPx {
                x: rect.x + 108.0,
                y: y - 3.0,
                width: (rect.width - 120.0).max(1.0),
                height: 18.0,
            },
            text,
        );
        y += 27.0;
    }
    draw_lock_glyph(
        rect.x + rect.width - 24.0,
        rect.y + 84.0,
        10.0,
        TEXT_SECONDARY,
        quads,
    );
    true
}

pub(super) fn render_pane(
    state: &ReviewWorkspaceState,
    pane: RevisionPane,
    rect: RectPx,
    quads: &mut Vec<Quad>,
    text: &mut Vec<TextRun>,
    hits: &mut Vec<HitRegion>,
) {
    let quad_start = quads.len();
    let text_start = text.len();
    let hit_start = hits.len();
    quads.push(Quad::from_rect(rect, VIEWPORT_BG));
    let pad = if rect.width < 620.0 { 18.0 } else { 28.0 };
    let x = rect.x + pad;
    let mut y = rect.y + 28.0;
    draw_text_clipped(
        match pane {
            RevisionPane::Surface(RevisionSurface::Impact) => "Impact · CHG-0031",
            RevisionPane::Surface(RevisionSurface::Release) => "RC-0009 · EVT2 release",
            RevisionPane::Surface(RevisionSurface::Evidence) => "RLS-0007 evidence",
            RevisionPane::Surface(surface) => surface.label(),
            RevisionPane::Witness => "Canonical witness",
        },
        x,
        y,
        18.0,
        TEXT_PRIMARY,
        TextFace::UiStrong,
        RectPx {
            x,
            y: rect.y,
            width: (rect.x + rect.width - 44.0 - x).max(1.0),
            height: 60.0,
        },
        text,
    );
    let close = RectPx {
        x: rect.x + rect.width - 38.0,
        y: rect.y + 16.0,
        width: 24.0,
        height: 24.0,
    };
    draw_text(
        "×",
        close.x + 7.0,
        close.y + 4.0,
        14.0,
        TEXT_SECONDARY,
        TextFace::Ui,
        text,
    );
    hits.push(HitRegion {
        target: HitTarget::CloseRevisionSurface,
        rect: close,
    });
    y += 42.0;
    match pane {
        RevisionPane::Surface(RevisionSurface::Release) => {
            render_release(state, x, y, rect, quads, text, hits)
        }
        RevisionPane::Surface(RevisionSurface::Impact) => {
            render_impact(x, y, rect, quads, text, hits)
        }
        RevisionPane::Surface(RevisionSurface::Change) => render_change(x, y, rect, quads, text),
        RevisionPane::Surface(RevisionSurface::Evidence) => {
            render_evidence(x, y, rect, quads, text, hits)
        }
        RevisionPane::Witness => render_witness(state, x, y, rect, quads, text),
    }
    crate::hit_clipping::clip_content(quads, text, hits, quad_start, text_start, hit_start, rect);
}

fn render_witness(
    state: &ReviewWorkspaceState,
    x: f32,
    mut y: f32,
    rect: RectPx,
    quads: &mut Vec<Quad>,
    text: &mut Vec<TextRun>,
) {
    draw_text(
        state
            .ui
            .revision
            .selected_witness
            .as_deref()
            .unwrap_or("No witness selected"),
        x,
        y,
        13.0,
        TEXT_SECONDARY,
        TextFace::UiStrong,
        text,
    );
    y += 32.0;
    for (label, value) in [
        ("SOURCE", "resolver-owned immutable authority"),
        ("PATH", "J3 → U7 → regulated output"),
        ("VERDICT", "IMPACT UNKNOWN"),
        ("WHY", "required evidence input is absent"),
    ] {
        section(
            label,
            value,
            x,
            y,
            (rect.width - 2.0 * (x - rect.x)).max(180.0),
            quads,
            text,
        );
        y += 44.0;
    }
}

fn section(
    title: &str,
    value: &str,
    x: f32,
    y: f32,
    width: f32,
    quads: &mut Vec<Quad>,
    text: &mut Vec<TextRun>,
) {
    let row = RectPx {
        x,
        y,
        width,
        height: 40.0,
    };
    quads.push(Quad::from_rect(row, PANEL_CARD_BG));
    push_rect_border(quads, row, PANEL_CARD_BORDER, 1.0);
    if width < 420.0 {
        draw_text_clipped(
            title,
            x + 10.0,
            y + 5.0,
            9.0,
            TEXT_MUTED,
            TextFace::UiStrong,
            RectPx {
                x,
                y,
                width,
                height: 18.0,
            },
            text,
        );
        draw_text_clipped(
            value,
            x + 10.0,
            y + 21.0,
            9.5,
            TEXT_PRIMARY,
            TextFace::Ui,
            RectPx {
                x,
                y: y + 17.0,
                width,
                height: 21.0,
            },
            text,
        );
    } else {
        draw_text_clipped(
            title,
            x + 10.0,
            y + 8.0,
            10.0,
            TEXT_MUTED,
            TextFace::UiStrong,
            RectPx {
                x,
                y,
                width: 140.0,
                height: 40.0,
            },
            text,
        );
        draw_text_clipped(
            value,
            x + 150.0,
            y + 8.0,
            11.0,
            TEXT_PRIMARY,
            TextFace::Ui,
            RectPx {
                x: x + 145.0,
                y,
                width: (width - 145.0).max(1.0),
                height: 40.0,
            },
            text,
        );
    }
}

fn draw_lock_glyph(x: f32, y: f32, size: f32, color: [f32; 3], quads: &mut Vec<Quad>) {
    let stroke = (size * 0.16).max(1.0);
    quads.push(Quad::from_rect(
        RectPx {
            x,
            y: y + size * 0.42,
            width: size,
            height: size * 0.58,
        },
        color,
    ));
    quads.push(Quad::from_rect(
        RectPx {
            x: x + size * 0.2,
            y,
            width: stroke,
            height: size * 0.52,
        },
        color,
    ));
    quads.push(Quad::from_rect(
        RectPx {
            x: x + size * 0.2,
            y,
            width: size * 0.6,
            height: stroke,
        },
        color,
    ));
    quads.push(Quad::from_rect(
        RectPx {
            x: x + size * 0.8 - stroke,
            y,
            width: stroke,
            height: size * 0.52,
        },
        color,
    ));
}

fn render_release(
    state: &ReviewWorkspaceState,
    x: f32,
    mut y: f32,
    rect: RectPx,
    quads: &mut Vec<Quad>,
    text: &mut Vec<TextRun>,
    hits: &mut Vec<HitRegion>,
) {
    draw_text_clipped(
        "Preparing → Ready for Review → In Approval → Ready to Release",
        x,
        y,
        13.0,
        TEXT_SECONDARY,
        TextFace::UiStrong,
        RectPx {
            x,
            y,
            width: (rect.x + rect.width - x - 12.0).max(1.0),
            height: 24.0,
        },
        text,
    );
    y += 24.0;
    draw_text_clipped(
        "exits: Stale · Rejected · Cancelled",
        x,
        y,
        10.5,
        TEXT_MUTED,
        TextFace::Ui,
        RectPx {
            x,
            y,
            width: (rect.x + rect.width - x - 12.0).max(1.0),
            height: 22.0,
        },
        text,
    );
    y += 26.0;
    for (name, value) in [
        ("1 · SCOPE + MEMBERS ✓", "3 proposed"),
        ("2 · REVISION ALLOCATIONS ◌", "2 reserved + 1 reuse"),
        ("3 · CHANGES + DEPARTURES ✓", "1 + 1"),
        ("4 · IMPACT ⚠", "1 Unknown — expanded"),
        ("5 · EVIDENCE ⚠", "1 blocker — expanded"),
        ("6 · EFFECTIVITY ✓", "units from SN-0450"),
        ("7 · ATTESTATIONS ⚠", "1 of 2 · digest-bound"),
        ("8 · DOCUMENTS ✓", "1 issue proposed"),
        ("9 · PACKAGE ✓", "manifest ready"),
    ] {
        section(
            name,
            value,
            x,
            y,
            (rect.width - 2.0 * (x - rect.x)).max(200.0),
            quads,
            text,
        );
        y += 44.0;
    }
    let bar = RectPx {
        x,
        y: rect.y + rect.height - 64.0,
        width: (rect.width - 2.0 * (x - rect.x)).max(200.0),
        height: 42.0,
    };
    quads.push(Quad::from_rect(bar, PANEL_CARD_BG));
    push_rect_border(quads, bar, TEXT_ACCENT, 1.0);
    let label = if state.ui.revision.issuance_armed {
        "CONFIRM ISSUE RELEASE B · creates immutable Release + Package"
    } else {
        "ARM ISSUE · no record is created"
    };
    draw_text(
        label,
        bar.x + 12.0,
        bar.y + 12.0,
        11.0,
        TEXT_PRIMARY,
        TextFace::UiStrong,
        text,
    );
    hits.push(HitRegion {
        target: HitTarget::ToggleRevisionIssuanceArm,
        rect: bar,
    });
}

fn render_impact(
    x: f32,
    mut y: f32,
    rect: RectPx,
    quads: &mut Vec<Quad>,
    text: &mut Vec<TextRun>,
    hits: &mut Vec<HitRegion>,
) {
    draw_text_clipped(
        "Evaluation IMP-0009 · vs BL-2026-08-24-01",
        x,
        y,
        13.0,
        TEXT_SECONDARY,
        TextFace::UiStrong,
        RectPx {
            x,
            y,
            width: (rect.x + rect.width - x - 12.0).max(1.0),
            height: 24.0,
        },
        text,
    );
    y += 30.0;
    for (label, value) in [
        ("Changed", "1 · lib footprint SOIC-8 rev 5→6"),
        ("Affected !", "2 · witness paths recorded"),
        ("Unaffected", "3 ✓ · proof: outside sensitivity"),
        ("ImpactUnknown", "? 1 — evaluator missing: 3D-model edge"),
        ("Graph scope", "complete except 3D + harness evaluators"),
    ] {
        section(
            label,
            value,
            x,
            y,
            (rect.width * 0.45).max(220.0),
            quads,
            text,
        );
        y += 44.0;
    }
    draw_text_clipped(
        "Filters — ⚠ affected  ·  ✓ unaffected  ·  ? unknown  ·  → required action",
        x,
        y + 8.0,
        11.0,
        TEXT_MUTED,
        TextFace::Ui,
        RectPx {
            x,
            y,
            width: (rect.x + rect.width - x - 12.0).max(1.0),
            height: 22.0,
        },
        text,
    );
    let witness = RectPx {
        x,
        y: y + 34.0,
        width: (rect.width * 0.45).max(220.0),
        height: 36.0,
    };
    quads.push(Quad::from_rect(witness, REVIEW_ROW_BADGE));
    draw_text(
        "J3 → U7 → regulated output  · ? ImpactUnknown",
        x + 10.0,
        y + 44.0,
        10.0,
        TEXT_PRIMARY,
        TextFace::Ui,
        text,
    );
    hits.push(HitRegion {
        target: HitTarget::OpenRevisionWitness("J3/U7/output".to_owned()),
        rect: witness,
    });
}

fn render_change(x: f32, mut y: f32, rect: RectPx, quads: &mut Vec<Quad>, text: &mut Vec<TextRun>) {
    draw_text(
        "Engineering Change · EC-024",
        x,
        y,
        13.0,
        TEXT_SECONDARY,
        TextFace::UiStrong,
        text,
    );
    y += 32.0;
    for (label, value) in [
        ("INTENT", "Correct regulator thermal margin"),
        ("AFFECTED ITEMS", "7 items · 2 unknown"),
        ("IMPACT", "evaluation current"),
        ("EVIDENCE", "12 inputs"),
        ("DEPARTURES", "none unresolved"),
        ("APPROVALS", "required · quorum 2/2"),
        ("EFFECTIVITY", "next issued release"),
        ("CLOSURE", "eligible after release"),
    ] {
        section(
            label,
            value,
            x,
            y,
            (rect.width - 2.0 * (x - rect.x)).max(240.0),
            quads,
            text,
        );
        y += 44.0;
    }
}

fn render_evidence(
    x: f32,
    mut y: f32,
    rect: RectPx,
    quads: &mut Vec<Quad>,
    text: &mut Vec<TextRun>,
    hits: &mut Vec<HitRegion>,
) {
    draw_text_clipped(
        "Configuration BL-2026-08-24-01 · LOCKED",
        x,
        y,
        13.0,
        TEXT_ACCENT,
        TextFace::UiStrong,
        RectPx {
            x,
            y,
            width: (rect.x + rect.width - x - 12.0).max(1.0),
            height: 24.0,
        },
        text,
    );
    draw_lock_glyph(x + 225.0, y - 4.0, 10.0, TEXT_ACCENT, quads);
    y += 28.0;
    draw_text_clipped(
        "Reproduction ✓ ByteIdentical · 12/12 outputs",
        x,
        y,
        12.0,
        TEXT_ACCENT,
        TextFace::UiStrong,
        RectPx {
            x,
            y,
            width: (rect.x + rect.width - x - 12.0).max(1.0),
            height: 22.0,
        },
        text,
    );
    y += 24.0;
    draw_text_clipped(
        "Authenticity signature valid · signer authorized ✓ (separate fact)",
        x,
        y,
        10.5,
        TEXT_SECONDARY,
        TextFace::Ui,
        RectPx {
            x,
            y,
            width: (rect.x + rect.width - x - 12.0).max(1.0),
            height: 22.0,
        },
        text,
    );
    y += 28.0;
    for (label, value) in [
        ("Sources", "✓ complete · 4 subjects · digests recorded"),
        ("Producer", "✓ recorded · datum-eda 1.4.2 · build 9917"),
        ("Invocation", "✓ recorded · policy P-FAB-1"),
        ("Environment", "✓ captured · digest e77a… · fonts pinned"),
        ("Outputs", "✓ 12 files verified · per-file sha256"),
        ("Attempts", "✓ 2 · latest independent ci-2"),
    ] {
        section(
            label,
            value,
            x,
            y,
            (rect.width - 2.0 * (x - rect.x)).max(240.0),
            quads,
            text,
        );
        y += 48.0;
    }
    let manifest = RectPx {
        x,
        y,
        width: 260.0,
        height: 34.0,
    };
    quads.push(Quad::from_rect(manifest, REVIEW_ROW_BADGE));
    draw_text(
        "Open immutable reproduction manifest beside",
        x + 10.0,
        y + 9.0,
        10.0,
        TEXT_PRIMARY,
        TextFace::Ui,
        text,
    );
    hits.push(HitRegion {
        target: HitTarget::OpenRevisionWitness("reproduction-manifest".to_owned()),
        rect: manifest,
    });
}
