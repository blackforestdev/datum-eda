//! Shared design-object clipboard data; platform clipboard ownership stays in the app.
use super::{
    schematic_edit::cardinal_symbol_point,
    schematic_selected_source::SelectedSchematicSource,
    schematic_symbol_payload::{member_payload, set_point, source_sheet_payload},
};
use crate::{
    error::EngineError,
    ir::geometry::Point,
    schematic::{Junction, SchematicWire},
    substrate::{DesignModel, ElectricalSelectionSnapshot},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeSet;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClipboardClass {
    Wire,
    Junction,
    Symbol,
}
impl ClipboardClass {
    pub(super) fn source_class(self) -> &'static str {
        match self {
            Self::Wire => "wires",
            Self::Junction => "junctions",
            Self::Symbol => "symbols",
        }
    }
    fn token(self) -> &'static str {
        match self {
            Self::Wire => "wire",
            Self::Junction => "junction",
            Self::Symbol => "symbol",
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClipboardTarget {
    pub class: ClipboardClass,
    pub id: Uuid,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClipboardItem {
    pub class: ClipboardClass,
    pub id: Uuid,
    pub raw: Value,
}

/// Source stamps are provenance only; pasted input never authorizes source edits.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DesignClipboardBundle {
    pub format: String,
    pub version: u32,
    pub source_project: Uuid,
    pub source_revision: String,
    pub source_sheet: Uuid,
    pub instance_path: Vec<Uuid>,
    pub reference: Point,
    pub items: Vec<ClipboardItem>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub symbols: Option<super::design_clipboard_symbols::SymbolClipboardSources>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClipboardPlacement {
    pub position: Point,
    pub rotation_deg: i32,
    pub mirrored: bool,
}

impl DesignClipboardBundle {
    /// Exact raw capture over one complete unique source occurrence. Copy is
    /// read-only, including locked objects; Cut applies its own write guards.
    pub fn capture(
        model: &DesignModel,
        sheet: Uuid,
        path: &[Uuid],
        targets: &[ClipboardTarget],
        reference: Point,
    ) -> Result<Self, EngineError> {
        let first = targets
            .first()
            .ok_or_else(|| invalid("Select design objects to copy"))?;
        let snapshot = ElectricalSelectionSnapshot::capture(model)
            .map_err(|e| invalid(format!("Clipboard source unavailable: {e:?}")))?;
        let source = SelectedSchematicSource::from_snapshot(&snapshot)?;
        let raw = source_sheet_payload(model, sheet, first.id)?;
        let mut items = Vec::new();
        let mut ids = BTreeSet::new();
        for target in targets {
            if !ids.insert(target.id)
                || source.sheet(target.class.source_class(), target.id, path)? != sheet
            {
                return Err(invalid(
                    "Copy requires distinct objects in one exact source occurrence",
                ));
            }
            let source_sheet = source
                .source
                .sheets
                .get(&sheet)
                .ok_or_else(|| invalid("Clipboard sheet unavailable"))?;
            let raw = match target.class {
                ClipboardClass::Symbol => member_payload(
                    &raw,
                    "symbols",
                    target.id,
                    &source_sheet.symbols[&target.id],
                )?,
                ClipboardClass::Wire => {
                    member_payload(&raw, "wires", target.id, &source_sheet.wires[&target.id])?
                }
                ClipboardClass::Junction => member_payload(
                    &raw,
                    "junctions",
                    target.id,
                    &source_sheet.junctions[&target.id],
                )?,
            };
            items.push(ClipboardItem {
                class: target.class,
                id: target.id,
                raw,
            });
        }
        items.sort_by_key(|item| (item.class, item.id));
        let symbols = super::design_clipboard_symbols::capture(model, &items)?;
        let bundle = Self {
            format: "datum.design-clipboard".into(),
            version: 1,
            source_project: model.project.project_id,
            source_revision: model.model_revision.0.clone(),
            source_sheet: sheet,
            instance_path: path.to_vec(),
            reference,
            items,
            symbols,
        };
        bundle.validate()?;
        Ok(bundle)
    }
    pub(super) fn validate(&self) -> Result<(), EngineError> {
        if self.format != "datum.design-clipboard"
            || self.version != 1
            || self.source_project.is_nil()
            || self.source_sheet.is_nil()
            || self.source_revision.is_empty()
            || self.items.is_empty()
            || self.instance_path.iter().any(Uuid::is_nil)
        {
            return Err(invalid("Unsupported or incomplete design clipboard bundle"));
        }
        let mut ids = BTreeSet::new();
        for item in &self.items {
            if item.id.is_nil() || !ids.insert(item.id) {
                return Err(invalid(
                    "Clipboard object identities must be nonempty and distinct",
                ));
            }
            if item
                .raw
                .get("locked")
                .is_some_and(|value| !value.is_boolean())
            {
                return Err(invalid("Clipboard lock metadata must be boolean"));
            }
            match item.class {
                ClipboardClass::Symbol => {
                    let symbol: crate::schematic::PlacedSymbol =
                        serde_json::from_value(item.raw.clone())?;
                    if symbol.uuid != item.id {
                        return Err(invalid("Clipboard Symbol identity is invalid"));
                    }
                }
                ClipboardClass::Wire => {
                    let wire: SchematicWire = serde_json::from_value(item.raw.clone())?;
                    if wire.uuid != item.id || wire.from == wire.to {
                        return Err(invalid("Clipboard wire identity or geometry is invalid"));
                    }
                }
                ClipboardClass::Junction => {
                    let junction: Junction = serde_json::from_value(item.raw.clone())?;
                    if junction.uuid != item.id {
                        return Err(invalid("Clipboard junction identity is invalid"));
                    }
                }
            }
        }
        super::design_clipboard_symbols::validate(self)?;
        Ok(())
    }
    /// Refresh target reservations without changing captured component sources.
    pub fn prepare_placement(&mut self, model: &DesignModel) -> Result<(), EngineError> {
        super::design_clipboard_symbols::prepare(model, self)
    }
    /// The same checked mirror/cardinal transform used by native Symbol edits.
    /// A fresh physical map is stable for one gesture. Unknown metadata remains
    /// opaque: only known object IDs and coordinate fields are replaced.
    pub fn placed(
        &self,
        placement: ClipboardPlacement,
        seed: Uuid,
    ) -> Result<Vec<ClipboardItem>, EngineError> {
        self.validate()?;
        if seed.is_nil() {
            return Err(invalid("Paste requires a nonempty gesture identity seed"));
        }
        let mut result = Vec::new();
        let mut ids = BTreeSet::new();
        let mut junction_positions = BTreeSet::new();
        let references = super::design_clipboard_symbols::references(self)?;
        for item in &self.items {
            let id = Uuid::new_v5(
                &seed,
                format!("design-clipboard:{}:{}", item.class.token(), item.id).as_bytes(),
            );
            if !ids.insert(id) {
                return Err(invalid("Pasted object identities collide"));
            }
            let mut raw = item.raw.clone();
            raw["uuid"] = serde_json::to_value(id)?;
            match item.class {
                ClipboardClass::Symbol => {
                    raw = super::design_clipboard_symbols::placed(
                        &raw,
                        id,
                        seed,
                        self.reference,
                        placement,
                        &references[&item.id],
                    )?;
                }
                ClipboardClass::Wire => {
                    let wire: SchematicWire = serde_json::from_value(item.raw.clone())?;
                    set_point(
                        &mut raw["from"],
                        self.transform_point(wire.from, placement)?,
                    );
                    set_point(&mut raw["to"], self.transform_point(wire.to, placement)?);
                }
                ClipboardClass::Junction => {
                    let junction: Junction = serde_json::from_value(item.raw.clone())?;
                    let point = self.transform_point(junction.position, placement)?;
                    if !junction_positions.insert((point.x, point.y)) {
                        return Err(invalid("Pasted Junctions have duplicate positions"));
                    }
                    set_point(&mut raw["position"], point);
                }
            }
            result.push(ClipboardItem {
                class: item.class,
                id,
                raw,
            });
        }
        Ok(result)
    }
    fn transform_point(
        &self,
        point: Point,
        placement: ClipboardPlacement,
    ) -> Result<Point, EngineError> {
        checked_cardinal_point(point, self.reference, placement)
    }
}
/// Checked common pose for existing wiring and newly placed clipboard geometry.
pub fn checked_cardinal_point(
    point: Point,
    reference: Point,
    placement: ClipboardPlacement,
) -> Result<Point, EngineError> {
    let offset = (
        i128::from(point.x) - i128::from(reference.x),
        i128::from(point.y) - i128::from(reference.y),
    );
    let (x, y) = cardinal_symbol_point(offset, placement.rotation_deg, placement.mirrored)?;
    Ok(Point::new(
        i64::try_from(x + i128::from(placement.position.x))
            .map_err(|_| invalid("Clipboard X coordinate overflow"))?,
        i64::try_from(y + i128::from(placement.position.y))
            .map_err(|_| invalid("Clipboard Y coordinate overflow"))?,
    ))
}

fn invalid(reason: impl Into<String>) -> EngineError {
    EngineError::Validation(reason.into())
}
