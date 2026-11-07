use crate::{
    ir::geometry::Point,
    schematic::{Junction, SchematicWire, Sheet},
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WireEndpoint {
    From,
    To,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum WireHandle {
    Endpoint { end: WireEndpoint },
    Vertex { point: Point },
    Segment { from: Point, to: Point },
    InsertVertex { from: Point, to: Point },
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WireRouting {
    #[default]
    Free,
    Orthogonal,
    #[serde(rename = "diagonal_45")]
    Diagonal45,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WireCornerOrder {
    #[default]
    Start,
    End,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct WireRouteConfig {
    pub routing: WireRouting,
    pub corner: WireCornerOrder,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WireEditBasis {
    pub(super) sheet: Sheet,
    pub(super) wire_id: Uuid,
    pub(super) edit_seed: Uuid,
    pub(super) handles: Vec<WireHandle>,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WireEditAction {
    #[default]
    Move,
    Split,
    RemoveSpan,
    RemoveVertex,
}

/// Replacements are in `wires`; removals are disjoint, true deletions.
/// Each new physical edge names an existing raw source member to clone.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WireEditPlan {
    pub wires: BTreeMap<Uuid, SchematicWire>,
    pub removed_wire_ids: BTreeSet<Uuid>,
    pub junctions: BTreeMap<Uuid, Junction>,
    pub removed_junction_ids: BTreeSet<Uuid>,
    pub new_wire_origins: BTreeMap<Uuid, Uuid>,
    /// Every original raw member contributing to a merged output edge.
    #[serde(default)]
    pub coalesced_wire_origins: BTreeMap<Uuid, BTreeSet<Uuid>>,
}
impl WireEditPlan {
    pub fn apply_to_sheet(&self, source: &Sheet) -> Sheet {
        let mut sheet = source.clone();
        for id in &self.removed_wire_ids {
            sheet.wires.remove(id);
        }
        for (id, wire) in &self.wires {
            sheet.wires.insert(*id, wire.clone());
        }
        for id in &self.removed_junction_ids {
            sheet.junctions.remove(id);
        }
        for (id, junction) in &self.junctions {
            sheet.junctions.insert(*id, junction.clone());
        }
        sheet
    }
}
