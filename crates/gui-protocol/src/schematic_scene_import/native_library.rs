//! Native symbol bodies come from their qualified pool source, not an IEC stand-in.
//! Existing display tessellation remains a renderer projection; source coordinates
//! and electrical attachment positions are never changed by this composition.
use super::*;
use eda_engine::api::native_write::schematic_edit::place_local_point;
use eda_engine::ir::geometry::Point;
use eda_engine::pool::{Primitive, Symbol};
use eda_engine::schematic::PlacedSymbol;
use eda_engine::substrate::DesignModel;
use uuid::Uuid;

pub(crate) fn project_bodies(
    scene: &mut BoardReviewSceneV1,
    model: &DesignModel,
    source: &Schematic,
    sheet_id: Uuid,
) -> Result<()> {
    let sheet = &source.sheets[&sheet_id];
    for placed in sheet.symbols.values() {
        let Some(id) = placed.lib_id.as_ref().and_then(|s| s.parse::<Uuid>().ok()) else {
            continue;
        };
        let Some(object) = model
            .objects
            .get(&id)
            .filter(|o| o.domain == "pool" && o.kind == "symbols")
        else {
            continue;
        };
        let shard = model
            .source_shards
            .iter()
            .find(|s| s.shard_id == object.source_shard_id)
            .context("Native symbol source unavailable")?;
        let symbol: Symbol = serde_json::from_value(
            model.materialized_source_shard_value_by_relative_path(&shard.relative_path)?,
        )?;
        // Older native placements already use the compatibility projection
        // for unsupported poses; this first-placement slice does not reinterpret them.
        if placed.rotation.rem_euclid(90) == 0 {
            replace_body(scene, placed, &symbol)?;
        }
    }
    Ok(())
}
pub(crate) fn replace_body(
    scene: &mut BoardReviewSceneV1,
    placed: &PlacedSymbol,
    symbol: &Symbol,
) -> Result<()> {
    let base_id = format!("schematic-symbol:{}", placed.uuid);
    scene.board_graphics.retain(|g| g.object_id != base_id);
    let transform =
        |p| place_local_point(p, placed.position, placed.rotation, placed.mirrored).map(point_nm);
    let mut text = SchematicTextSink::default();
    let mut text_points = Vec::new();
    for (index, drawing) in symbol.drawings.iter().enumerate() {
        let (path, kind, width) = match drawing {
            Primitive::Line { from, to, width } => {
                (vec![transform(*from)?, transform(*to)?], "line", *width)
            }
            Primitive::Rect { min, max, width } => (
                vec![
                    transform(*min)?,
                    transform(Point { x: max.x, y: min.y })?,
                    transform(*max)?,
                    transform(Point { x: min.x, y: max.y })?,
                    transform(*min)?,
                ],
                "polyline",
                *width,
            ),
            Primitive::Polygon { polygon, width } => {
                let mut path = polygon
                    .vertices
                    .iter()
                    .map(|p| transform(*p))
                    .collect::<std::result::Result<Vec<_>, _>>()?;
                if let Some(first) = path.first().copied() {
                    path.push(first);
                }
                (path, "polyline", *width)
            }
            Primitive::Circle {
                center,
                radius,
                width,
            } => {
                let mut path = arc_path_points(eda_engine::ir::geometry::Arc {
                    center: *center,
                    radius: *radius,
                    start_angle: 0,
                    end_angle: 3600,
                });
                for p in &mut path {
                    *p = transform(Point { x: p.x, y: p.y })?;
                }
                (path, "polyline", *width)
            }
            Primitive::Arc { arc, width } => {
                let mut path = arc_path_points(*arc);
                for p in &mut path {
                    *p = transform(Point { x: p.x, y: p.y })?;
                }
                (path, "polyline", *width)
            }
            Primitive::Text {
                text: content,
                position,
                rotation,
            } => {
                let rotation = if placed.mirrored {
                    rotation
                        .checked_neg()
                        .context("Library text reflection overflow")?
                } else {
                    *rotation
                };
                text.push_rotated(
                    &mut text_points,
                    placed.uuid,
                    &format!("library:{index}"),
                    content,
                    transform(*position)?,
                    LABEL_TEXT_HEIGHT_NM,
                    TextHAlign::Left,
                    TextVAlign::Center,
                    SCHEMATIC_ANNOTATION_TEXT_LAYER_INT,
                    rotation
                        .checked_add(placed.rotation)
                        .context("Library text rotation overflow")?,
                );
                continue;
            }
        };
        scene.board_graphics.push(BoardGraphicPrimitive {
            object_id: format!("{base_id}:library:{index}"),
            object_kind: "schematic_symbol".into(),
            primitive_kind: kind.into(),
            source_object_uuid: placed.uuid.to_string(),
            layer_id: SCHEMATIC_SYMBOL_LAYER.into(),
            path,
            holes: Vec::new(),
            width_nm: Some(width),
        });
    }
    let (texts, geometries, assets) = text.into_parts();
    scene.board_texts.extend(texts);
    scene.board_text_geometries.extend(geometries);
    for asset in assets {
        if !scene
            .glyph_mesh_assets
            .iter()
            .any(|existing| existing.handle == asset.handle)
        {
            scene.glyph_mesh_assets.push(asset);
        }
    }
    Ok(())
}
