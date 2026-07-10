use anyhow::{Context, Result};
use eda_engine::ir::geometry::Point;
use eda_engine::schematic::{Schematic, SchematicPrimitive, Sheet};
use std::path::{Path, PathBuf};
use uuid::Uuid;

use super::*;

const SCHEMATIC_DRAWING_LAYER: &str = "L37";
const SCHEMATIC_FRAME_LAYER: &str = "L44";
const SCHEMATIC_STROKE_NM: i64 = 120_000;
const SYMBOL_HALF_WIDTH_NM: i64 = 1_600_000;
const SYMBOL_HALF_HEIGHT_NM: i64 = 900_000;
const JUNCTION_RADIUS_NM: i64 = 180_000;
const LABEL_HALF_WIDTH_NM: i64 = 1_100_000;
const LABEL_HALF_HEIGHT_NM: i64 = 300_000;
const PORT_HALF_WIDTH_NM: i64 = 1_300_000;
const PORT_HALF_HEIGHT_NM: i64 = 450_000;
const NOCONNECT_HALF_NM: i64 = 260_000;
const SHEET_INSTANCE_HALF_WIDTH_NM: i64 = 2_200_000;
const SHEET_INSTANCE_HALF_HEIGHT_NM: i64 = 1_400_000;
const SCHEMATIC_BOUNDS_PAD_NM: i64 = 5_000_000;

pub fn load_kicad_schematic_workspace_state(schematic_file: &Path) -> Result<ReviewWorkspaceState> {
    let source = schematic_file.canonicalize().with_context(|| {
        format!(
            "failed to resolve KiCad schematic {}",
            schematic_file.display()
        )
    })?;
    let (scene, schematic_path) = load_scene_from_kicad_schematic_import(&source)?;
    let request = LiveReviewRequest {
        project_root: source
            .parent()
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(".")),
        board_file: None,
        artifact_path: None,
        net_uuid: None,
        from_anchor_pad_uuid: None,
        to_anchor_pad_uuid: None,
        profile: None,
        kicad_board_source: None,
    };
    let mut state = ReviewWorkspaceState::new(scene, empty_route_review_payload(&request));
    state.backing = Some(WorkspaceBacking {
        request,
        board_path: schematic_path,
    });
    Ok(state)
}

pub(crate) fn load_scene_from_kicad_schematic_import(
    schematic_file: &Path,
) -> Result<(BoardReviewSceneV1, PathBuf)> {
    let import_started = std::time::Instant::now();
    let (schematic, report) = eda_engine::import::kicad::import_schematic_document(schematic_file)
        .map_err(|e| anyhow::anyhow!("import {}: {e}", schematic_file.display()))?;
    for warning in &report.warnings {
        eprintln!(
            "datum-import warning [{}]: {warning}",
            schematic_file.display()
        );
    }
    trace_protocol_timing(format!(
        "kicad schematic import {}ms warnings={}",
        import_started.elapsed().as_millis(),
        report.warnings.len()
    ));

    let root_sheet = root_sheet(&schematic).context("imported schematic has no root sheet")?;
    let project_name = schematic_file
        .file_stem()
        .and_then(|value| value.to_str())
        .filter(|value| !value.is_empty())
        .unwrap_or("schematic")
        .to_string();
    let project_uuid = schematic.uuid.to_string();
    let sheet_uuid = root_sheet.uuid.to_string();
    let inspect = ProjectInspectPayload {
        project_root: schematic_file
            .parent()
            .unwrap_or(Path::new("."))
            .display()
            .to_string(),
        project_name: project_name.clone(),
        project_uuid: project_uuid.clone(),
        board_uuid: sheet_uuid.clone(),
        board_path: schematic_file.display().to_string(),
    };

    let mut graphics = Vec::new();
    let mut points = Vec::new();
    push_root_sheet_graphics(root_sheet, &schematic, &mut graphics, &mut points);
    let outline = schematic_outline(&points);
    let mut scene = build_board_review_scene(
        &inspect,
        outline,
        Vec::new(),
        Vec::new(),
        Vec::new(),
        ScenePadExpansionSetup::default(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        graphics,
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        SCHEMATIC_FRAME_LAYER.to_string(),
    );
    scene.kind = "schematic_review_scene".to_string();
    scene.scene_id = format!("schematic-review-scene:{sheet_uuid}");
    scene.board_name = root_sheet.name.clone();
    scene.source_revision = format!("schematic:{project_uuid}:sheet:{sheet_uuid}");
    scene.layers = schematic_scene_layers();
    Ok((scene, schematic_file.to_path_buf()))
}

fn root_sheet(schematic: &Schematic) -> Option<&Sheet> {
    schematic
        .sheet_definitions
        .values()
        .min_by(|a, b| a.name.cmp(&b.name).then_with(|| a.uuid.cmp(&b.uuid)))
        .and_then(|definition| schematic.sheets.get(&definition.root_sheet))
        .or_else(|| {
            schematic
                .sheets
                .values()
                .min_by(|a, b| a.name.cmp(&b.name).then_with(|| a.uuid.cmp(&b.uuid)))
        })
}

fn push_root_sheet_graphics(
    sheet: &Sheet,
    schematic: &Schematic,
    graphics: &mut Vec<BoardGraphicPrimitive>,
    points: &mut Vec<PointNm>,
) {
    for wire in sorted_values(&sheet.wires) {
        push_line_graphic(
            graphics,
            points,
            format!("schematic-wire:{}", wire.uuid),
            wire.uuid,
            wire.from,
            wire.to,
            SCHEMATIC_STROKE_NM,
        );
    }
    for drawing in sorted_values(&sheet.drawings) {
        push_drawing_graphic(graphics, points, drawing);
    }
    for symbol in sorted_values(&sheet.symbols) {
        let label = if symbol.reference.is_empty() {
            symbol
                .lib_id
                .clone()
                .unwrap_or_else(|| "symbol".to_string())
        } else {
            symbol.reference.clone()
        };
        push_rect_graphic(
            graphics,
            points,
            format!("schematic-symbol:{}", symbol.uuid),
            symbol.uuid,
            symbol.position,
            SYMBOL_HALF_WIDTH_NM,
            SYMBOL_HALF_HEIGHT_NM,
            Some(label),
        );
    }
    for label in sorted_values(&sheet.labels) {
        push_rect_graphic(
            graphics,
            points,
            format!("schematic-label:{}", label.uuid),
            label.uuid,
            label.position,
            LABEL_HALF_WIDTH_NM,
            LABEL_HALF_HEIGHT_NM,
            Some(label.name.clone()),
        );
    }
    for port in sorted_values(&sheet.ports) {
        push_rect_graphic(
            graphics,
            points,
            format!("schematic-port:{}", port.uuid),
            port.uuid,
            port.position,
            PORT_HALF_WIDTH_NM,
            PORT_HALF_HEIGHT_NM,
            Some(port.name.clone()),
        );
    }
    for junction in sorted_values(&sheet.junctions) {
        push_circle_graphic(
            graphics,
            points,
            format!("schematic-junction:{}", junction.uuid),
            junction.uuid,
            junction.position,
            JUNCTION_RADIUS_NM,
        );
    }
    for noconnect in sorted_values(&sheet.noconnects) {
        push_cross_graphic(
            graphics,
            points,
            format!("schematic-noconnect:{}", noconnect.uuid),
            noconnect.uuid,
            noconnect.position,
            NOCONNECT_HALF_NM,
        );
    }
    for text in sorted_values(&sheet.texts) {
        push_rect_graphic(
            graphics,
            points,
            format!("schematic-text:{}", text.uuid),
            text.uuid,
            text.position,
            LABEL_HALF_WIDTH_NM,
            LABEL_HALF_HEIGHT_NM,
            Some(text.text.clone()),
        );
    }
    for instance in sorted_values(&schematic.sheet_instances) {
        if instance.parent_sheet != Some(sheet.uuid) {
            continue;
        }
        push_rect_graphic(
            graphics,
            points,
            format!("schematic-sheet-instance:{}", instance.uuid),
            instance.uuid,
            instance.position,
            SHEET_INSTANCE_HALF_WIDTH_NM,
            SHEET_INSTANCE_HALF_HEIGHT_NM,
            Some(instance.name.clone()),
        );
    }
}

fn sorted_values<T>(map: &std::collections::HashMap<Uuid, T>) -> Vec<&T> {
    let mut entries = map.iter().collect::<Vec<_>>();
    entries.sort_by_key(|(uuid, _)| **uuid);
    entries.into_iter().map(|(_, value)| value).collect()
}

fn push_drawing_graphic(
    graphics: &mut Vec<BoardGraphicPrimitive>,
    points: &mut Vec<PointNm>,
    drawing: &SchematicPrimitive,
) {
    match drawing {
        SchematicPrimitive::Line { uuid, from, to } => push_line_graphic(
            graphics,
            points,
            format!("schematic-drawing-line:{uuid}"),
            *uuid,
            *from,
            *to,
            SCHEMATIC_STROKE_NM,
        ),
        SchematicPrimitive::Rect { uuid, min, max } => {
            let center = Point {
                x: (min.x + max.x) / 2,
                y: (min.y + max.y) / 2,
            };
            push_rect_graphic(
                graphics,
                points,
                format!("schematic-drawing-rect:{uuid}"),
                *uuid,
                center,
                (max.x - min.x).abs() / 2,
                (max.y - min.y).abs() / 2,
                None,
            );
        }
        SchematicPrimitive::Circle {
            uuid,
            center,
            radius,
        } => push_circle_graphic(
            graphics,
            points,
            format!("schematic-drawing-circle:{uuid}"),
            *uuid,
            *center,
            (*radius).max(SCHEMATIC_STROKE_NM),
        ),
        SchematicPrimitive::Arc { uuid, arc } => {
            let path = arc_path_points(*arc);
            points.extend(path.iter().copied());
            graphics.push(BoardGraphicPrimitive {
                object_id: format!("schematic-drawing-arc:{uuid}"),
                object_kind: "schematic_graphic".to_string(),
                primitive_kind: "polyline".to_string(),
                source_object_uuid: uuid.to_string(),
                layer_id: SCHEMATIC_DRAWING_LAYER.to_string(),
                path,
                holes: Vec::new(),
                width_nm: Some(SCHEMATIC_STROKE_NM),
            });
        }
    }
}

fn arc_path_points(arc: eda_engine::ir::geometry::Arc) -> Vec<PointNm> {
    let mut sweep = arc.end_angle - arc.start_angle;
    if sweep == 0 {
        sweep = 3600;
    }
    if sweep < 0 {
        sweep += 3600;
    }
    let steps = ((sweep.abs() / 150).max(4) as usize).min(48);
    (0..=steps)
        .map(|step| {
            let angle_tenths = arc.start_angle as f64 + (sweep as f64 * step as f64 / steps as f64);
            let radians = (angle_tenths / 10.0).to_radians();
            PointNm {
                x: arc.center.x + (radians.cos() * arc.radius as f64).round() as i64,
                y: arc.center.y + (radians.sin() * arc.radius as f64).round() as i64,
            }
        })
        .collect()
}

fn push_line_graphic(
    graphics: &mut Vec<BoardGraphicPrimitive>,
    points: &mut Vec<PointNm>,
    object_id: String,
    source_uuid: Uuid,
    from: Point,
    to: Point,
    width_nm: i64,
) {
    let path = vec![point_nm(from), point_nm(to)];
    points.extend(path.iter().copied());
    graphics.push(BoardGraphicPrimitive {
        object_id,
        object_kind: "schematic_graphic".to_string(),
        primitive_kind: "line".to_string(),
        source_object_uuid: source_uuid.to_string(),
        layer_id: SCHEMATIC_DRAWING_LAYER.to_string(),
        path,
        holes: Vec::new(),
        width_nm: Some(width_nm),
    });
}

// Scene import threads many primitive-geometry parameters.
#[allow(clippy::too_many_arguments)]
fn push_rect_graphic(
    graphics: &mut Vec<BoardGraphicPrimitive>,
    points: &mut Vec<PointNm>,
    object_id: String,
    source_uuid: Uuid,
    center: Point,
    half_width_nm: i64,
    half_height_nm: i64,
    _label: Option<String>,
) {
    let center = point_nm(center);
    let path = vec![
        PointNm {
            x: center.x - half_width_nm,
            y: center.y - half_height_nm,
        },
        PointNm {
            x: center.x + half_width_nm,
            y: center.y - half_height_nm,
        },
        PointNm {
            x: center.x + half_width_nm,
            y: center.y + half_height_nm,
        },
        PointNm {
            x: center.x - half_width_nm,
            y: center.y + half_height_nm,
        },
    ];
    points.extend(path.iter().copied());
    graphics.push(BoardGraphicPrimitive {
        object_id,
        object_kind: "schematic_graphic".to_string(),
        primitive_kind: "polygon".to_string(),
        source_object_uuid: source_uuid.to_string(),
        layer_id: SCHEMATIC_DRAWING_LAYER.to_string(),
        path,
        holes: Vec::new(),
        width_nm: Some(SCHEMATIC_STROKE_NM),
    });
}

fn push_circle_graphic(
    graphics: &mut Vec<BoardGraphicPrimitive>,
    points: &mut Vec<PointNm>,
    object_id: String,
    source_uuid: Uuid,
    center: Point,
    radius_nm: i64,
) {
    let center = point_nm(center);
    let mut path = Vec::new();
    for step in 0..24 {
        let theta = (step as f64 / 24.0) * std::f64::consts::TAU;
        path.push(PointNm {
            x: center.x + (theta.cos() * radius_nm as f64).round() as i64,
            y: center.y + (theta.sin() * radius_nm as f64).round() as i64,
        });
    }
    points.extend(path.iter().copied());
    graphics.push(BoardGraphicPrimitive {
        object_id,
        object_kind: "schematic_graphic".to_string(),
        primitive_kind: "polygon".to_string(),
        source_object_uuid: source_uuid.to_string(),
        layer_id: SCHEMATIC_DRAWING_LAYER.to_string(),
        path,
        holes: Vec::new(),
        width_nm: Some(SCHEMATIC_STROKE_NM),
    });
}

fn push_cross_graphic(
    graphics: &mut Vec<BoardGraphicPrimitive>,
    points: &mut Vec<PointNm>,
    object_id: String,
    source_uuid: Uuid,
    center: Point,
    half_nm: i64,
) {
    let center = point_nm(center);
    let a = PointNm {
        x: center.x - half_nm,
        y: center.y - half_nm,
    };
    let b = PointNm {
        x: center.x + half_nm,
        y: center.y + half_nm,
    };
    let c = PointNm {
        x: center.x - half_nm,
        y: center.y + half_nm,
    };
    let d = PointNm {
        x: center.x + half_nm,
        y: center.y - half_nm,
    };
    points.extend([a, b, c, d]);
    graphics.push(BoardGraphicPrimitive {
        object_id: format!("{object_id}:a"),
        object_kind: "schematic_graphic".to_string(),
        primitive_kind: "line".to_string(),
        source_object_uuid: source_uuid.to_string(),
        layer_id: SCHEMATIC_DRAWING_LAYER.to_string(),
        path: vec![a, b],
        holes: Vec::new(),
        width_nm: Some(SCHEMATIC_STROKE_NM),
    });
    graphics.push(BoardGraphicPrimitive {
        object_id: format!("{object_id}:b"),
        object_kind: "schematic_graphic".to_string(),
        primitive_kind: "line".to_string(),
        source_object_uuid: source_uuid.to_string(),
        layer_id: SCHEMATIC_DRAWING_LAYER.to_string(),
        path: vec![c, d],
        holes: Vec::new(),
        width_nm: Some(SCHEMATIC_STROKE_NM),
    });
}

fn schematic_outline(points: &[PointNm]) -> OutlinePayload {
    let (min_x, min_y, max_x, max_y) = if points.is_empty() {
        (
            -SCHEMATIC_BOUNDS_PAD_NM,
            -SCHEMATIC_BOUNDS_PAD_NM,
            SCHEMATIC_BOUNDS_PAD_NM,
            SCHEMATIC_BOUNDS_PAD_NM,
        )
    } else {
        let min_x = points.iter().map(|point| point.x).min().unwrap() - SCHEMATIC_BOUNDS_PAD_NM;
        let min_y = points.iter().map(|point| point.y).min().unwrap() - SCHEMATIC_BOUNDS_PAD_NM;
        let max_x = points.iter().map(|point| point.x).max().unwrap() + SCHEMATIC_BOUNDS_PAD_NM;
        let max_y = points.iter().map(|point| point.y).max().unwrap() + SCHEMATIC_BOUNDS_PAD_NM;
        (min_x, min_y, max_x, max_y)
    };
    OutlinePayload {
        vertices: vec![
            PointNm { x: min_x, y: min_y },
            PointNm { x: max_x, y: min_y },
            PointNm { x: max_x, y: max_y },
            PointNm { x: min_x, y: max_y },
        ],
        closed: true,
    }
}

fn schematic_scene_layers() -> Vec<SceneLayer> {
    vec![
        SceneLayer {
            layer_id: SCHEMATIC_FRAME_LAYER.to_string(),
            name: "Edge.Cuts".to_string(),
            kind: "mechanical".to_string(),
            render_order: 0,
            visible_by_default: true,
        },
        SceneLayer {
            layer_id: SCHEMATIC_DRAWING_LAYER.to_string(),
            name: "F.SilkS".to_string(),
            kind: "schematic".to_string(),
            render_order: 1,
            visible_by_default: true,
        },
    ]
}

fn point_nm(point: Point) -> PointNm {
    PointNm {
        x: point.x,
        y: point.y,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_kicad_schematic_projects_to_visible_review_scene() {
        let schematic = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../engine/testdata/import/kicad/simple-demo.kicad_sch");
        let state = load_kicad_schematic_workspace_state(&schematic)
            .expect("simple schematic should load as a review scene");

        assert_eq!(state.scene.kind, "schematic_review_scene");
        assert_eq!(state.scene.board_name, "Sub");
        assert!(
            state
                .scene
                .layers
                .iter()
                .any(|layer| layer.kind == "schematic" && layer.visible_by_default)
        );
        assert!(
            state
                .scene
                .board_graphics
                .iter()
                .any(|graphic| graphic.object_id.starts_with("schematic-wire:")),
            "schematic wires should be visible through review-scene graphics"
        );
        assert!(
            state
                .scene
                .board_graphics
                .iter()
                .any(|graphic| graphic.object_id.starts_with("schematic-symbol:")),
            "schematic symbols should have first-pass visible placeholders"
        );
    }
}
