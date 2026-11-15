//! One continuous result viewport beside the existing typed acquisition preview.
use super::*;
use datum_gui_protocol::library_browser::acquisition::{
    COMPONENT_PICKER_ROW_HEIGHT, ComponentPickerResultKind,
};

pub(crate) fn render(
    state: &ReviewWorkspaceState,
    layout: &ShellLayout,
    quads: &mut Vec<Quad>,
    text: &mut Vec<TextRun>,
    hits: &mut Vec<HitRegion>,
) {
    let a = state.library.acquisition.as_ref().unwrap();
    let width = 720f32.min((layout.top_menu_bar.width - 32.).max(1.));
    let outer = RectPx {
        x: (layout.top_menu_bar.width - width) / 2.,
        y: 64.,
        width,
        height: (layout.status_bar.y - 80.).max(1.).min(590.),
    };
    quads.push(Quad::from_rect(outer, design_tokens::chrome::SURFACE_01));
    push_rect_border(quads, outer, design_tokens::chrome::BORDER_STRONG, 1.);
    hits.push(HitRegion {
        target: HitTarget::SchematicPicker,
        rect: outer,
    });
    let mut p = Painter {
        rect: outer,
        y: outer.y + 6.,
        unbounded: true,
        state,
        quads,
        text,
        hits,
    };
    p.line("Place Component");
    let target = a.placement_document.as_ref().and_then(|key| {
        state
            .schematic_documents
            .rows
            .iter()
            .find(|row| &row.key == key)
    });
    small_line(
        &mut p,
        &target.map_or_else(
            || "Captured schematic unavailable".into(),
            |row| {
                format!(
                    "Target: {} · occurrence /{}",
                    row.source_sheet_name,
                    row.breadcrumb.join("/")
                )
            },
        ),
    );
    p.line("Search components and Symbols (Ctrl+F)");
    field(&mut p, &a.search, a.search_editor, C::AcquireSearch);
    let results = a.results();
    let count_y = p.y;
    small_line(
        &mut p,
        &format!("{} matching components and Symbols", results.len()),
    );
    if a.unavailable_count() > 0 {
        let label = if a.include_unavailable {
            "Hide unavailable"
        } else {
            "Show unavailable"
        };
        let rect = RectPx {
            x: outer.x + width - 158.,
            y: count_y,
            width: 150.,
            height: 22.,
        };
        p.quads
            .push(Quad::from_rect(rect, design_tokens::chrome::SURFACE_02));
        label_at(&mut p, label, rect, 12., TEXT_SECONDARY);
        p.hits.push(HitRegion {
            target: HitTarget::Library(C::AcquireShowUnavailable),
            rect,
        });
        if state.library.focus == Some(C::AcquireShowUnavailable) {
            push_rect_border(p.quads, rect, TEXT_ACCENT, 1.);
        }
    }
    let footer_y = outer.y + outer.height - 64.;
    let hint_y = footer_y - 40.;
    let body = RectPx {
        y: p.y + 2.,
        height: (hint_y - p.y - 4.).max(1.),
        ..outer
    };
    let normal = width >= 640.;
    let list = if normal {
        RectPx {
            width: width - 300.,
            ..body
        }
    } else {
        RectPx {
            height: (body.height * 0.33).min(108.),
            ..body
        }
    };
    let details = if normal {
        RectPx {
            x: list.x + list.width,
            width: 300.,
            ..body
        }
    } else {
        RectPx {
            y: list.y + list.height + 4.,
            height: (body.height - list.height - 4.).max(1.),
            ..body
        }
    };
    p.rect = list;
    p.y = list.y;
    let starts = (p.quads.len(), p.text.len(), p.hits.len());
    if !a.loaded {
        p.line(if a.loading {
            "Loading project components and shipped Symbols…"
        } else {
            "Library catalog unavailable. Cancel and open Place Component again."
        });
    } else if results.is_empty() {
        p.line("No matches. Refine the search.");
    } else {
        // Paint only the visible slice; the shared viewport retains the full list extent.
        let maximum = (results.len() as f32 * COMPONENT_PICKER_ROW_HEIGHT - list.height).max(0.);
        let first = (state.library.scroll_offset as f32).min(maximum) / COMPONENT_PICKER_ROW_HEIGHT;
        let first = first as usize;
        let count = (list.height / COMPONENT_PICKER_ROW_HEIGHT).ceil() as usize + 2;
        for (position, index) in results.iter().enumerate().skip(first).take(count) {
            p.y = list.y + position as f32 * COMPONENT_PICKER_ROW_HEIGHT;
            result_row(&mut p, *index);
        }
        p.y = list.y + results.len() as f32 * COMPONENT_PICKER_ROW_HEIGHT;
    }
    scroll_content(&mut p, starts, 0);
    p.rect = details;
    p.y = details.y;
    p.unbounded = false;
    push_rect_border(p.quads, details, design_tokens::chrome::BORDER_STRONG, 1.);
    let starts = (p.quads.len(), p.text.len(), p.hits.len());
    preview(&mut p, normal);
    crate::hit_clipping::clip_content(
        p.quads, p.text, p.hits, starts.0, starts.1, starts.2, details,
    );
    p.rect = RectPx {
        y: hint_y,
        height: 40.,
        ..outer
    };
    p.y = hint_y;
    small_line(&mut p, "↓ Search to results · ↑/↓ select + preview");
    small_line(&mut p, "PgUp/PgDn page · wheel scrolls · Tab controls");
    p.rect = outer;
    p.y = footer_y;
    p.unbounded = true;
    p.rect.width = outer.width / 3.;
    p.button("Cancel", C::Cancel);
    p.rect = RectPx {
        x: outer.x + outer.width / 3.,
        width: outer.width * 2. / 3.,
        ..outer
    };
    p.y = footer_y;
    if let Some(preview) = &a.preview
        && a.picker_can_place(&state.library)
    {
        p.button(
            &if preview.native_part.is_some() {
                "Place Component".into()
            } else {
                format!("Place {} — schematic only", preview.name)
            },
            C::AcquireImport,
        );
    }
}

fn label_at(p: &mut Painter<'_>, label: &str, rect: RectPx, size: f32, color: design_tokens::Rgb) {
    let label =
        crate::terminal_tab_strip::fit_end(label, (rect.width - 8.).max(0.), size, TextFace::Ui);
    draw_text_clipped(
        &label,
        rect.x + 4.,
        rect.y + 3.,
        size,
        color,
        TextFace::Ui,
        rect,
        p.text,
    );
    p.text.last_mut().unwrap().layout_size =
        Some(((rect.width - 8.).max(1.), (rect.height - 4.).max(1.)));
}

fn small_line(p: &mut Painter<'_>, label: &str) {
    let rect = RectPx {
        x: p.rect.x + 8.,
        y: p.y,
        width: (p.rect.width - 16.).max(1.),
        height: 20.,
    };
    label_at(p, label, rect, 12., TEXT_SECONDARY);
    p.y += 20.;
}

fn result_row(p: &mut Painter<'_>, index: usize) {
    let a = p.state.library.acquisition.as_ref().unwrap();
    let source = &a.sources[index];
    let selected = a.selected.as_deref() == Some(source.name.as_str())
        && a.selected_source.as_ref() == Some(&source.path);
    let rect = RectPx {
        x: p.rect.x,
        y: p.y,
        width: p.rect.width,
        height: COMPONENT_PICKER_ROW_HEIGHT,
    };
    p.quads.push(Quad::from_rect(
        rect,
        if selected {
            design_tokens::chrome::SURFACE_03
        } else {
            design_tokens::chrome::SURFACE_02
        },
    ));
    if p.state.library.focus == Some(C::AcquireResult(index)) {
        push_rect_border(p.quads, rect, TEXT_ACCENT, 1.);
    }
    let kind = a.picker_result_kind(source);
    label_at(
        p,
        match kind {
            ComponentPickerResultKind::CompletePart => "Part",
            ComponentPickerResultKind::Symbol => "Symbol",
        },
        RectPx {
            width: 60.,
            y: rect.y + 5.,
            height: 23.,
            ..rect
        },
        12.,
        TEXT_SECONDARY,
    );
    let status = if source.is_unavailable() {
        "unavailable"
    } else if kind == ComponentPickerResultKind::Symbol {
        "sch only"
    } else {
        "complete"
    };
    let status_width = if source.is_unavailable() { 80. } else { 58. };
    let identity = RectPx {
        x: rect.x + 60.,
        y: rect.y + 4.,
        width: (rect.width - 64. - status_width).max(1.),
        height: 25.,
    };
    label_at(p, &source.label, identity, 16., TEXT_PRIMARY);
    let name_width =
        measured_text_run_width_px(&source.label, 16., TextFace::Ui).unwrap_or(identity.width);
    if name_width + 24. < identity.width {
        label_at(
            p,
            &format!("— {}", source.description),
            RectPx {
                x: identity.x + name_width + 8.,
                width: identity.width - name_width - 8.,
                ..identity
            },
            13.,
            TEXT_SECONDARY,
        );
    }
    label_at(
        p,
        status,
        RectPx {
            x: rect.x + rect.width - status_width,
            width: status_width,
            y: rect.y + 5.,
            height: 23.,
        },
        12.,
        TEXT_SECONDARY,
    );
    p.hits.push(HitRegion {
        target: HitTarget::Library(C::AcquireResult(index)),
        rect,
    });
}

fn preview(p: &mut Painter<'_>, normal: bool) {
    let a = p.state.library.acquisition.as_ref().unwrap();
    let source = a.sources.iter().find(|source| {
        Some(&source.path) == a.selected_source.as_ref()
            && Some(source.name.as_str()) == a.selected.as_deref()
    });
    if let Some(source) = source {
        let identity = RectPx {
            y: p.y,
            height: 28.,
            ..p.rect
        };
        label_at(p, &source.label, identity, 16., TEXT_PRIMARY);
        p.y += 28.;
        p.line(match a.picker_result_kind(source) {
            ComponentPickerResultKind::CompletePart => "Part — complete declared graph",
            ComponentPickerResultKind::Symbol => "Symbol — SCHEMATIC ONLY",
        });
        if a.picker_result_kind(source) == ComponentPickerResultKind::Symbol {
            p.line("No Package, Footprint or pin-pad map.");
        }
    } else {
        p.line("Select a result to preview");
    }
    if let Some(preview) = &a.preview {
        let geometry = RectPx {
            x: p.rect.x + 8.,
            y: p.y,
            width: (p.rect.width - 16.).max(1.),
            height: if normal { 116. } else { 88. },
        };
        if let Some(part) = &preview.native_part {
            let (_, unit, symbol) = &part.gate_symbols[0];
            let half = RectPx {
                width: geometry.width / 2.,
                ..geometry
            };
            super::super::super::library_preview::electrical::symbol(
                symbol,
                Some(unit),
                half,
                p.quads,
                p.text,
            );
            let physical =
                datum_gui_protocol::library_browser::acquisition::FootprintAcquisitionPreview {
                    package: part.package.clone(),
                    footprint: part.footprint.clone(),
                    padstacks: part.padstacks.clone(),
                };
            footprint::render(
                &physical,
                RectPx {
                    x: half.x + half.width,
                    ..half
                },
                p.quads,
                p.text,
            );
        } else {
            super::super::super::library_preview::electrical::symbol(
                &preview.symbol,
                Some(&preview.unit),
                geometry,
                p.quads,
                p.text,
            );
        }
        p.y += geometry.height + 4.;
    }
    p.rect = RectPx {
        y: p.y,
        height: (p.rect.y + p.rect.height - p.y).max(1.),
        ..p.rect
    };
    p.unbounded = true;
    let starts = (p.quads.len(), p.text.len(), p.hits.len());
    if let Some(reason) = &a.refusal {
        p.line(reason);
    }
    if let Some(preview) = &a.preview {
        if let Some(part) = &preview.native_part {
            let name = |raw: &Value, key| {
                raw.get(key)
                    .and_then(Value::as_str)
                    .unwrap_or("Unavailable")
                    .to_owned()
            };
            p.line("Complete Part · declared physical graph");
            p.line(&format!(
                "Value {} · Package {}",
                name(&part.part, "value"),
                name(&part.package, "name")
            ));
            p.line(&format!(
                "Gate {} · Footprint {}",
                part.gate_symbols[0].0,
                name(&part.footprint, "name")
            ));
            match part.mapping_labels() {
                Ok(rows) => p.line(&rows.join(" · ")),
                Err(reason) => p.line(&reason),
            }
        }
        if let Some(source) = source {
            p.line(&source.description);
        }
        small_line(p, "Library approval is not established by acquisition.");
    } else if a.loading && a.loaded {
        p.line("Reading the selected preview…");
    }
    scroll_acquisition_content(p, starts, 0, true);
}

#[cfg(test)]
#[path = "component_picker_tests.rs"]
mod tests;
