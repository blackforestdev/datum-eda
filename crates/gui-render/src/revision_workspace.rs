use super::*;
use datum_gui_protocol::{PaneContent, RevisionPane, RevisionSurface};

const NAV_ROWS: [(&str, RevisionSurface); 5] = [
    ("Changes", RevisionSurface::Change),
    ("Baselines", RevisionSurface::Impact),
    ("Releases", RevisionSurface::Release),
    ("Controlled Documents", RevisionSurface::Release),
    ("Evidence", RevisionSurface::Evidence),
];

pub(super) fn render_navigator(
    state: &ReviewWorkspaceState,
    rect: RectPx,
    quads: &mut Vec<Quad>,
    text: &mut Vec<TextRun>,
    hits: &mut Vec<HitRegion>,
) {
    let x = rect.x + 12.0;
    let mut y = rect.y + 82.0;
    draw_text("REVISION", x, y, 10.0, TEXT_MUTED, TextFace::UiStrong, text);
    y += 21.0;
    for (label, surface) in NAV_ROWS {
        let row = RectPx {
            x,
            y: y - 5.0,
            width: (rect.width - 24.0).max(1.0),
            height: 21.0,
        };
        if state.ui.layout.leaves().into_iter().any(|id| {
            state.ui.layout.content_for(id)
                == Some(PaneContent::Revision(RevisionPane::Surface(surface)))
        }) {
            quads.push(Quad::from_rect(row, REVIEW_ROW_BADGE));
            quads.push(Quad::from_rect(
                RectPx {
                    x: row.x,
                    y: row.y,
                    width: 2.0,
                    height: row.height,
                },
                TEXT_ACCENT,
            ));
        }
        draw_text(label, x + 8.0, y, 11.0, TEXT_SECONDARY, TextFace::Ui, text);
        let empty = match surface {
            RevisionSurface::Change => "none — begins at first divergence",
            RevisionSurface::Impact => "none — created by first release",
            RevisionSurface::Release => "none — issue your first release",
            RevisionSurface::Evidence => "none — recorded by checks",
        };
        if rect.width > 260.0 {
            draw_text(empty, x + 112.0, y, 9.0, TEXT_MUTED, TextFace::Ui, text);
        }
        hits.push(HitRegion {
            target: HitTarget::OpenRevisionSurface(surface),
            rect: row,
        });
        y += 24.0;
    }
}

pub(super) fn render_pane(
    state: &ReviewWorkspaceState,
    pane: RevisionPane,
    rect: RectPx,
    quads: &mut Vec<Quad>,
    text: &mut Vec<TextRun>,
    hits: &mut Vec<HitRegion>,
) {
    quads.push(Quad::from_rect(rect, VIEWPORT_BG));
    let pad = if rect.width < 620.0 { 18.0 } else { 28.0 };
    let x = rect.x + pad;
    let mut y = rect.y + 28.0;
    draw_text(
        match pane {
            RevisionPane::Surface(surface) => surface.label(),
            RevisionPane::Witness => "Canonical witness",
        },
        x,
        y,
        18.0,
        TEXT_PRIMARY,
        TextFace::UiStrong,
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
    draw_text(
        title,
        x + 10.0,
        y + 8.0,
        10.0,
        TEXT_MUTED,
        TextFace::UiStrong,
        text,
    );
    draw_text(
        value,
        x + 150.0,
        y + 8.0,
        11.0,
        TEXT_PRIMARY,
        TextFace::Ui,
        text,
    );
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
    draw_text(
        "Release candidate · readiness",
        x,
        y,
        13.0,
        TEXT_SECONDARY,
        TextFace::UiStrong,
        text,
    );
    y += 28.0;
    for (name, value) in [
        ("SCOPE", "12 affected items"),
        ("CHANGE", "EC-024 · Authorized"),
        ("REVISION", "B · reservation pending"),
        ("BASELINE", "BL-024-01"),
        ("EVIDENCE", "12 current · 0 stale"),
        ("DEPARTURES", "0 unresolved"),
        ("APPROVALS", "2 of 2 valid"),
        ("DOCUMENTS", "4 controlled issues"),
        ("PACKAGE", "manifest ready"),
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
    draw_text(
        "Impact summary",
        x,
        y,
        13.0,
        TEXT_SECONDARY,
        TextFace::UiStrong,
        text,
    );
    y += 30.0;
    for (label, value) in [
        ("Affected", "7"),
        ("Unaffected", "18"),
        ("Impact unknown", "2 — evidence incomplete"),
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
        y += 48.0;
    }
    draw_text(
        "Select a result to open its canonical witness tree beside this summary.",
        x,
        y + 8.0,
        11.0,
        TEXT_MUTED,
        TextFace::Ui,
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
        "J3 → U7 → regulated output  · IMPACT UNKNOWN",
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
    draw_text(
        "REPRODUCIBLE · 12 of 12 outputs byte-identical",
        x,
        y,
        13.0,
        TEXT_ACCENT,
        TextFace::UiStrong,
        text,
    );
    y += 32.0;
    for (label, value) in [
        ("Sources", "8 immutable inputs"),
        ("Producer", "datum-cli 0.1"),
        ("Invocation", "typed command captured"),
        ("Environment", "platform + toolchain captured"),
        ("Outputs", "12 / 12 matched"),
        ("Attempts", "2 retained"),
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
