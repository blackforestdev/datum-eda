//! Complete native Part source intent uses the existing acquisition controls.
use super::{AcquisitionSource, LibraryAcquisition, LibraryAcquisitionPreview};
use crate::single_line_editor::SingleLineEditor;
use serde_json::Value;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq)]
pub struct NativePartAcquisitionPreview {
    pub part: Value,
    pub entity: Value,
    pub package: Value,
    pub footprint: Value,
    pub pin_pad_map: Value,
    pub gate_count: usize,
    pub object_count: usize,
    pub padstacks: Vec<Value>,
    pub gate_symbols: Vec<(String, Value, Value)>,
}
impl NativePartAcquisitionPreview {
    /// Labels follow the authored UUID relationships, never matching pin/pad names.
    pub fn mapping_labels(&self) -> Result<Vec<String>, String> {
        let reference = |raw: &Value, key: &str| -> Result<Uuid, String> {
            raw.get(key)
                .and_then(Value::as_str)
                .and_then(|s| s.parse().ok())
                .ok_or_else(|| format!("Native mapping {key} identity unavailable"))
        };
        let rows = self
            .pin_pad_map
            .get("mappings")
            .and_then(Value::as_object)
            .filter(|r| !r.is_empty())
            .ok_or("Native pin/pad map unavailable")?;
        rows.iter()
            .map(|(pad_key, cell)| {
                let pad: Uuid = pad_key
                    .parse()
                    .map_err(|_| "Native pad identity unavailable")?;
                let pad = self
                    .footprint
                    .get("pads")
                    .and_then(|p| p.get(pad.to_string()))
                    .ok_or("Native footprint pad unavailable")?;
                let gate = reference(cell, "gate")?;
                let gate = self
                    .entity
                    .get("gates")
                    .and_then(|g| g.get(gate.to_string()))
                    .ok_or("Native mapping gate unavailable")?;
                let unit = reference(gate, "unit")?;
                let unit = self
                    .gate_symbols
                    .iter()
                    .find(|(_, u, _)| reference(u, "uuid").ok() == Some(unit))
                    .map(|(_, u, _)| u)
                    .ok_or("Native mapping Unit unavailable")?;
                let pin = reference(cell, "pin")?;
                let pin = unit
                    .get("pins")
                    .and_then(|p| p.get(pin.to_string()))
                    .ok_or("Native logical pin unavailable")?;
                let name = |raw: &Value| {
                    raw.get("name")
                        .and_then(Value::as_str)
                        .unwrap_or("Unnamed")
                        .to_owned()
                };
                Ok(format!(
                    "Pad {} → {} / {}",
                    name(pad),
                    name(gate),
                    name(pin)
                ))
            })
            .collect()
    }
}
impl LibraryAcquisitionPreview {
    pub fn selection_key(&self) -> String {
        self.native_part
            .as_ref()
            .and_then(|p| p.part.get("uuid")?.as_str()?.parse::<Uuid>().ok())
            .map_or_else(|| self.name.clone(), |id| id.to_string())
    }
}
impl LibraryAcquisition {
    pub fn bundled(&self) -> bool {
        self.source == AcquisitionSource::BundledSymbols
    }
    pub fn footprints(&self) -> bool {
        self.source == AcquisitionSource::KiCadFootprints
    }
    pub fn native_parts(&self) -> bool {
        self.source == AcquisitionSource::NativeParts
    }
    pub fn is_native_part(&self) -> bool {
        self.native_parts()
    }
    pub fn selected_native_part(&self) -> Option<Uuid> {
        let preview = self.preview.as_ref()?;
        if !self.native_parts()
            || self.loading
            || self.selected.as_deref() != Some(preview.selection_key().as_str())
            || self.selected_source.as_ref() != Some(&preview.source_path)
        {
            return None;
        }
        preview
            .native_part
            .as_ref()?
            .part
            .get("uuid")?
            .as_str()?
            .parse()
            .ok()
    }
    pub fn use_native_part_source(&mut self) {
        self.source_changed();
        self.source = AcquisitionSource::NativeParts;
        self.path.clear();
        self.path_editor = SingleLineEditor::default();
    }
    pub fn native_part_preview(
        snapshot: &eda_engine::api::native_write::library_acquisition::native_part::NativePartPreview,
    ) -> Result<LibraryAcquisitionPreview, String> {
        let get = |kind, id| {
            snapshot
                .object(kind, id)
                .cloned()
                .ok_or_else(|| format!("Native {kind} source unavailable"))
        };
        let part = get("parts", snapshot.part)?;
        let reference = |raw: &Value, key: &str| -> Result<Uuid, String> {
            raw.get(key)
                .and_then(Value::as_str)
                .and_then(|s| s.parse().ok())
                .ok_or_else(|| format!("Native Part {key} unavailable"))
        };
        let entity = get("entities", reference(&part, "entity")?)?;
        let gates = entity
            .get("gates")
            .and_then(Value::as_object)
            .ok_or("Native gates unavailable")?;
        let gate_count = gates.len();
        let gate_symbols = gates
            .values()
            .map(|g| {
                Ok((
                    g.get("name")
                        .and_then(Value::as_str)
                        .unwrap_or("Unnamed gate")
                        .to_owned(),
                    get("units", reference(g, "unit")?)?,
                    get("symbols", reference(g, "symbol")?)?,
                ))
            })
            .collect::<Result<Vec<_>, String>>()?;
        let gate = gates.values().next().ok_or("Native gates unavailable")?;
        Ok(LibraryAcquisitionPreview {
            name: snapshot.name.clone(), source_digest: snapshot.digest.clone(), source_path: snapshot.source_path.clone(),
            unit: get("units", reference(gate, "unit")?)?, symbol: get("symbols", reference(gate, "symbol")?)?,
            footprint: None,
            native_part: Some(NativePartAcquisitionPreview {
                package: get("packages", reference(&part, "package")?)?,
                footprint: get("footprints", reference(&part, "default_footprint")?)?,
                pin_pad_map: get("pin_pad_maps", reference(&part, "default_pin_pad_map")?)?,
                entity, part, gate_count, object_count: snapshot.nodes.len(), gate_symbols,
                padstacks: snapshot.nodes.iter().filter(|n| n.kind == "padstacks").map(|n| n.object.clone()).collect(),
            }),
            notices: vec!["Identity-preserving native Part snapshot; source definitions and review metadata are retained. Acquisition does not approve the source.".into(),
                "Identical Project definitions are reused unchanged; conflicting identities refuse the entire acquisition.".into(),
                "Mask/paste process basis and manufacturing qualification are not supplied by acquisition.".into()],
        })
    }
}

#[cfg(test)]
#[path = "library_native_part_acquisition_tests.rs"]
mod tests;
