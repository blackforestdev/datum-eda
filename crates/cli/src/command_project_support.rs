use super::*;

pub(crate) fn parse_native_polygon_vertices(vertices: &[String]) -> Result<Polygon> {
    if vertices.len() < 3 {
        bail!("polygon requires at least three --vertex entries");
    }
    let points = vertices
        .iter()
        .map(|value| {
            let (x, y) = value
                .split_once(':')
                .ok_or_else(|| anyhow::anyhow!("vertex must be x_nm:y_nm, got `{value}`"))?;
            Ok(Point {
                x: x.parse::<i64>()?,
                y: y.parse::<i64>()?,
            })
        })
        .collect::<Result<Vec<Point>>>()?;
    Ok(Polygon {
        vertices: points,
        closed: true,
    })
}

pub(crate) fn parse_native_stackup_layers(layers: &[String]) -> Result<Vec<StackupLayer>> {
    if layers.is_empty() {
        bail!("stackup requires at least one --layer entry");
    }
    layers
        .iter()
        .map(|value| {
            let parts = value.splitn(4, ':').collect::<Vec<_>>();
            if parts.len() != 4 {
                bail!("layer must be id:name:type:thickness_nm, got `{value}`");
            }
            Ok(StackupLayer {
                id: parts[0].parse::<i32>()?,
                name: parts[1].to_string(),
                layer_type: parse_stackup_layer_type(parts[2])?,
                thickness_nm: parts[3].parse::<i64>()?,
            })
        })
        .collect()
}

fn parse_stackup_layer_type(value: &str) -> Result<StackupLayerType> {
    match value {
        "Copper" | "copper" => Ok(StackupLayerType::Copper),
        "Dielectric" | "dielectric" => Ok(StackupLayerType::Dielectric),
        "SolderMask" | "soldermask" | "solder_mask" => Ok(StackupLayerType::SolderMask),
        "Silkscreen" | "silkscreen" => Ok(StackupLayerType::Silkscreen),
        "Paste" | "paste" => Ok(StackupLayerType::Paste),
        "Mechanical" | "mechanical" => Ok(StackupLayerType::Mechanical),
        _ => bail!("unknown stackup layer type `{value}`"),
    }
}

pub(super) fn native_project_board_net_class_report(
    action: &str,
    project: &LoadedNativeProject,
    net_class: NetClass,
) -> NativeProjectBoardNetClassMutationReportView {
    NativeProjectBoardNetClassMutationReportView {
        action: action.to_string(),
        project_root: project.root.display().to_string(),
        board_path: project.board_path.display().to_string(),
        net_class_uuid: net_class.uuid.to_string(),
        name: net_class.name,
        clearance_nm: net_class.clearance,
        track_width_nm: net_class.track_width,
        via_drill_nm: net_class.via_drill,
        via_diameter_nm: net_class.via_diameter,
        diffpair_width_nm: net_class.diffpair_width,
        diffpair_gap_nm: net_class.diffpair_gap,
    }
}

pub(super) fn native_project_board_net_report(
    action: &str,
    project: &LoadedNativeProject,
    net: Net,
) -> NativeProjectBoardNetMutationReportView {
    NativeProjectBoardNetMutationReportView {
        action: action.to_string(),
        project_root: project.root.display().to_string(),
        board_path: project.board_path.display().to_string(),
        net_uuid: net.uuid.to_string(),
        name: net.name,
        class_uuid: net.class.to_string(),
    }
}

pub(super) fn native_project_board_component_report(
    action: &str,
    project: &LoadedNativeProject,
    component: PlacedPackage,
) -> NativeProjectBoardComponentMutationReportView {
    let key = component.uuid.to_string();
    NativeProjectBoardComponentMutationReportView {
        action: action.to_string(),
        project_root: project.root.display().to_string(),
        board_path: project.board_path.display().to_string(),
        component_uuid: key.clone(),
        part_uuid: component.part.to_string(),
        package_uuid: component.package.to_string(),
        reference: component.reference,
        value: component.value,
        x_nm: component.position.x,
        y_nm: component.position.y,
        rotation_deg: component.rotation,
        layer: component.layer,
        locked: component.locked,
        has_persisted_component_silkscreen: component_has_persisted_silkscreen(project, &key),
        persisted_component_silkscreen_text_count: component_graphic_count(
            &project.board.component_silkscreen_texts,
            &key,
        ),
        persisted_component_silkscreen_line_count: component_graphic_count(
            &project.board.component_silkscreen,
            &key,
        ),
        persisted_component_silkscreen_arc_count: component_graphic_count(
            &project.board.component_silkscreen_arcs,
            &key,
        ),
        persisted_component_silkscreen_circle_count: component_graphic_count(
            &project.board.component_silkscreen_circles,
            &key,
        ),
        persisted_component_silkscreen_polygon_count: component_graphic_count(
            &project.board.component_silkscreen_polygons,
            &key,
        ),
        persisted_component_silkscreen_polyline_count: component_graphic_count(
            &project.board.component_silkscreen_polylines,
            &key,
        ),
        has_persisted_component_mechanical: component_has_persisted_mechanical(project, &key),
        persisted_component_mechanical_text_count: component_graphic_count(
            &project.board.component_mechanical_texts,
            &key,
        ),
        persisted_component_mechanical_line_count: component_graphic_count(
            &project.board.component_mechanical_lines,
            &key,
        ),
        persisted_component_mechanical_arc_count: component_graphic_count(
            &project.board.component_mechanical_arcs,
            &key,
        ),
        persisted_component_mechanical_circle_count: component_graphic_count(
            &project.board.component_mechanical_circles,
            &key,
        ),
        persisted_component_mechanical_polygon_count: component_graphic_count(
            &project.board.component_mechanical_polygons,
            &key,
        ),
        persisted_component_mechanical_polyline_count: component_graphic_count(
            &project.board.component_mechanical_polylines,
            &key,
        ),
        has_persisted_component_pads: component_package_pad_count(project, &key) > 0,
        persisted_component_pad_count: component_package_pad_count(project, &key),
        has_persisted_component_models_3d: component_model_count(project, &key) > 0,
        persisted_component_model_3d_count: component_model_count(project, &key),
    }
}

pub(super) fn native_project_board_track_report(
    action: &str,
    project: &LoadedNativeProject,
    track: Track,
) -> NativeProjectBoardTrackMutationReportView {
    NativeProjectBoardTrackMutationReportView {
        action: action.to_string(),
        project_root: project.root.display().to_string(),
        board_path: project.board_path.display().to_string(),
        track_uuid: track.uuid.to_string(),
        net_uuid: track.net.to_string(),
        from_x_nm: track.from.x,
        from_y_nm: track.from.y,
        to_x_nm: track.to.x,
        to_y_nm: track.to.y,
        width_nm: track.width,
        layer: track.layer,
    }
}

pub(super) fn native_project_board_via_report(
    action: &str,
    project: &LoadedNativeProject,
    via: Via,
) -> NativeProjectBoardViaMutationReportView {
    NativeProjectBoardViaMutationReportView {
        action: action.to_string(),
        project_root: project.root.display().to_string(),
        board_path: project.board_path.display().to_string(),
        via_uuid: via.uuid.to_string(),
        net_uuid: via.net.to_string(),
        x_nm: via.position.x,
        y_nm: via.position.y,
        drill_nm: via.drill,
        diameter_nm: via.diameter,
        from_layer: via.from_layer,
        to_layer: via.to_layer,
    }
}

pub(super) fn native_project_board_zone_report(
    action: &str,
    project: &LoadedNativeProject,
    zone: Zone,
) -> NativeProjectBoardZoneMutationReportView {
    NativeProjectBoardZoneMutationReportView {
        action: action.to_string(),
        project_root: project.root.display().to_string(),
        board_path: project.board_path.display().to_string(),
        zone_uuid: zone.uuid.to_string(),
        net_uuid: zone.net.to_string(),
        layer: zone.layer,
        priority: zone.priority,
        thermal_relief: zone.thermal_relief,
        thermal_gap_nm: zone.thermal_gap,
        thermal_spoke_width_nm: zone.thermal_spoke_width,
        vertex_count: zone.polygon.vertices.len(),
    }
}

pub(super) fn native_project_board_pad_report(
    action: &str,
    project: &LoadedNativeProject,
    pad: PlacedPad,
) -> NativeProjectBoardPadMutationReportView {
    NativeProjectBoardPadMutationReportView {
        action: action.to_string(),
        project_root: project.root.display().to_string(),
        board_path: project.board_path.display().to_string(),
        pad_uuid: pad.uuid.to_string(),
        package_uuid: pad.package.to_string(),
        name: pad.name,
        net_uuid: pad.net.map(|uuid| uuid.to_string()),
        x_nm: pad.position.x,
        y_nm: pad.position.y,
        layer: pad.layer,
        shape: match pad.shape {
            PadShape::Circle => "circle".to_string(),
            PadShape::Rect => "rect".to_string(),
        },
        diameter_nm: pad.diameter,
        width_nm: pad.width,
        height_nm: pad.height,
    }
}

pub(crate) fn parse_native_field_position(
    x_nm: Option<i64>,
    y_nm: Option<i64>,
) -> Result<Option<Point>> {
    match (x_nm, y_nm) {
        (None, None) => Ok(None),
        (Some(x), Some(y)) => Ok(Some(Point { x, y })),
        _ => bail!("field position requires both --x-nm and --y-nm"),
    }
}

pub(super) fn write_canonical_json<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    let json = to_json_deterministic(value).context("failed to serialize canonical JSON")?;
    std::fs::write(path, format!("{json}\n"))
        .with_context(|| format!("failed to write {}", path.display()))
}

pub(super) fn csv_escape(value: &str) -> String {
    if value.contains([',', '"', '\n']) {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_string()
    }
}

pub(super) fn parse_csv_line(line: &str) -> Result<Vec<String>> {
    let mut fields = Vec::new();
    let mut current = String::new();
    let mut chars = line.chars().peekable();
    let mut in_quotes = false;
    while let Some(ch) = chars.next() {
        match ch {
            '"' => {
                if in_quotes && chars.peek() == Some(&'"') {
                    current.push('"');
                    chars.next();
                } else {
                    in_quotes = !in_quotes;
                }
            }
            ',' if !in_quotes => {
                fields.push(current);
                current = String::new();
            }
            _ => current.push(ch),
        }
    }
    if in_quotes {
        bail!("unterminated quoted field");
    }
    fields.push(current);
    Ok(fields)
}

pub(super) fn native_outline_to_polygon(outline: &NativeOutline) -> Polygon {
    Polygon {
        vertices: outline
            .vertices
            .iter()
            .map(|point| Point {
                x: point.x,
                y: point.y,
            })
            .collect(),
        closed: outline.closed,
    }
}
