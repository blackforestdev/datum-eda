//! Logical gate rows and explicit named reference review for new Entities.
use super::*;
use datum_gui_protocol::library_authoring::LibraryDraft;

fn named(p: &Painter<'_>, kind: &str, id: &Value) -> String {
    let candidates: Vec<_> = p
        .state
        .library
        .entries
        .iter()
        .filter(|e| e.kind == kind && id.as_str() == Some(e.id.to_string().as_str()))
        .collect();
    if candidates.len() == 1 && candidates[0].warning.is_none() {
        candidates[0].name.clone()
    } else {
        "Unavailable / ambiguous".into()
    }
}
fn gate_label(p: &Painter<'_>, gate: &Value) -> String {
    format!(
        "{}  |  {}  |  {}",
        gate["name"].as_str().unwrap_or("Name required"),
        named(p, "units", &gate["unit"]),
        named(p, "symbols", &gate["symbol"])
    )
}
fn row(p: &mut Painter<'_>, labels: [&str; 4], control: Option<C>, active: bool) {
    let width = (p.rect.width - 16.).max(1.);
    let cell_width = width / 4.;
    let height = labels
        .iter()
        .map(|label| {
            crate::text_metrics::measured_text_run_height_px(
                label,
                (cell_width - 10.).max(1.),
                12.,
                TextFace::Ui,
            )
            .unwrap_or(16.)
                + 12.
        })
        .fold(28., f32::max);
    let rect = RectPx {
        x: p.rect.x + 8.,
        y: p.y,
        width,
        height,
    };
    if control.is_some() {
        p.quads
            .push(Quad::from_rect(rect, design_tokens::chrome::SURFACE_02));
    }
    if active
        || control
            .as_ref()
            .is_some_and(|c| p.state.library.focus.as_ref() == Some(c))
    {
        push_rect_border(p.quads, rect, TEXT_ACCENT, 1.);
    }
    for (column, label) in labels.into_iter().enumerate() {
        let cell = RectPx {
            x: rect.x + column as f32 * cell_width,
            width: cell_width,
            ..rect
        };
        draw_text_clipped(
            label,
            cell.x + 5.,
            cell.y + 6.,
            12.,
            TEXT_PRIMARY,
            if control.is_none() {
                TextFace::UiStrong
            } else {
                TextFace::Ui
            },
            cell,
            p.text,
        );
        p.text.last_mut().unwrap().layout_size = Some(((cell_width - 10.).max(1.), height - 12.));
    }
    if let Some(control) = control {
        p.hits.push(HitRegion {
            target: HitTarget::Library(control),
            rect,
        });
    }
    p.y += height + 4.;
}
fn gate_row(p: &mut Painter<'_>, d: &LibraryDraft, gate: &Value, control: C, active: bool) {
    let unit = named(p, "units", &gate["unit"]);
    let symbol = named(p, "symbols", &gate["symbol"]);
    row(
        p,
        [
            gate["name"]
                .as_str()
                .filter(|s| !s.is_empty())
                .unwrap_or("Name required"),
            &unit,
            &symbol,
            if d.preview.is_some() {
                "Validated preview"
            } else {
                "Preview required"
            },
        ],
        Some(control),
        active,
    );
}
pub(super) fn table(p: &mut Painter<'_>, d: &LibraryDraft) {
    row(
        p,
        ["Gate", "Unit", "Symbol", "Reference state"],
        None,
        false,
    );
    for (i, (id, raw)) in d.entity.gates.iter().enumerate() {
        let active = d.entity.gate_id.is_some_and(|v| v.to_string() == *id);
        let gate = if active { pending_gate(d) } else { raw.clone() };
        gate_row(p, d, &gate, C::Gate(i), active);
    }
    if d.entity
        .gate_id
        .is_some_and(|id| !d.entity.gates.contains_key(&id.to_string()))
    {
        let i = d.fields.iter().position(|f| f.key == "gate_name").unwrap();
        gate_row(p, d, &pending_gate(d), C::Field(i), true);
    }
    p.button("Add gate", C::AddGate);
    p.button("Remove draft gate", C::RemoveGate);
}
fn pending_gate(d: &LibraryDraft) -> Value {
    let mut gate = Value::Object(Default::default());
    for (key, field) in [
        ("name", "gate_name"),
        ("unit", "gate_unit"),
        ("symbol", "gate_symbol"),
    ] {
        gate.as_object_mut()
            .unwrap()
            .insert(key.into(), Value::String(d.value(field).into()));
    }
    gate
}
pub(super) fn review(p: &mut Painter<'_>, raw: &Value) {
    p.line(&format!(
        "Review: create Entity {}",
        raw["name"].as_str().unwrap_or("Unavailable")
    ));
    p.line(&format!(
        "Reference prefix: {} · Manufacturer: {}",
        raw["prefix"].as_str().unwrap_or(""),
        raw["manufacturer"].as_str().unwrap_or("")
    ));
    let gates = raw.get("gates").and_then(Value::as_object);
    p.line(&format!(
        "{} logical gates · explicit Unit / Symbol links",
        gates.map_or(0, |g| g.len())
    ));
    for gate in gates.into_iter().flat_map(|g| g.values()) {
        p.line(&gate_label(p, gate));
    }
    p.line(&format!("Tags: {}", raw["tags"]));
}
