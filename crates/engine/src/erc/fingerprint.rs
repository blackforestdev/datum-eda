//! Electrical observations, separated from human-readable finding presentation.
use super::{ErcFinding, electrical::canonical_pin_electrical_type_name};
use crate::{ir::serialization::to_json_deterministic, schematic::SchematicNetInfo};
use sha2::{Digest, Sha256};

pub(super) fn attach(findings: &mut [ErcFinding], net: &SchematicNetInfo) {
    let mut pins: Vec<_> = net
        .pins
        .iter()
        .map(|pin| {
            (
                pin.uuid,
                &pin.instance_path,
                canonical_pin_electrical_type_name(&pin.electrical_type),
            )
        })
        .collect();
    pins.sort();
    let mut ports = net.port_uuids.clone();
    ports.sort();
    for finding in findings {
        let evidence: Vec<_> = finding
            .pin_evidence
            .iter()
            .map(|pin| {
                serde_json::json!({
                    "pin": pin.pin_uuid, "symbol": pin.symbol_uuid,
                    "instance_path": pin.instance_path,
                    "type": pin.canonical_pin_type, "taxonomy": pin.pin_taxonomy_revision,
                    "part": pin.part, "entity": pin.entity, "gate": pin.gate,
                    "unit": pin.unit_selection,
                })
            })
            .collect();
        let material = serde_json::json!({
            "contract": "datum-eda:erc-observation-fingerprint:v1",
            "rule": finding.code, "severity": finding.severity,
            "targets": finding.object_uuids,
            "net_uuid": finding.net_uuid, "occurrences": net.occurrences,
            "pins": pins, "ports": ports, "labels": net.labels,
            "semantic_class": net.semantic_class, "evidence": evidence,
        });
        let canonical = to_json_deterministic(&material).expect("ERC observation JSON");
        finding.fingerprint = Some(format!("sha256:{:x}", Sha256::digest(canonical.as_bytes())));
    }
}
