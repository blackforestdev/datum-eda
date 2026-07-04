use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use eda_engine::board::PadShape;
use eda_engine::board::{PlacedPackage, StackupLayer, StackupLayerType};
use eda_engine::ir::geometry::{Point, Polygon};
use eda_engine::pool::{
    Footprint, ModelRef, Package, Pad, Padstack, PadstackAperture, Part, Primitive,
};
use eda_engine::substrate::{DesignModel, ProjectResolver, SourceShardTaxon};

use crate::NativeComponentMechanicalPolygon;
use crate::{
    NativeComponentSilkscreenArc, NativeComponentSilkscreenCircle, NativeComponentSilkscreenLine,
    NativeComponentSilkscreenPolygon, NativeComponentSilkscreenPolyline,
    NativeComponentSilkscreenText,
};
use super::command_project_native_types::NativeComponentPad;
use super::{LoadedNativeProject, NativePoint, query_native_project_board_stackup};

const PACKAGE_COURTYARD_MECHANICAL_LAYER_ID: i32 = 41;

pub(crate) fn materialize_supported_pool_package_graphics(
    project: &mut LoadedNativeProject,
    component: &PlacedPackage,
) -> Result<()> {
    initialize_component_graphic_maps(project, component.uuid.to_string());
    let model = ProjectResolver::new(&project.root)
        .resolve()
        .with_context(|| {
            format!(
                "failed to resolve native project {}",
                project.root.display()
            )
        })?;

    let Some(package) = resolve_native_project_pool_package(project, &model, component.package)?
    else {
        return Ok(());
    };
    let footprint =
        resolve_native_project_pool_footprint_for_component(project, &model, component)?;
    let Some(silkscreen_layer) = resolve_component_silkscreen_layer(project, component.layer)?
    else {
        return Ok(());
    };
    let geometry = PoolLandPatternGeometry::from_footprint_or_package(footprint, package);

    let key = component.uuid.to_string();
    let mut silkscreen_lines = Vec::new();
    let mut silkscreen_arcs = Vec::new();
    let mut silkscreen_circles = Vec::new();
    let mut silkscreen_texts = Vec::new();
    let mut silkscreen_polygons = Vec::new();
    let mut silkscreen_polylines = Vec::new();
    let mut mechanical_polygons = Vec::new();
    let mut pads = geometry
        .pads
        .into_iter()
        .map(|pad| {
            let padstack = resolve_native_project_pool_padstack(project, &model, pad.padstack)?;
            Ok(native_component_pad(pad, padstack.as_ref()))
        })
        .collect::<Result<Vec<_>>>()?;
    pads.sort_by(|a, b| a.name.cmp(&b.name).then_with(|| a.uuid.cmp(&b.uuid)));
    let models_3d = geometry.models_3d;

    for primitive in geometry.silkscreen {
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
            Primitive::Text {
                text,
                position,
                rotation,
            } => silkscreen_texts.push(NativeComponentSilkscreenText {
                text,
                position: native_point(position),
                rotation,
                height_nm: 1_000_000,
                stroke_width_nm: 150_000,
                layer: silkscreen_layer.id,
            }),
        }
    }

    if geometry.courtyard.closed && geometry.courtyard.vertices.len() >= 2 {
        mechanical_polygons.push(NativeComponentMechanicalPolygon {
            vertices: geometry
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
        .component_silkscreen_texts
        .insert(key.clone(), silkscreen_texts);
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

struct PoolLandPatternGeometry {
    pads: Vec<Pad>,
    courtyard: Polygon,
    silkscreen: Vec<Primitive>,
    models_3d: Vec<ModelRef>,
}

impl PoolLandPatternGeometry {
    fn from_footprint_or_package(footprint: Option<Footprint>, package: Package) -> Self {
        if let Some(footprint) = footprint {
            return Self {
                pads: footprint.pads.into_values().collect(),
                courtyard: footprint.courtyard,
                silkscreen: footprint.silkscreen,
                models_3d: if footprint.models_3d.is_empty() {
                    package.models_3d
                } else {
                    footprint.models_3d
                },
            };
        }
        Self {
            pads: package.pads.into_values().collect(),
            courtyard: package.courtyard,
            silkscreen: package.silkscreen,
            models_3d: package.models_3d,
        }
    }
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
    let (shape, diameter_nm, width_nm, height_nm) =
        match padstack.and_then(|padstack| padstack.aperture.as_ref()) {
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
    model: &DesignModel,
    package_uuid: uuid::Uuid,
) -> Result<Option<Package>> {
    let mut refs = project.manifest.pools.clone();
    refs.sort_by(|a, b| {
        a.priority
            .cmp(&b.priority)
            .then_with(|| a.path.cmp(&b.path))
    });
    for pool_ref in refs {
        let pool_path = PathBuf::from(&pool_ref.path);
        if pool_path.is_absolute() {
            let package_path = pool_path
                .join("packages")
                .join(format!("{package_uuid}.json"));
            if !package_path.exists() {
                continue;
            }
            let package: Package = serde_json::from_str(
                &std::fs::read_to_string(&package_path)
                    .with_context(|| format!("failed to read {}", package_path.display()))?,
            )
            .with_context(|| format!("failed to parse {}", package_path.display()))?;
            return Ok(Some(package));
        }
        let relative_path = format!("{}/packages/{package_uuid}.json", pool_ref.path);
        if let Ok(package_value) =
            model.materialized_source_shard_value_by_relative_path(&relative_path)
        {
            let package: Package = serde_json::from_value(package_value)
                .with_context(|| format!("failed to parse materialized {relative_path}"))?;
            return Ok(Some(package));
        }
        let package_path = project.root.join(&relative_path);
        if !package_path.exists() {
            continue;
        }
        let package: Package = serde_json::from_str(
            &std::fs::read_to_string(&package_path)
                .with_context(|| format!("failed to read {}", package_path.display()))?,
        )
        .with_context(|| format!("failed to parse {}", package_path.display()))?;
        return Ok(Some(package));
    }
    Ok(None)
}

fn resolve_native_project_pool_footprint_for_package(
    project: &LoadedNativeProject,
    model: &DesignModel,
    package_uuid: uuid::Uuid,
) -> Result<Option<Footprint>> {
    let mut refs = project.manifest.pools.clone();
    refs.sort_by(|a, b| {
        a.priority
            .cmp(&b.priority)
            .then_with(|| a.path.cmp(&b.path))
    });
    for pool_ref in refs {
        let prefix = format!("{}/footprints/", pool_ref.path);
        let pool_path = PathBuf::from(&pool_ref.path);
        if pool_path.is_absolute() {
            let Some(footprint) = read_matching_footprint_from_directory(
                &pool_path.join("footprints"),
                package_uuid,
            )?
            else {
                continue;
            };
            return Ok(Some(footprint));
        }

        let mut relative_paths = model
            .source_shards
            .iter()
            .filter(|shard| shard.taxon == Some(SourceShardTaxon::PoolFootprint))
            .filter(|shard| shard.relative_path.starts_with(&prefix))
            .map(|shard| shard.relative_path.clone())
            .collect::<Vec<_>>();
        relative_paths.sort();
        for relative_path in relative_paths {
            let footprint_value = model
                .materialized_source_shard_value_by_relative_path(&relative_path)
                .with_context(|| format!("failed to materialize {relative_path}"))?;
            let footprint: Footprint = serde_json::from_value(footprint_value)
                .with_context(|| format!("failed to parse materialized {relative_path}"))?;
            if footprint.package == package_uuid {
                return Ok(Some(footprint));
            }
        }

        let directory = project.root.join(&pool_ref.path).join("footprints");
        if let Some(footprint) = read_matching_footprint_from_directory(&directory, package_uuid)? {
            return Ok(Some(footprint));
        }
    }
    Ok(None)
}

fn resolve_native_project_pool_footprint_for_component(
    project: &LoadedNativeProject,
    model: &DesignModel,
    component: &PlacedPackage,
) -> Result<Option<Footprint>> {
    if component.part != uuid::Uuid::nil()
        && let Some(part) = resolve_native_project_pool_part(project, model, component.part)?
        && let Some(default_footprint) = part.default_footprint
    {
        return resolve_native_project_pool_footprint(project, model, default_footprint);
    }
    resolve_native_project_pool_footprint_for_package(project, model, component.package)
}

fn resolve_native_project_pool_part(
    project: &LoadedNativeProject,
    model: &DesignModel,
    part_uuid: uuid::Uuid,
) -> Result<Option<Part>> {
    let mut refs = project.manifest.pools.clone();
    refs.sort_by(|a, b| {
        a.priority
            .cmp(&b.priority)
            .then_with(|| a.path.cmp(&b.path))
    });
    for pool_ref in refs {
        let pool_path = PathBuf::from(&pool_ref.path);
        if pool_path.is_absolute() {
            let part_path = pool_path.join("parts").join(format!("{part_uuid}.json"));
            if !part_path.exists() {
                continue;
            }
            let part: Part = serde_json::from_str(
                &std::fs::read_to_string(&part_path)
                    .with_context(|| format!("failed to read {}", part_path.display()))?,
            )
            .with_context(|| format!("failed to parse {}", part_path.display()))?;
            return Ok(Some(part));
        }
        let relative_path = format!("{}/parts/{part_uuid}.json", pool_ref.path);
        if let Ok(part_value) =
            model.materialized_source_shard_value_by_relative_path(&relative_path)
        {
            let part: Part = serde_json::from_value(part_value)
                .with_context(|| format!("failed to parse materialized {relative_path}"))?;
            return Ok(Some(part));
        }
        let part_path = project.root.join(&relative_path);
        if !part_path.exists() {
            continue;
        }
        let part: Part = serde_json::from_str(
            &std::fs::read_to_string(&part_path)
                .with_context(|| format!("failed to read {}", part_path.display()))?,
        )
        .with_context(|| format!("failed to parse {}", part_path.display()))?;
        return Ok(Some(part));
    }
    Ok(None)
}

fn resolve_native_project_pool_footprint(
    project: &LoadedNativeProject,
    model: &DesignModel,
    footprint_uuid: uuid::Uuid,
) -> Result<Option<Footprint>> {
    let mut refs = project.manifest.pools.clone();
    refs.sort_by(|a, b| {
        a.priority
            .cmp(&b.priority)
            .then_with(|| a.path.cmp(&b.path))
    });
    for pool_ref in refs {
        let pool_path = PathBuf::from(&pool_ref.path);
        if pool_path.is_absolute() {
            let footprint_path = pool_path
                .join("footprints")
                .join(format!("{footprint_uuid}.json"));
            if !footprint_path.exists() {
                continue;
            }
            let footprint: Footprint = serde_json::from_str(
                &std::fs::read_to_string(&footprint_path)
                    .with_context(|| format!("failed to read {}", footprint_path.display()))?,
            )
            .with_context(|| format!("failed to parse {}", footprint_path.display()))?;
            return Ok(Some(footprint));
        }
        let relative_path = format!("{}/footprints/{footprint_uuid}.json", pool_ref.path);
        if let Ok(footprint_value) =
            model.materialized_source_shard_value_by_relative_path(&relative_path)
        {
            let footprint: Footprint = serde_json::from_value(footprint_value)
                .with_context(|| format!("failed to parse materialized {relative_path}"))?;
            return Ok(Some(footprint));
        }
        let footprint_path = project.root.join(&relative_path);
        if !footprint_path.exists() {
            continue;
        }
        let footprint: Footprint = serde_json::from_str(
            &std::fs::read_to_string(&footprint_path)
                .with_context(|| format!("failed to read {}", footprint_path.display()))?,
        )
        .with_context(|| format!("failed to parse {}", footprint_path.display()))?;
        return Ok(Some(footprint));
    }
    Ok(None)
}

fn read_matching_footprint_from_directory(
    directory: &Path,
    package_uuid: uuid::Uuid,
) -> Result<Option<Footprint>> {
    let Ok(entries) = std::fs::read_dir(directory) else {
        return Ok(None);
    };
    let mut paths = entries
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|value| value.to_str()) == Some("json"))
        .collect::<Vec<_>>();
    paths.sort();
    for path in paths {
        let footprint: Footprint = serde_json::from_str(
            &std::fs::read_to_string(&path)
                .with_context(|| format!("failed to read {}", path.display()))?,
        )
        .with_context(|| format!("failed to parse {}", path.display()))?;
        if footprint.package == package_uuid {
            return Ok(Some(footprint));
        }
    }
    Ok(None)
}

fn resolve_native_project_pool_padstack(
    project: &LoadedNativeProject,
    model: &DesignModel,
    padstack_uuid: uuid::Uuid,
) -> Result<Option<Padstack>> {
    let mut refs = project.manifest.pools.clone();
    refs.sort_by(|a, b| {
        a.priority
            .cmp(&b.priority)
            .then_with(|| a.path.cmp(&b.path))
    });
    for pool_ref in refs {
        let pool_path = PathBuf::from(&pool_ref.path);
        if pool_path.is_absolute() {
            let padstack_path = pool_path
                .join("padstacks")
                .join(format!("{padstack_uuid}.json"));
            if !padstack_path.exists() {
                continue;
            }
            let padstack: Padstack = serde_json::from_str(
                &std::fs::read_to_string(&padstack_path)
                    .with_context(|| format!("failed to read {}", padstack_path.display()))?,
            )
            .with_context(|| format!("failed to parse {}", padstack_path.display()))?;
            return Ok(Some(padstack));
        }
        let relative_path = format!("{}/padstacks/{padstack_uuid}.json", pool_ref.path);
        if let Ok(padstack_value) =
            model.materialized_source_shard_value_by_relative_path(&relative_path)
        {
            let padstack: Padstack = serde_json::from_value(padstack_value)
                .with_context(|| format!("failed to parse materialized {relative_path}"))?;
            return Ok(Some(padstack));
        }
        let padstack_path = project.root.join(&relative_path);
        if !padstack_path.exists() {
            continue;
        }
        let padstack: Padstack = serde_json::from_str(
            &std::fs::read_to_string(&padstack_path)
                .with_context(|| format!("failed to read {}", padstack_path.display()))?,
        )
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
