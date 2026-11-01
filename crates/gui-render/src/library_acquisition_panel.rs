//! Local-source acquisition in the existing library pane and focus sequence.
use super::*;
use crate::global_preferences_primitives::{ControlMeshCache, ControlPainter};

pub(super) fn render(p: &mut Painter<'_>) {
    let a = p.state.library.acquisition.as_ref().unwrap();
    p.line("Acquire local KiCad symbol");
    p.line("Source .kicad_sym file");
    field(p, &a.path, a.path_editor, C::AcquirePath);
    p.button("Load source names", C::AcquireLoad);
    p.line("Search symbol names");
    field(p, &a.search, a.search_editor, C::AcquireSearch);
    if a.loading {
        p.line("Reading local source…");
    } else if !a.loaded {
        p.line("Load a .kicad_sym file to browse its named symbols");
    } else if a.names.is_empty() {
        p.line("This source contains no named symbols");
    } else {
        let results = a.results();
        p.line(&format!(
            "{} matching names · exact first, then name order",
            results.len()
        ));
        for index in results.iter().skip(a.offset).take(4) {
            p.button(
                &format!("Preview {}", a.names[*index]),
                C::AcquireResult(*index),
            );
        }
        if results.len() > 4 {
            p.button("Previous names", C::AcquirePage(false));
            p.button("Next names", C::AcquirePage(true));
        }
    }
    if let Some(reason) = &a.refusal {
        p.line(reason);
    }
    if let Some(preview) = &a.preview {
        p.line(&format!("{} · preview only", preview.name));
        p.line("Checks: unchecked · approval: unavailable · standards basis: unknown");
        for notice in &preview.notices {
            p.line(notice);
        }
        let pins = preview.unit.get("pins").and_then(Value::as_object);
        p.line(&format!(
            "{} logical pins · native Unit and Symbol",
            pins.map_or(0, |pins| pins.len())
        ));
        let rect = RectPx {
            x: p.rect.x + 8.,
            y: p.y,
            width: (p.rect.width - 16.).max(0.),
            height: 190.,
        };
        super::super::library_preview::electrical::symbol(
            &preview.symbol,
            Some(&preview.unit),
            rect,
            p.quads,
            p.text,
        );
        p.y += rect.height;
        p.button("Import into Project pool", C::AcquireImport);
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
        "",
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
    p.line("Local source acquisition · preview only");
    p.line(&a.path);
    p.line("Checks: unchecked · approval: unavailable");
    p.line("Standards basis and production qualification: unknown");
    if let Some(preview) = &a.preview {
        p.line(&preview.name);
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
