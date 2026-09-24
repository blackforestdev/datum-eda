use super::*;

pub(crate) fn query_native_project_board_tracks(root: &Path) -> Result<Vec<Track>> {
    let project = load_native_project_with_resolved_board(root)?;
    let mut tracks = project
        .board
        .tracks
        .into_values()
        .map(|value| serde_json::from_value(value).context("failed to parse board track"))
        .collect::<Result<Vec<Track>>>()?;
    tracks.sort_by_key(|a| a.uuid);
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
    vias.sort_by_key(|a| a.uuid);
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
    zones.sort_by_key(|a| a.uuid);
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
