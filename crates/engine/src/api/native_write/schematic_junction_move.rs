//! Contact-preserving junction Move geometry shared by preview and commit.
//! Occurrence and raw-source admission remain with the canonical selected editor.
use crate::{
    connectivity::SymbolContactBasis,
    error::EngineError,
    ir::geometry::Point,
    schematic::{Junction, SchematicWire, Sheet},
};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq)]
pub struct JunctionMoveBasis {
    pub junction: Junction,
    contacts: Option<SymbolContactBasis>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct JunctionMoveDestination {
    pub junction: Junction,
    pub wires: Vec<SchematicWire>,
}

impl JunctionMoveBasis {
    /// Capture one immutable sheet geometry; the caller owns its qualified basis.
    pub fn capture(sheet: &Sheet, id: Uuid) -> Result<Self, EngineError> {
        let junction = sheet
            .junctions
            .get(&id)
            .filter(|junction| junction.uuid == id)
            .ok_or_else(|| EngineError::Validation("Junction Move source unavailable".into()))?
            .clone();
        let contacts = SymbolContactBasis::capture_junction(sheet, id)?;
        // A free marker retains the existing absolute placement behavior.
        // A connected marker instead owns its incident wire endpoints and must
        // preserve the complete electrical membership of its source contact.
        let contacts = contacts.has_source_wire_contact().then_some(contacts);
        Ok(Self { junction, contacts })
    }

    pub fn destination_edit(
        &self,
        position: Point,
    ) -> Result<JunctionMoveDestination, EngineError> {
        let mut junction = self.junction.clone();
        junction.position = position;
        let wires = match &self.contacts {
            Some(contacts) => contacts.moved_junction_wires(&junction)?,
            None => Vec::new(),
        };
        Ok(JunctionMoveDestination { junction, wires })
    }
}
