//! Generated copper projects through the engine's existing current-fill owner.
use crate::PointNm;
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ZonePrimitive {
    pub object_id: String,
    pub object_kind: String,
    pub source_object_uuid: String,
    pub zone_uuid: String,
    pub net_uuid: Option<String>,
    pub layer_id: String,
    pub polygon: Vec<PointNm>,
    #[serde(default, skip_serializing_if = "ZoneFillProjection::is_legacy")]
    pub fill_projection: ZoneFillProjection,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(tag = "state", content = "value", rename_all = "snake_case")]
pub enum ZoneFillProjection {
    /// Existing compatibility/import scenes keep their recorded nominal paint.
    #[default]
    LegacyNominal,
    /// Complete certified current occupied polygons; empty is a positive state.
    Current(Vec<Vec<PointNm>>),
    Unavailable(String),
}
impl ZoneFillProjection {
    pub fn is_legacy(&self) -> bool {
        matches!(self, Self::LegacyNominal)
    }
    pub fn capture(model: &eda_engine::substrate::DesignModel, zone: uuid::Uuid) -> Self {
        let Some(fill) = model.current_zone_fill(zone) else {
            return Self::Unavailable(
                "Current generated fill is absent, stale or unavailable".into(),
            );
        };
        for polygon in &fill.islands {
            if let Err(reason) = eda_engine::board::occupied_copper::validate_polygon(polygon) {
                return Self::Unavailable(format!(
                    "Generated occupied geometry is not certified: {reason:?}"
                ));
            }
        }
        Self::Current(
            fill.islands
                .iter()
                .map(|p| {
                    p.vertices
                        .iter()
                        .map(|p| PointNm { x: p.x, y: p.y })
                        .collect()
                })
                .collect(),
        )
    }
}
impl ZonePrimitive {
    pub fn occupied_polygons(&self) -> impl Iterator<Item = &[PointNm]> {
        let legacy = (self.fill_projection.is_legacy() && self.polygon.len() >= 4)
            .then_some(self.polygon.as_slice());
        let current = match &self.fill_projection {
            ZoneFillProjection::Current(polygons) => Some(polygons.as_slice()),
            _ => None,
        };
        legacy
            .into_iter()
            .chain(current.into_iter().flatten().map(Vec::as_slice))
    }
}
