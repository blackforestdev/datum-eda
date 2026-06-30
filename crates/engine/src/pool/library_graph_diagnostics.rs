use std::collections::BTreeSet;

use uuid::Uuid;

use super::library_graph::{LibraryGraph, LibraryGraphDiagnostic};

impl LibraryGraph {
    pub(super) fn validate_symbol_refs(&self, diagnostics: &mut Vec<LibraryGraphDiagnostic>) {
        let unit_ids = self.units.keys().copied().collect();
        for (symbol_id, symbol) in &self.symbols {
            let subject = self.subject(symbol_id);
            self.validate_uuid_ref(symbol, "unit", &unit_ids, &subject, "symbol", diagnostics);
        }
    }

    pub(super) fn validate_entity_refs(&self, diagnostics: &mut Vec<LibraryGraphDiagnostic>) {
        let unit_ids = self.units.keys().copied().collect();
        let symbol_ids = self.symbols.keys().copied().collect();
        for (entity_id, entity) in &self.entities {
            let subject = self.subject(entity_id);
            let Some(gates) = entity.get("gates").and_then(serde_json::Value::as_object) else {
                continue;
            };
            for (gate_key, gate) in gates {
                let gate_subject = format!("{subject}#gates/{gate_key}");
                self.validate_uuid_ref(
                    gate,
                    "unit",
                    &unit_ids,
                    &gate_subject,
                    "entity gate",
                    diagnostics,
                );
                self.validate_uuid_ref(
                    gate,
                    "symbol",
                    &symbol_ids,
                    &gate_subject,
                    "entity gate",
                    diagnostics,
                );
            }
        }
    }

    pub(super) fn validate_part_refs(&self, diagnostics: &mut Vec<LibraryGraphDiagnostic>) {
        let entity_ids = self.entities.keys().copied().collect();
        let package_ids = self.packages.keys().copied().collect();
        let footprint_ids = self.footprints.keys().copied().collect();
        let pin_pad_map_ids = self.pin_pad_maps.keys().copied().collect();
        for (part_id, part) in &self.parts {
            let subject = self.subject(part_id);
            let package_id = self.parse_uuid_field(part, "package", &subject, "part", diagnostics);
            let default_footprint_id = self.validate_optional_uuid_ref(
                part,
                "default_footprint",
                &footprint_ids,
                &subject,
                "part",
                diagnostics,
            );
            let default_pin_pad_map_id = self.validate_optional_uuid_ref(
                part,
                "default_pin_pad_map",
                &pin_pad_map_ids,
                &subject,
                "part",
                diagnostics,
            );
            self.validate_uuid_ref(part, "entity", &entity_ids, &subject, "part", diagnostics);
            self.validate_uuid_ref(part, "package", &package_ids, &subject, "part", diagnostics);
            if let Some(pad_map) = part.get("pad_map").and_then(serde_json::Value::as_object) {
                for (map_key, entry) in pad_map {
                    let map_subject = format!("{subject}#pad_map/{map_key}");
                    if let Some(package_id) = package_id {
                        self.validate_part_pad_map_key_against_package(
                            map_key,
                            package_id,
                            &map_subject,
                            diagnostics,
                        );
                    }
                    self.validate_part_pad_map_entry(entry, part, &map_subject, diagnostics);
                }
            }
            if let (Some(package_id), Some(default_footprint_id)) =
                (package_id, default_footprint_id)
            {
                self.validate_part_default_footprint_package(
                    default_footprint_id,
                    package_id,
                    &subject,
                    diagnostics,
                );
            }
            if let Some(default_pin_pad_map_id) = default_pin_pad_map_id {
                self.validate_part_default_pin_pad_map(
                    default_pin_pad_map_id,
                    *part_id,
                    default_footprint_id,
                    package_id,
                    &subject,
                    diagnostics,
                );
            }
            self.validate_part_model_attachments(part, &subject, diagnostics);
        }
    }

    pub(super) fn validate_package_refs(&self, diagnostics: &mut Vec<LibraryGraphDiagnostic>) {
        let padstack_ids = self.padstacks.keys().copied().collect();
        for (package_id, package) in &self.packages {
            let subject = self.subject(package_id);
            let Some(pads) = package.get("pads").and_then(serde_json::Value::as_object) else {
                continue;
            };
            for (pad_key, pad) in pads {
                let pad_subject = format!("{subject}#pads/{pad_key}");
                self.validate_uuid_ref(
                    pad,
                    "padstack",
                    &padstack_ids,
                    &pad_subject,
                    "package pad",
                    diagnostics,
                );
            }
        }
    }

    pub(super) fn validate_footprint_refs(&self, diagnostics: &mut Vec<LibraryGraphDiagnostic>) {
        let package_ids = self.packages.keys().copied().collect();
        let padstack_ids = self.padstacks.keys().copied().collect();
        for (footprint_id, footprint) in &self.footprints {
            let subject = self.subject(footprint_id);
            self.validate_uuid_ref(
                footprint,
                "package",
                &package_ids,
                &subject,
                "footprint",
                diagnostics,
            );
            let Some(pads) = footprint.get("pads").and_then(serde_json::Value::as_object) else {
                continue;
            };
            for (pad_key, pad) in pads {
                let pad_subject = format!("{subject}#pads/{pad_key}");
                self.validate_uuid_ref(
                    pad,
                    "padstack",
                    &padstack_ids,
                    &pad_subject,
                    "footprint pad",
                    diagnostics,
                );
            }
        }
    }

    pub(super) fn validate_pin_pad_map_refs(&self, diagnostics: &mut Vec<LibraryGraphDiagnostic>) {
        let part_ids = self.parts.keys().copied().collect();
        for (map_id, map) in &self.pin_pad_maps {
            let subject = self.subject(map_id);
            if let Some(part_id) =
                self.validate_uuid_ref(map, "part", &part_ids, &subject, "pin_pad_map", diagnostics)
            {
                self.validate_pin_pad_map_mappings(map, part_id, &subject, diagnostics);
            }
        }
    }

    fn validate_part_model_attachments(
        &self,
        part: &serde_json::Value,
        subject: &str,
        diagnostics: &mut Vec<LibraryGraphDiagnostic>,
    ) {
        let Some(models) = part
            .get("behavioural_models")
            .and_then(serde_json::Value::as_array)
        else {
            return;
        };
        for (index, attachment) in models.iter().enumerate() {
            let attachment_subject = format!("{subject}#behavioural_models/{index}");
            let Some(model_uuid) = self.parse_uuid_field(
                attachment,
                "model_uuid",
                &attachment_subject,
                "model attachment",
                diagnostics,
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
                diagnostics.push(LibraryGraphDiagnostic {
                    severity: "error",
                    code: "missing_required_field",
                    subject: attachment_subject,
                    message: "model attachment provenance missing sha256".to_string(),
                });
                continue;
            };
            let Some(blob) = self.model_blobs.get(sha256) else {
                diagnostics.push(LibraryGraphDiagnostic {
                    severity: "error",
                    code: "missing_model_blob",
                    subject: attachment_subject,
                    message: format!(
                        "model attachment references missing pool model blob sha256 {sha256}"
                    ),
                });
                continue;
            };
            if model_uuid != blob.model_uuid {
                diagnostics.push(LibraryGraphDiagnostic {
                    severity: "error",
                    code: "bad_deterministic_model_uuid",
                    subject: attachment_subject,
                    message: "model attachment model_uuid does not match deterministic UUID"
                        .to_string(),
                });
            }
        }
    }

    fn validate_part_pad_map_key_against_package(
        &self,
        map_key: &str,
        package_id: Uuid,
        subject: &str,
        diagnostics: &mut Vec<LibraryGraphDiagnostic>,
    ) {
        let Some(pad_id) = Uuid::parse_str(map_key).ok() else {
            diagnostics.push(LibraryGraphDiagnostic {
                severity: "error",
                code: "invalid_uuid_key",
                subject: subject.to_string(),
                message: format!("part pad_map key `{map_key}` is not a valid UUID"),
            });
            return;
        };
        let Some(package) = self.packages.get(&package_id) else {
            return;
        };
        let Some(pads) = package.get("pads").and_then(serde_json::Value::as_object) else {
            return;
        };
        if !pads.contains_key(&pad_id.to_string()) {
            diagnostics.push(LibraryGraphDiagnostic {
                severity: "error",
                code: "dangling_reference",
                subject: subject.to_string(),
                message: format!("part pad_map key {pad_id} references missing package pad"),
            });
        }
    }

    fn validate_part_pad_map_entry(
        &self,
        entry: &serde_json::Value,
        part: &serde_json::Value,
        subject: &str,
        diagnostics: &mut Vec<LibraryGraphDiagnostic>,
    ) {
        let gate_id =
            self.validate_nested_uuid_field(entry, "gate", subject, "part pad_map", diagnostics);
        let pin_id =
            self.validate_nested_uuid_field(entry, "pin", subject, "part pad_map", diagnostics);
        let Some(entity_id) = self.parse_uuid_field(part, "entity", subject, "part", diagnostics)
        else {
            return;
        };
        let Some(entity) = self.entities.get(&entity_id) else {
            return;
        };
        let Some(gate_id) = gate_id else {
            return;
        };
        let Some(gates) = entity.get("gates").and_then(serde_json::Value::as_object) else {
            return;
        };
        let Some(gate) = gates.get(&gate_id.to_string()) else {
            diagnostics.push(LibraryGraphDiagnostic {
                severity: "error",
                code: "dangling_reference",
                subject: subject.to_string(),
                message: format!("part pad_map references missing entity gate {gate_id}"),
            });
            return;
        };
        let Some(unit_id) =
            self.parse_uuid_field(gate, "unit", subject, "entity gate", diagnostics)
        else {
            return;
        };
        if self.units.contains_key(&unit_id)
            && let Some(pin_id) = pin_id
        {
            self.validate_unit_pin_exists(unit_id, pin_id, subject, diagnostics);
        }
    }

    fn validate_pin_pad_map_mappings(
        &self,
        map: &serde_json::Value,
        part_id: Uuid,
        subject: &str,
        diagnostics: &mut Vec<LibraryGraphDiagnostic>,
    ) {
        let Some(part) = self.parts.get(&part_id) else {
            return;
        };
        let Some(mappings) = map.get("mappings").and_then(serde_json::Value::as_object) else {
            return;
        };
        let package_id =
            self.parse_uuid_field(part, "package", subject, "pin_pad_map part", diagnostics);
        let footprint_id = self.validate_optional_uuid_ref(
            map,
            "footprint",
            &self.footprints.keys().copied().collect(),
            subject,
            "pin_pad_map",
            diagnostics,
        );
        let entity_id =
            self.parse_uuid_field(part, "entity", subject, "pin_pad_map part", diagnostics);
        for (pad_key, entry) in mappings {
            let mapping_subject = format!("{subject}#mappings/{pad_key}");
            let Some(pad_id) = Uuid::parse_str(pad_key).ok() else {
                diagnostics.push(LibraryGraphDiagnostic {
                    severity: "error",
                    code: "invalid_uuid_key",
                    subject: mapping_subject.clone(),
                    message: format!("pin_pad_map mapping key `{pad_key}` is not a valid UUID"),
                });
                continue;
            };
            let Some(gate_id) = self.parse_uuid_field(
                entry,
                "gate",
                &mapping_subject,
                "pin_pad_map mapping",
                diagnostics,
            ) else {
                continue;
            };
            let pin_id = self.parse_uuid_field(
                entry,
                "pin",
                &mapping_subject,
                "pin_pad_map mapping",
                diagnostics,
            );
            if let Some(entity_id) = entity_id {
                self.validate_entity_gate_pin_exists(
                    entity_id,
                    gate_id,
                    pin_id,
                    &mapping_subject,
                    diagnostics,
                );
            } else {
                continue;
            }
            if let Some(footprint_id) = footprint_id {
                self.validate_footprint_pad_exists(
                    footprint_id,
                    pad_id,
                    &mapping_subject,
                    diagnostics,
                );
            } else if let Some(package_id) = package_id {
                self.validate_package_pad_exists(package_id, pad_id, &mapping_subject, diagnostics);
            }
        }
    }

    fn validate_part_default_footprint_package(
        &self,
        footprint_id: Uuid,
        package_id: Uuid,
        subject: &str,
        diagnostics: &mut Vec<LibraryGraphDiagnostic>,
    ) {
        let Some(footprint) = self.footprints.get(&footprint_id) else {
            return;
        };
        let Some(footprint_package_id) = self.parse_uuid_field(
            footprint,
            "package",
            subject,
            "part default_footprint",
            diagnostics,
        ) else {
            return;
        };
        if footprint_package_id != package_id {
            diagnostics.push(LibraryGraphDiagnostic {
                severity: "error",
                code: "package_mismatch",
                subject: subject.to_string(),
                message: format!(
                    "part default_footprint {footprint_id} belongs to package {footprint_package_id}, not part package {package_id}"
                ),
            });
        }
    }

    fn validate_part_default_pin_pad_map(
        &self,
        pin_pad_map_id: Uuid,
        part_id: Uuid,
        default_footprint_id: Option<Uuid>,
        package_id: Option<Uuid>,
        subject: &str,
        diagnostics: &mut Vec<LibraryGraphDiagnostic>,
    ) {
        let Some(pin_pad_map) = self.pin_pad_maps.get(&pin_pad_map_id) else {
            return;
        };
        let Some(map_part_id) = self.parse_uuid_field(
            pin_pad_map,
            "part",
            subject,
            "part default_pin_pad_map",
            diagnostics,
        ) else {
            return;
        };
        if map_part_id != part_id {
            diagnostics.push(LibraryGraphDiagnostic {
                severity: "error",
                code: "part_mismatch",
                subject: subject.to_string(),
                message: format!(
                    "part default_pin_pad_map {pin_pad_map_id} belongs to part {map_part_id}, not {part_id}"
                ),
            });
        }
        let map_footprint_id = self.validate_optional_uuid_ref(
            pin_pad_map,
            "footprint",
            &self.footprints.keys().copied().collect(),
            subject,
            "part default_pin_pad_map",
            diagnostics,
        );
        if let (Some(default_footprint_id), Some(map_footprint_id)) =
            (default_footprint_id, map_footprint_id)
            && map_footprint_id != default_footprint_id
        {
            diagnostics.push(LibraryGraphDiagnostic {
                severity: "error",
                code: "footprint_mismatch",
                subject: subject.to_string(),
                message: format!(
                    "part default_pin_pad_map {pin_pad_map_id} targets footprint {map_footprint_id}, not default_footprint {default_footprint_id}"
                ),
            });
        }
        if let (Some(package_id), Some(map_footprint_id)) = (package_id, map_footprint_id) {
            self.validate_part_default_footprint_package(
                map_footprint_id,
                package_id,
                subject,
                diagnostics,
            );
        }
    }

    fn validate_unit_pin_exists(
        &self,
        unit_id: Uuid,
        pin_id: Uuid,
        subject: &str,
        diagnostics: &mut Vec<LibraryGraphDiagnostic>,
    ) {
        let Some(unit) = self.units.get(&unit_id) else {
            return;
        };
        let Some(pins) = unit.get("pins").and_then(serde_json::Value::as_object) else {
            return;
        };
        if !pins.contains_key(&pin_id.to_string()) {
            diagnostics.push(LibraryGraphDiagnostic {
                severity: "error",
                code: "dangling_reference",
                subject: subject.to_string(),
                message: format!("part pad_map references missing unit pin {pin_id}"),
            });
        }
    }

    fn validate_entity_gate_pin_exists(
        &self,
        entity_id: Uuid,
        gate_id: Uuid,
        pin_id: Option<Uuid>,
        subject: &str,
        diagnostics: &mut Vec<LibraryGraphDiagnostic>,
    ) {
        let Some(entity) = self.entities.get(&entity_id) else {
            return;
        };
        let Some(gate) = entity
            .get("gates")
            .and_then(serde_json::Value::as_object)
            .and_then(|gates| gates.get(&gate_id.to_string()))
        else {
            diagnostics.push(LibraryGraphDiagnostic {
                severity: "error",
                code: "dangling_reference",
                subject: subject.to_string(),
                message: format!("pin_pad_map mapping references missing entity gate {gate_id}"),
            });
            return;
        };
        let Some(pin_id) = pin_id else {
            return;
        };
        let Some(unit_id) = gate
            .get("unit")
            .and_then(serde_json::Value::as_str)
            .and_then(|unit| Uuid::parse_str(unit).ok())
        else {
            return;
        };
        self.validate_unit_pin_exists(unit_id, pin_id, subject, diagnostics);
    }

    fn validate_package_pad_exists(
        &self,
        package_id: Uuid,
        pad_id: Uuid,
        subject: &str,
        diagnostics: &mut Vec<LibraryGraphDiagnostic>,
    ) {
        let Some(package) = self.packages.get(&package_id) else {
            return;
        };
        let Some(pads) = package.get("pads").and_then(serde_json::Value::as_object) else {
            return;
        };
        if !pads.contains_key(&pad_id.to_string()) {
            diagnostics.push(LibraryGraphDiagnostic {
                severity: "error",
                code: "dangling_reference",
                subject: subject.to_string(),
                message: format!("pin_pad_map mapping references missing package pad {pad_id}"),
            });
        }
    }

    fn validate_footprint_pad_exists(
        &self,
        footprint_id: Uuid,
        pad_id: Uuid,
        subject: &str,
        diagnostics: &mut Vec<LibraryGraphDiagnostic>,
    ) {
        let Some(footprint) = self.footprints.get(&footprint_id) else {
            return;
        };
        let Some(pads) = footprint.get("pads").and_then(serde_json::Value::as_object) else {
            return;
        };
        if !pads.contains_key(&pad_id.to_string()) {
            diagnostics.push(LibraryGraphDiagnostic {
                severity: "error",
                code: "dangling_reference",
                subject: subject.to_string(),
                message: format!("pin_pad_map mapping references missing footprint pad {pad_id}"),
            });
        }
    }

    fn validate_uuid_ref(
        &self,
        value: &serde_json::Value,
        field: &'static str,
        valid_ids: &BTreeSet<Uuid>,
        subject: &str,
        label: &'static str,
        diagnostics: &mut Vec<LibraryGraphDiagnostic>,
    ) -> Option<Uuid> {
        let Some(reference) = self.parse_uuid_field(value, field, subject, label, diagnostics)
        else {
            return None;
        };
        if !valid_ids.contains(&reference) {
            diagnostics.push(LibraryGraphDiagnostic {
                severity: "error",
                code: "dangling_reference",
                subject: subject.to_string(),
                message: format!("{label} field {field} references missing object {reference}"),
            });
        }
        Some(reference)
    }

    fn validate_optional_uuid_ref(
        &self,
        value: &serde_json::Value,
        field: &'static str,
        valid_ids: &BTreeSet<Uuid>,
        subject: &str,
        label: &'static str,
        diagnostics: &mut Vec<LibraryGraphDiagnostic>,
    ) -> Option<Uuid> {
        match value.get(field) {
            None | Some(serde_json::Value::Null) => None,
            Some(_) => self.validate_uuid_ref(value, field, valid_ids, subject, label, diagnostics),
        }
    }

    fn validate_nested_uuid_field(
        &self,
        value: &serde_json::Value,
        field: &'static str,
        subject: &str,
        label: &'static str,
        diagnostics: &mut Vec<LibraryGraphDiagnostic>,
    ) -> Option<Uuid> {
        self.parse_uuid_field(value, field, subject, label, diagnostics)
    }

    fn parse_uuid_field(
        &self,
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

    fn subject(&self, object_id: &Uuid) -> String {
        self.subjects.get(object_id).cloned().unwrap_or_default()
    }
}
