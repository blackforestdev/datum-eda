use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::electrical::{PIN_ELECTRICAL_TAXONOMY_REVISION, canonical_pin_electrical_type_name};
use crate::schematic::{NetPinRef, Schematic};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ErcPinEvidence {
    pub pin_uuid: Uuid,
    pub symbol_uuid: Uuid,
    pub reference: String,
    pub pin_number: String,
    pub pin_name: String,
    pub canonical_pin_type: &'static str,
    pub pin_taxonomy_revision: &'static str,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lib_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unit_selection: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub part: Option<Uuid>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub entity: Option<Uuid>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gate: Option<Uuid>,
}

pub(super) fn pin_evidence_by_uuid(schematic: &Schematic) -> BTreeMap<Uuid, ErcPinEvidence> {
    let mut evidence = BTreeMap::new();
    for sheet in schematic.sheets.values() {
        for symbol in sheet.symbols.values() {
            for pin in &symbol.pins {
                evidence.insert(
                    pin.uuid,
                    ErcPinEvidence {
                        pin_uuid: pin.uuid,
                        symbol_uuid: symbol.uuid,
                        reference: symbol.reference.clone(),
                        pin_number: pin.number.clone(),
                        pin_name: pin.name.clone(),
                        canonical_pin_type: canonical_pin_electrical_type_name(
                            &pin.electrical_type,
                        ),
                        pin_taxonomy_revision: PIN_ELECTRICAL_TAXONOMY_REVISION,
                        lib_id: symbol.lib_id.clone(),
                        unit_selection: symbol.unit_selection.clone(),
                        part: symbol.part,
                        entity: symbol.entity,
                        gate: symbol.gate,
                    },
                );
            }
        }
    }
    evidence
}

pub(super) fn evidence_for_pins<'a>(
    evidence_by_uuid: &BTreeMap<Uuid, ErcPinEvidence>,
    pins: impl IntoIterator<Item = &'a NetPinRef>,
) -> Vec<ErcPinEvidence> {
    pins.into_iter()
        .filter_map(|pin| evidence_by_uuid.get(&pin.uuid).cloned())
        .collect()
}
