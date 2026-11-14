//! One complete library component choice shared by capture and board handoff.
//! This is captured source intent, not a new persisted component or a writer.
use super::*;
use crate::api::native_write::library_pin_pad_map::{
    PinPadMapMapping, validate_pin_pad_map_payload,
};
use crate::pool::{Footprint, PinPadMap};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CompletePartSelection {
    pub project: Uuid,
    pub part: PartGateSelection,
    pub value: String,
    pub package: Uuid,
    pub package_name: String,
    pub footprint: Uuid,
    pub footprint_name: String,
    pub pin_pad_map: Uuid,
    pub source_revisions: BTreeMap<Uuid, u64>,
}

impl CompletePartSelection {
    pub fn capture(
        model: &DesignModel,
        part_id: Uuid,
        gate: Uuid,
        symbol: Uuid,
    ) -> Result<Self, EngineError> {
        let selection = PartGateSelection::capture(model, part_id, gate, symbol)?;
        let part: Part =
            serde_json::from_value(materialized_pool_object(model, part_id, "parts")?)?;
        if part.value.trim().is_empty() {
            return Err(invalid("Choose a component value before placement"));
        }
        let footprint_id = part.default_footprint.ok_or_else(|| {
            invalid("Complete this component's default footprint before placement")
        })?;
        let map_id = part.default_pin_pad_map.ok_or_else(|| {
            invalid("Complete this component's default pin/pad map before placement")
        })?;
        let package = materialized_pool_object(model, part.package, "packages")?;
        let footprint: Footprint =
            serde_json::from_value(materialized_pool_object(model, footprint_id, "footprints")?)?;
        let map: PinPadMap =
            serde_json::from_value(materialized_pool_object(model, map_id, "pin_pad_maps")?)?;
        if footprint.uuid != footprint_id
            || footprint.package != part.package
            || map.uuid != map_id
            || map.part != part_id
            || map.footprint != Some(footprint_id)
        {
            return Err(invalid(
                "Component package, footprint and pin/pad map disagree",
            ));
        }
        let mappings: Vec<_> = map
            .mappings
            .iter()
            .map(|(pad, entry)| PinPadMapMapping {
                pad: *pad,
                gate: entry.gate,
                pin: entry.pin,
            })
            .collect();
        validate_pin_pad_map_payload(model, part_id, Some(footprint_id), &mappings)?;
        if footprint.pads.is_empty()
            || footprint.pads.keys().copied().collect::<BTreeSet<_>>()
                != map.mappings.keys().copied().collect()
        {
            return Err(invalid(
                "Every footprint pad needs an explicit component pin mapping",
            ));
        }
        let entity: Entity =
            serde_json::from_value(materialized_pool_object(model, part.entity, "entities")?)?;
        let mut source_revisions = BTreeMap::new();
        let mut expected = BTreeSet::new();
        for (gate_id, gate) in &entity.gates {
            let unit: Unit =
                serde_json::from_value(materialized_pool_object(model, gate.unit, "units")?)?;
            let body: Symbol =
                serde_json::from_value(materialized_pool_object(model, gate.symbol, "symbols")?)?;
            if gate_id != &gate.uuid
                || unit.uuid != gate.unit
                || body.uuid != gate.symbol
                || body.unit != gate.unit
            {
                return Err(invalid(
                    "Component gate, Symbol and Unit identities disagree",
                ));
            }
            expected.extend(unit.pins.keys().map(|pin| (gate.uuid, *pin)));
            source_revisions.insert(
                gate.unit,
                object_revision(model, gate.unit, "component Unit")?,
            );
            source_revisions.insert(
                gate.symbol,
                object_revision(model, gate.symbol, "component Symbol")?,
            );
        }
        if expected
            != mappings
                .iter()
                .map(|entry| (entry.gate, entry.pin))
                .collect::<BTreeSet<_>>()
        {
            return Err(invalid(
                "Every component gate pin needs an explicit footprint pad mapping",
            ));
        }
        for pad in footprint.pads.values() {
            materialized_pool_object(model, pad.padstack, "padstacks")?;
            source_revisions.insert(
                pad.padstack,
                object_revision(model, pad.padstack, "component Padstack")?,
            );
        }
        for (id, label) in [
            (part_id, "component Part"),
            (part.entity, "component Entity"),
            (symbol, "component Symbol"),
            (part.package, "component Package"),
            (footprint_id, "component Footprint"),
            (map_id, "component PinPadMap"),
        ] {
            source_revisions.insert(id, object_revision(model, id, label)?);
        }
        Ok(Self {
            project: model.project.project_id,
            part: selection,
            value: part.value,
            package: part.package,
            package_name: package["name"].as_str().unwrap_or("").to_owned(),
            footprint: footprint_id,
            footprint_name: footprint.name,
            pin_pad_map: map_id,
            source_revisions,
        })
    }

    pub fn verify(&self, model: &DesignModel) -> Result<(), EngineError> {
        if Self::capture(model, self.part.part, self.part.gate, self.part.symbol)? != *self {
            return Err(invalid(
                "Component definition changed; review the current component again",
            ));
        }
        Ok(())
    }

    pub fn verify_declared(&self, model: &DesignModel, component: Uuid) -> Result<(), EngineError> {
        self.verify(model)?;
        let declared = model.declared_component_part_sources(component, self.part.part)?;
        if declared.len() != self.source_revisions.len()
            || declared
                .iter()
                .any(|(id, revision)| self.source_revisions.get(id) != Some(&revision.0))
        {
            return Err(invalid(
                "Component physical definition changed since selection; review the component update before PCB handoff",
            ));
        }
        Ok(())
    }

    pub(crate) fn guard(&self, prepared: &mut crate::api::native_write::context::PreparedWrite) {
        self.part.guard(prepared);
        for (id, revision) in &self.source_revisions {
            if !prepared.batch.operations.iter().any(
                |op| matches!(op, Operation::GuardObjectRevision { object_id, .. } if object_id == id),
            ) {
                prepared.batch.operations.insert(0, Operation::GuardObjectRevision {
                    object_id: *id,
                    expected_object_revision: ObjectRevision(*revision),
                });
            }
        }
    }
}
