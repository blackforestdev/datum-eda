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
        replace_body(scene, placed, &symbol)?;
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
    let transform = |p| display_point(p, placed.position, placed.rotation, placed.mirrored);
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

/// Display projection only. Electrical pin anchors and exact placement
/// acceptance continue to use the engine's cardinal transform unchanged.
fn display_point(point: Point, origin: Point, rotation: i32, mirrored: bool) -> Result<PointNm> {
    if rotation.rem_euclid(90) == 0 {
        return Ok(point_nm(place_local_point(
            point, origin, rotation, mirrored,
        )?));
    }
    let x = if mirrored {
        point
            .x
            .checked_neg()
            .context("Symbol display reflection exceeds integer coordinates")?
    } else {
        point.x
    } as f64;
    let y = point.y as f64;
    let (sin, cos) = f64::from(rotation.rem_euclid(360)).to_radians().sin_cos();
    let rounded = |value: f64| -> Result<i64> {
        let value = value.round();
        // i64::MAX as f64 rounds UP to 2^63. Use an exclusive upper
        // bound so a saturating float cast cannot conceal overflow.
        anyhow::ensure!(
            value.is_finite() && value >= i64::MIN as f64 && value < -(i64::MIN as f64),
            "Symbol display rotation exceeds integer coordinates"
        );
        Ok(value as i64)
    };
    Ok(PointNm {
        x: origin
            .x
            .checked_add(rounded(x * cos - y * sin)?)
            .context("Symbol display translation exceeds integer coordinates")?,
        y: origin
            .y
            .checked_add(rounded(x * sin + y * cos)?)
            .context("Symbol display translation exceeds integer coordinates")?,
    })
}

#[cfg(test)]
#[path = "native_library_tests.rs"]
mod tests;
