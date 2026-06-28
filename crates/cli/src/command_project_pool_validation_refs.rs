use std::collections::BTreeSet;

use uuid::Uuid;

use super::{
    NativeProjectValidationIssueView, PoolValidationGraph, parse_uuid_field, parse_uuid_value,
    push_issue,
};

pub(super) fn validate_pool_refs(
    graph: &PoolValidationGraph,
    issues: &mut Vec<NativeProjectValidationIssueView>,
) {
    for (symbol_id, symbol) in &graph.symbols {
        let subject = graph.subjects.get(symbol_id).cloned().unwrap_or_default();
        validate_uuid_ref(
            symbol,
            "unit",
            &graph.units.keys().copied().collect(),
            &subject,
            "symbol",
            issues,
        );
    }
    for (entity_id, entity) in &graph.entities {
        let subject = graph.subjects.get(entity_id).cloned().unwrap_or_default();
        let Some(gates) = entity.get("gates").and_then(serde_json::Value::as_object) else {
            continue;
        };
        for (gate_key, gate) in gates {
            let gate_subject = format!("{subject}#gates/{gate_key}");
            validate_uuid_ref(
                gate,
                "unit",
                &graph.units.keys().copied().collect(),
                &gate_subject,
                "entity gate",
                issues,
            );
            validate_uuid_ref(
                gate,
                "symbol",
                &graph.symbols.keys().copied().collect(),
                &gate_subject,
                "entity gate",
                issues,
            );
        }
    }
    for (part_id, part) in &graph.parts {
        let subject = graph.subjects.get(part_id).cloned().unwrap_or_default();
        let package_id = parse_uuid_field(part, "package", &subject, "part", issues);
        validate_uuid_ref(
            part,
            "entity",
            &graph.entities.keys().copied().collect(),
            &subject,
            "part",
            issues,
        );
        validate_uuid_ref(
            part,
            "package",
            &graph.packages.keys().copied().collect(),
            &subject,
            "part",
            issues,
        );
        if let Some(pad_map) = part.get("pad_map").and_then(serde_json::Value::as_object) {
            for (map_key, entry) in pad_map {
                let map_subject = format!("{subject}#pad_map/{map_key}");
                if let Some(package_id) = package_id {
                    validate_part_pad_map_key(map_key, package_id, graph, &map_subject, issues);
                }
                validate_part_pad_map_entry(entry, part, graph, &map_subject, issues);
            }
        }
        validate_part_model_attachments(part, graph, &subject, issues);
    }
    for (package_id, package) in &graph.packages {
        let subject = graph.subjects.get(package_id).cloned().unwrap_or_default();
        let Some(pads) = package.get("pads").and_then(serde_json::Value::as_object) else {
            continue;
        };
        for (pad_key, pad) in pads {
            let pad_subject = format!("{subject}#pads/{pad_key}");
            validate_uuid_ref(
                pad,
                "padstack",
                &graph.padstacks.keys().copied().collect(),
                &pad_subject,
                "package pad",
                issues,
            );
        }
    }
    for (map_id, map) in &graph.pin_pad_maps {
        let subject = graph.subjects.get(map_id).cloned().unwrap_or_default();
        if let Some(part_id) = validate_uuid_ref(
            map,
            "part",
            &graph.parts.keys().copied().collect(),
            &subject,
            "pin_pad_map",
            issues,
        ) {
            validate_pin_pad_map_mappings(map, part_id, graph, &subject, issues);
        }
    }
}

fn validate_part_model_attachments(
    part: &serde_json::Value,
    graph: &PoolValidationGraph,
    subject: &str,
    issues: &mut Vec<NativeProjectValidationIssueView>,
) {
    let Some(models) = part
        .get("behavioural_models")
        .and_then(serde_json::Value::as_array)
    else {
        return;
    };
    for (index, attachment) in models.iter().enumerate() {
        let attachment_subject = format!("{subject}#behavioural_models/{index}");
        let Some(model_uuid) = parse_uuid_field(
            attachment,
            "model_uuid",
            &attachment_subject,
            "model attachment",
            issues,
        ) else {
            continue;
        };
        let Some(provenance) = attachment.get("provenance") else {
            continue;
        };
        if provenance.is_null() {
            continue;
        }
        let Some(sha256) = provenance.get("sha256").and_then(serde_json::Value::as_str) else {
            push_issue(
                issues,
                "error",
                "missing_required_field",
                attachment_subject,
                "model attachment provenance missing sha256",
            );
            continue;
        };
        let Some(blob) = graph.model_blobs.get(sha256) else {
            push_issue(
                issues,
                "error",
                "missing_model_blob",
                attachment_subject,
                format!("model attachment references missing pool model blob sha256 {sha256}"),
            );
            continue;
        };
        if model_uuid != blob.model_uuid {
            push_issue(
                issues,
                "error",
                "bad_deterministic_model_uuid",
                attachment_subject,
                "model attachment model_uuid does not match deterministic UUID",
            );
        }
    }
}

fn validate_part_pad_map_key(
    map_key: &str,
    package_id: Uuid,
    graph: &PoolValidationGraph,
    subject: &str,
    issues: &mut Vec<NativeProjectValidationIssueView>,
) {
    let Some(pad_id) = Uuid::parse_str(map_key).ok() else {
        push_issue(
            issues,
            "error",
            "invalid_uuid_key",
            subject.to_string(),
            format!("part pad_map key `{map_key}` is not a valid UUID"),
        );
        return;
    };
    let Some(package) = graph.packages.get(&package_id) else {
        return;
    };
    let Some(pads) = package.get("pads").and_then(serde_json::Value::as_object) else {
        return;
    };
    if !pads.contains_key(&pad_id.to_string()) {
        push_issue(
            issues,
            "error",
            "dangling_reference",
            subject.to_string(),
            format!("part pad_map key {pad_id} references missing package pad"),
        );
    }
}

fn validate_part_pad_map_entry(
    entry: &serde_json::Value,
    part: &serde_json::Value,
    graph: &PoolValidationGraph,
    subject: &str,
    issues: &mut Vec<NativeProjectValidationIssueView>,
) {
    let gate_id = validate_nested_uuid_field(entry, "gate", subject, "part pad_map", issues);
    let pin_id = validate_nested_uuid_field(entry, "pin", subject, "part pad_map", issues);
    let Some(entity_id) = parse_uuid_field(part, "entity", subject, "part", issues) else {
        return;
    };
    let Some(entity) = graph.entities.get(&entity_id) else {
        return;
    };
    let Some(gate_id) = gate_id else {
        return;
    };
    let Some(gates) = entity.get("gates").and_then(serde_json::Value::as_object) else {
        return;
    };
    let Some(gate) = gates.get(&gate_id.to_string()) else {
        push_issue(
            issues,
            "error",
            "dangling_reference",
            subject.to_string(),
            format!("part pad_map references missing entity gate {gate_id}"),
        );
        return;
    };
    let Some(unit_id) = parse_uuid_field(gate, "unit", subject, "entity gate", issues) else {
        return;
    };
    if graph.units.contains_key(&unit_id) {
        if let Some(pin_id) = pin_id {
            validate_unit_pin_exists(unit_id, pin_id, graph, subject, issues);
        }
    }
}

fn validate_pin_pad_map_mappings(
    map: &serde_json::Value,
    part_id: Uuid,
    graph: &PoolValidationGraph,
    subject: &str,
    issues: &mut Vec<NativeProjectValidationIssueView>,
) {
    let Some(part) = graph.parts.get(&part_id) else {
        return;
    };
    let Some(mappings) = map.get("mappings").and_then(serde_json::Value::as_object) else {
        return;
    };
    let package_id = parse_uuid_field(part, "package", subject, "pin_pad_map part", issues);
    let entity_id = parse_uuid_field(part, "entity", subject, "pin_pad_map part", issues);
    for (pin_key, pad_value) in mappings {
        let mapping_subject = format!("{subject}#mappings/{pin_key}");
        if let Some(pin_id) = Uuid::parse_str(pin_key).ok() {
            if let Some(entity_id) = entity_id {
                validate_entity_pin_exists(entity_id, pin_id, graph, &mapping_subject, issues);
            }
        } else {
            push_issue(
                issues,
                "error",
                "invalid_uuid_key",
                mapping_subject.clone(),
                format!("pin_pad_map mapping key `{pin_key}` is not a valid UUID"),
            );
        }
        let Some(pad_id) = parse_uuid_value(pad_value, &mapping_subject, "pad", issues) else {
            continue;
        };
        if let Some(package_id) = package_id {
            validate_package_pad_exists(package_id, pad_id, graph, &mapping_subject, issues);
        }
    }
}

fn validate_unit_pin_exists(
    unit_id: Uuid,
    pin_id: Uuid,
    graph: &PoolValidationGraph,
    subject: &str,
    issues: &mut Vec<NativeProjectValidationIssueView>,
) {
    let Some(unit) = graph.units.get(&unit_id) else {
        return;
    };
    let Some(pins) = unit.get("pins").and_then(serde_json::Value::as_object) else {
        return;
    };
    if !pins.contains_key(&pin_id.to_string()) {
        push_issue(
            issues,
            "error",
            "dangling_reference",
            subject.to_string(),
            format!("part pad_map references missing unit pin {pin_id}"),
        );
    }
}

fn validate_entity_pin_exists(
    entity_id: Uuid,
    pin_id: Uuid,
    graph: &PoolValidationGraph,
    subject: &str,
    issues: &mut Vec<NativeProjectValidationIssueView>,
) {
    let Some(entity) = graph.entities.get(&entity_id) else {
        return;
    };
    let Some(gates) = entity.get("gates").and_then(serde_json::Value::as_object) else {
        return;
    };
    for gate in gates.values() {
        let Some(unit_id) = gate
            .get("unit")
            .and_then(serde_json::Value::as_str)
            .and_then(|unit| Uuid::parse_str(unit).ok())
        else {
            continue;
        };
        let Some(unit) = graph.units.get(&unit_id) else {
            continue;
        };
        if unit
            .get("pins")
            .and_then(serde_json::Value::as_object)
            .is_some_and(|pins| pins.contains_key(&pin_id.to_string()))
        {
            return;
        }
    }
    push_issue(
        issues,
        "error",
        "dangling_reference",
        subject.to_string(),
        format!("pin_pad_map mapping references missing entity pin {pin_id}"),
    );
}

fn validate_package_pad_exists(
    package_id: Uuid,
    pad_id: Uuid,
    graph: &PoolValidationGraph,
    subject: &str,
    issues: &mut Vec<NativeProjectValidationIssueView>,
) {
    let Some(package) = graph.packages.get(&package_id) else {
        return;
    };
    let Some(pads) = package.get("pads").and_then(serde_json::Value::as_object) else {
        return;
    };
    if !pads.contains_key(&pad_id.to_string()) {
        push_issue(
            issues,
            "error",
            "dangling_reference",
            subject.to_string(),
            format!("pin_pad_map mapping references missing package pad {pad_id}"),
        );
    }
}

fn validate_uuid_ref(
    value: &serde_json::Value,
    field: &str,
    allowed: &BTreeSet<Uuid>,
    subject: &str,
    label: &str,
    issues: &mut Vec<NativeProjectValidationIssueView>,
) -> Option<Uuid> {
    let Some(reference) = value.get(field) else {
        return None;
    };
    let Some(reference) = parse_uuid_value(reference, subject, field, issues) else {
        return None;
    };
    if !allowed.contains(&reference) {
        push_issue(
            issues,
            "error",
            "dangling_reference",
            subject.to_string(),
            format!("{label} references missing {field} {reference}"),
        );
        return None;
    }
    Some(reference)
}

fn validate_nested_uuid_field(
    value: &serde_json::Value,
    field: &str,
    subject: &str,
    label: &str,
    issues: &mut Vec<NativeProjectValidationIssueView>,
) -> Option<Uuid> {
    if let Some(reference) = value.get(field) {
        parse_uuid_value(reference, subject, field, issues)
    } else {
        push_issue(
            issues,
            "error",
            "missing_required_field",
            subject.to_string(),
            format!("{label} missing {field}"),
        );
        None
    }
}
