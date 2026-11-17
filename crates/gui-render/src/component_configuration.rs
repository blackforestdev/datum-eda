//! Shared disposable component form and footprint chooser for both entrypoints.
use super::*;
use datum_gui_protocol::library_browser::component_resolution::configuration::ConfigurationControl as F;

pub(super) fn form(p: &mut Painter<'_>) {
    let Some(configuration) = p.state.library.configuration.as_ref() else {
        p.line("Select a component to configure");
        return;
    };
    if configuration.target.is_some() {
        p.line("Reference");
        field(
            p,
            &configuration.reference,
            configuration.reference_editor,
            C::Field(0),
        );
    }
    p.line("Value");
    let value_runs = p.text.len();
    field(
        p,
        &configuration.value,
        configuration.value_editor,
        C::Field(1),
    );
    if std::env::var_os("DATUM_TRACE_CLICKS").is_some() {
        eprintln!(
            "[datum-component-render] {:?} session={} value_bytes={} emitted_bytes={:?} caret={} anchor={}",
            std::time::SystemTime::now(),
            configuration.session,
            configuration.value.len(),
            p.text[value_runs..]
                .iter()
                .map(|run| run.text.len())
                .collect::<Vec<_>>(),
            configuration.value_editor.caret,
            configuration.value_editor.anchor
        );
    }
    p.line(&configuration.footprint().map_or_else(
        |_| "Footprint: not chosen".into(),
        |f| format!("Footprint: {}", f.name),
    ));
    if configuration.footprint_change_refusal.is_none() && !configuration.pending() {
        p.button(
            if configuration.footprint_preview.is_some() {
                "Change Footprint…"
            } else {
                "Choose Footprint…"
            },
            C::Configure(F::ChooseFootprint),
        );
    }
    let geometry = RectPx {
        x: p.rect.x + 8.,
        y: p.y,
        width: (p.rect.width - 16.).max(1.) / 2.,
        height: 92.,
    };
    super::super::super::super::library_preview::electrical::symbol(
        &configuration.symbol_preview.symbol,
        Some(&configuration.symbol_preview.unit),
        geometry,
        p.quads,
        p.text,
    );
    if let Some(physical) = configuration
        .footprint_preview
        .as_ref()
        .and_then(|p| p.footprint.as_ref())
    {
        footprint::render(
            physical,
            RectPx {
                x: geometry.x + geometry.width,
                ..geometry
            },
            p.quads,
            p.text,
        );
    }
    p.y += 100.;
    if let Some(reason) = &configuration.footprint_change_refusal {
        p.line(reason);
    } else if let (Ok(physical), Ok(unit)) = (configuration.footprint(), configuration.unit()) {
        if configuration.mapping_confirmed {
            p.line("Pin mapping confirmed for these sources");
            // Complete multi-gate Parts retain their full map labels.
            if let Some(part) = configuration
                .symbol_preview
                .native_part
                .as_ref()
                .filter(|_| configuration.placement_component().is_some())
            {
                match part.mapping_labels() {
                    Ok(rows) => p.line(&rows.join(" · ")),
                    Err(reason) => p.line(&reason),
                }
            } else {
                for (pad, pin) in &configuration.mappings {
                    if let Some(pad) = physical
                        .pads
                        .get(pad)
                        .filter(|_| unit.pins.contains_key(pin))
                    {
                        p.line(&format!(
                            "Pad {} → Pin {}",
                            pad.name,
                            configuration.pin_label(*pin)
                        ));
                    }
                }
            }
        } else {
            p.line("Select a pad, then its pin. Confirm the completed mapping.");
            let mut pads: Vec<_> = physical.pads.iter().collect();
            pads.sort_by_key(|(id, _)| **id);
            for (index, (id, pad)) in pads.into_iter().enumerate() {
                let mapped = configuration
                    .mappings
                    .get(id)
                    .and_then(|pin| unit.pins.get(pin));
                p.button(
                    &format!(
                        "{}Pad {} → {}",
                        if configuration.selected_pad == Some(*id) {
                            "Selected · "
                        } else {
                            ""
                        },
                        pad.name,
                        mapped.map_or_else(
                            || "unassigned".into(),
                            |pin| configuration.pin_label(pin.uuid)
                        )
                    ),
                    C::Configure(F::MappingPad(index)),
                );
            }
            if configuration.selected_pad.is_some() {
                let mut pins: Vec<_> = unit.pins.iter().collect();
                pins.sort_by_key(|(id, _)| **id);
                for (index, (_, pin)) in pins.into_iter().enumerate() {
                    p.button(
                        &format!("Connect to Pin {}", configuration.pin_label(pin.uuid)),
                        C::Configure(F::MappingPin(index)),
                    );
                }
            }
            p.button("Confirm mapping", C::Configure(F::ConfirmMapping));
        }
    }
    if configuration.pending() {
        p.line("Saving component… Wait for confirmation.");
    }
    if let Some(reason) = &configuration.refusal {
        p.line(reason);
    }
    small_line(p, "Pending edits are saved only by Apply or Place.");
}

fn chooser(p: &mut Painter<'_>) {
    let Some(a) = p.state.library.acquisition.as_ref() else {
        return;
    };
    p.line("Local footprint library");
    field(p, &a.path, a.path_editor, C::AcquirePath);
    p.button("Refresh source", C::AcquireLoad);
    p.line("Search footprints");
    field(p, &a.search, a.search_editor, C::AcquireSearch);
    let results = a.results();
    small_line(p, &format!("{} matching footprints", results.len()));
    for index in results.iter().skip(a.offset).take(4) {
        p.button(&a.sources[*index].display_label(), C::AcquireResult(*index));
    }
    if results.len() > 4 {
        p.button("Previous results", C::AcquirePage(false));
        p.button("Next results", C::AcquirePage(true));
    }
    if let Some(preview) = a.preview.as_ref().and_then(|p| p.footprint.as_ref()) {
        let rect = RectPx {
            x: p.rect.x + 8.,
            y: p.y,
            width: (p.rect.width - 16.).max(1.),
            height: 110.,
        };
        footprint::render(preview, rect, p.quads, p.text);
        p.y += 118.;
    }
    if a.loading {
        p.line("Reading footprint preview…");
    }
    if let Some(reason) = &a.refusal {
        p.line(reason);
    }
    small_line(
        p,
        "Choose returns a pending footprint to the component form. No edits are saved yet.",
    );
}

pub(crate) fn modal(
    state: &ReviewWorkspaceState,
    layout: &ShellLayout,
    quads: &mut Vec<Quad>,
    text: &mut Vec<TextRun>,
    hits: &mut Vec<HitRegion>,
) {
    let configuration = state.library.configuration.as_ref().unwrap();
    let choosing = configuration.footprint_chooser.is_some();
    let width = 720f32.min((layout.top_menu_bar.width - 32.).max(1.));
    let rect = RectPx {
        x: (layout.top_menu_bar.width - width) / 2.,
        y: 64.,
        width,
        height: (layout.status_bar.y - 80.).clamp(1., 590.),
    };
    quads.push(Quad::from_rect(rect, design_tokens::chrome::SURFACE_01));
    push_rect_border(quads, rect, design_tokens::chrome::BORDER_STRONG, 1.);
    hits.push(HitRegion {
        rect,
        target: HitTarget::SchematicPicker,
    });
    let mut p = Painter {
        rect,
        y: rect.y + 6.,
        unbounded: true,
        state,
        quads,
        text,
        hits,
    };
    p.line(if choosing {
        "Choose Footprint"
    } else {
        "Component Properties"
    });
    let footer = rect.y + rect.height - 54.;
    p.rect = RectPx {
        y: p.y,
        height: (footer - p.y - 6.).max(1.),
        ..rect
    };
    let starts = (p.quads.len(), p.text.len(), p.hits.len());
    if choosing {
        chooser(&mut p);
    } else {
        form(&mut p);
    }
    scroll_content(&mut p, starts, 0);
    p.rect = RectPx {
        width: width / 2.,
        ..rect
    };
    p.y = footer;
    if !configuration.pending() {
        p.button("Cancel", C::Configure(F::Cancel));
    }
    p.rect = RectPx {
        x: rect.x + width / 2.,
        width: width / 2.,
        ..rect
    };
    p.y = footer;
    if choosing {
        if state.library.acquisition.as_ref().is_some_and(|a| {
            !a.loading
                && a.request().is_ok()
                && a.preview.as_ref().is_some_and(|p| p.footprint.is_some())
        }) {
            p.button("Choose Footprint", C::AcquireImport);
        }
    } else if configuration.can_apply() {
        p.button("Apply", C::Configure(F::Apply));
    }
}
