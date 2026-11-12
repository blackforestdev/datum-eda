//! Typed menu presets for the engine-owned schematic alignment discipline.
use crate::SelectionSubject;
use eda_engine::api::native_write::schematic_alignment::{
    AlignmentAxis, AlignmentOperation, AlignmentPosition, AlignmentReference,
    SymbolAlignmentRequest,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum MenuActionPreset {
    SchematicAlignment {
        axis: AlignmentAxis,
        position: AlignmentPosition,
    },
}

impl MenuActionPreset {
    pub fn alignment_request(
        &self,
        subject: &SelectionSubject,
    ) -> anyhow::Result<SymbolAlignmentRequest> {
        let members = crate::footprint_move_preview::native::group::symbol_members(subject)?;
        anyhow::ensure!(
            members.len() >= 2,
            "Select at least two Symbols for alignment"
        );
        let Self::SchematicAlignment { axis, position } = self;
        Ok(SymbolAlignmentRequest {
            symbols: members.iter().map(|id| id.id).collect(),
            instance_path: members[0].instance_path.clone(),
            op: AlignmentOperation::Align,
            reference: AlignmentReference::SelectionBounds,
            axis: *axis,
            position: *position,
        })
    }
}
