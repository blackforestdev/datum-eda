//! Read-only qualification geometry from the captured native source and text layout.
use crate::selection_geometry::{GeometryError, Rational as R};
use crate::selection_resolution::NativeSelectionResolution;
use crate::{
    AuthoredSelectionClass as Class, AuthoredSelectionIdentity as Id, BoardReviewSceneV1, PointNm,
};
use eda_engine::substrate::{DesignModel, SourceShardKind, ZoneFillState};
use serde_json::Value;
use std::collections::BTreeMap;
pub type LayoutPoint = [R; 2];
#[derive(Debug, Clone)]
pub enum QualificationGeometry {
    Anchors(Vec<PointNm>),
    Layout(Vec<LayoutPoint>),
    Filled(Vec<Vec<LayoutPoint>>),
    Disk { center: PointNm, radius: i64 },
}
pub fn layout_polygon(geometry: &crate::BoardTextGeometryPrimitive) -> Option<[LayoutPoint; 4]> {
    let t = geometry.world_transform_nm?;
    let b = geometry.block_bbox_em_nm?;
    let corners = [
        (b.min_x_em_nm, b.min_y_em_nm),
        (b.max_x_em_nm, b.min_y_em_nm),
        (b.max_x_em_nm, b.max_y_em_nm),
        (b.min_x_em_nm, b.max_y_em_nm),
    ];
    let mut result = [[R::integer(0); 2]; 4];
    for (index, (x, y)) in corners.into_iter().enumerate() {
        let coordinate = |a: i64, b: i64, translate: i64| {
            R::integer(a.into())
                .checked_mul(R::integer(x.into()))?
                .checked_add(R::integer(b.into()).checked_mul(R::integer(y.into()))?)?
                .checked_div(R::integer(1_000_000))?
                .checked_add(R::integer(translate.into()))
        };
        result[index] = [
            coordinate(t.m11_ppm, t.m12_ppm, t.tx_nm).ok()?,
            coordinate(t.m21_ppm, t.m22_ppm, t.ty_nm).ok()?,
        ];
    }
    Some(result)
}
fn point(value: &Value) -> Option<PointNm> {
    Some(PointNm {
        x: value.get("x")?.as_i64()?,
        y: value.get("y")?.as_i64()?,
    })
}
fn polygon(path: impl IntoIterator<Item = PointNm>) -> Vec<LayoutPoint> {
    path.into_iter()
        .map(|p| [R::integer(p.x.into()), R::integer(p.y.into())])
        .collect()
}
fn source<'a>(board: &'a Value, class: &str, id: &Id) -> Option<&'a Value> {
    let values = board.get(class)?;
    match values {
        Value::Object(map) => map.get(&id.id.to_string()),
        Value::Array(list) => list
            .iter()
            .find(|v| v.get("uuid").and_then(Value::as_str) == Some(&id.id.to_string())),
        _ => None,
    }
}
pub fn qualification_catalog(
    model: &DesignModel,
    authority: &NativeSelectionResolution,
    scene: &BoardReviewSceneV1,
) -> BTreeMap<Id, Result<QualificationGeometry, String>> {
    let board = model
        .materialized_source_shard_value(SourceShardKind::BoardRoot)
        .unwrap_or(Value::Null);
    let sheet = authority.snapshot().schematic_source().and_then(|s| {
        scene
            .board_uuid
            .parse()
            .ok()
            .and_then(|uuid| s.sheets.get(&uuid))
    });
    let mut result = BTreeMap::new();
    let bindings = crate::selection_scene::SceneSelectionBindings::capture(scene, authority);
    for (_, binding) in bindings.iter() {
        let id = binding.acquired.clone();
        if result.contains_key(&id) {
            continue;
        }
        let geometry = (|| -> Option<QualificationGeometry> {
            match id.class {
                Class::Footprint => {
                    let package = source(&board, "packages", &id)?;
                    let pads = board.get("pads")?.as_object()?;
                    let mut anchors: Vec<_> = pads
                        .values()
                        .filter(|p| {
                            p.get("package").and_then(Value::as_str) == Some(&id.id.to_string())
                        })
                        .filter_map(|p| point(p.get("position")?))
                        .collect();
                    if anchors.is_empty() {
                        anchors.push(point(package.get("position")?)?);
                    }
                    Some(QualificationGeometry::Anchors(anchors))
                }
                Class::Track => {
                    let track = source(&board, "tracks", &id)?;
                    let mut anchors = vec![point(track.get("from")?)?, point(track.get("to")?)?];
                    if let Some(mid) = track.get("midpoint").filter(|v| !v.is_null()) {
                        anchors.push(point(mid)?);
                    }
                    Some(QualificationGeometry::Anchors(anchors))
                }
                Class::Via => Some(QualificationGeometry::Anchors(vec![point(
                    source(&board, "vias", &id)?.get("position")?,
                )?])),
                Class::Zone => {
                    let fill = model.zone_fills.get(&id.id)?;
                    let source = model.objects.get(&id.id)?;
                    if fill.state != ZoneFillState::Filled
                        || fill.model_revision != model.model_revision
                        || fill.source_zone_revision != source.object_revision
                    {
                        return None;
                    }
                    // Successful empty fill has no authored filled area to enclose.
                    Some(QualificationGeometry::Filled(
                        fill.islands
                            .iter()
                            .map(|p| polygon(p.vertices.iter().map(|p| PointNm { x: p.x, y: p.y })))
                            .collect(),
                    ))
                }
                Class::BoardOutline => {
                    let points: Vec<PointNm> = board
                        .get("outline")?
                        .get("vertices")?
                        .as_array()?
                        .iter()
                        .map(point)
                        .collect::<Option<_>>()?;
                    Some(QualificationGeometry::Anchors(points))
                }
                Class::Symbol => {
                    let symbol = sheet?.symbols.get(&id.id)?;
                    let anchors = if symbol.pins.is_empty() {
                        vec![PointNm {
                            x: symbol.position.x,
                            y: symbol.position.y,
                        }]
                    } else {
                        symbol
                            .pins
                            .iter()
                            .map(|p| PointNm {
                                x: p.position.x,
                                y: p.position.y,
                            })
                            .collect()
                    };
                    Some(QualificationGeometry::Anchors(anchors))
                }
                Class::Wire => {
                    let wire = sheet?.wires.get(&id.id)?;
                    Some(QualificationGeometry::Anchors(vec![
                        PointNm {
                            x: wire.from.x,
                            y: wire.from.y,
                        },
                        PointNm {
                            x: wire.to.x,
                            y: wire.to.y,
                        },
                    ]))
                }
                Class::BusSection => {
                    let bus = sheet?.buses.get(&id.id)?;
                    Some(QualificationGeometry::Anchors(
                        bus.segments
                            .iter()
                            .map(|p| PointNm { x: p.x, y: p.y })
                            .collect(),
                    ))
                }
                Class::Junction => {
                    let p = sheet?.junctions.get(&id.id)?.position;
                    Some(QualificationGeometry::Anchors(vec![PointNm {
                        x: p.x,
                        y: p.y,
                    }]))
                }
                Class::NoConnect => {
                    let p = sheet?.noconnects.get(&id.id)?.position;
                    Some(QualificationGeometry::Anchors(vec![PointNm {
                        x: p.x,
                        y: p.y,
                    }]))
                }
                Class::BoardText | Class::SchematicText | Class::Label | Class::Port => {
                    let text = scene.board_texts.iter().find(|text| {
                        text.text_uuid == id.id.to_string()
                            || text.source_object_uuid.as_deref() == Some(&id.id.to_string())
                    })?;
                    let geometry = scene
                        .board_text_geometries
                        .iter()
                        .find(|g| g.object_id == text.object_id)?;
                    Some(QualificationGeometry::Layout(
                        layout_polygon(geometry)?.to_vec(),
                    ))
                }
                Class::SchematicGraphic => {
                    use eda_engine::schematic::SchematicPrimitive as P;
                    match sheet?.drawings.get(&id.id)? {
                        P::Line { from, to, .. } => Some(QualificationGeometry::Anchors(vec![
                            PointNm {
                                x: from.x,
                                y: from.y,
                            },
                            PointNm { x: to.x, y: to.y },
                        ])),
                        P::Rect { min, max, .. } => {
                            Some(QualificationGeometry::Filled(vec![polygon([
                                PointNm { x: min.x, y: min.y },
                                PointNm { x: max.x, y: min.y },
                                PointNm { x: max.x, y: max.y },
                                PointNm { x: min.x, y: max.y },
                            ])]))
                        }
                        P::Circle { center, radius, .. } => Some(QualificationGeometry::Disk {
                            center: PointNm {
                                x: center.x,
                                y: center.y,
                            },
                            radius: *radius,
                        }),
                        P::Arc { arc, .. } => {
                            let sweep = arc.end_angle.checked_sub(arc.start_angle)?;
                            let at = |angle: i32| {
                                let (dx, dy) = match angle.rem_euclid(360) {
                                    0 => (arc.radius, 0),
                                    90 => (0, arc.radius),
                                    180 => (-arc.radius, 0),
                                    270 => (0, -arc.radius),
                                    _ => return None,
                                };
                                Some(PointNm {
                                    x: arc.center.x.checked_add(dx)?,
                                    y: arc.center.y.checked_add(dy)?,
                                })
                            };
                            if sweep % 2 != 0 {
                                return None;
                            }
                            Some(QualificationGeometry::Anchors(vec![
                                at(arc.start_angle)?,
                                at(arc.start_angle + sweep / 2)?,
                                at(arc.end_angle)?,
                            ]))
                        }
                    }
                }
                _ => None,
            }
        })()
        .ok_or_else(|| {
            format!(
                "Certified {:?} region geometry unavailable for {}",
                id.class, id.id
            )
        });
        result.insert(id, geometry);
    }
    result
}
pub fn geometry_error(error: GeometryError) -> String {
    format!("Exact region predicate unavailable: {error:?}")
}
