//! PM057 per-placement Pad materialization. Source IDs are persisted by the
//! existing package journal owner; queries never allocate or rematerialize.
use super::{DesignModel, EngineError, ModelRevision, ObjectRevision, RevisionedRef};
use crate::{
    board::PadShape,
    ir::geometry::Point,
    pool::{PadstackAperture, PadstackLayerSpan},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
mod pins;
mod projection;
mod validation;
pub(crate) use pins::{capture_pad, capture_root, check_pin, continuity, source_value};
pub(crate) use projection::project_board_value;
pub use projection::{ComponentPadProjection, component_pad_projection};
pub(crate) use validation::{touches, touches_batch, validate_batch, validate_native};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PinnedPadLibrarySource {
    pub object_id: Uuid,
    pub owner_id: Uuid,
    pub source_shard_id: Uuid,
    pub relative_path: String,
    pub object_revision: ObjectRevision,
    pub owner_value_hash: String,
    pub record_hash: String,
    pub incarnation: Option<Uuid>,
    pub journal_tip: Option<Uuid>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FrozenPadGeometry {
    pub name: String,
    pub position: Point,
    pub layer: i32,
    pub aperture: PadstackAperture,
    pub drill_nm: Option<i64>,
    pub layer_span: PadstackLayerSpan,
    pub plated: Option<bool>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ComponentPadSourceBinding {
    pub project_id: Uuid,
    pub board_id: Uuid,
    pub package_id: Uuid,
    pub materialized_against: ModelRevision,
    pub part: PinnedPadLibrarySource,
    pub package: PinnedPadLibrarySource,
    pub land_pattern: PinnedPadLibrarySource,
    pub library_pad: PinnedPadLibrarySource,
    pub padstack: PinnedPadLibrarySource,
    pub component_instance: Option<RevisionedRef>,
    pub pin_pad_map: Option<PinnedPadLibrarySource>,
    pub frozen_geometry: FrozenPadGeometry,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComponentPadSource {
    pub uuid: Uuid,
    pub name: String,
    pub position: Point,
    pub padstack: Uuid,
    pub layer: i32,
    #[serde(default)]
    pub drill_nm: Option<i64>,
    #[serde(default)]
    pub shape: Option<PadShape>,
    #[serde(default)]
    pub diameter_nm: i64,
    #[serde(default)]
    pub width_nm: i64,
    #[serde(default)]
    pub height_nm: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_binding: Option<ComponentPadSourceBinding>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ComponentPadMaterializationStatus {
    Current,
    Stale,
    Unverified,
}
pub(crate) fn invalid(reason: impl Into<String>) -> EngineError {
    EngineError::Validation(format!("component Pad authority: {}", reason.into()))
}
impl ComponentPadSource {
    pub fn validate_frozen(&self) -> Result<(), EngineError> {
        let b = self
            .source_binding
            .as_ref()
            .ok_or_else(|| invalid("explicit journaled adoption required"))?;
        let g = &b.frozen_geometry;
        if b.library_pad.owner_id != b.land_pattern.object_id
            || b.library_pad.relative_path != b.land_pattern.relative_path
            || b.library_pad.source_shard_id != b.land_pattern.source_shard_id
            || b.padstack.object_id != self.padstack
        {
            return Err(invalid(
                "foreign pinned land-pattern/Padstack correspondence",
            ));
        }
        let aperture = match self.shape {
            Some(PadShape::Circle) => PadstackAperture::Circle {
                diameter_nm: self.diameter_nm,
            },
            Some(PadShape::Rect) => PadstackAperture::Rect {
                width_nm: self.width_nm,
                height_nm: self.height_nm,
            },
            _ => return Err(invalid("unsupported frozen aperture")),
        };
        if self.uuid.is_nil()
            || self.uuid == b.library_pad.object_id
            || self.name != g.name
            || self.position != g.position
            || self.layer != g.layer
            || self.padstack != b.padstack.object_id
            || self.drill_nm != g.drill_nm
            || aperture != g.aperture
        {
            return Err(invalid("frozen materialization/source binding mismatch"));
        }
        match aperture {
            PadstackAperture::Circle { diameter_nm } if diameter_nm > 0 => {}
            PadstackAperture::Rect {
                width_nm,
                height_nm,
            } if width_nm > 0 && height_nm > 0 => {}
            _ => return Err(invalid("nonpositive frozen aperture")),
        }
        if self.drill_nm.is_some_and(|d| d < 0) {
            return Err(invalid("negative frozen drill"));
        }
        Ok(())
    }
    pub fn materialization_status(
        &self,
        model: &DesignModel,
        part: Uuid,
        package: Uuid,
    ) -> ComponentPadMaterializationStatus {
        let Some(b) = &self.source_binding else {
            return ComponentPadMaterializationStatus::Unverified;
        };
        if self.validate_frozen().is_err() || b.project_id != model.project.project_id {
            return ComponentPadMaterializationStatus::Unverified;
        }
        if b.part.object_id != part
            || b.package.object_id != package
            || [
                &b.part,
                &b.package,
                &b.land_pattern,
                &b.library_pad,
                &b.padstack,
            ]
            .into_iter()
            .any(|p| check_pin(model, p).is_err())
        {
            ComponentPadMaterializationStatus::Stale
        } else {
            ComponentPadMaterializationStatus::Current
        }
    }
}
