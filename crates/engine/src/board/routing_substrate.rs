use crate::ir::geometry::{LayerId, Point, Polygon};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::board::{
    Board, Keepout, Net, NetClass, PadShape, PlacedPad, StackupLayer, StackupLayerType, Track,
    Via, Zone,
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RoutingPadSource {
    BoardPad,
    ComponentPad,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutingComponentPad {
    pub component_uuid: Uuid,
    pub uuid: Uuid,
    pub name: String,
    pub position: Point,
    pub padstack_uuid: Uuid,
    pub layer: LayerId,
    pub drill_nm: Option<i64>,
    pub shape: Option<PadShape>,
    pub diameter_nm: i64,
    pub width_nm: i64,
    pub height_nm: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutingPadFact {
    pub source: RoutingPadSource,
    pub owner_uuid: Uuid,
    pub uuid: Uuid,
    pub name: String,
    pub net: Option<Uuid>,
    pub position: Point,
    pub layer: LayerId,
    pub padstack_uuid: Option<Uuid>,
    pub drill_nm: Option<i64>,
    pub shape: Option<PadShape>,
    pub diameter_nm: i64,
    pub width_nm: i64,
    pub height_nm: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutingSubstrateSummary {
    pub outline_vertex_count: usize,
    pub layer_count: usize,
    pub copper_layer_count: usize,
    pub keepout_count: usize,
    pub board_pad_count: usize,
    pub component_pad_count: usize,
    pub track_count: usize,
    pub via_count: usize,
    pub zone_count: usize,
    pub net_count: usize,
    pub net_class_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutingSubstrateReport {
    pub contract: String,
    pub persisted_native_board_state_only: bool,
    pub summary: RoutingSubstrateSummary,
    pub outline: Polygon,
    pub layers: Vec<StackupLayer>,
    pub copper_layer_ids: Vec<LayerId>,
    pub keepouts: Vec<Keepout>,
    pub pads: Vec<RoutingPadFact>,
    pub tracks: Vec<Track>,
    pub vias: Vec<Via>,
    pub zones: Vec<Zone>,
    pub nets: Vec<Net>,
    pub net_classes: Vec<NetClass>,
}

impl Board {
    pub fn routing_substrate(
        &self,
        component_pads: &[RoutingComponentPad],
    ) -> RoutingSubstrateReport {
        let mut layers = self.stackup.layers.clone();
        layers.sort_by(|a, b| a.id.cmp(&b.id).then_with(|| a.name.cmp(&b.name)));

        let copper_layer_ids = layers
            .iter()
            .filter(|layer| matches!(layer.layer_type, StackupLayerType::Copper))
            .map(|layer| layer.id)
            .collect::<Vec<_>>();

        let mut keepouts = self.keepouts.clone();
        keepouts.sort_by(|a, b| a.kind.cmp(&b.kind).then_with(|| a.uuid.cmp(&b.uuid)));

        let mut pads = self
            .pads
            .values()
            .map(RoutingPadFact::from_board_pad)
            .collect::<Vec<_>>();
        pads.extend(
            component_pads
                .iter()
                .cloned()
                .map(RoutingPadFact::from_component_pad),
        );
        pads.sort_by(|a, b| {
            a.source
                .cmp(&b.source)
                .then_with(|| a.owner_uuid.cmp(&b.owner_uuid))
                .then_with(|| a.name.cmp(&b.name))
                .then_with(|| a.uuid.cmp(&b.uuid))
        });

        let mut tracks = self.tracks.values().cloned().collect::<Vec<_>>();
        tracks.sort_by(|a, b| a.uuid.cmp(&b.uuid));

        let mut vias = self.vias.values().cloned().collect::<Vec<_>>();
        vias.sort_by(|a, b| a.uuid.cmp(&b.uuid));

        let mut zones = self.zones.values().cloned().collect::<Vec<_>>();
        zones.sort_by(|a, b| a.uuid.cmp(&b.uuid));

        let mut nets = self.nets.values().cloned().collect::<Vec<_>>();
        nets.sort_by(|a, b| a.name.cmp(&b.name).then_with(|| a.uuid.cmp(&b.uuid)));

        let mut net_classes = self.net_classes.values().cloned().collect::<Vec<_>>();
        net_classes.sort_by(|a, b| a.name.cmp(&b.name).then_with(|| a.uuid.cmp(&b.uuid)));

        let board_pad_count = pads
            .iter()
            .filter(|pad| matches!(pad.source, RoutingPadSource::BoardPad))
            .count();
        let component_pad_count = pads.len().saturating_sub(board_pad_count);

        RoutingSubstrateReport {
            contract: "m5_routing_substrate_v1".to_string(),
            persisted_native_board_state_only: true,
            summary: RoutingSubstrateSummary {
                outline_vertex_count: self.outline.vertices.len(),
                layer_count: layers.len(),
                copper_layer_count: copper_layer_ids.len(),
                keepout_count: keepouts.len(),
                board_pad_count,
                component_pad_count,
                track_count: tracks.len(),
                via_count: vias.len(),
                zone_count: zones.len(),
                net_count: nets.len(),
                net_class_count: net_classes.len(),
            },
            outline: self.outline.clone(),
            layers,
            copper_layer_ids,
            keepouts,
            pads,
            tracks,
            vias,
            zones,
            nets,
            net_classes,
        }
    }
}

impl RoutingPadFact {
    fn from_board_pad(pad: &PlacedPad) -> Self {
        Self {
            source: RoutingPadSource::BoardPad,
            owner_uuid: pad.package,
            uuid: pad.uuid,
            name: pad.name.clone(),
            net: pad.net,
            position: pad.position,
            layer: pad.layer,
            padstack_uuid: None,
            drill_nm: None,
            shape: Some(pad.shape),
            diameter_nm: pad.diameter,
            width_nm: pad.width,
            height_nm: pad.height,
        }
    }

    fn from_component_pad(pad: RoutingComponentPad) -> Self {
        Self {
            source: RoutingPadSource::ComponentPad,
            owner_uuid: pad.component_uuid,
            uuid: pad.uuid,
            name: pad.name,
            net: None,
            position: pad.position,
            layer: pad.layer,
            padstack_uuid: Some(pad.padstack_uuid),
            drill_nm: pad.drill_nm,
            shape: pad.shape,
            diameter_nm: pad.diameter_nm,
            width_nm: pad.width_nm,
            height_nm: pad.height_nm,
        }
    }
}
