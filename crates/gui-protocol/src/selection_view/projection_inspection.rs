//! Source-bound projection aggregates; scene coordinates never establish membership.
use super::*;
use crate::selection_geometry::Rational as R;
use crate::selection_scene::SceneSelectionBindings;
use crate::{BoardReviewSceneV1 as Scene, PointNm, RectNm};
use eda_engine::{
    connectivity::BoardRunMembership,
    substrate::{BusRunMembership, ElectricalMembership},
};
use std::cmp::Ordering;

#[derive(Default)]
struct Coverage {
    bounds: BTreeMap<String, [R; 4]>,
    layers: BTreeMap<String, BTreeSet<Id>>,
    unavailable: BTreeSet<String>,
}
fn points(points: impl IntoIterator<Item = [R; 2]>, radius: R) -> Option<[R; 4]> {
    let mut bounds: Option<[R; 4]> = None;
    for [x, y] in points {
        let next = [
            x.checked_sub(radius).ok()?,
            y.checked_sub(radius).ok()?,
            x.checked_add(radius).ok()?,
            y.checked_add(radius).ok()?,
        ];
        bounds = Some(match bounds {
            None => next,
            Some(prior) => union(prior, next)?,
        });
    }
    bounds
}
fn union(mut a: [R; 4], b: [R; 4]) -> Option<[R; 4]> {
    for i in 0..4 {
        let ordering = b[i].compare(a[i]).ok()?;
        if (i < 2 && ordering == Ordering::Less) || (i >= 2 && ordering == Ordering::Greater) {
            a[i] = b[i];
        }
    }
    Some(a)
}
fn path(path: &[PointNm], width: i64) -> Option<[R; 4]> {
    if width < 0 {
        return None;
    }
    points(
        path.iter()
            .map(|p| [R::integer(p.x.into()), R::integer(p.y.into())]),
        R::new(width.into(), 2).ok()?,
    )
}
fn rect(r: RectNm) -> [R; 4] {
    [r.min_x, r.min_y, r.max_x, r.max_y].map(|v| R::integer(v.into()))
}
fn geometry<'a>(
    scene: &'a Scene,
    zones: &BTreeMap<String, Vec<Vec<PointNm>>>,
) -> BTreeMap<&'a str, Option<[R; 4]>> {
    let mut result = BTreeMap::new();
    for v in &scene.components {
        result.insert(v.object_id.as_str(), Some(rect(v.bounds)));
    }
    for v in &scene.pads {
        result.insert(v.object_id.as_str(), Some(rect(v.bounds)));
    }
    for v in &scene.tracks {
        result.insert(v.object_id.as_str(), path(&v.path, v.width_nm));
    }
    for v in &scene.vias {
        result.insert(v.object_id.as_str(), path(&[v.position], v.diameter_nm));
    }
    for v in &scene.zones {
        let bounds = zones.get(&v.object_id).and_then(|polygons| {
            points(
                polygons
                    .iter()
                    .flatten()
                    .map(|p| [R::integer(p.x.into()), R::integer(p.y.into())]),
                R::integer(0),
            )
        });
        result.insert(v.object_id.as_str(), bounds);
    }
    for v in &scene.outline {
        result.insert(v.object_id.as_str(), path(&v.path, 0));
    }
    for v in &scene.board_graphics {
        result.insert(v.object_id.as_str(), path(&v.path, v.width_nm.unwrap_or(0)));
    }
    for v in &scene.board_text_geometries {
        result.insert(
            v.object_id.as_str(),
            crate::selection_geometry_catalog::layout_polygon(v)
                .and_then(|p| points(p, R::integer(0))),
        );
    }
    result
}
fn rational(r: R) -> Value {
    // Exact rational strings avoid JSON's finite integer/float range and any
    // rounded display becoming the canonical aggregate comparison basis.
    serde_json::json!({"numerator":r.numerator.to_string(),"denominator":r.denominator.to_string()})
}
fn value(bounds: &BTreeMap<String, [R; 4]>) -> Value {
    Value::Object(bounds.iter().map(|(frame,b)| (frame.clone(),serde_json::json!({"min_x":rational(b[0]),"min_y":rational(b[1]),"max_x":rational(b[2]),"max_y":rational(b[3])}))).collect())
}
pub(super) fn append(
    view: &mut SelectionView,
    authority: &NativeSelectionResolution,
    electrical: Option<&ElectricalMembership>,
    run: Option<&BoardRunMembership>,
    bus: Option<&BusRunMembership>,
    board: &Scene,
    schematic: Option<&Scene>,
) {
    let mut coverage: BTreeMap<Option<Class>, Coverage> = view
        .inspection
        .iter()
        .map(|s| (s.scope, Coverage::default()))
        .collect();
    for (surface, scene) in
        std::iter::once(("board", board)).chain(schematic.map(|scene| ("schematic", scene)))
    {
        if scene.project_uuid != view.project.to_string()
            || scene.source_revision != view.revision.0
        {
            continue;
        }
        let frame = format!("{surface}:{}", scene.board_uuid);
        let geometries = geometry(scene, &view.zone_regions);
        for (key, binding) in SceneSelectionBindings::capture(scene, authority).iter() {
            // A parent owns its projected children; each stable member is
            // counted once per layer even when it has several display records.
            let ids: BTreeSet<_> = [&binding.projected, &binding.acquired]
                .into_iter()
                .filter(|id| view.members.contains(*id))
                .collect();
            for id in ids {
                for scope in [None, Some(id.class)] {
                    let c = coverage.get_mut(&scope).expect("complete scope");
                    for layer in &binding.layers {
                        c.layers
                            .entry(format!("{frame}:{layer}"))
                            .or_default()
                            .insert(id.clone());
                    }
                    match geometries.get(key).copied().flatten() {
                        Some(next) => {
                            let merged = match c.bounds.get(&frame) {
                                Some(prior) => union(*prior, next),
                                None => Some(next),
                            };
                            if let Some(bounds) = merged {
                                c.bounds.insert(frame.clone(), bounds);
                            } else {
                                c.unavailable.insert(format!("Arithmetic range: {key}"));
                            }
                        }
                        None => {
                            // A successful empty Zone projects no occupied area.
                            if !view.zone_regions.get(key).is_some_and(Vec::is_empty) {
                                c.unavailable
                                    .insert(format!("Projection bounds unavailable: {key}"));
                            }
                        }
                    }
                }
            }
        }
    }
    for s in &mut view.inspection {
        let c = coverage.remove(&s.scope).expect("complete scope");
        let missing: Vec<_> = view
            .unprojected
            .iter()
            .filter(|id| s.scope.is_none_or(|class| class == id.class))
            .collect();
        s.fields.insert(
            "projected_bounds_nm".into(),
            InspectionValue::Common(value(&c.bounds)),
        );
        s.fields.insert(
            "combined_bounds_nm".into(),
            if !missing.is_empty() || !c.unavailable.is_empty() {
                InspectionValue::Unavailable(format!(
                    "{} unprojected members: {}; {}",
                    missing.len(),
                    serde_json::to_string(&missing).unwrap(),
                    c.unavailable.into_iter().collect::<Vec<_>>().join("; ")
                ))
            } else {
                InspectionValue::Common(value(&c.bounds))
            },
        );
        s.fields.insert(
            "projected_layer_coverage".into(),
            InspectionValue::Common(serde_json::json!(c.layers)),
        );
        s.fields.insert(
            "layer_coverage".into(),
            if missing.is_empty() {
                InspectionValue::Common(serde_json::json!(c.layers))
            } else {
                InspectionValue::Unavailable(format!(
                    "Layer projection unavailable for {} members: {}",
                    missing.len(),
                    serde_json::to_string(&missing).unwrap()
                ))
            },
        );
    }
    if let Some(all) = view.inspection.iter_mut().find(|s| s.scope.is_none()) {
        if let Some(e) = electrical {
            all.fields.insert(
                "derived_subject_id".into(),
                InspectionValue::Common(serde_json::json!(e.subject_id)),
            );
            all.fields.insert(
                "related_subject_ids".into(),
                InspectionValue::Common(serde_json::json!(e.related_subjects)),
            );
        }
        if let Some(r) = run {
            all.fields.insert(
                "board_assignment_id".into(),
                InspectionValue::Common(serde_json::json!(r.board_net)),
            );
            all.fields.insert(
                "foreign_net_contacts".into(),
                match &r.contact_evidence {
                    Ok(contacts) => InspectionValue::Common(serde_json::json!(contacts)),
                    Err(error) => InspectionValue::Unavailable(format!(
                        "Contact evidence unavailable: {error:?}"
                    )),
                },
            );
        }
        if let Some(b) = bus {
            all.fields.insert(
                "foreign_bus_contacts".into(),
                InspectionValue::Common(serde_json::json!(b.contacts)),
            );
        }
    }
}
