use uuid::Uuid;

use super::library_graph::{
    LibraryGraph, LibraryGraphDiagnostic, LibraryGraphLegacyPinPadMapMigrationReport,
};

impl LibraryGraph {
    pub fn legacy_pin_pad_map_migration_report(
        &self,
    ) -> LibraryGraphLegacyPinPadMapMigrationReport {
        let mut report = LibraryGraphLegacyPinPadMapMigrationReport::default();
        for (part_id, part) in &self.parts {
            let Some(pad_map) = part.get("pad_map").and_then(serde_json::Value::as_object) else {
                continue;
            };
            if pad_map.is_empty() {
                continue;
            }
            report.parts += 1;
            let subject = self
                .subjects
                .get(part_id)
                .cloned()
                .unwrap_or_else(|| part_id.to_string());
            let default_map_is_authoritative = part
                .get("default_pin_pad_map")
                .and_then(serde_json::Value::as_str)
                .and_then(|raw| Uuid::parse_str(raw).ok())
                .and_then(|map_id| self.pin_pad_maps.get(&map_id))
                .and_then(|map| map.get("part").and_then(serde_json::Value::as_str))
                .and_then(|raw| Uuid::parse_str(raw).ok())
                == Some(*part_id);
            let package_id = part
                .get("package")
                .and_then(serde_json::Value::as_str)
                .and_then(|raw| Uuid::parse_str(raw).ok());
            let footprint_id = part
                .get("default_footprint")
                .and_then(serde_json::Value::as_str)
                .and_then(|raw| Uuid::parse_str(raw).ok());
            for (pad_key, entry) in pad_map {
                report.rows += 1;
                let row_subject = format!("{subject}#pad_map/{pad_key}");
                if default_map_is_authoritative {
                    report.shadowed_rows += 1;
                    report.shadowed_subjects.push(row_subject);
                    continue;
                }
                if legacy_part_pad_map_row_is_migratable(
                    self,
                    package_id,
                    footprint_id,
                    pad_key,
                    entry,
                ) {
                    report.migratable_rows += 1;
                    report.migratable_subjects.push(row_subject);
                } else {
                    report.blocked_rows += 1;
                    report.blocked_subjects.push(row_subject);
                }
            }
        }
        report
    }

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

impl LibraryGraphLegacyPinPadMapMigrationReport {
    pub fn diagnostics(&self) -> Vec<LibraryGraphDiagnostic> {
        let mut diagnostics = Vec::new();
        if self.migratable_rows > 0 {
            diagnostics.push(LibraryGraphDiagnostic {
                severity: "info",
                code: "legacy_part_pad_map_migratable",
                subject: "pool/parts#pad_map".to_string(),
                message: format!(
                    "{} legacy Part.pad_map row(s) can seed first-class PinPadMap records",
                    self.migratable_rows
                ),
            });
        }
        if self.shadowed_rows > 0 {
            diagnostics.push(LibraryGraphDiagnostic {
                severity: "info",
                code: "legacy_part_pad_map_shadowed",
                subject: "pool/parts#pad_map".to_string(),
                message: format!(
                    "{} legacy Part.pad_map row(s) are ignored because Part.default_pin_pad_map already names an authoritative first-class PinPadMap",
                    self.shadowed_rows
                ),
            });
        }
        if self.blocked_rows > 0 {
            for subject in &self.blocked_subjects {
                diagnostics.push(LibraryGraphDiagnostic {
                    severity: "warning",
                    code: "legacy_part_pad_map_migration_blocked",
                    subject: subject.clone(),
                    message: "legacy Part.pad_map row cannot be migrated without resolving malformed references or footprint-pad ambiguity".to_string(),
                });
            }
        }
        diagnostics
    }
}

fn legacy_part_pad_map_row_is_migratable(
    graph: &LibraryGraph,
    package_id: Option<Uuid>,
    footprint_id: Option<Uuid>,
    pad_key: &str,
    entry: &serde_json::Value,
) -> bool {
    let Some(pad_id) = Uuid::parse_str(pad_key).ok() else {
        return false;
    };
    if !entry.get("gate").is_some_and(serde_json::Value::is_string)
        || !entry.get("pin").is_some_and(serde_json::Value::is_string)
        || entry
            .get("gate")
            .and_then(serde_json::Value::as_str)
            .and_then(|raw| Uuid::parse_str(raw).ok())
            .is_none()
        || entry
            .get("pin")
            .and_then(serde_json::Value::as_str)
            .and_then(|raw| Uuid::parse_str(raw).ok())
            .is_none()
    {
        return false;
    }
    if let Some(footprint_id) = footprint_id {
        return legacy_part_pad_map_pad_resolves_to_footprint(
            graph,
            package_id,
            footprint_id,
            pad_id,
        );
    }
    package_id.is_some_and(|package_id| {
        graph
            .packages
            .get(&package_id)
            .and_then(|package| package.get("pads"))
            .and_then(serde_json::Value::as_object)
            .is_some_and(|pads| pads.contains_key(&pad_id.to_string()))
    })
}

fn legacy_part_pad_map_pad_resolves_to_footprint(
    graph: &LibraryGraph,
    package_id: Option<Uuid>,
    footprint_id: Uuid,
    pad_id: Uuid,
) -> bool {
    let Some(footprint_pads) = graph
        .footprints
        .get(&footprint_id)
        .and_then(|footprint| footprint.get("pads"))
        .and_then(serde_json::Value::as_object)
    else {
        return false;
    };
    if footprint_pads.contains_key(&pad_id.to_string()) {
        return true;
    }
    let Some(package_pads) = package_id
        .and_then(|package_id| graph.packages.get(&package_id))
        .and_then(|package| package.get("pads"))
        .and_then(serde_json::Value::as_object)
    else {
        return false;
    };
    let Some(package_pad_name) = package_pads
        .get(&pad_id.to_string())
        .and_then(|pad| pad.get("name"))
        .and_then(serde_json::Value::as_str)
    else {
        return false;
    };
    footprint_pads
        .values()
        .filter(|pad| pad.get("name").and_then(serde_json::Value::as_str) == Some(package_pad_name))
        .take(2)
        .count()
        == 1
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
