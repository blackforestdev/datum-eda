//! One schematic picker, using the existing library controls and preview geometry.
use super::*;

pub(crate) fn render(
    state: &ReviewWorkspaceState,
    layout: &ShellLayout,
    quads: &mut Vec<Quad>,
    text: &mut Vec<TextRun>,
    hits: &mut Vec<HitRegion>,
) {
    let a = state.library.acquisition.as_ref().unwrap();
    let width = 720f32.min((layout.top_menu_bar.width - 32.).max(1.));
    let rect = RectPx {
        x: (layout.top_menu_bar.width - width) / 2.,
        y: 64.,
        width,
        height: (layout.status_bar.y - 80.).max(1.).min(590.),
    };
    quads.push(Quad::from_rect(rect, design_tokens::chrome::SURFACE_01));
    push_rect_border(quads, rect, design_tokens::chrome::BORDER_STRONG, 1.);
    hits.push(HitRegion {
        target: HitTarget::SchematicPicker,
        rect,
    });
    let mut p = Painter {
        rect,
        y: rect.y + 8.,
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
    p.line(&target.map_or_else(
        || "Captured schematic unavailable".into(),
        |row| {
            format!(
                "Target: {} · occurrence /{}",
                row.source_sheet_name,
                if row.key.instance_path.is_empty() {
                    String::new()
                } else {
                    row.breadcrumb.join("/")
                }
            )
        },
    ));
    let outer = rect;
    let rect = RectPx {
        y: p.y,
        height: (outer.y + outer.height - p.y - 96.).max(1.),
        ..outer
    };
    p.rect = rect;
    p.line("Search components and Symbols (Ctrl+F)");
    field(&mut p, &a.search, a.search_editor, C::AcquireSearch);
    let rect = RectPx {
        y: p.y,
        height: (outer.y + outer.height - p.y - 96.).max(1.),
        ..outer
    };
    p.rect = rect;
    let starts = (p.quads.len(), p.text.len(), p.hits.len());
    let results = a.results();
    if a.loading {
        p.line(if a.loaded {
            "Reading the selected preview…"
        } else {
            "Loading project components and shipped Symbols…"
        });
    } else if !a.loaded {
        p.line("Library catalog unavailable. Cancel and open Place Component again.");
    } else if a.names.is_empty() {
        p.line("No project components or shipped Symbols are available.");
    } else if results.is_empty() {
        p.line("No matches in project components or shipped Symbols. Refine the search.");
    } else {
        p.line(&format!(
            "{} matching components and Symbols",
            results.len()
        ));
        for index in results.iter().skip(a.offset).take(4) {
            let source = &a.sources[*index];
            p.button(
                &format!(
                    "{} · {}{}",
                    match a.picker_result_kind(source) {
                        datum_gui_protocol::library_browser::acquisition::ComponentPickerResultKind::CompletePart => "Part",
                        datum_gui_protocol::library_browser::acquisition::ComponentPickerResultKind::Symbol => "Symbol · schematic only",
                    },
                    source.display_label(),
                    if source.is_unavailable() {
                        " · unavailable"
                    } else {
                        ""
                    }
                ),
                C::AcquireResult(*index),
            );
        }
        if results.len() > 4 {
            let y = p.y;
            p.rect.width = width / 2.;
            p.button("Previous", C::AcquirePage(false));
            let bottom = p.y;
            p.rect.x += width / 2.;
            p.y = y;
            p.button("Next", C::AcquirePage(true));
            p.y = bottom.max(p.y);
            p.rect = rect;
        }
    }
    if a.unavailable_count() > 0 {
        p.button(
            if a.include_unavailable {
                "Hide unavailable results"
            } else {
                "Show unavailable results"
            },
            C::AcquireShowUnavailable,
        );
    }
    if let Some(preview) = &a.preview
        && let Some(part) = &preview.native_part
    {
        let name = |raw: &Value, key| {
            raw.get(key)
                .and_then(Value::as_str)
                .unwrap_or("Unavailable")
                .to_owned()
        };
        p.line(&format!(
            "Value {} · Package {}",
            name(&part.part, "value"),
            name(&part.package, "name")
        ));
        let (gate, unit, symbol) = &part.gate_symbols[0];
        p.line(&format!(
            "Symbol: gate {gate} · Footprint: {} · declared pin/pad map",
            name(&part.footprint, "name")
        ));
        let preview_rect = RectPx {
            x: rect.x + 8.,
            y: p.y,
            width: (width - 24.) / 2.,
            height: 120.,
        };
        super::super::super::library_preview::electrical::symbol(
            symbol,
            Some(unit),
            preview_rect,
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
                x: rect.x + width / 2.,
                ..preview_rect
            },
            p.quads,
            p.text,
        );
        p.y += 124.;
        match part.mapping_labels() {
            Ok(rows) => p.line(&rows.join(" · ")),
            Err(reason) => p.line(&reason),
        }
        p.line("Complete declared graph · existing engine checks apply. Approval/basis not established by acquisition.");
    }
    if let Some(preview) = &a.preview
        && preview.native_part.is_none()
    {
        p.line(&format!(
            "Symbol: {} — SCHEMATIC ONLY",
            a.sources
                .iter()
                .find(|source| Some(&source.path) == a.selected_source.as_ref()
                    && Some(source.name.as_str()) == a.selected.as_deref())
                .map_or(preview.name.as_str(), |source| source.label.as_str())
        ));
        let side_by_side = width >= 600.;
        let preview_y = p.y;
        let preview_rect = RectPx {
            x: rect.x + 8.,
            y: preview_y,
            width: if side_by_side {
                width / 2. - 16.
            } else {
                width - 16.
            },
            height: 140.,
        };
        super::super::super::library_preview::electrical::symbol(
            &preview.symbol,
            Some(&preview.unit),
            preview_rect,
            p.quads,
            p.text,
        );
        if side_by_side {
            p.rect = RectPx {
                x: rect.x + width / 2.,
                width: width / 2.,
                ..rect
            };
            p.line("Physical");
            p.line("SCHEMATIC ONLY. No Package, Footprint or pin-pad map.");
            p.y = p.y.max(preview_y + 144.);
            p.rect = rect;
        } else {
            p.y += 144.;
            p.line("SCHEMATIC ONLY. No Package, Footprint or pin-pad map.");
        }
        p.line("Library approval is not established by acquisition.");
    }

    if let Some(reason) = &a.refusal {
        p.line(reason);
    }
    scroll_content(&mut p, starts, 0);
    p.rect = outer;
    p.y = outer.y + outer.height - 88.;
    let footer_y = p.y;
    p.rect.width = outer.width / 3.;
    p.button("Cancel", C::Cancel);
    p.rect.x = outer.x + outer.width / 3.;
    p.rect.width = outer.width * 2. / 3.;
    p.y = footer_y;
    if let Some(preview) = &a.preview
        && a.picker_can_place(&state.library)
    {
        if preview.native_part.is_some() {
            p.button("Place Component", C::AcquireImport);
        } else {
            p.button(
                &format!("Place {} — schematic only", preview.name),
                C::AcquireImport,
            );
        }
    }
}

#[cfg(test)]
#[path = "component_picker_tests.rs"]
mod tests;
