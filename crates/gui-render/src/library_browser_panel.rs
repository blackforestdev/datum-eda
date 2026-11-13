//! Browser navigation, source acquisition and named results in the shared Library pane.
use super::*;

pub(super) fn render(p: &mut Painter<'_>) {
    let state = p.state;
    let rect = p.rect;
    p.line("Project pool · source identity / revision remain separate");
    p.button("Browse shipped symbols", C::BrowseBundled);
    p.button(
        if KINDS
            .get(state.library.kind)
            .is_some_and(|(kind, _)| matches!(*kind, "footprints" | "packages" | "padstacks"))
        {
            "Acquire local KiCad footprint"
        } else {
            "Acquire local KiCad symbol"
        },
        C::Acquire,
    );
    let cell = (rect.width - 12.) / 4.;
    let row_height = KINDS
        .iter()
        .map(|(_, label)| {
            crate::text_metrics::measured_text_run_height_px(
                label,
                (cell - 9.).max(1.),
                11.,
                TextFace::Ui,
            )
            .unwrap_or(16.)
                + 12.
        })
        .fold(25., f32::max)
        + 4.;
    for (i, (_, label)) in KINDS.iter().enumerate() {
        let r = RectPx {
            x: rect.x + 6. + (i % 4) as f32 * cell,
            y: p.y + (i / 4) as f32 * row_height,
            width: (cell - 3.).max(0.),
            height: row_height - 4.,
        };
        p.quads.push(Quad::from_rect(
            r,
            if state.library.kind == i {
                design_tokens::chrome::SURFACE_03
            } else {
                design_tokens::chrome::SURFACE_02
            },
        ));
        if state.library.focus == Some(C::Kind(i)) {
            push_rect_border(p.quads, r, TEXT_ACCENT, 1.);
        }
        draw_text_clipped(
            label,
            r.x + 3.,
            r.y + 6.,
            11.,
            TEXT_PRIMARY,
            TextFace::Ui,
            r,
            p.text,
        );
        p.text.last_mut().unwrap().layout_size = Some(((r.width - 6.).max(1.), r.height - 12.));
        p.hits.push(HitRegion {
            target: HitTarget::Library(C::Kind(i)),
            rect: r,
        });
    }
    p.y += row_height * 2.;
    p.button(&format!("Search: {}", state.library.search), C::Search);
    let results = state.library.results();
    p.line(&format!(
        "{} named {} results",
        results.len(),
        KINDS[state.library.kind].1
    ));
    for &i in results.iter().skip(state.library.result_offset).take(3) {
        let e = &state.library.entries[i];
        p.button(
            &format!(
                "{}{}",
                e.name,
                if e.warning.is_some() {
                    " · inspection only"
                } else {
                    ""
                }
            ),
            C::Result(i),
        );
    }
    if results.len() > 3 {
        p.button("Previous results", C::ResultsPage(false));
        p.button("Next results", C::ResultsPage(true));
    }
    if LibraryDraft::can_create(KINDS[state.library.kind].0) {
        p.button(
            &format!("Create {} · target pool", KINDS[state.library.kind].1),
            C::New,
        );
    }
    if let Some(refusal) = &state.library.refusal {
        p.line(refusal);
    }
}
