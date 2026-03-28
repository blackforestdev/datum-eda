use std::path::Path;

use anyhow::{Context, Result, bail};
use eda_engine::board::{Net, Track, Via, Zone};
use eda_engine::ir::geometry::{Point, Polygon};
use uuid::Uuid;

use super::{
    NativeProjectBoardNetMutationReportView, NativeProjectBoardTrackMutationReportView,
    NativeProjectBoardViaMutationReportView, NativeProjectBoardZoneMutationReportView,
    load_native_project, native_project_board_net_report, native_project_board_track_report,
    native_project_board_via_report, native_project_board_zone_report, write_canonical_json,
};

pub(crate) fn query_native_project_board_tracks(root: &Path) -> Result<Vec<Track>> {
    let project = load_native_project(root)?;
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
    let project = load_native_project(root)?;
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
    let project = load_native_project(root)?;
    let mut zones = project
        .board
        .zones
        .into_values()
        .map(|value| serde_json::from_value(value).context("failed to parse board zone"))
        .collect::<Result<Vec<Zone>>>()?;
    zones.sort_by(|a, b| a.uuid.cmp(&b.uuid));
    Ok(zones)
}

pub(crate) fn query_native_project_board_nets(root: &Path) -> Result<Vec<Net>> {
    let project = load_native_project(root)?;
    let mut nets = project
        .board
        .nets
        .into_values()
        .map(|value| serde_json::from_value(value).context("failed to parse board net"))
        .collect::<Result<Vec<Net>>>()?;
    nets.sort_by(|a, b| a.name.cmp(&b.name).then_with(|| a.uuid.cmp(&b.uuid)));
    Ok(nets)
}

pub(crate) fn place_native_project_board_net(
    root: &Path,
    name: String,
    class_uuid: Uuid,
) -> Result<NativeProjectBoardNetMutationReportView> {
    let mut project = load_native_project(root)?;
    let net_uuid = Uuid::new_v4();
    let net = Net {
        uuid: net_uuid,
        name,
        class: class_uuid,
    };
    project.board.nets.insert(
        net_uuid.to_string(),
        serde_json::to_value(&net).expect("native board net serialization must succeed"),
    );
    write_canonical_json(&project.board_path, &project.board)?;
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
    let mut project = load_native_project(root)?;
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
    project.board.tracks.insert(
        track_uuid.to_string(),
        serde_json::to_value(&track).expect("native board track serialization must succeed"),
    );
    write_canonical_json(&project.board_path, &project.board)?;
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
    let mut project = load_native_project(root)?;
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
    project.board.vias.insert(
        via_uuid.to_string(),
        serde_json::to_value(&via).expect("native board via serialization must succeed"),
    );
    write_canonical_json(&project.board_path, &project.board)?;
    Ok(native_project_board_via_report(
        "place_board_via",
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
    let mut project = load_native_project(root)?;
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
    project.board.zones.insert(
        zone_uuid.to_string(),
        serde_json::to_value(&zone).expect("native board zone serialization must succeed"),
    );
    write_canonical_json(&project.board_path, &project.board)?;
    Ok(native_project_board_zone_report(
        "place_board_zone",
        &project,
        zone,
    ))
}

pub(crate) fn edit_native_project_board_net(
    root: &Path,
    net_uuid: Uuid,
    name: Option<String>,
    class_uuid: Option<Uuid>,
) -> Result<NativeProjectBoardNetMutationReportView> {
    let mut project = load_native_project(root)?;
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
    project.board.nets.insert(
        key,
        serde_json::to_value(&net).expect("native board net serialization must succeed"),
    );
    write_canonical_json(&project.board_path, &project.board)?;
    Ok(native_project_board_net_report(
        "edit_board_net",
        &project,
        net,
    ))
}

pub(crate) fn delete_native_project_board_track(
    root: &Path,
    track_uuid: Uuid,
) -> Result<NativeProjectBoardTrackMutationReportView> {
    let mut project = load_native_project(root)?;
    let value = project
        .board
        .tracks
        .remove(&track_uuid.to_string())
        .ok_or_else(|| anyhow::anyhow!("board track not found in native project: {track_uuid}"))?;
    let track: Track = serde_json::from_value(value).with_context(|| {
        format!(
            "failed to parse board track in {}",
            project.board_path.display()
        )
    })?;
    write_canonical_json(&project.board_path, &project.board)?;
    Ok(native_project_board_track_report(
        "delete_board_track",
        &project,
        track,
    ))
}

pub(crate) fn delete_native_project_board_via(
    root: &Path,
    via_uuid: Uuid,
) -> Result<NativeProjectBoardViaMutationReportView> {
    let mut project = load_native_project(root)?;
    let value = project
        .board
        .vias
        .remove(&via_uuid.to_string())
        .ok_or_else(|| anyhow::anyhow!("board via not found in native project: {via_uuid}"))?;
    let via: Via = serde_json::from_value(value).with_context(|| {
        format!(
            "failed to parse board via in {}",
            project.board_path.display()
        )
    })?;
    write_canonical_json(&project.board_path, &project.board)?;
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
    let mut project = load_native_project(root)?;
    let value = project
        .board
        .zones
        .remove(&zone_uuid.to_string())
        .ok_or_else(|| anyhow::anyhow!("board zone not found in native project: {zone_uuid}"))?;
    let zone: Zone = serde_json::from_value(value).with_context(|| {
        format!(
            "failed to parse board zone in {}",
            project.board_path.display()
        )
    })?;
    write_canonical_json(&project.board_path, &project.board)?;
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
    let mut project = load_native_project(root)?;
    let value = project
        .board
        .nets
        .remove(&net_uuid.to_string())
        .ok_or_else(|| anyhow::anyhow!("board net not found in native project: {net_uuid}"))?;
    let net: Net = serde_json::from_value(value).with_context(|| {
        format!(
            "failed to parse board net in {}",
            project.board_path.display()
        )
    })?;
    write_canonical_json(&project.board_path, &project.board)?;
    Ok(native_project_board_net_report(
        "delete_board_net",
        &project,
        net,
    ))
}
