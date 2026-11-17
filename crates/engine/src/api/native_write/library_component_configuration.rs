//! One designer component choice over existing source readers and pool writers.
//!
//! This is invocation state, not a new persisted Part kind or mutation path.
use crate::api::native_write::{
    context::{BatchComposer, PreparedWrite, WriteProvenance},
    library::{self, PoolLibraryObjectTarget, PoolLibraryOperationSpec},
    registry::NativeWriteContext,
};
use crate::api::native_write::{
    library_acquisition::LibraryAcquisitionRequest,
    schematic_library::part_placement::CompletePartSelection,
};
use crate::substrate::Operation;
use crate::{
    error::EngineError,
    pool::{Footprint, Symbol, Unit},
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use uuid::Uuid;

pub const CONFIGURED_DESCRIPTION: &str =
    "Designer-configured component; procurement and rating unspecified.";

/// Actual source identities and explicit engineering choices, never GUI object payloads.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ComponentConfigurationRequest {
    pub symbol: Uuid,
    pub footprint: Uuid,
    pub value: String,
    pub mappings: BTreeMap<Uuid, Uuid>,
    pub mapping_confirmed: bool,
    #[serde(default)]
    pub symbol_source: Option<LibraryAcquisitionRequest>,
    #[serde(default)]
    pub footprint_source: Option<LibraryAcquisitionRequest>,
    /// A reusable, revision-qualified mapping; commercial metadata is not inherited.
    #[serde(default)]
    pub mapping_source: Option<CompletePartSelection>,
}

fn refusal(message: impl Into<String>) -> EngineError {
    EngineError::Validation(message.into())
}

/// Reusable correspondence comes from current complete components, never from
/// similar-looking names, pad order or a guessed numerical convention.
pub fn qualified_mapping_choices(
    model: &crate::substrate::DesignModel,
    symbol: Uuid,
    footprint: Uuid,
) -> Vec<CompletePartSelection> {
    let mut choices = Vec::new();
    for record in model
        .objects
        .values()
        .filter(|o| o.domain == "pool" && o.kind == "parts")
    {
        let result = (|| -> Result<(), EngineError> {
            let raw = super::super::schematic_library::materialized_pool_object(
                model,
                record.object_id,
                "parts",
            )?;
            let part: crate::pool::Part = serde_json::from_value(raw)?;
            if part.default_footprint != Some(footprint) {
                return Ok(());
            }
            let raw = super::super::schematic_library::materialized_pool_object(
                model,
                part.entity,
                "entities",
            )?;
            let entity: crate::pool::Entity = serde_json::from_value(raw)?;
            // A bare single-Symbol configuration cannot silently discard the
            // other gates of a complete multi-gate component.
            if entity.gates.len() != 1 {
                return Ok(());
            }
            for gate in entity.gates.values().filter(|g| g.symbol == symbol) {
                if let Ok(choice) =
                    CompletePartSelection::capture(model, part.uuid, gate.uuid, symbol)
                {
                    choices.push(choice);
                }
            }
            Ok(())
        })();
        let _ = result; // An unrelated incomplete library entry is not a global gate.
    }
    choices.sort_by_key(|c| (c.part.part, c.part.gate));
    choices
}

/// Reuse a previously saved instance of this exact user configuration. A
/// commercial Part or a different rating is never inferred from matching pads.
pub fn reusable_configurations(
    model: &crate::substrate::DesignModel,
    request: &ComponentConfigurationRequest,
) -> Vec<CompletePartSelection> {
    qualified_mapping_choices(model, request.symbol, request.footprint)
        .into_iter()
        .filter(|choice| {
            if choice.value != request.value.trim() {
                return false;
            }
            let result = (|| -> Result<bool, EngineError> {
                let raw = super::super::schematic_library::materialized_pool_object(
                    model,
                    choice.part.part,
                    "parts",
                )?;
                let part: crate::pool::Part = serde_json::from_value(raw.clone())?;
                if !part.mpn.trim().is_empty()
                    || !part.manufacturer.trim().is_empty()
                    || part.description != CONFIGURED_DESCRIPTION
                    || !part.parametric.is_empty()
                    || !part.orderable_mpns.is_empty()
                    || !part.behavioural_models.is_empty()
                    || part.base.is_some()
                    || part.thermal.is_some()
                    || !part.datasheet.is_empty()
                    || !part.packaging_options.is_empty()
                    || !part.pad_map.is_empty()
                    || part.manufacturer_jep106.is_some()
                    || part.supply_chain_offers.is_some()
                    || part.last_supply_chain_check.is_some()
                {
                    return Ok(false);
                }
                let map = super::super::schematic_library::materialized_pool_object(
                    model,
                    choice.pin_pad_map,
                    "pin_pad_maps",
                )?;
                let map: crate::pool::PinPadMap = serde_json::from_value(map)?;
                Ok(map.mappings.len() == request.mappings.len()
                    && map.mappings.iter().all(|(pad, target)| {
                        target.gate == choice.part.gate
                            && request.mappings.get(pad) == Some(&target.pin)
                    }))
            })();
            result.unwrap_or(false)
        })
        .collect()
}

impl ComponentConfigurationRequest {
    /// A display name describes the user's actual choice, not a supplier identity.
    pub fn display_name(
        &self,
        symbol: &Symbol,
        footprint: &Footprint,
    ) -> Result<String, EngineError> {
        self.validate_value()?;
        if symbol.uuid != self.symbol || footprint.uuid != self.footprint {
            return Err(refusal(
                "Component source identities differ from the selected preview",
            ));
        }
        Ok(format!(
            "{} {} · {}",
            symbol.name,
            self.value.trim(),
            footprint.name
        ))
    }

    pub fn validate_value(&self) -> Result<(), EngineError> {
        if self.value.trim().is_empty() || self.value.chars().any(char::is_control) {
            return Err(refusal("Enter the component value before Apply or Place"));
        }
        Ok(())
    }

    /// Mapping authority is explicit confirmation or a separately verified saved map.
    /// Pin names/numbers are captions; only the actual source UUIDs establish links.
    pub fn validate_mapping(
        &self,
        unit: &Unit,
        symbol: &Symbol,
        footprint: &Footprint,
    ) -> Result<(), EngineError> {
        self.validate_value()?;
        if !self.mapping_confirmed {
            return Err(refusal(
                "Confirm the pin-to-pad correspondence before Apply or Place",
            ));
        }
        if symbol.uuid != self.symbol
            || symbol.unit != unit.uuid
            || footprint.uuid != self.footprint
        {
            return Err(refusal(
                "The selected Symbol, Unit or Footprint source changed",
            ));
        }
        let pins: BTreeSet<_> = unit.pins.keys().copied().collect();
        let pads: BTreeSet<_> = footprint.pads.keys().copied().collect();
        if pins.is_empty() || pads.is_empty() {
            return Err(refusal("Choose actual component pins and physical pads"));
        }
        if symbol
            .pin_anchors
            .iter()
            .any(|anchor| !pins.contains(&anchor.pin))
        {
            return Err(refusal("The Symbol contains an unavailable Unit pin"));
        }
        if self.mappings.keys().copied().collect::<BTreeSet<_>>() != pads
            || self.mappings.values().copied().collect::<BTreeSet<_>>() != pins
        {
            return Err(refusal(
                "Map every physical pad and every component pin using the selected sources",
            ));
        }
        Ok(())
    }
}

/// The ordinary component task creates only project-local definitions. It does
/// not edit a shared Part/default or write an instance behind the user's back.
pub(super) fn build(
    context: &NativeWriteContext<'_>,
    provenance: WriteProvenance,
    part_id: Uuid,
    request: ComponentConfigurationRequest,
) -> Result<PreparedWrite, EngineError> {
    request.validate_value()?;
    if context.model.objects.contains_key(&part_id) {
        return Err(refusal(
            "This component choice was already saved; refresh the configuration",
        ));
    }
    let mut sources = BTreeMap::<Uuid, (String, Value)>::new();
    let mut operations = Vec::new();
    for (source, id, kind) in [
        (request.symbol_source.as_ref(), request.symbol, "symbols"),
        (
            request.footprint_source.as_ref(),
            request.footprint,
            "footprints",
        ),
    ] {
        if let Some(source) = source {
            source_operations(
                context,
                provenance.clone(),
                source,
                id,
                kind,
                &mut sources,
                &mut operations,
            )?;
        }
    }
    let symbol_raw = object(context, &sources, request.symbol, "symbols")?;
    let symbol: Symbol = serde_json::from_value(symbol_raw.clone())?;
    let unit_raw = object(context, &sources, symbol.unit, "units")?;
    let unit: Unit = serde_json::from_value(unit_raw.clone())?;
    let footprint_raw = object(context, &sources, request.footprint, "footprints")?;
    let footprint: Footprint = serde_json::from_value(footprint_raw.clone())?;
    request.validate_mapping(&unit, &symbol, &footprint)?;
    if let Some(saved) = &request.mapping_source {
        saved.verify(context.model)?;
        if saved.part.symbol != request.symbol || saved.footprint != request.footprint {
            return Err(refusal(
                "The saved mapping belongs to another Symbol or Footprint",
            ));
        }
        let raw = object(context, &sources, saved.pin_pad_map, "pin_pad_maps")?;
        let map: crate::pool::PinPadMap = serde_json::from_value(raw)?;
        let selected: BTreeMap<_, _> = map
            .mappings
            .iter()
            .filter(|(_, target)| target.gate == saved.part.gate)
            .map(|(pad, target)| (*pad, target.pin))
            .collect();
        if selected != request.mappings || selected.len() != map.mappings.len() {
            return Err(refusal(
                "The saved qualified mapping differs from this pending choice",
            ));
        }
    }
    let package = object(context, &sources, footprint.package, "packages")?;
    let entity_id = Uuid::new_v5(&part_id, b"component-entity");
    let gate_id = Uuid::new_v5(&part_id, b"component-gate");
    let map_id = Uuid::new_v5(&part_id, b"component-pin-pad-map");
    let name = request.display_name(&symbol, &footprint)?;
    let prefix = symbol
        .default_refdes_prefix
        .as_deref()
        .filter(|p| !p.trim().is_empty())
        .ok_or_else(|| {
            refusal(
                "The source Symbol has no reference prefix; choose a complete library component",
            )
        })?;
    let entity = library::pool_entity_payload(
        entity_id,
        gate_id,
        unit.uuid,
        symbol.uuid,
        &symbol.name,
        prefix,
        "",
        &symbol.name,
    );
    let mut part = library::pool_part_payload(
        part_id,
        entity_id,
        footprint.package,
        "",
        "",
        request.value.trim(),
        CONFIGURED_DESCRIPTION,
        "",
        "Unknown",
    );
    part["name"] = json!(name);
    part["default_footprint"] = json!(footprint.uuid);
    part["default_pin_pad_map"] = json!(map_id);
    let source_library = footprint_raw
        .pointer("/source_provenance/source_path")
        .and_then(Value::as_str)
        .and_then(|p| std::path::Path::new(p).parent()?.file_name()?.to_str());
    part["tags"] = json!(
        source_library
            .into_iter()
            .chain(std::iter::once(symbol.name.as_str()))
            .collect::<BTreeSet<_>>()
    );
    super::super::library_definition::validate_part_identity(&part)?;
    let mappings: serde_json::Map<String, Value> = request
        .mappings
        .iter()
        .map(|(pad, pin)| (pad.to_string(), json!({"gate":gate_id,"pin":pin})))
        .collect();
    let map = json!({"schema_version":1,"uuid":map_id,"part":part_id,
        "footprint":footprint.uuid,"mappings":mappings,"tags":[]});
    let mut graph = crate::pool::LibraryGraph::default();
    for (kind, id, raw) in [
        ("units", unit.uuid, unit_raw),
        ("symbols", symbol.uuid, symbol_raw),
        ("packages", footprint.package, package),
        ("footprints", footprint.uuid, footprint_raw),
        ("entities", entity_id, entity.clone()),
        ("parts", part_id, part.clone()),
        ("pin_pad_maps", map_id, map.clone()),
    ] {
        let _ = graph.insert_pool_object(kind, id, format!("pool/{kind}/{id}.json"), raw);
    }
    for pad in footprint.pads.values() {
        if !graph.padstacks.contains_key(&pad.padstack) {
            let raw = object(context, &sources, pad.padstack, "padstacks")?;
            let _ = graph.insert_pool_object(
                "padstacks",
                pad.padstack,
                format!("pool/padstacks/{}.json", pad.padstack),
                raw,
            );
        }
    }
    let report = graph.validation_report();
    if !report.valid {
        let reasons = report
            .diagnostics
            .iter()
            .filter(|d| d.severity == "error")
            .map(|d| d.message.as_str())
            .collect::<Vec<_>>()
            .join("; ");
        return Err(refusal(format!(
            "Component source is incomplete: {reasons}"
        )));
    }
    let definitions = [
        ("entities", entity_id, entity),
        ("parts", part_id, part),
        ("pin_pad_maps", map_id, map),
    ]
    .into_iter()
    .map(|(kind, id, object)| PoolLibraryOperationSpec::Create {
        target: PoolLibraryObjectTarget::new("pool", kind, id),
        object,
    })
    .collect();
    let prepared = library::build_pool_library_write(
        context.model,
        provenance.clone(),
        Some("pool"),
        definitions,
    )?;
    operations.extend(prepared.batch.operations);
    // Separate acquisition closures can request the same pool reference. Keep
    // the existing canonical operation once, rather than adding another writer.
    let mut pool_refs = BTreeSet::new();
    operations.retain(|op| match op {
        Operation::AddProjectPoolRef { path, .. } => pool_refs.insert(path.clone()),
        _ => true,
    });
    let mut prepared = BatchComposer::compose(context.model, provenance)
        .push_ops(operations)
        .primary_object(part_id)
        .finish()?;
    if let Some(source) = &request.mapping_source {
        source.guard(&mut prepared);
    }
    Ok(prepared)
}

fn object(
    context: &NativeWriteContext<'_>,
    sources: &BTreeMap<Uuid, (String, Value)>,
    id: Uuid,
    kind: &str,
) -> Result<Value, EngineError> {
    if let Some((found, value)) = sources.get(&id) {
        if found != kind {
            return Err(refusal("Component source object has the wrong kind"));
        }
        return Ok(value.clone());
    }
    super::super::schematic_library::materialized_pool_object(context.model, id, kind)
}

fn source_operations(
    context: &NativeWriteContext<'_>,
    provenance: WriteProvenance,
    source: &LibraryAcquisitionRequest,
    expected: Uuid,
    kind: &str,
    objects: &mut BTreeMap<Uuid, (String, Value)>,
    operations: &mut Vec<Operation>,
) -> Result<(), EngineError> {
    // Even an already-adopted source must match the pending reader result.
    let (id, hash) = match (kind, source.source_format.as_str()) {
        ("symbols", "datum_bundle") => {
            let converted =
                crate::import::library_bundle::preview(&source.source_path, &source.symbol_name)?;
            (converted.symbol.uuid, converted.source_hash)
        }
        ("symbols", "kicad_symbol") => {
            let converted =
                crate::import::kicad_library::convert(&source.source_path, &source.symbol_name)?;
            (converted.symbol.uuid, converted.source_hash)
        }
        ("footprints", "kicad_footprint") => {
            let converted = crate::import::kicad_footprint_library::convert(&source.source_path)?;
            if context.model.objects.contains_key(&expected) {
                let raw = super::super::schematic_library::materialized_pool_object(
                    context.model,
                    expected,
                    kind,
                )?;
                let current: Footprint = serde_json::from_value(raw.clone())?;
                if current != converted.footprint
                    || raw
                        .pointer("/source_provenance/source_hash")
                        .and_then(Value::as_str)
                        != Some(source.expected_source_hash.as_str())
                {
                    return Err(refusal(
                        "The Project footprint differs from this file preview; select its saved Project definition explicitly",
                    ));
                }
            }
            (converted.footprint.uuid, converted.source_hash)
        }
        _ => {
            return Err(refusal(
                "Unsupported component source; choose an actual Symbol or Footprint",
            ));
        }
    };
    if id != expected || hash != source.expected_source_hash {
        return Err(refusal(
            "Component source changed since preview; select it again",
        ));
    }
    if context.model.objects.contains_key(&expected) {
        return Ok(());
    }
    let prepared =
        super::super::library_acquisition::build_acquisition(context, provenance, source.clone())?;
    for op in &prepared.batch.operations {
        if let Operation::CreatePoolLibraryObject {
            object_id,
            object_kind,
            object,
            ..
        } = op
        {
            if let Some((previous_kind, previous)) =
                objects.insert(*object_id, (object_kind.clone(), object.clone()))
            {
                if previous_kind != *object_kind || previous != *object {
                    return Err(refusal(
                        "The selected sources disagree about a library identity",
                    ));
                }
            }
        }
    }
    operations.extend(prepared.batch.operations);
    Ok(())
}
