// commands/prelude — the shared scope prelude for command families.
//
// Wave 2 endgame move: absorbs the legacy command_project_prelude.rs (std
// re-exports) and the external-crate re-export block that lived at the
// bottom of command_project_surface.rs (both hosts dissolved). Family files
// reach these names through their `use super::*;` / `use crate::*;` chains,
// exactly as they did through the legacy command_project scope.

pub(crate) use std::collections::{BTreeMap, BTreeSet, HashMap};
pub(crate) use std::path::{Path, PathBuf};

pub(crate) use anyhow::{Context, Result, bail};
pub(crate) use eda_engine::api::{CheckCodeCount, CheckReport, CheckStatus, CheckSummary};
pub(crate) use eda_engine::board::{
    Board, BoardText, Dimension, Keepout, Net, NetClass, PadAperture, PadShape, PlacedPackage,
    PlacedPad, Stackup, StackupLayer, StackupLayerType, Track, Via, Zone,
};
pub(crate) use eda_engine::connectivity::{schematic_diagnostics, schematic_net_info};
pub(crate) use eda_engine::erc::{ErcFinding, run_prechecks};
pub(crate) use eda_engine::export::{
    render_rs274x_copper_layer, render_rs274x_outline_default, render_rs274x_paste_layer,
    render_rs274x_silkscreen_layer, render_rs274x_soldermask_layer,
};
pub(crate) use eda_engine::import::ids_sidecar::compute_source_hash_bytes;
pub(crate) use eda_engine::ir::geometry::Polygon;
pub(crate) use eda_engine::ir::geometry::{Arc, Point};
pub(crate) use eda_engine::ir::serialization::to_json_deterministic;
pub(crate) use eda_engine::rules::ast::Rule;
pub(crate) use eda_engine::schematic::{
    Bus, BusEntry, BusEntryInfo, BusInfo, CheckWaiver, ConnectivityDiagnosticInfo,
    HiddenPowerBehavior, HierarchicalPort, HierarchyInfo, Junction, LabelInfo, LabelKind, NetLabel,
    NoConnectInfo, NoConnectMarker, PinDisplayOverride, PlacedSymbol, PortDirection, PortInfo,
    Schematic, SchematicNetInfo, SchematicPrimitive, SchematicText, SchematicWire, Sheet,
    SheetDefinition, SheetFrame, SheetInstance, SymbolDisplayMode, SymbolField, SymbolFieldInfo,
    SymbolInfo, SymbolPin,
};
pub(crate) use serde::{Deserialize, Serialize};
pub(crate) use uuid::Uuid;
