//! Shared supported Footprint graphics projection, authenticated from current sources.
use crate::{
    board::{PlacedPackage, StackupLayer, StackupLayerType},
    error::EngineError,
    pool::{Footprint, Package, Primitive},
    substrate::{DesignModel, SourceShardKind, SourceShardTaxon, component_pad_source as source},
};
use serde_json::{Value, json};
use uuid::Uuid;

pub fn materialize(
    model: &DesignModel,
    placement: &PlacedPackage,
    footprint: Uuid,
) -> Result<Value, EngineError> {
    let pin = source::capture_root(model, footprint, SourceShardTaxon::PoolFootprint)?;
    let footprint: Footprint = serde_json::from_value(source::source_value(model, &pin)?)?;
    let pin = source::capture_root(model, placement.package, SourceShardTaxon::PoolPackage)?;
    let package: Package = serde_json::from_value(source::source_value(model, &pin)?)?;
    if footprint.package != placement.package {
        return Err(EngineError::Validation(
            "Footprint graphics Package mismatch".into(),
        ));
    }
    let board = model.materialized_source_shard_value(SourceShardKind::BoardRoot)?;
    let layers: Vec<StackupLayer> = serde_json::from_value(board["stackup"]["layers"].clone())?;
    let layer = layers
        .iter()
        .filter(|l| l.layer_type == StackupLayerType::Silkscreen)
        .min_by_key(|l| ((i64::from(l.id) - i64::from(placement.layer)).abs(), l.id))
        .ok_or_else(|| {
            EngineError::Validation("Footprint graphics require a board silkscreen layer".into())
        })?
        .id;
    let mut lines = vec![];
    let mut arcs = vec![];
    let mut circles = vec![];
    let mut texts = vec![];
    let mut polygons = vec![];
    let mut polylines = vec![];
    for primitive in footprint.silkscreen {
        match primitive {
            Primitive::Line { from, to, width } => lines.push(json!({"from":from,"to":to,"width_nm":width,"layer":layer})),
            Primitive::Rect { min, max, width } => polygons.push(json!({"vertices":[min,crate::ir::geometry::Point::new(max.x,min.y),max,crate::ir::geometry::Point::new(min.x,max.y)],"width_nm":width,"layer":layer})),
            Primitive::Circle { center, radius, width } => circles.push(json!({"center":center,"radius_nm":radius,"width_nm":width,"layer":layer})),
            Primitive::Polygon { polygon, width } => {
                let payload = json!({"vertices":polygon.vertices,"width_nm":width,"layer":layer});
                if polygon.closed { polygons.push(payload); } else { polylines.push(payload); }
            }
            Primitive::Arc { arc, width } => arcs.push(json!({"center":arc.center,"radius_nm":arc.radius,"start_angle":arc.start_angle,"end_angle":arc.end_angle,"width_nm":width,"layer":layer})),
            Primitive::Text { text, position, rotation } => texts.push(json!({"text":text,"position":position,"rotation":rotation,"height_nm":1000000,"stroke_width_nm":150000,"layer":layer})),
        }
    }
    let courtyard = if footprint.courtyard.closed && footprint.courtyard.vertices.len() >= 2 {
        vec![json!({"vertices":footprint.courtyard.vertices,"layer":41})]
    } else {
        vec![]
    };
    let models = if footprint.models_3d.is_empty() {
        package.models_3d
    } else {
        footprint.models_3d
    };
    Ok(
        json!({"component_silkscreen":lines,"component_silkscreen_arcs":arcs,
        "component_silkscreen_circles":circles,"component_silkscreen_texts":texts,
        "component_silkscreen_polygons":polygons,"component_silkscreen_polylines":polylines,
        "component_mechanical_polygons":courtyard,"component_models_3d":models}),
    )
}
