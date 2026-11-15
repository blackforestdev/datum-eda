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
    let width = 760f32.min((layout.top_menu_bar.width - 32.).max(1.));
    let rect = RectPx {
        x: (layout.top_menu_bar.width - width) / 2.,
        y: 76.,
        width,
        height: (layout.status_bar.y - 92.).max(1.).min(820.),
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
        height: (outer.y + outer.height - p.y - 82.).max(1.),
        ..outer
    };
    p.rect = rect;
    let starts = (p.quads.len(), p.text.len(), p.hits.len());
    p.line("Source · saved native project folder");
    let source_y = p.y;
    p.rect.width = (width - 132.).max(1.);
    field(&mut p, &a.path, a.path_editor, C::AcquirePath);
    p.rect = RectPx {
        x: rect.x + width - 132.,
        width: 132.,
        ..rect
    };
    p.y = source_y;
    p.button("Read source", C::AcquireLoad);
    p.rect = rect;
    p.y = source_y + 40.;
    p.line("Search components (Ctrl+F)");
    field(&mut p, &a.search, a.search_editor, C::AcquireSearch);
    let results = a.results();
    if a.loading {
        p.line("Reading selected source…");
    } else if !a.loaded {
        p.line(if a.path.trim().is_empty() { "Choose a saved source containing complete Parts. No component catalog is configured yet." } else { "Read the source to search its declared components." });
    } else if a.names.is_empty() {
        p.line("This source contains no declared Parts.");
    } else if results.is_empty() {
        p.line("No matching complete components. Refine the search or show unavailable Parts.");
    } else {
        p.line(&format!("{} matching components", results.len()));
        for index in results.iter().skip(a.offset).take(4) {
            let source = &a.sources[*index];
            p.button(
                &format!(
                    "{}{}",
                    source.label,
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
                "Hide unavailable Parts"
            } else {
                "Show unavailable Parts"
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
    if let Some(reason) = &a.refusal {
        p.line(reason);
    }
    scroll_content(&mut p, starts, 0);
    p.rect = outer;
    p.y = outer.y + outer.height - 78.;
    if let Some(part) = a
        .preview
        .as_ref()
        .and_then(|preview| preview.native_part.as_ref())
        && !a.loading
        && a.selected_native_part().is_some()
    {
        p.button(
            if part.in_project(&state.library) {
                "Place Component · start placement"
            } else {
                "Adopt into project library · separate undoable commit"
            },
            C::AcquireImport,
        );
    } else {
        p.y += 36.;
    }
    p.button("Cancel", C::Cancel);
}

#[cfg(test)]
#[path = "component_picker_tests.rs"]
mod tests;
