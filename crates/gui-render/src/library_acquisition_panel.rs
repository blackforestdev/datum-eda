//! Local-source acquisition in the existing library pane and focus sequence.
use super::*;
use crate::global_preferences_primitives::{ControlMeshCache, ControlPainter};
#[path = "library_footprint_preview.rs"]
mod footprint;

pub(super) fn render(p: &mut Painter<'_>) {
    let a = p.state.library.acquisition.as_ref().unwrap();
    if a.bundled() {
        p.line("Datum symbol library");
        p.line(if a.placement_pane.is_some() {
            "Search, then Enter or choose a result to place"
        } else {
            "Choose a symbol to inspect"
        });
    } else {
        p.line(if a.native_parts() {
            "Browse reusable native Parts"
        } else if a.footprints() {
            "Browse KiCad footprints"
        } else {
            "Browse KiCad symbols"
        });
        p.line(if a.native_parts() {
            "Saved native Datum project folder"
        } else if a.footprints() {
            "Library folder, .pretty folder or .kicad_mod file"
        } else {
            "Library folder or .kicad_sym file"
        });
        field(p, &a.path, a.path_editor, C::AcquirePath);
        p.button("Refresh source", C::AcquireLoad);
    }
    p.line(&format!("Search {}s (Ctrl+F)", a.object_label()));
    field(p, &a.search, a.search_editor, C::AcquireSearch);
    if a.placement_pane.is_some() && !a.is_native_part() {
        p.button("Project symbols…", C::ProjectSymbols);
    }
    if let Some(preview) = &a.preview {
        p.line(&format!("Selected: {}", preview.name));
        if let Some(part) = &preview.native_part {
            p.line(&format!(
                "{} gates · {} native definitions",
                part.gate_count, part.object_count
            ));
            p.line(&format!(
                "Package: {} · Footprint: {}",
                part.package["name"].as_str().unwrap_or("Unavailable"),
                part.footprint["name"].as_str().unwrap_or("Unavailable")
            ));
            p.line("Complete explicit pin/pad map · identical saved definitions reused");
            match part.mapping_labels() {
                Ok(rows) => {
                    for row in rows {
                        p.line(&row);
                    }
                }
                Err(reason) => p.line(&reason),
            }
        }
        if preview
            .notices
            .iter()
            .any(|n| n.starts_with("Already in this Project:"))
        {
            p.line("Uses this Project's saved symbol.");
        }
        p.button(
            if a.placement_pane.is_some() && a.is_native_part() {
                "Add component"
            } else if a.placement_pane.is_some() {
                "Place symbol"
            } else if a.bundled() {
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
            height: if a.native_parts() {
                300.
            } else if a.footprints() {
                210.
            } else {
                120.
            },
        };
        if let Some(part) = &preview.native_part {
            for (name, unit, symbol) in &part.gate_symbols {
                p.line(&format!("Gate: {name}"));
                super::super::library_preview::electrical::symbol(
                    symbol,
                    Some(unit),
                    RectPx {
                        y: p.y,
                        height: 120.,
                        ..rect
                    },
                    p.quads,
                    p.text,
                );
                p.y += 125.;
            }
            let physical =
                datum_gui_protocol::library_browser::acquisition::FootprintAcquisitionPreview {
                    package: part.package.clone(),
                    footprint: part.footprint.clone(),
                    padstacks: part.padstacks.clone(),
                };
            footprint::render(
                &physical,
                RectPx {
                    y: p.y,
                    height: 170.,
                    ..rect
                },
                p.quads,
                p.text,
            );
            p.y += 175.;
        } else if let Some(physical) = &preview.footprint {
            footprint::render(physical, rect, p.quads, p.text);
        } else {
            super::super::library_preview::electrical::symbol(
                &preview.symbol,
                Some(&preview.unit),
                rect,
                p.quads,
                p.text,
            );
        }
        if preview.native_part.is_none() {
            p.y += rect.height;
        }
    }
    if a.loading {
        p.line("Reading local source…");
    } else if !a.loaded {
        p.line("Choose a library source, then refresh");
    } else if a.names.is_empty() {
        p.line(&format!(
            "This source contains no named {}s",
            a.object_label()
        ));
    } else {
        let results = a.results();
        p.line(&format!("{} matching {}s", results.len(), a.object_label()));
        if results.is_empty() {
            p.line(if a.is_native_part() {
                "No matching complete Parts. Try a value, manufacturer or part number."
            } else if a.footprints() {
                "No matching footprints. Try R_0603, SOT or connector."
            } else {
                "No matching symbols. Try a name such as resistor or capacitor."
            });
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
    if a.bundled() || a.is_native_part() {
        p.button(
            if a.is_native_part() && a.include_unavailable {
                "Hide unavailable Parts"
            } else if a.is_native_part() {
                "Show unavailable Parts"
            } else if a.include_unavailable {
                "Hide unavailable symbols"
            } else {
                "Show unavailable symbols"
            },
            C::AcquireShowUnavailable,
        );
    }
    if !a.catalog_notices.is_empty() {
        if !a.bundled() && !a.is_native_part() {
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

pub(super) fn field(
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
    p.line(if a.bundled() {
        "Shipped symbol · preview only"
    } else {
        "Local source acquisition · preview only"
    });
    if !a.bundled() {
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
        if let Some(physical) = &preview.footprint {
            p.line(&format!(
                "{} padstacks · manual Part binding after acquisition",
                physical.padstacks.len()
            ));
            p.line("Source dimensions retained; manufacturer qualification unknown");
        }
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
