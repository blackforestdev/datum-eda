use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use eda_engine::board::PadShape;
use eda_engine::board::{PlacedPackage, StackupLayer, StackupLayerType};
use eda_engine::ir::geometry::{Point, Polygon};
use eda_engine::pool::{Package, Padstack, PadstackAperture, Primitive};

use super::command_project_native_types::NativeComponentPad;
use super::command_project_gerber_mechanical::NativeComponentMechanicalPolygon;
use super::command_project_gerber_silkscreen::{
    NativeComponentSilkscreenArc, NativeComponentSilkscreenCircle, NativeComponentSilkscreenLine,
    NativeComponentSilkscreenPolygon, NativeComponentSilkscreenPolyline,
};
use super::{LoadedNativeProject, NativePoint, query_native_project_board_stackup};

const PACKAGE_COURTYARD_MECHANICAL_LAYER_ID: i32 = 41;

pub(crate) fn materialize_supported_pool_package_graphics(
    project: &mut LoadedNativeProject,
    component: &PlacedPackage,
) -> Result<()> {
    initialize_component_graphic_maps(project, component.uuid.to_string());

    let Some(package) = resolve_native_project_pool_package(project, component.package)? else {
        return Ok(());
    };
    let Some(silkscreen_layer) = resolve_component_silkscreen_layer(project, component.layer)?
    else {
        return Ok(());
    };

    let key = component.uuid.to_string();
    let mut silkscreen_lines = Vec::new();
    let mut silkscreen_arcs = Vec::new();
    let mut silkscreen_circles = Vec::new();
    let mut silkscreen_polygons = Vec::new();
    let mut silkscreen_polylines = Vec::new();
    let mut mechanical_polygons = Vec::new();
    let mut pads = package
        .pads
        .values()
        .cloned()
        .map(|pad| {
            let padstack = resolve_native_project_pool_padstack(project, pad.padstack)?;
            Ok(native_component_pad(pad, padstack.as_ref()))
        })
        .collect::<Result<Vec<_>>>()?;
    pads.sort_by(|a, b| a.name.cmp(&b.name).then_with(|| a.uuid.cmp(&b.uuid)));
    let models_3d = package.models_3d.clone();

    for primitive in package.silkscreen {
        match primitive {
            Primitive::Line { from, to, width } => {
                silkscreen_lines.push(NativeComponentSilkscreenLine {
                    from: native_point(from),
                    to: native_point(to),
                    width_nm: width,
                    layer: silkscreen_layer.id,
                })
            }
            Primitive::Rect { min, max, width } => {
                silkscreen_polygons.push(NativeComponentSilkscreenPolygon {
                    vertices: rect_vertices(min, max),
                    width_nm: width,
                    layer: silkscreen_layer.id,
                });
            }
            Primitive::Circle {
                center,
                radius,
                width,
            } => {
                silkscreen_circles.push(NativeComponentSilkscreenCircle {
                    center: native_point(center),
                    radius_nm: radius,
                    width_nm: width,
                    layer: silkscreen_layer.id,
                });
            }
            Primitive::Polygon { polygon, width } => {
                let vertices = polygon
                    .vertices
                    .into_iter()
                    .map(native_point)
                    .collect::<Vec<_>>();
                if polygon.closed {
                    silkscreen_polygons.push(NativeComponentSilkscreenPolygon {
                        vertices,
                        width_nm: width,
                        layer: silkscreen_layer.id,
                    });
                } else {
                    silkscreen_polylines.push(NativeComponentSilkscreenPolyline {
                        vertices,
                        width_nm: width,
                        layer: silkscreen_layer.id,
                    });
                }
            }
            Primitive::Arc { arc, width } => silkscreen_arcs.push(NativeComponentSilkscreenArc {
                center: native_point(arc.center),
                radius_nm: arc.radius,
                start_angle: arc.start_angle,
                end_angle: arc.end_angle,
                width_nm: width,
                layer: silkscreen_layer.id,
            }),
            Primitive::Text { .. } => {}
        }
    }

    if package.courtyard.closed && package.courtyard.vertices.len() >= 2 {
        mechanical_polygons.push(NativeComponentMechanicalPolygon {
            vertices: package
                .courtyard
                .vertices
                .into_iter()
                .map(native_point)
                .collect(),
            layer: PACKAGE_COURTYARD_MECHANICAL_LAYER_ID,
        });
    }

    project
        .board
        .component_silkscreen
        .insert(key.clone(), silkscreen_lines);
    project
        .board
        .component_silkscreen_arcs
        .insert(key.clone(), silkscreen_arcs);
    project
        .board
        .component_silkscreen_circles
        .insert(key.clone(), silkscreen_circles);
    project
        .board
        .component_silkscreen_polygons
        .insert(key.clone(), silkscreen_polygons);
    project
        .board
        .component_silkscreen_polylines
        .insert(key, silkscreen_polylines);
    project
        .board
        .component_mechanical_polygons
        .insert(component.uuid.to_string(), mechanical_polygons);
    project
        .board
        .component_pads
        .insert(component.uuid.to_string(), std::mem::take(&mut pads));
    project
        .board
        .component_models_3d
        .insert(component.uuid.to_string(), models_3d);

    Ok(())
}

fn initialize_component_graphic_maps(project: &mut LoadedNativeProject, key: String) {
    project
        .board
        .component_silkscreen
        .entry(key.clone())
        .or_default();
    project
        .board
        .component_silkscreen_texts
        .entry(key.clone())
        .or_default();
    project
        .board
        .component_silkscreen_arcs
        .entry(key.clone())
        .or_default();
    project
        .board
        .component_silkscreen_circles
        .entry(key.clone())
        .or_default();
    project
        .board
        .component_silkscreen_polygons
        .entry(key.clone())
        .or_default();
    project
        .board
        .component_silkscreen_polylines
        .entry(key.clone())
        .or_default();
    project
        .board
        .component_mechanical_lines
        .entry(key.clone())
        .or_default();
    project
        .board
        .component_mechanical_texts
        .entry(key.clone())
        .or_default();
    project
        .board
        .component_mechanical_polygons
        .entry(key.clone())
        .or_default();
    project
        .board
        .component_mechanical_polylines
        .entry(key.clone())
        .or_default();
    project
        .board
        .component_mechanical_circles
        .entry(key.clone())
        .or_default();
    project
        .board
        .component_mechanical_arcs
        .entry(key.clone())
        .or_default();
    project.board.component_pads.entry(key.clone()).or_default();
    project.board.component_models_3d.entry(key).or_default();
}

fn native_component_pad(
    pad: eda_engine::pool::Pad,
    padstack: Option<&Padstack>,
) -> NativeComponentPad {
    let (shape, diameter_nm, width_nm, height_nm) = match padstack.and_then(|padstack| padstack.aperture.as_ref()) {
        Some(PadstackAperture::Circle { diameter_nm }) => {
            (Some(PadShape::Circle), *diameter_nm, 0, 0)
        }
        Some(PadstackAperture::Rect {
            width_nm,
            height_nm,
        }) => (Some(PadShape::Rect), 0, *width_nm, *height_nm),
        None => (None, 0, 0, 0),
    };
    NativeComponentPad {
        uuid: pad.uuid,
        name: pad.name,
        position: native_point(pad.position),
        padstack: pad.padstack,
        layer: pad.layer,
        drill_nm: padstack.and_then(|padstack| padstack.drill_nm),
        shape,
        diameter_nm,
        width_nm,
        height_nm,
    }
}

fn resolve_native_project_pool_package(
    project: &LoadedNativeProject,
    package_uuid: uuid::Uuid,
) -> Result<Option<Package>> {
    let mut refs = project.manifest.pools.clone();
    refs.sort_by(|a, b| {
        a.priority
            .cmp(&b.priority)
            .then_with(|| a.path.cmp(&b.path))
    });
    for pool_ref in refs {
        let pool_root = resolve_native_project_pool_path(&project.root, &pool_ref.path);
        if !pool_root.exists() {
            return Err(anyhow::anyhow!(
                "native project pool path does not exist: {}",
                pool_root.display()
            ));
        }
        let package_path = pool_root
            .join("packages")
            .join(format!("{}.json", package_uuid));
        if !package_path.exists() {
            continue;
        }
        let package_text = std::fs::read_to_string(&package_path)
            .with_context(|| format!("failed to read {}", package_path.display()))?;
        let package: Package = serde_json::from_str(&package_text)
            .with_context(|| format!("failed to parse {}", package_path.display()))?;
        return Ok(Some(package));
    }
    Ok(None)
}

fn resolve_native_project_pool_padstack(
    project: &LoadedNativeProject,
    padstack_uuid: uuid::Uuid,
) -> Result<Option<Padstack>> {
    let mut refs = project.manifest.pools.clone();
    refs.sort_by(|a, b| {
        a.priority
            .cmp(&b.priority)
            .then_with(|| a.path.cmp(&b.path))
    });
    for pool_ref in refs {
        let pool_root = resolve_native_project_pool_path(&project.root, &pool_ref.path);
        if !pool_root.exists() {
            return Err(anyhow::anyhow!(
                "native project pool path does not exist: {}",
                pool_root.display()
            ));
        }
        let padstack_path = pool_root
            .join("padstacks")
            .join(format!("{}.json", padstack_uuid));
        if !padstack_path.exists() {
            continue;
        }
        let padstack_text = std::fs::read_to_string(&padstack_path)
            .with_context(|| format!("failed to read {}", padstack_path.display()))?;
        let padstack: Padstack = serde_json::from_str(&padstack_text)
            .with_context(|| format!("failed to parse {}", padstack_path.display()))?;
        return Ok(Some(padstack));
    }
    Ok(None)
}

pub(crate) fn resolve_native_project_pool_path(project_root: &Path, pool_path: &str) -> PathBuf {
    let path = PathBuf::from(pool_path);
    if path.is_absolute() {
        path
    } else {
        project_root.join(path)
    }
}

fn resolve_component_silkscreen_layer(
    project: &LoadedNativeProject,
    component_layer: i32,
) -> Result<Option<StackupLayer>> {
    let stackup = query_native_project_board_stackup(&project.root)?;
    Ok(stackup
        .into_iter()
        .filter(|entry| matches!(entry.layer_type, StackupLayerType::Silkscreen))
        .min_by(|a, b| {
            (i64::from((a.id - component_layer).abs()), a.id)
                .cmp(&(i64::from((b.id - component_layer).abs()), b.id))
        }))
}

fn native_point(point: Point) -> NativePoint {
    NativePoint {
        x: point.x,
        y: point.y,
    }
}

fn rect_vertices(min: Point, max: Point) -> Vec<NativePoint> {
    Polygon {
        vertices: vec![
            Point { x: min.x, y: min.y },
            Point { x: max.x, y: min.y },
            Point { x: max.x, y: max.y },
            Point { x: min.x, y: max.y },
        ],
        closed: true,
    }
    .vertices
    .into_iter()
    .map(native_point)
    .collect()
}
