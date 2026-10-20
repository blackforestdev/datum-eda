//! Read-only qualification geometry from the captured native source and text layout.
use crate::selection_geometry::{GeometryError, Rational as R};
use crate::selection_resolution::NativeSelectionResolution;
use crate::{
    AuthoredSelectionClass as Class, AuthoredSelectionIdentity as Id, BoardReviewSceneV1, PointNm,
};
use eda_engine::substrate::{DesignModel, SourceShardKind};
use serde_json::Value;
use std::collections::BTreeMap;
mod text_layout_index;
pub type LayoutPoint = [R; 2];
#[derive(Debug, Clone)]
pub enum QualificationGeometry {
    Anchors(Vec<PointNm>),
    ExactAnchors(Vec<crate::ExactPointNm>),
    Layout(Vec<LayoutPoint>),
    Filled(Vec<Vec<LayoutPoint>>),
    Disk { center: PointNm, radius: i64 },
}
pub fn layout_polygon(geometry: &crate::BoardTextGeometryPrimitive) -> Option<[LayoutPoint; 4]> {
    layout_polygon_from_basis(geometry.world_transform_nm?, geometry.block_bbox_em_nm?)
}

/// Exact layout corners from the same captured affine/em basis, without
/// requiring each retained hit to own a duplicated rational corner array.
pub fn layout_polygon_from_basis(
    t: crate::Affine2DFixedPrimitive,
    b: crate::MeshRectEmPrimitive,
) -> Option<[LayoutPoint; 4]> {
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
    let mut board = model
        .materialized_source_shard_value(SourceShardKind::BoardRoot)
        .unwrap_or(Value::Null);
    let projection =
        eda_engine::substrate::component_pad_source::component_pad_projection(model, &board);
    if let Ok(p) = &projection {
        if !p.pads.is_empty() && board.get("pads").is_none() {
            board["pads"] = serde_json::json!({});
        }
        if let Some(map) = board.get_mut("pads").and_then(Value::as_object_mut) {
            for (id, pad) in &p.pads {
                map.entry(id.to_string())
                    .or_insert_with(|| serde_json::to_value(pad).unwrap());
            }
        }
    }
    let sheet = authority.snapshot().schematic_source().and_then(|s| {
        scene
            .board_uuid
            .parse()
            .ok()
            .and_then(|uuid| s.sheets.get(&uuid))
    });
    let mut result = BTreeMap::new();
    let text_layouts =
        text_layout_index::TextLayoutIndex::new(&scene.board_texts, &scene.board_text_geometries);
    let bindings = crate::selection_scene::SceneSelectionBindings::capture(scene, authority);
    for (_, binding) in bindings.iter() {
        let id = binding.acquired.clone();
        if result.contains_key(&id) {
            continue;
        }
        if id.class == Class::Footprint
            && board
                .get("component_pads")
                .and_then(|v| v.get(id.id.to_string()))
                .and_then(Value::as_array)
                .is_some_and(|pads| {
                    !pads.is_empty()
                        && (projection.as_ref().is_err()
                            || pads.iter().any(|v| {
                                v.get("uuid")
                                    .and_then(Value::as_str)
                                    .and_then(|s| s.parse().ok())
                                    .is_none_or(|id| {
                                        !projection.as_ref().unwrap().query_pads.contains_key(&id)
                                    })
                            }))
                })
        {
            result.insert(id, Err("Complete native component Pad identity/geometry authority unavailable; placement-point fallback refused".into()));
            continue;
        }
        let geometry = (|| -> Option<QualificationGeometry> {
            match id.class {
                Class::Footprint => {
                    if let Ok(projection) = &projection {
                        let mut anchors = BTreeMap::new();
                        if let Some(pads) = board.get("pads").and_then(Value::as_object) {
                            for (key, pad) in pads {
                                if pad.get("package").and_then(Value::as_str)
                                    == Some(&id.id.to_string())
                                {
                                    let center = point(pad.get("position")?)?;
                                    anchors.insert(
                                        key.clone(),
                                        crate::PadCenter::Integer(center).exact_point(),
                                    );
                                }
                            }
                        }
                        for (key, pad) in &projection.query_pads {
                            if pad.terminal().package == id.id {
                                anchors.insert(key.to_string(), pad.center());
                            }
                        }
                        if !anchors.is_empty()
                            && projection.query_pads.values().any(|p| {
                                p.terminal().package == id.id
                                    && matches!(
                                    p,
                                    eda_engine::board::query_pad::QueryPadGeometry::SourceRelative(
                                        _
                                    )
                                )
                            })
                        {
                            return Some(QualificationGeometry::ExactAnchors(
                                anchors.into_values().collect(),
                            ));
                        }
                    }
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
                // Bindings already collapse owned Pads to their Footprint.
                // An independent source Pad uses the one-anchor family.
                Class::Pad => Some(QualificationGeometry::Anchors(vec![point(
                    source(&board, "pads", &id)?.get("position")?,
                )?])),
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
                    let crate::ZoneFillProjection::Current(polygons) =
                        crate::ZoneFillProjection::capture(model, id.id)
                    else {
                        return None;
                    };
                    Some(QualificationGeometry::Filled(
                        polygons.into_iter().map(polygon).collect(),
                    ))
                }
                Class::BoardGraphic => {
                    crate::native_board_graphics::qualification(source(&board, "graphics", &id)?)
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
                    Some(QualificationGeometry::Layout(
                        text_layouts.layout(&id.id.to_string())?.to_vec(),
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
                            // Canonical IR and the native projector use tenths
                            // of a degree and a positive wrapped sweep.
                            let mut sweep = arc.end_angle.checked_sub(arc.start_angle)?;
                            if sweep == 0 {
                                sweep = 3600;
                            }
                            if sweep < 0 {
                                sweep = sweep.checked_add(3600)?;
                            }
                            let at = |angle: i32| {
                                let (dx, dy) = match angle.rem_euclid(3600) {
                                    0 => (arc.radius, 0),
                                    900 => (0, arc.radius),
                                    1800 => (arc.radius.checked_neg()?, 0),
                                    2700 => (0, arc.radius.checked_neg()?),
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
                                at(arc.start_angle.checked_add(sweep / 2)?)?,
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
