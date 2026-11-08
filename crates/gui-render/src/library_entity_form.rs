//! Logical gate rows and explicit named reference review for new Entities.
use super::definition_table::row;
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
