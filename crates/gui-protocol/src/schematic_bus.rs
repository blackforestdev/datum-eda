//! Bus form values and qualified member edits; names are validated against engine identity on submit.
use super::*;
use crate::{AuthoredSelectionIdentity, AuthoringViewBasis};
use eda_engine::schematic::Bus;

pub fn bus_fields(name: &str, members: &str) -> Result<(String, Vec<String>), String> {
    if name.trim().is_empty() || name.chars().any(char::is_control) {
        return Err("Enter a nonempty single-line bus name".into());
    }
    let members: Vec<String> = members.split(',').map(|v| v.trim().to_owned()).collect();
    if members
        .iter()
        .any(|v| v.is_empty() || v.chars().any(char::is_control))
    {
        return Err("Enter existing scalar net names separated by commas".into());
    }
    let unique: std::collections::BTreeSet<_> = members.iter().collect();
    if unique.len() != members.len() {
        return Err("Each bus member must be distinct".into());
    }
    Ok((name.trim().into(), members))
}

#[derive(Debug, Clone, PartialEq)]
pub struct BusEditDraft {
    pub target: AuthoredSelectionIdentity,
    pub basis: AuthoringViewBasis,
    pub tip: Option<Uuid>,
    pub original: Bus,
    pub entries: Vec<eda_engine::schematic::BusEntry>,
}
impl BusEditDraft {
    pub fn delete_entry(&self, state: &SchematicAuthoring) -> Result<SchematicEditRequest, String> {
        if self.request(&state.reference, &state.value)?.is_some() {
            return Err("Apply or cancel bus field changes before deleting an entry".into());
        }
        let sheet = state
            .source
            .as_ref()
            .and_then(|s| s.sheets.get(&self.basis.source))
            .ok_or("Bus source unavailable")?;
        if sheet.buses.get(&self.target.id) != Some(&self.original) {
            return Err("Bus changed; reopen the edit".into());
        }
        let entry = self
            .entries
            .get(state.bus_entry_choice)
            .ok_or("Choose an attached Bus Entry")?;
        if sheet.bus_entries.get(&entry.uuid) != Some(entry) || entry.bus != self.target.id {
            return Err("Bus Entry changed; reopen the edit".into());
        }
        Ok(SchematicEditRequest::DeleteBusEntry {
            bus: self.target.id,
            bus_entry: entry.uuid,
            instance_path: self.target.instance_path.clone(),
        })
    }
    pub fn request(
        &self,
        name: &str,
        members: &str,
    ) -> Result<Option<SchematicEditRequest>, String> {
        let (name, members) = bus_fields(name, members)?;
        if name == self.original.name && members == self.original.members {
            return Ok(None);
        }
        Ok(Some(SchematicEditRequest::CreateBus {
            sheet: self.basis.source,
            bus: Some(self.target.id),
            instance_path: self.target.instance_path.clone(),
            name,
            members,
            points: None,
        }))
    }
}

impl SchematicAuthoring {
    pub fn polyline_request(&self) -> Result<SchematicEditRequest, String> {
        let sheet = self.sheet.ok_or("Native schematic sheet unavailable")?;
        if !self.wire || self.vertices.len() < 2 {
            return Err("Draw at least two distinct vertices".into());
        }
        let points = self
            .vertices
            .iter()
            .map(|p| eda_engine::ir::geometry::Point { x: p.x, y: p.y })
            .collect();
        if self.bus {
            let (name, members) = bus_fields(&self.reference, &self.value)?;
            Ok(SchematicEditRequest::CreateBus {
                sheet,
                bus: None,
                instance_path: vec![],
                name,
                members,
                points: Some(points),
            })
        } else {
            Ok(SchematicEditRequest::DrawWire { sheet, points })
        }
    }
}
