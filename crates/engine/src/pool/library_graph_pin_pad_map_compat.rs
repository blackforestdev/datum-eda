use uuid::Uuid;

use super::library_graph::{LibraryGraph, LibraryGraphDiagnostic};

impl LibraryGraph {
    pub(super) fn resolve_pin_pad_map_mapping(
        &self,
        mapping_key: &str,
        entry: &serde_json::Value,
        entity_id: Option<Uuid>,
        subject: &str,
        diagnostics: &mut Vec<LibraryGraphDiagnostic>,
    ) -> Option<(Uuid, Uuid, Option<Uuid>)> {
        if entry.get("gate").is_some() || entry.get("pin").is_some() {
            let Some(pad_id) = parse_uuid_key(mapping_key, subject, diagnostics) else {
                return None;
            };
            let gate_id =
                parse_uuid_field(entry, "gate", subject, "pin_pad_map mapping", diagnostics)?;
            let pin_id =
                parse_uuid_field(entry, "pin", subject, "pin_pad_map mapping", diagnostics);
            return Some((pad_id, gate_id, pin_id));
        }

        let Some(pin_id) = parse_uuid_key(mapping_key, subject, diagnostics) else {
            return None;
        };
        let pad_id = parse_legacy_pin_pad_map_pad(entry, subject, diagnostics)?;
        let entity_id = entity_id?;
        let gate_id =
            self.infer_legacy_pin_pad_map_gate(entity_id, pin_id, subject, diagnostics)?;
        Some((pad_id, gate_id, Some(pin_id)))
    }

    fn infer_legacy_pin_pad_map_gate(
        &self,
        entity_id: Uuid,
        pin_id: Uuid,
        subject: &str,
        diagnostics: &mut Vec<LibraryGraphDiagnostic>,
    ) -> Option<Uuid> {
        let Some(entity) = self.entities.get(&entity_id) else {
            return None;
        };
        let Some(gates) = entity.get("gates").and_then(serde_json::Value::as_object) else {
            return None;
        };
        let mut matches = Vec::new();
        for (gate_key, gate) in gates {
            let Some(gate_id) = Uuid::parse_str(gate_key).ok() else {
                continue;
            };
            let Some(unit_id) = gate
                .get("unit")
                .and_then(serde_json::Value::as_str)
                .and_then(|unit| Uuid::parse_str(unit).ok())
            else {
                continue;
            };
            if self
                .units
                .get(&unit_id)
                .and_then(|unit| unit.get("pins"))
                .and_then(serde_json::Value::as_object)
                .is_some_and(|pins| pins.contains_key(&pin_id.to_string()))
            {
                matches.push(gate_id);
            }
        }
        match matches.as_slice() {
            [gate_id] => Some(*gate_id),
            [] => {
                diagnostics.push(LibraryGraphDiagnostic {
                    severity: "error",
                    code: "dangling_reference",
                    subject: subject.to_string(),
                    message: format!(
                        "legacy pin_pad_map mapping references missing entity pin {pin_id}"
                    ),
                });
                None
            }
            _ => {
                diagnostics.push(LibraryGraphDiagnostic {
                    severity: "error",
                    code: "ambiguous_legacy_pin_pad_map",
                    subject: subject.to_string(),
                    message: format!(
                        "legacy pin_pad_map mapping for pin {pin_id} matches multiple entity gates; use canonical pad -> {{gate, pin}} rows"
                    ),
                });
                None
            }
        }
    }
}

fn parse_uuid_key(
    mapping_key: &str,
    subject: &str,
    diagnostics: &mut Vec<LibraryGraphDiagnostic>,
) -> Option<Uuid> {
    let Some(uuid) = Uuid::parse_str(mapping_key).ok() else {
        diagnostics.push(LibraryGraphDiagnostic {
            severity: "error",
            code: "invalid_uuid_key",
            subject: subject.to_string(),
            message: format!("pin_pad_map mapping key `{mapping_key}` is not a valid UUID"),
        });
        return None;
    };
    Some(uuid)
}

fn parse_legacy_pin_pad_map_pad(
    entry: &serde_json::Value,
    subject: &str,
    diagnostics: &mut Vec<LibraryGraphDiagnostic>,
) -> Option<Uuid> {
    let raw = if let Some(raw) = entry.as_str() {
        raw
    } else if let Some(raw) = entry.get("pad").and_then(serde_json::Value::as_str) {
        raw
    } else {
        diagnostics.push(LibraryGraphDiagnostic {
            severity: "error",
            code: "missing_required_field",
            subject: subject.to_string(),
            message: "legacy pin_pad_map mapping missing pad UUID".to_string(),
        });
        return None;
    };
    match Uuid::parse_str(raw) {
        Ok(pad_id) => Some(pad_id),
        Err(_) => {
            diagnostics.push(LibraryGraphDiagnostic {
                severity: "error",
                code: "invalid_uuid",
                subject: subject.to_string(),
                message: format!("legacy pin_pad_map mapping pad `{raw}` is not a valid UUID"),
            });
            None
        }
    }
}

fn parse_uuid_field(
    value: &serde_json::Value,
    field: &'static str,
    subject: &str,
    label: &'static str,
    diagnostics: &mut Vec<LibraryGraphDiagnostic>,
) -> Option<Uuid> {
    match value.get(field).and_then(serde_json::Value::as_str) {
        Some(raw) => match Uuid::parse_str(raw) {
            Ok(uuid) => Some(uuid),
            Err(_) => {
                diagnostics.push(LibraryGraphDiagnostic {
                    severity: "error",
                    code: "invalid_uuid",
                    subject: subject.to_string(),
                    message: format!("{label} field {field} is not a valid UUID: {raw}"),
                });
                None
            }
        },
        None => {
            diagnostics.push(LibraryGraphDiagnostic {
                severity: "error",
                code: "missing_required_field",
                subject: subject.to_string(),
                message: format!("{label} missing required field {field}"),
            });
            None
        }
    }
}
