use std::collections::HashMap;

use uuid::Uuid;

use crate::board::{Board, Keepout, PlacedPackage, Stackup, StackupLayer, StackupLayerType, Zone};
use crate::ir::geometry::{Point, Polygon};

pub(super) fn empty_board() -> Board {
    Board {
        uuid: Uuid::new_v4(),
        name: "drc-demo".into(),
        stackup: Stackup {
            layers: vec![StackupLayer::new(
                1,
                "F.Cu",
                StackupLayerType::Copper,
                35_000,
            )],
        },
        pad_expansion_setup: crate::board::PadExpansionSetup::default(),
        outline: Polygon::new(vec![
            Point::new(0, 0),
            Point::new(100_000_000, 0),
            Point::new(100_000_000, 100_000_000),
            Point::new(0, 100_000_000),
        ]),
        packages: HashMap::<Uuid, PlacedPackage>::new(),
        pads: HashMap::new(),
        tracks: HashMap::new(),
        vias: HashMap::new(),
        zones: HashMap::<Uuid, Zone>::new(),
        nets: HashMap::new(),
        net_classes: HashMap::new(),
        rules: Vec::new(),
        keepouts: Vec::<Keepout>::new(),
        dimensions: Vec::new(),
        texts: Vec::new(),
    }
}
