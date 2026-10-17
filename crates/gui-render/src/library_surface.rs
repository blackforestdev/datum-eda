//! Named browser and explicit native forms in the shared recursive pane shell.
use crate::*;
use datum_gui_protocol::LibraryJsonValue as Value;
use datum_gui_protocol::{
    ReviewWorkspaceState,
    library_browser::{KINDS, LibraryControl as C},
};

struct Painter<'a> {
    rect: RectPx,
    y: f32,
    unbounded: bool,
    state: &'a ReviewWorkspaceState,
    quads: &'a mut Vec<Quad>,
    text: &'a mut Vec<TextRun>,
    hits: &'a mut Vec<HitRegion>,
}
impl Painter<'_> {
    fn line(&mut self, label: &str) {
        let width = (self.rect.width - 16.).max(1.);
        let height =
            crate::text_metrics::measured_text_run_height_px(label, width, 12., TextFace::Ui)
                .unwrap_or(16.)
                + 8.;
        if self.unbounded || self.y + height <= self.rect.y + self.rect.height {
            draw_text_clipped(
                label,
                self.rect.x + 8.,
                self.y + 4.,
                12.,
                TEXT_PRIMARY,
                TextFace::Ui,
                RectPx {
                    y: self.y,
                    height,
                    ..self.rect
                },
                self.text,
            );
            self.text.last_mut().unwrap().layout_size = Some((width, height - 8.));
        }
        self.y += height;
    }
    fn button(&mut self, label: &str, control: C) {
        let rect = RectPx {
            x: self.rect.x + 6.,
            y: self.y,
            width: (self.rect.width - 12.).max(0.),
            height: (crate::text_metrics::measured_text_run_height_px(
                label,
                (self.rect.width - 24.).max(1.),
                12.,
                TextFace::Ui,
            )
            .unwrap_or(16.)
                + 12.)
                .max(25.),
        };
        if self.unbounded || rect.y + rect.height <= self.rect.y + self.rect.height {
            self.quads
                .push(Quad::from_rect(rect, design_tokens::chrome::SURFACE_02));
            if self.state.library.focus.as_ref() == Some(&control) {
                push_rect_border(self.quads, rect, TEXT_ACCENT, 1.);
            }
            draw_text_clipped(
                label,
                rect.x + 6.,
                rect.y + 6.,
                12.,
                TEXT_PRIMARY,
                TextFace::Ui,
                rect,
                self.text,
            );
            self.text.last_mut().unwrap().layout_size =
                Some(((rect.width - 12.).max(1.), rect.height - 12.));
            self.hits.push(HitRegion {
                target: HitTarget::Library(control),
                rect,
            });
        }
        self.y += rect.height + 4.;
    }
}
pub(crate) fn render(
    state: &ReviewWorkspaceState,
    rect: RectPx,
    quads: &mut Vec<Quad>,
    text: &mut Vec<TextRun>,
    hits: &mut Vec<HitRegion>,
) {
    let starts = (quads.len(), text.len(), hits.len());
    quads.push(Quad::from_rect(rect, design_tokens::chrome::SURFACE_01));
    let mut p = Painter {
        rect,
        y: rect.y + 8.,
        unbounded: true,
        state,
        quads,
        text,
        hits,
    };
    if let Some(d) = &state.library.draft {
        p.line(&format!(
            "{} {} · canonical target: pool",
            if d.create { "Create" } else { "Edit" },
            KINDS
                .iter()
                .find(|k| k.0 == d.kind)
                .map_or("Unavailable", |k| k.1)
        ));
        if rect.height > 420. {
            p.line("Tab: field / action · arrows: choices · Esc: cancel");
        }
        if d.kind == "packages" {
            p.line("Physical body / terminals · no Footprint lands");
            p.line("Tolerances, terminal positions and governed models unavailable");
        }
        if matches!(d.kind.as_str(), "footprints" | "packages") {
            let package = d.kind == "packages";
            let records = if package { &d.terminals } else { &d.pads };
            let focused = match state.library.focus {
                Some(C::Pad(i) | C::Terminal(i)) => i,
                _ => 0,
            };
            let count = records.len().min(4).max(1);
            let toolbar = RectPx {
                width: rect.width / count as f32,
                ..rect
            };
            let row = p.y;
            let mut bottom = row;
            for (i, pad) in records.values().enumerate().skip(focused / 4 * 4).take(4) {
                p.rect = RectPx {
                    x: rect.x + (i % 4) as f32 * toolbar.width,
                    ..toolbar
                };
                p.y = row;
                p.button(
                    &format!(
                        "{} {}",
                        if package { "Terminal" } else { "Pad" },
                        pad.get("name")
                            .and_then(Value::as_str)
                            .unwrap_or("Unavailable")
                    ),
                    if package { C::Terminal(i) } else { C::Pad(i) },
                );
                bottom = bottom.max(p.y);
            }
            p.rect = RectPx {
                width: rect.width / 2.,
                ..rect
            };
            p.y = bottom;
            p.button(
                if package { "Add terminal" } else { "Add pad" },
                if package { C::AddTerminal } else { C::AddPad },
            );
            let add_bottom = p.y;
            p.rect.x += rect.width / 2.;
            p.y = bottom;
            p.button(
                if package {
                    "Remove draft terminal"
                } else {
                    "Remove pad"
                },
                if package {
                    C::RemoveTerminal
                } else {
                    C::RemovePad
                },
            );
            p.y = p.y.max(add_bottom);
            p.rect = rect;
        }
        let compact = rect.height < 360.;
        let footer = if compact {
            56.
        } else {
            (rect.height * 0.45).clamp(112., 280.)
        };
        let rows = d.fields.len();
        for (index, field) in d.fields.iter().enumerate().take(rows) {
            p.line(field.label);
            let shown = field
                .choices
                .iter()
                .find(|c| c.0 == field.text)
                .map_or_else(
                    || {
                        if field.text.is_empty() {
                            "Unavailable / not authored".into()
                        } else {
                            field.text.clone()
                        }
                    },
                    |c| c.1.clone(),
                );
            p.button(&shown, C::Field(index));
        }
        if let Some(raw) = &d.preview {
            for (i, pad) in raw
                .get("pads")
                .and_then(Value::as_object)
                .into_iter()
                .flat_map(|p| p.values())
                .enumerate()
            {
                p.button(
                    &format!(
                        "Inspect pad {} process",
                        pad.get("name")
                            .and_then(Value::as_str)
                            .unwrap_or("Unavailable")
                    ),
                    C::PreviewPad(i),
                );
            }
        }
        p.button(
            if d.kind == "packages" {
                "Inspect complete refusal in Console"
            } else {
                "Inspect complete refusal / process reason in Console"
            },
            C::Notice,
        );
        p.y = (rect.y + rect.height - footer).max(p.y);
        if compact {
            let row = p.y;
            let mut bottom = row;
            let width = rect.width / 3.;
            for (i, (label, control)) in [
                ("Preview", C::Preview),
                ("Commit", C::Commit),
                ("Cancel", C::Cancel),
            ]
            .into_iter()
            .enumerate()
            {
                p.rect = RectPx {
                    x: rect.x + i as f32 * width,
                    width,
                    ..rect
                };
                p.y = row;
                p.button(label, control);
                bottom = bottom.max(p.y);
            }
            p.rect = rect;
            p.y = bottom;
        } else {
            p.button("Preview current fields", C::Preview);
            p.button(
                if d.preview.is_some() {
                    "Commit to pool"
                } else {
                    "Commit unavailable · preview required"
                },
                C::Commit,
            );
            p.button("Cancel · discard draft", C::Cancel);
        }
        if let Some(refusal) = d.refusal.as_ref().or(state.library.refusal.as_ref()) {
            p.line(refusal);
        } else if let Some(raw) = &d.preview {
            p.line("Preview only · source and journal unchanged");
            let preview = RectPx {
                x: rect.x + 8.,
                y: p.y,
                width: (rect.width - 16.).max(0.),
                height: footer,
            };
            super::library_preview::render(state, &d.kind, raw, preview, p.quads, p.text);
            p.y += preview.height;
        }
    } else {
        p.line("Project pool · source identity / revision remain separate");
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
        if matches!(
            KINDS[state.library.kind].0,
            "footprints" | "padstacks" | "packages"
        ) {
            p.button(
                &format!("Create {} · target pool", KINDS[state.library.kind].1),
                C::New,
            );
        }
        if let Some(refusal) = &state.library.refusal {
            p.line(refusal);
        }
        if let Some(e) = state.library.selected(&state.selection) {
            p.line(&format!("{} · {}", e.label(), e.name));
            p.line(&e.source_path);
            p.line(&format!(
                "UUID {} · revision {}",
                e.id,
                e.revision.map_or("Unavailable".into(), |r| r.to_string())
            ));
            p.line(&format!(
                "Approval: {}",
                e.raw
                    .get("approval_status")
                    .or_else(|| e.raw.pointer("/approval/status"))
                    .map_or("Unavailable".into(), |v| v.to_string())
            ));
            if let Some(w) = &e.warning {
                p.line(w);
            } else if e.editable() {
                p.button("Edit source · placed users remain frozen", C::Edit);
            }
            if let Some(w) = &state.library.refusal {
                p.line(w);
            }
            let links = state.library.links(e);
            for (index, link) in links.iter().enumerate() {
                if link.targets.len() == 1 {
                    p.button(
                        &format!(
                            "{}: {}",
                            link.role, state.library.entries[link.targets[0]].name
                        ),
                        C::Relationship(index),
                    );
                } else {
                    p.line(&format!(
                        "{}: {} ({})",
                        link.role,
                        if link.targets.is_empty() {
                            "Missing reference"
                        } else {
                            "Ambiguous reference"
                        },
                        link.id
                    ));
                }
            }
            if matches!(e.kind.as_str(), "footprints" | "padstacks") {
                for (i, pad) in e
                    .raw
                    .get("pads")
                    .and_then(Value::as_object)
                    .into_iter()
                    .flat_map(|p| p.values())
                    .enumerate()
                {
                    p.button(
                        &format!(
                            "Inspect pad {} process",
                            pad.get("name")
                                .and_then(Value::as_str)
                                .unwrap_or("Unavailable")
                        ),
                        C::PreviewPad(i),
                    );
                }

                p.button(
                    "Inspect complete refusal / process reason in Console",
                    C::Notice,
                );
            }
            p.button("Previous metadata row", C::DetailPage(false));
            p.button("Next metadata row", C::DetailPage(true));
        }
    }
    let content = p.y - rect.y;
    let mut scroll = datum_gui_viewport::scroll::ScrollViewport::default();
    scroll.layout(
        datum_gui_viewport::ScreenRectPx {
            x: rect.x,
            y: rect.y,
            width: rect.width,
            height: rect.height,
        },
        content,
    );
    scroll.set_offset(state.library.scroll_offset as f32);
    if state.library.reveal_focus {
        if let Some(region) = p.hits[starts.2..].iter().find(|r| matches!(&r.target, HitTarget::Library(c) if Some(c) == state.library.focus.as_ref())) {
            scroll.reveal(region.rect.y - rect.y, region.rect.y + region.rect.height - rect.y);
        }
    }
    let offset = scroll.offset();
    for quad in &mut p.quads[starts.0 + 1..] {
        for point in &mut quad.points {
            point.1 -= offset;
        }
    }
    for run in &mut p.text[starts.1..] {
        run.y -= offset;
        if let Some(clip) = &mut run.clip_bounds {
            clip.y -= offset;
        }
    }
    for hit in &mut p.hits[starts.2..] {
        hit.rect.y -= offset;
    }
    p.hits.insert(
        starts.2,
        HitRegion {
            target: HitTarget::Library(C::Scroll {
                content: content.ceil() as usize,
                height: rect.height as usize,
                offset: offset as usize,
            }),
            rect,
        },
    );
    crate::hit_clipping::clip_content(p.quads, p.text, p.hits, starts.0, starts.1, starts.2, rect);
}
pub(crate) fn inspector(
    state: &ReviewWorkspaceState,
    rect: RectPx,
    quads: &mut Vec<Quad>,
    text: &mut Vec<TextRun>,
) {
    let mut hits = Vec::new();
    let mut p = Painter {
        rect,
        y: rect.y + UI_CARD_DIVIDER_Y + 8.,
        unbounded: false,
        state,
        quads,
        text,
        hits: &mut hits,
    };
    p.line("Library source");
    if let Some(d) = &state.library.draft {
        p.line("Draft · target pool · not approved");
        if let Some(reason) = d.refusal.as_ref().or(state.library.refusal.as_ref()) {
            p.line("Preview refused");
            p.line(reason);
            p.line("Complete reason: Inspect refusal → Console");
        }
        if let Some(raw) = &d.preview {
            p.line("Validated preview · no source / journal writes");
            p.line(
                raw.get("name")
                    .and_then(Value::as_str)
                    .unwrap_or("Name unavailable"),
            );
            if d.kind == "footprints" {
                let package = raw.get("package").and_then(Value::as_str);
                let named = state
                    .library
                    .entries
                    .iter()
                    .filter(|e| e.kind == "packages" && Some(e.id.to_string().as_str()) == package)
                    .collect::<Vec<_>>();
                p.line(&format!(
                    "Package: {}",
                    if named.len() == 1 {
                        named[0].name.as_str()
                    } else {
                        "Missing / ambiguous"
                    }
                ));
                for pad in raw
                    .get("pads")
                    .and_then(Value::as_object)
                    .into_iter()
                    .flat_map(|p| p.values())
                    .take(4)
                {
                    let stack = pad.get("padstack").and_then(Value::as_str);
                    let names = state
                        .library
                        .entries
                        .iter()
                        .filter(|e| {
                            e.kind == "padstacks" && Some(e.id.to_string().as_str()) == stack
                        })
                        .collect::<Vec<_>>();
                    p.line(&format!(
                        "Pad {} → {}",
                        pad.get("name")
                            .and_then(Value::as_str)
                            .unwrap_or("Unavailable"),
                        if names.len() == 1 {
                            names[0].name.as_str()
                        } else {
                            "Missing / ambiguous"
                        }
                    ));
                }
            } else if d.kind == "packages" {
                p.line("Physical body · terminals have names / roles, no stored positions");
                p.line("Footprint lands are authored separately");
            } else {
                p.line(&format!(
                    "Drill: {}",
                    raw.get("drill_nm")
                        .map_or("Absent".into(), |v| format!("{v} nm"))
                ));
            }
            let preview = RectPx {
                x: rect.x + 8.,
                y: p.y,
                width: (rect.width - 16.).max(0.),
                height: (rect.y + rect.height - p.y - 4.).max(0.),
            };
            super::library_preview::render(state, &d.kind, raw, preview, p.quads, p.text);
        } else {
            p.line("Preview unavailable · enter explicit fields");
        }
        return;
    }
    if let Some(entry) = state.library.selected(&state.selection) {
        p.line(&entry.name);
        p.line(&entry.source_path);
        p.line("Authoring does not approve this source");
        for (label, value) in [
            (
                "Approval",
                entry
                    .raw
                    .get("approval_status")
                    .or_else(|| entry.raw.pointer("/approval/status")),
            ),
            ("Lifecycle", entry.raw.get("lifecycle")),
            (
                "Basis",
                entry
                    .raw
                    .get("process_basis")
                    .or_else(|| entry.raw.get("ipc_basis"))
                    .or_else(|| entry.raw.get("standards_basis")),
            ),
            ("Provenance", entry.raw.get("provenance")),
        ] {
            p.line(&format!(
                "{label}: {}",
                value
                    .filter(|v| !v.is_null())
                    .map_or("Unavailable".into(), |v| v.to_string())
            ));
        }
        p.line("Library checks: no authoritative object result / freshness unavailable");
        if let Some(check) = entry.raw.get("check_state") {
            p.line(&format!("Declared check metadata: {check}"));
            p.line("Declared status does not establish fresh checks or approval");
        }
        let rows = ((rect.y + rect.height - p.y - 160.) / 24.).max(1.) as usize;
        for line in state
            .library
            .detail_lines(entry)
            .iter()
            .skip(state.library.detail_offset)
            .take(rows)
        {
            p.line(line);
        }
        let preview = RectPx {
            x: rect.x + 8.,
            y: p.y,
            width: (rect.width - 16.).max(0.),
            height: (rect.y + rect.height - p.y - 4.).max(0.),
        };
        super::library_preview::render(state, &entry.kind, &entry.raw, preview, p.quads, p.text);
    } else {
        p.line("Choose a named library source");
    }
}
