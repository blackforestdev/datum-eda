use std::path::Path;

use anyhow::{Context, Result, bail};
use eda_engine::api::native_write::board_routing::{
    build_delete_board_net, build_delete_board_track, build_delete_board_via,
    build_delete_board_zone, build_place_board_net, build_place_board_track,
    build_place_board_via, build_place_board_zone, build_set_board_net, build_set_board_track,
    build_set_board_via, build_set_board_zone, build_set_zone_fills,
};
use eda_engine::api::native_write::{PreparedWrite, WriteProvenance, commit_prepared};
use eda_engine::board::{ImpedanceSpec, Net, Track, Via, Zone};
use eda_engine::error::EngineError;
use eda_engine::ir::geometry::{Point, Polygon};
use eda_engine::substrate::{
    DesignModel, ModelRevision, ProjectResolver, ZONE_FILL_SCHEMA_VERSION, ZoneFill,
    compute_bounded_zone_fill,
};
use serde::Serialize;
use uuid::Uuid;

use super::{
    NativeProjectBoardNetMutationReportView, NativeProjectBoardTrackMutationReportView,
    NativeProjectBoardViaMutationReportView, NativeProjectBoardZoneMutationReportView,
    load_native_project_with_resolved_board, native_project_board_net_report,
    native_project_board_track_report, native_project_board_via_report,
    native_project_board_zone_report,
};

use crate::command_project::cli_commit_source;

#[path = "command_project_zone_fill_context.rs"]
mod command_project_zone_fill_context;
pub(crate) use command_project_zone_fill_context::zone_fill_copper_context;

#[derive(Debug, Serialize)]
pub(crate) struct NativeProjectZoneFillsQueryView {
    pub(crate) contract: &'static str,
    pub(crate) project_id: String,
    pub(crate) model_revision: ModelRevision,
    pub(crate) zone_fill_count: usize,
    pub(crate) zone_fills: Vec<ZoneFill>,
}

#[derive(Debug, Serialize)]
pub(crate) struct NativeProjectFillZonesView {
    pub(crate) contract: &'static str,
    pub(crate) action: &'static str,
    pub(crate) project_id: String,
    pub(crate) model_revision: ModelRevision,
    pub(crate) requested_zone: Option<Uuid>,
    pub(crate) requested_net: Option<Uuid>,
    pub(crate) zone_fill_count: usize,
    pub(crate) zone_fills: Vec<ZoneFill>,
    pub(crate) zone_fill_paths: Vec<String>,
}

pub(crate) fn query_native_project_board_tracks(root: &Path) -> Result<Vec<Track>> {
    let project = load_native_project_with_resolved_board(root)?;
    let mut tracks = project
        .board
        .tracks
        .into_values()
        .map(|value| serde_json::from_value(value).context("failed to parse board track"))
        .collect::<Result<Vec<Track>>>()?;
    tracks.sort_by(|a, b| a.uuid.cmp(&b.uuid));
    Ok(tracks)
}

pub(crate) fn query_native_project_board_vias(root: &Path) -> Result<Vec<Via>> {
    let project = load_native_project_with_resolved_board(root)?;
    let mut vias = project
        .board
        .vias
        .into_values()
        .map(|value| serde_json::from_value(value).context("failed to parse board via"))
        .collect::<Result<Vec<Via>>>()?;
    vias.sort_by(|a, b| a.uuid.cmp(&b.uuid));
    Ok(vias)
}

pub(crate) fn query_native_project_board_zones(root: &Path) -> Result<Vec<Zone>> {
    let project = load_native_project_with_resolved_board(root)?;
    let mut zones = project
        .board
        .zones
        .values()
        .cloned()
        .map(|value| serde_json::from_value(value).context("failed to parse board zone"))
        .collect::<Result<Vec<Zone>>>()?;
    zones.sort_by(|a, b| a.uuid.cmp(&b.uuid));
    Ok(zones)
}

pub(crate) fn query_native_project_zone_fills(
    root: &Path,
) -> Result<NativeProjectZoneFillsQueryView> {
    let model = ProjectResolver::new(root).resolve()?;
    let zone_fills: Vec<ZoneFill> = model.zone_fills.into_values().collect();
    Ok(NativeProjectZoneFillsQueryView {
        contract: "zone_fills_query_v1",
        project_id: model.project.project_id.to_string(),
        model_revision: model.model_revision,
        zone_fill_count: zone_fills.len(),
        zone_fills,
    })
}

pub(crate) fn fill_native_project_zones(
    root: &Path,
    requested_zone: Option<Uuid>,
    requested_net: Option<Uuid>,
) -> Result<NativeProjectFillZonesView> {
    let model = ProjectResolver::new(root).resolve()?;
    let project = load_native_project_with_resolved_board(root)?;
    let mut zones = project
        .board
        .zones
        .values()
        .cloned()
        .map(|value| serde_json::from_value(value).context("failed to parse board zone"))
        .collect::<Result<Vec<Zone>>>()?;
    zones.retain(|zone| {
        requested_zone.is_none_or(|filter| zone.uuid == filter)
            && requested_net.is_none_or(|filter| zone.net == filter)
    });
    if zones.is_empty() {
        bail!("no board zones matched fill request");
    }
    zones.sort_by(|a, b| a.uuid.cmp(&b.uuid));

    let project_id = model.project.project_id.to_string();
    let expected_model_revision = model.model_revision.clone();
    let mut zone_fills = Vec::new();
    let mut zone_fill_paths = Vec::new();
    for zone in zones {
        let source_zone_revision = model
            .objects
            .get(&zone.uuid)
            .map(|object| object.object_revision)
            .unwrap_or(eda_engine::substrate::ObjectRevision(0));
        let fill_context = zone_fill_copper_context(&project.board)?;
        let (state, islands, provenance) = compute_bounded_zone_fill(&zone, &fill_context);
        let fill = ZoneFill {
            schema_version: ZONE_FILL_SCHEMA_VERSION,
            zone_id: zone.uuid,
            state,
            source_zone_revision,
            model_revision: model.model_revision.clone(),
            islands,
            provenance: Some(provenance),
        };
        let relative_path = format!(".datum/zone_fills/{}.json", fill.zone_id);
        zone_fill_paths.push(root.join(&relative_path).display().to_string());
        zone_fills.push(fill);
    }
    if !zone_fills.is_empty() {
        let mut model = model;
        let prepared = build_set_zone_fills(&model, cli_provenance("fill zones")?, &zone_fills)?;
        commit_prepared(&mut model, root, prepared)?;
    }

    Ok(NativeProjectFillZonesView {
        contract: "zone_fill_generate_v1",
        action: "fill_zones",
        project_id,
        model_revision: expected_model_revision,
        requested_zone,
        requested_net,
        zone_fill_count: zone_fills.len(),
        zone_fills,
        zone_fill_paths,
    })
}

pub(crate) fn query_native_project_board_nets(root: &Path) -> Result<Vec<Net>> {
    let project = load_native_project_with_resolved_board(root)?;
    let mut nets = project
        .board
        .nets
        .into_values()
        .map(|value| serde_json::from_value(value).context("failed to parse board net"))
        .collect::<Result<Vec<Net>>>()?;
    nets.sort_by(|a, b| a.name.cmp(&b.name).then_with(|| a.uuid.cmp(&b.uuid)));
    Ok(nets)
}

pub(crate) fn query_native_project_board_net(root: &Path, net_uuid: Uuid) -> Result<Net> {
    let project = load_native_project_with_resolved_board(root)?;
    let key = net_uuid.to_string();
    let entry = project
        .board
        .nets
        .get(&key)
        .cloned()
        .with_context(|| format!("board net not found in native project: {net_uuid}"))?;
    serde_json::from_value(entry).context("failed to parse board net")
}

pub(crate) fn place_native_project_board_net(
    root: &Path,
    name: String,
    class_uuid: Uuid,
    impedance_target_ohms: Option<String>,
    impedance_tolerance_pct: Option<String>,
    controlled_dielectric_layer: Option<i32>,
) -> Result<NativeProjectBoardNetMutationReportView> {
    let net_uuid = Uuid::new_v4();
    let controlled_impedance = parse_controlled_impedance(
        impedance_target_ohms,
        impedance_tolerance_pct,
        controlled_dielectric_layer,
    )?;
    let net = Net {
        uuid: net_uuid,
        name,
        class: class_uuid,
        controlled_impedance,
    };
    commit_board_routing_write(root, "place board net", |model, provenance| {
        build_place_board_net(model, provenance, &net)
    })?;
    let project = load_native_project_with_resolved_board(root)?;
    Ok(native_project_board_net_report(
        "place_board_net",
        &project,
        net,
    ))
}

pub(crate) fn place_native_project_board_track(
    root: &Path,
    net_uuid: Uuid,
    from: Point,
    to: Point,
    width_nm: i64,
    layer: i32,
) -> Result<NativeProjectBoardTrackMutationReportView> {
    let project = load_native_project_with_resolved_board(root)?;
    if !project.board.nets.contains_key(&net_uuid.to_string()) {
        bail!("board net not found in native project: {net_uuid}");
    }
    let track_uuid = Uuid::new_v4();
    let track = Track {
        uuid: track_uuid,
        net: net_uuid,
        from,
        to,
        width: width_nm,
        layer,
    };
    commit_board_routing_write(root, "draw board track", |model, provenance| {
        build_place_board_track(model, provenance, &track)
    })?;
    let project = load_native_project_with_resolved_board(root)?;
    Ok(native_project_board_track_report(
        "draw_board_track",
        &project,
        track,
    ))
}

pub(crate) fn place_native_project_board_via(
    root: &Path,
    net_uuid: Uuid,
    position: Point,
    drill_nm: i64,
    diameter_nm: i64,
    from_layer: i32,
    to_layer: i32,
) -> Result<NativeProjectBoardViaMutationReportView> {
    let project = load_native_project_with_resolved_board(root)?;
    if !project.board.nets.contains_key(&net_uuid.to_string()) {
        bail!("board net not found in native project: {net_uuid}");
    }
    let via_uuid = Uuid::new_v4();
    let via = Via {
        uuid: via_uuid,
        net: net_uuid,
        position,
        drill: drill_nm,
        diameter: diameter_nm,
        from_layer,
        to_layer,
    };
    commit_board_routing_write(root, "place board via", |model, provenance| {
        build_place_board_via(model, provenance, &via)
    })?;
    let project = load_native_project_with_resolved_board(root)?;
    Ok(native_project_board_via_report(
        "place_board_via",
        &project,
        via,
    ))
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn edit_native_project_board_track(
    root: &Path,
    track_uuid: Uuid,
    net_uuid: Option<Uuid>,
    from: Option<Point>,
    to: Option<Point>,
    width_nm: Option<i64>,
    layer: Option<i32>,
) -> Result<NativeProjectBoardTrackMutationReportView> {
    let project = load_native_project_with_resolved_board(root)?;
    if let Some(net_uuid) = net_uuid {
        if !project.board.nets.contains_key(&net_uuid.to_string()) {
            bail!("board net not found in native project: {net_uuid}");
        }
    }
    let value = project
        .board
        .tracks
        .get(&track_uuid.to_string())
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("board track not found in native project: {track_uuid}"))?;
    let mut track: Track = serde_json::from_value(value).with_context(|| {
        format!(
            "failed to parse board track in {}",
            project.board_path.display()
        )
    })?;
    if let Some(net_uuid) = net_uuid {
        track.net = net_uuid;
    }
    if let Some(from) = from {
        track.from = from;
    }
    if let Some(to) = to {
        track.to = to;
    }
    if let Some(width_nm) = width_nm {
        track.width = width_nm;
    }
    if let Some(layer) = layer {
        track.layer = layer;
    }
    commit_board_routing_write(root, "edit board track", |model, provenance| {
        build_set_board_track(model, provenance, &track)
    })?;
    let project = load_native_project_with_resolved_board(root)?;
    Ok(native_project_board_track_report(
        "edit_board_track",
        &project,
        track,
    ))
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn edit_native_project_board_via(
    root: &Path,
    via_uuid: Uuid,
    net_uuid: Option<Uuid>,
    position: Option<Point>,
    drill_nm: Option<i64>,
    diameter_nm: Option<i64>,
    from_layer: Option<i32>,
    to_layer: Option<i32>,
) -> Result<NativeProjectBoardViaMutationReportView> {
    let project = load_native_project_with_resolved_board(root)?;
    if let Some(net_uuid) = net_uuid {
        if !project.board.nets.contains_key(&net_uuid.to_string()) {
            bail!("board net not found in native project: {net_uuid}");
        }
    }
    let value = project
        .board
        .vias
        .get(&via_uuid.to_string())
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("board via not found in native project: {via_uuid}"))?;
    let mut via: Via = serde_json::from_value(value).with_context(|| {
        format!(
            "failed to parse board via in {}",
            project.board_path.display()
        )
    })?;
    if let Some(net_uuid) = net_uuid {
        via.net = net_uuid;
    }
    if let Some(position) = position {
        via.position = position;
    }
    if let Some(drill_nm) = drill_nm {
        via.drill = drill_nm;
    }
    if let Some(diameter_nm) = diameter_nm {
        via.diameter = diameter_nm;
    }
    if let Some(from_layer) = from_layer {
        via.from_layer = from_layer;
    }
    if let Some(to_layer) = to_layer {
        via.to_layer = to_layer;
    }
    commit_board_routing_write(root, "edit board via", |model, provenance| {
        build_set_board_via(model, provenance, &via)
    })?;
    let project = load_native_project_with_resolved_board(root)?;
    Ok(native_project_board_via_report(
        "edit_board_via",
        &project,
        via,
    ))
}

pub(crate) fn place_native_project_board_zone(
    root: &Path,
    net_uuid: Uuid,
    polygon: Polygon,
    layer: i32,
    priority: u32,
    thermal_relief: bool,
    thermal_gap_nm: i64,
    thermal_spoke_width_nm: i64,
) -> Result<NativeProjectBoardZoneMutationReportView> {
    let project = load_native_project_with_resolved_board(root)?;
    if !project.board.nets.contains_key(&net_uuid.to_string()) {
        bail!("board net not found in native project: {net_uuid}");
    }
    let zone_uuid = Uuid::new_v4();
    let zone = Zone {
        uuid: zone_uuid,
        net: net_uuid,
        polygon,
        layer,
        priority,
        thermal_relief,
        thermal_gap: thermal_gap_nm,
        thermal_spoke_width: thermal_spoke_width_nm,
    };
    commit_board_routing_write(root, "place board zone", |model, provenance| {
        build_place_board_zone(model, provenance, &zone)
    })?;
    let project = load_native_project_with_resolved_board(root)?;
    Ok(native_project_board_zone_report(
        "place_board_zone",
        &project,
        zone,
    ))
}

pub(crate) fn edit_native_project_board_zone(
    root: &Path,
    zone_uuid: Uuid,
    net_uuid: Option<Uuid>,
    polygon: Option<Polygon>,
    layer: Option<i32>,
    priority: Option<u32>,
    thermal_relief: Option<bool>,
    thermal_gap_nm: Option<i64>,
    thermal_spoke_width_nm: Option<i64>,
) -> Result<NativeProjectBoardZoneMutationReportView> {
    if net_uuid.is_none()
        && polygon.is_none()
        && layer.is_none()
        && priority.is_none()
        && thermal_relief.is_none()
        && thermal_gap_nm.is_none()
        && thermal_spoke_width_nm.is_none()
    {
        bail!("edit-board-zone requires at least one replacement field");
    }
    let project = load_native_project_with_resolved_board(root)?;
    let value = project
        .board
        .zones
        .get(&zone_uuid.to_string())
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("board zone not found in native project: {zone_uuid}"))?;
    let mut zone: Zone = serde_json::from_value(value).with_context(|| {
        format!(
            "failed to parse board zone in {}",
            project.board_path.display()
        )
    })?;
    if let Some(net_uuid) = net_uuid {
        if !project.board.nets.contains_key(&net_uuid.to_string()) {
            bail!("board net not found in native project: {net_uuid}");
        }
        zone.net = net_uuid;
    }
    if let Some(polygon) = polygon {
        zone.polygon = polygon;
    }
    if let Some(layer) = layer {
        zone.layer = layer;
    }
    if let Some(priority) = priority {
        zone.priority = priority;
    }
    if let Some(thermal_relief) = thermal_relief {
        zone.thermal_relief = thermal_relief;
    }
    if let Some(thermal_gap_nm) = thermal_gap_nm {
        zone.thermal_gap = thermal_gap_nm;
    }
    if let Some(thermal_spoke_width_nm) = thermal_spoke_width_nm {
        zone.thermal_spoke_width = thermal_spoke_width_nm;
    }
    commit_board_routing_write(root, "edit board zone", |model, provenance| {
        build_set_board_zone(model, provenance, &zone)
    })?;
    let project = load_native_project_with_resolved_board(root)?;
    Ok(native_project_board_zone_report(
        "edit_board_zone",
        &project,
        zone,
    ))
}

pub(crate) fn edit_native_project_board_net(
    root: &Path,
    net_uuid: Uuid,
    name: Option<String>,
    class_uuid: Option<Uuid>,
    impedance_target_ohms: Option<String>,
    impedance_tolerance_pct: Option<String>,
    controlled_dielectric_layer: Option<i32>,
    clear_controlled_impedance: bool,
) -> Result<NativeProjectBoardNetMutationReportView> {
    let project = load_native_project_with_resolved_board(root)?;
    let key = net_uuid.to_string();
    let entry = project
        .board
        .nets
        .get(&key)
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("board net not found in native project: {net_uuid}"))?;
    let mut net: Net = serde_json::from_value(entry).with_context(|| {
        format!(
            "failed to parse board net in {}",
            project.board_path.display()
        )
    })?;
    if let Some(name) = name {
        net.name = name;
    }
    if let Some(class_uuid) = class_uuid {
        net.class = class_uuid;
    }
    if clear_controlled_impedance {
        if impedance_target_ohms.is_some()
            || impedance_tolerance_pct.is_some()
            || controlled_dielectric_layer.is_some()
        {
            bail!(
                "--clear-controlled-impedance cannot be combined with impedance replacement fields"
            );
        }
        net.controlled_impedance = None;
    } else if impedance_target_ohms.is_some()
        || impedance_tolerance_pct.is_some()
        || controlled_dielectric_layer.is_some()
    {
        net.controlled_impedance = Some(update_controlled_impedance(
            net.controlled_impedance,
            impedance_target_ohms,
            impedance_tolerance_pct,
            controlled_dielectric_layer,
        )?);
    }
    commit_board_routing_write(root, "edit board net", |model, provenance| {
        build_set_board_net(model, provenance, &net)
    })?;
    let project = load_native_project_with_resolved_board(root)?;
    Ok(native_project_board_net_report(
        "edit_board_net",
        &project,
        net,
    ))
}

fn parse_controlled_impedance(
    target_ohms: Option<String>,
    tolerance_pct: Option<String>,
    controlled_dielectric: Option<i32>,
) -> Result<Option<ImpedanceSpec>> {
    if target_ohms.is_none() && tolerance_pct.is_none() && controlled_dielectric.is_none() {
        return Ok(None);
    }
    let target_ohms = target_ohms.ok_or_else(|| {
        anyhow::anyhow!("--impedance-target-ohms is required when authoring controlled impedance")
    })?;
    let tolerance_pct = tolerance_pct.ok_or_else(|| {
        anyhow::anyhow!("--impedance-tolerance-pct is required when authoring controlled impedance")
    })?;
    Ok(Some(ImpedanceSpec {
        target_ohms: parse_positive_json_number(&target_ohms, "impedance target ohms")?,
        tolerance_pct: parse_non_negative_json_number(&tolerance_pct, "impedance tolerance pct")?,
        controlled_dielectric,
    }))
}

fn update_controlled_impedance(
    existing: Option<ImpedanceSpec>,
    target_ohms: Option<String>,
    tolerance_pct: Option<String>,
    controlled_dielectric: Option<i32>,
) -> Result<ImpedanceSpec> {
    let Some(mut spec) = existing else {
        return parse_controlled_impedance(target_ohms, tolerance_pct, controlled_dielectric)?
            .ok_or_else(|| anyhow::anyhow!("controlled impedance replacement is empty"));
    };
    if let Some(target_ohms) = target_ohms {
        spec.target_ohms = parse_positive_json_number(&target_ohms, "impedance target ohms")?;
    }
    if let Some(tolerance_pct) = tolerance_pct {
        spec.tolerance_pct =
            parse_non_negative_json_number(&tolerance_pct, "impedance tolerance pct")?;
    }
    if let Some(controlled_dielectric) = controlled_dielectric {
        spec.controlled_dielectric = Some(controlled_dielectric);
    }
    Ok(spec)
}

fn parse_positive_json_number(value: &str, field: &str) -> Result<serde_json::Number> {
    let number = parse_non_negative_json_number(value, field)?;
    if number.as_f64().is_some_and(|value| value > 0.0) {
        Ok(number)
    } else {
        bail!("{field} must be a positive JSON number");
    }
}

fn parse_non_negative_json_number(value: &str, field: &str) -> Result<serde_json::Number> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        bail!("{field} must be a non-negative JSON number");
    }
    let number: serde_json::Number = serde_json::from_str(trimmed)
        .with_context(|| format!("{field} must be a non-negative JSON number"))?;
    if number.as_f64().is_some_and(|value| value >= 0.0) {
        Ok(number)
    } else {
        bail!("{field} must be a non-negative JSON number");
    }
}

pub(crate) fn delete_native_project_board_track(
    root: &Path,
    track_uuid: Uuid,
) -> Result<NativeProjectBoardTrackMutationReportView> {
    let project = load_native_project_with_resolved_board(root)?;
    let value = project
        .board
        .tracks
        .get(&track_uuid.to_string())
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("board track not found in native project: {track_uuid}"))?;
    let track: Track = serde_json::from_value(value.clone()).with_context(|| {
        format!(
            "failed to parse board track in {}",
            project.board_path.display()
        )
    })?;
    commit_board_routing_write(root, "delete board track", |model, provenance| {
        build_delete_board_track(model, provenance, track_uuid, value)
    })?;
    Ok(native_project_board_track_report(
        "delete_board_track",
        &project,
        track,
    ))
}

fn cli_provenance(reason: &str) -> Result<WriteProvenance> {
    Ok(WriteProvenance::new("datum-eda-cli", cli_commit_source()?, reason))
}

fn commit_board_routing_write<F>(root: &Path, reason: &str, build: F) -> Result<()>
where
    F: FnOnce(&DesignModel, WriteProvenance) -> Result<PreparedWrite, EngineError>,
{
    let mut model = ProjectResolver::new(root).resolve()?;
    let prepared = build(&model, cli_provenance(reason)?)?;
    commit_prepared(&mut model, root, prepared)?;
    Ok(())
}

pub(crate) fn delete_native_project_board_via(
    root: &Path,
    via_uuid: Uuid,
) -> Result<NativeProjectBoardViaMutationReportView> {
    let project = load_native_project_with_resolved_board(root)?;
    let value = project
        .board
        .vias
        .get(&via_uuid.to_string())
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("board via not found in native project: {via_uuid}"))?;
    let via: Via = serde_json::from_value(value.clone()).with_context(|| {
        format!(
            "failed to parse board via in {}",
            project.board_path.display()
        )
    })?;
    commit_board_routing_write(root, "delete board via", |model, provenance| {
        build_delete_board_via(model, provenance, via_uuid, value)
    })?;
    Ok(native_project_board_via_report(
        "delete_board_via",
        &project,
        via,
    ))
}

pub(crate) fn delete_native_project_board_zone(
    root: &Path,
    zone_uuid: Uuid,
) -> Result<NativeProjectBoardZoneMutationReportView> {
    let project = load_native_project_with_resolved_board(root)?;
    let value = project
        .board
        .zones
        .get(&zone_uuid.to_string())
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("board zone not found in native project: {zone_uuid}"))?;
    let zone: Zone = serde_json::from_value(value.clone()).with_context(|| {
        format!(
            "failed to parse board zone in {}",
            project.board_path.display()
        )
    })?;
    commit_board_routing_write(root, "delete board zone", |model, provenance| {
        build_delete_board_zone(model, provenance, zone_uuid, value)
    })?;
    Ok(native_project_board_zone_report(
        "delete_board_zone",
        &project,
        zone,
    ))
}

pub(crate) fn delete_native_project_board_net(
    root: &Path,
    net_uuid: Uuid,
) -> Result<NativeProjectBoardNetMutationReportView> {
    let project = load_native_project_with_resolved_board(root)?;
    let value = project
        .board
        .nets
        .get(&net_uuid.to_string())
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("board net not found in native project: {net_uuid}"))?;
    let net: Net = serde_json::from_value(value.clone()).with_context(|| {
        format!(
            "failed to parse board net in {}",
            project.board_path.display()
        )
    })?;
    commit_board_routing_write(root, "delete board net", |model, provenance| {
        build_delete_board_net(model, provenance, net_uuid, value)
    })?;
    Ok(native_project_board_net_report("delete_board_net", &project, net))
}
