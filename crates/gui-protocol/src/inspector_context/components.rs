//! Explicit placed references and captured library facts remain separate from readiness.
use super::*;
use crate::AuthoredSelectionClass as C;
use crate::library_browser::LibraryEntry;

pub(super) fn append<'a>(context: &mut InspectorContext<'a>, state: &'a ReviewWorkspaceState) {
    let Some(view) = context.view else {
        return;
    };
    let mut members = view.scoped_members();
    let Some(id) = members.next() else {
        return;
    };
    if members.next().is_some() || !matches!(id.class, C::Symbol | C::Footprint) {
        return;
    }
    let Some(values) = view.values.get(id) else {
        return;
    };
    let symbol = values
        .get("library_symbol")
        .and_then(serde_json::Value::as_str)
        .and_then(|s| s.parse::<Uuid>().ok());
    let part = values
        .get("library_part")
        .and_then(serde_json::Value::as_str)
        .and_then(|s| s.parse::<Uuid>().ok());
    if let Some(part) = part {
        let candidates: Vec<_> = state
            .library
            .entries
            .iter()
            .filter(|e| e.kind == "parts" && e.id == part)
            .collect();
        let label = if candidates.len() == 1 && candidates[0].warning.is_none() {
            format!(
                "Resolved source reference — {} ({part})",
                candidates[0].name
            )
        } else if candidates.len() > 1 {
            format!(
                "Ambiguous source reference — {part}; {} candidate records",
                candidates.len()
            )
        } else {
            format!("Unresolved source reference — {part}")
        };
        context.push(
            InspectorSectionId::LibraryInfo,
            InspectorField::text("Placed Part", label),
        );
    } else {
        context.push(
            InspectorSectionId::LibraryInfo,
            InspectorField::text("Placed Part", "Unresolved — no explicit Part reference"),
        );
    }
    if id.class == C::Footprint {
        footprint(context, state, values);
        return;
    }
    let candidates: Vec<_> = state
        .library
        .entries
        .iter()
        .filter(|e| e.kind == "symbols" && Some(e.id) == symbol)
        .collect();
    if candidates.len() != 1 || candidates[0].warning.is_some() {
        context.push(
            InspectorSectionId::LibraryInfo,
            InspectorField::text(
                "Symbol",
                if candidates.len() > 1 {
                    "Ambiguous — multiple captured Symbol sources"
                } else {
                    "Unavailable — captured Symbol source is unresolved"
                },
            ),
        );
        context.push(
            InspectorSectionId::Readiness,
            InspectorField::text("Place Symbol", "Unavailable — source binding is unresolved"),
        );
        return;
    }
    let entry = candidates[0];
    library_fields(context, entry);
    let unit = entry
        .raw
        .get("unit")
        .and_then(serde_json::Value::as_str)
        .and_then(|s| s.parse::<Uuid>().ok());
    let units: Vec<_> = state
        .library
        .entries
        .iter()
        .filter(|e| e.kind == "units" && Some(e.id) == unit)
        .collect();
    if units.len() == 1 {
        let pins = units[0]
            .raw
            .get("pins")
            .and_then(serde_json::Value::as_object)
            .map(|pins| pins.len());
        context.push(
            InspectorSectionId::LibraryInfo,
            InspectorField::text("Unit", format!("{} ({})", units[0].name, units[0].id)),
        );
        context.push(
            InspectorSectionId::LibraryInfo,
            InspectorField::text(
                "Unit pins",
                pins.map_or_else(
                    || "Unavailable — Unit pins are not captured".into(),
                    |n| n.to_string(),
                ),
            ),
        );
    } else {
        context.push(
            InspectorSectionId::LibraryInfo,
            InspectorField::text(
                "Unit",
                "Unavailable — Unit reference is missing or ambiguous",
            ),
        );
    }
    if let Some(choice) = state
        .schematic_authoring
        .choices
        .iter()
        .find(|choice| Some(choice.id) == symbol)
    {
        context.push(
            InspectorSectionId::Readiness,
            InspectorField::text("Command binding", choice.binding_status.as_str()),
        );
        context.push(
            InspectorSectionId::Readiness,
            InspectorField::text(
                "Place Symbol",
                choice
                    .refusal
                    .as_deref()
                    .unwrap_or("Ready on captured binding basis"),
            ),
        );
    } else {
        context.push(
            InspectorSectionId::Readiness,
            InspectorField::text(
                "Place Symbol",
                "Unavailable — no captured command binding result",
            ),
        );
    }
}

fn footprint<'a>(
    context: &mut InspectorContext<'a>,
    state: &'a ReviewWorkspaceState,
    values: &'a std::collections::BTreeMap<String, serde_json::Value>,
) {
    let metadata = values.get("library_land_pattern_sources");
    let sources = metadata
        .and_then(|v| v.get("sources"))
        .and_then(serde_json::Value::as_array);
    let complete = metadata.is_some_and(|v| {
        v["pad_count"]
            .as_u64()
            .is_some_and(|n| n > 0 && Some(n) == v["source_pad_count"].as_u64())
    });
    if !complete
        || values
            .get("pad_materialization_status")
            .and_then(serde_json::Value::as_str)
            != Some("current")
    {
        context.push(
            InspectorSectionId::LibraryInfo,
            InspectorField::text(
                "Footprint source",
                "Unavailable — materialized pad source binding is missing, stale or unverified",
            ),
        );
        return;
    }
    let Some(sources) = sources.filter(|sources| sources.len() == 1) else {
        context.push(
            InspectorSectionId::LibraryInfo,
            InspectorField::text(
                "Footprint source",
                "Ambiguous — materialized pads name multiple land-pattern sources",
            ),
        );
        return;
    };
    let source = &sources[0];
    let id = source["object_id"]
        .as_str()
        .and_then(|s| s.parse::<Uuid>().ok());
    let entries: Vec<_> = state
        .library
        .entries
        .iter()
        .filter(|entry| {
            entry.kind == "footprints"
                && Some(entry.id) == id
                && Some(entry.source_path.as_str()) == source["relative_path"].as_str()
                && entry.revision == source["object_revision"].as_u64()
                && entry.warning.is_none()
        })
        .collect();
    if entries.len() == 1 {
        library_fields(context, entries[0]);
        context.push(InspectorSectionId::LibraryInfo, InspectorField::text("Binding basis", "Complete materialized pad land-pattern pins; exact captured pool path and revision"));
    } else {
        context.push(
            InspectorSectionId::LibraryInfo,
            InspectorField::text(
                "Footprint source",
                "Unavailable — pinned land pattern has no unique matching captured pool source",
            ),
        );
    }
}

pub(super) fn library_fields<'a>(context: &mut InspectorContext<'a>, entry: &'a LibraryEntry) {
    let section = InspectorSectionId::LibraryInfo;
    context.push(
        section,
        InspectorField::text("Library object", entry.name.as_str()),
    );
    context.push(
        section,
        InspectorField::text("Object UUID", entry.id.to_string()),
    );
    context.push(
        section,
        InspectorField::text("Origin", entry.source_path.as_str()),
    );
    context.push(
        section,
        InspectorField::text(
            "Source revision",
            entry
                .revision
                .map_or_else(|| "Unavailable".into(), |r| r.to_string()),
        ),
    );
    for (name, key) in [
        ("Approval", "approval"),
        ("Standards basis", "standards_basis"),
        ("Check state", "check_state"),
    ] {
        context.push(
            InspectorSectionId::Readiness,
            if let Some(value) = entry.raw.get(key).filter(|value| !value.is_null()) {
                InspectorField {
                    label: name.into(),
                    value: InspectorValue::Json(value),
                }
            } else {
                InspectorField::text(name, "Not recorded")
            },
        );
    }
}

#[cfg(test)]
#[path = "components_tests.rs"]
mod tests;
