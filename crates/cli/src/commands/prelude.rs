// commands/prelude — the shared scope prelude for command families.
//
// Wave 2 endgame move: absorbs the legacy command_project_prelude.rs (std
// re-exports) and the external-crate re-export block that lived at the
// bottom of command_project_surface.rs (both hosts dissolved). Family files
// reach these names through their `use super::*;` / `use crate::*;` chains,
// exactly as they did through the legacy command_project scope.

pub(crate) use std::collections::{BTreeMap, BTreeSet, HashMap};

pub(crate) use eda_engine::api::{CheckCodeCount, CheckSummary};
pub(crate) use eda_engine::board::{
    Board, BoardText, Dimension, Keepout, Net, NetClass, PadAperture, PadShape, PlacedPackage,
    PlacedPad, Stackup, StackupLayer, StackupLayerType, Track, Via, Zone,
};
pub(crate) use eda_engine::connectivity::{schematic_diagnostics, schematic_net_info};
pub(crate) use eda_engine::erc::run_prechecks;
pub(crate) use eda_engine::export::{
    render_rs274x_copper_layer, render_rs274x_outline_default, render_rs274x_paste_layer,
    render_rs274x_silkscreen_layer, render_rs274x_soldermask_layer,
};
pub(crate) use eda_engine::import::ids_sidecar::compute_source_hash_bytes;
pub(crate) use eda_engine::ir::geometry::Polygon;
pub(crate) use eda_engine::ir::geometry::{Arc, Point};
pub(crate) use eda_engine::ir::serialization::to_json_deterministic;
pub(crate) use eda_engine::schematic::{
    Bus, BusEntry, CheckWaiver, HiddenPowerBehavior, HierarchicalPort, Junction, LabelKind,
    NetLabel, NoConnectMarker, PinDisplayOverride, PlacedSymbol, PortDirection, Schematic,
    SchematicPrimitive, SchematicText, SchematicWire, Sheet, SheetDefinition, SheetFrame,
    SheetInstance, SymbolDisplayMode, SymbolField, SymbolFieldInfo, SymbolPin,
};
