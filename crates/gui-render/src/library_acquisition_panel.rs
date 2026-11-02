//! Local-source acquisition in the existing library pane and focus sequence.
use super::*;
use crate::global_preferences_primitives::{ControlMeshCache, ControlPainter};

pub(super) fn render(p: &mut Painter<'_>) {
    let a = p.state.library.acquisition.as_ref().unwrap();
    if a.bundled {
        p.line("Datum symbol library");
        p.line(if a.placement_pane.is_some() {
            "Search, then Enter or choose a result to place"
        } else {
            "Choose a symbol to inspect"
        });
    } else {
        p.line("Browse KiCad symbols");
        p.line("Library folder or .kicad_sym file");
        field(p, &a.path, a.path_editor, C::AcquirePath);
        p.button("Refresh source", C::AcquireLoad);
    }
    p.line("Search symbols (Ctrl+F)");
    field(p, &a.search, a.search_editor, C::AcquireSearch);
    if a.placement_pane.is_some() {
        p.button("Project symbols…", C::ProjectSymbols);
    }
    if let Some(preview) = &a.preview {
        p.line(&format!("Selected: {}", preview.name));
        if preview
            .notices
            .iter()
            .any(|n| n.starts_with("Already in this Project:"))
        {
            p.line("Uses this Project's saved symbol.");
        }
        p.button(
            if a.placement_pane.is_some() {
                "Place symbol"
            } else if a.bundled {
                "Add to Project library"
            } else {
                "Import into Project pool"
            },
            C::AcquireImport,
        );
        let rect = RectPx {
            x: p.rect.x + 8.,
            y: p.y,
            width: (p.rect.width - 16.).max(0.),
            height: 120.,
        };
        super::super::library_preview::electrical::symbol(
            &preview.symbol,
            Some(&preview.unit),
            rect,
            p.quads,
            p.text,
        );
        p.y += rect.height;
    }
    if a.loading {
        p.line("Reading local source…");
    } else if !a.loaded {
        p.line("Choose a library folder or .kicad_sym file, then refresh");
    } else if a.names.is_empty() {
        p.line("This source contains no named symbols");
    } else {
        let results = a.results();
        p.line(&format!("{} matching symbols", results.len()));
        if results.is_empty() {
            p.line("No matching symbols. Try a name such as resistor or capacitor.");
        }
        for index in results.iter().skip(a.offset).take(4) {
            p.button(
                &a.sources
                    .get(*index)
                    .map(|s| s.display_label())
                    .unwrap_or_else(|| a.names[*index].clone()),
                C::AcquireResult(*index),
            );
        }
        if results.len() > 4 {
            p.button("Previous results", C::AcquirePage(false));
            p.button("Next results", C::AcquirePage(true));
        }
    }
    if a.bundled {
        p.button(
            if a.include_unavailable {
                "Hide unavailable symbols"
            } else {
                "Show unavailable symbols"
            },
            C::AcquireShowUnavailable,
        );
    }
    if !a.catalog_notices.is_empty() {
        if !a.bundled {
            p.line(&format!(
                "{} source files could not be read",
                a.catalog_notices.len()
            ));
        }
        for notice in a.catalog_notices.iter().take(2) {
            p.line(notice);
        }
    }
    if let Some(reason) = &a.refusal {
        p.line(reason);
    }
    p.button("Cancel · leave source and Project unchanged", C::Cancel);
}

fn field(
    p: &mut Painter<'_>,
    value: &str,
    editor: datum_gui_protocol::single_line_editor::SingleLineEditor,
    control: C,
) {
    let rect = RectPx {
        x: p.rect.x + 6.,
        y: p.y,
        width: (p.rect.width - 12.).max(0.),
        height: 34.,
    };
    let mut cache = ControlMeshCache::default();
    let _ = crate::single_line_control::paint(
        value,
        if control == C::AcquireSearch {
            "resistor, capacitor, connector…"
        } else {
            ""
        },
        editor,
        p.state.library.focus.as_ref() == Some(&control),
        true,
        rect,
        &mut ControlPainter::new(p.quads, &mut cache, 1.),
        p.text,
        p.hits,
        HitTarget::Library(control),
    );
    p.y += 38.;
}

pub(super) fn inspector(p: &mut Painter<'_>) {
    let a = p.state.library.acquisition.as_ref().unwrap();
    p.line(if a.bundled {
        "Shipped symbol · preview only"
    } else {
        "Local source acquisition · preview only"
    });
    if !a.bundled {
        p.line(&a.path);
    }
    p.line("Checks: unchecked · approval: unavailable");
    p.line("Standards basis and production qualification: unknown");
    if let Some(preview) = &a.preview {
        p.line(&preview.name);
        for notice in &preview.notices {
            p.line(notice);
        }
        p.line(&format!(
            "Original source: {}",
            preview.source_path.display()
        ));
        p.line(&format!("Source hash: {}", preview.source_digest));
        for pin in preview
            .unit
            .get("pins")
            .and_then(Value::as_object)
            .into_iter()
            .flat_map(|pins| pins.values())
        {
            p.line(&format!(
                "{} · {}",
                pin.get("name").and_then(Value::as_str).unwrap_or("Unnamed"),
                pin.get("electrical_type")
                    .or_else(|| pin.get("direction"))
                    .and_then(Value::as_str)
                    .unwrap_or("Unspecified")
            ));
        }
    }
    if let Some(reason) = &a.refusal {
        p.line(reason);
    }
}
