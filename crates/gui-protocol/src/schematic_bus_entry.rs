//! A bus entry binds an existing scalar wire; it is a child of the selected Bus.
use super::*;
use crate::{AuthoredSelectionIdentity, AuthoringViewBasis};
use eda_engine::api::native_write::schematic_bus::{WireEnd, entry_geometry};
use eda_engine::schematic::{Bus, BusEntry, SchematicWire, Sheet};

#[derive(Debug, Clone, PartialEq)]
pub struct BusEntryDraft {
    pub wire: AuthoredSelectionIdentity,
    pub basis: AuthoringViewBasis,
    pub tip: Option<Uuid>,
    pub original: SchematicWire,
    pub buses: Vec<Bus>,
    pub choice: usize,
    pub wire_end: WireEnd,
}
impl BusEntryDraft {
    pub fn bus(&self) -> Option<&Bus> {
        self.buses.get(self.choice)
    }
    pub fn end_name(&self) -> &'static str {
        match self.wire_end {
            WireEnd::From => "From",
            WireEnd::To => "To",
        }
    }
    pub fn cycle_end(&mut self) {
        self.wire_end = match self.wire_end {
            WireEnd::From => WireEnd::To,
            WireEnd::To => WireEnd::From,
        };
    }
    pub fn preview(&self, at: PointNm, sheet: &Sheet) -> Result<BusEntry, String> {
        let bus = self.bus().ok_or("Choose an existing Bus")?;
        if sheet.buses.get(&bus.uuid) != Some(bus)
            || sheet.wires.get(&self.wire.id) != Some(&self.original)
        {
            return Err("Bus or member wire changed; cancel and reopen Bus Entry".into());
        }
        let position = eda_engine::ir::geometry::Point { x: at.x, y: at.y };
        let size = entry_geometry(bus, &self.original, position, self.wire_end)
            .map_err(|e| e.to_string())?;
        Ok(BusEntry {
            uuid: Uuid::nil(),
            bus: bus.uuid,
            wire: Some(self.wire.id),
            position,
            size,
        })
    }
    pub fn request(&self, at: PointNm, sheet: &Sheet) -> Result<SchematicEditRequest, String> {
        let entry = self.preview(at, sheet)?;
        Ok(SchematicEditRequest::AddBusEntry {
            sheet: self.basis.source,
            bus: entry.bus,
            instance_path: self.wire.instance_path.clone(),
            wire: self.wire.id,
            at,
            wire_end: self.wire_end,
        })
    }
}
