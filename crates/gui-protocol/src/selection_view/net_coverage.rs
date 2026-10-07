//! Canonical logical Net coverage, distinct from board-local assignments.
use super::*;
use eda_engine::substrate::{DesignModel, ElectricalOccurrence as Occurrence, SourceShardKind};

fn class(class: Class) -> Option<&'static str> {
    Some(match class {
        Class::Track => "tracks",
        Class::Via => "vias",
        Class::Zone => "zones",
        Class::Pad => "pads",
        Class::Wire => "wires",
        Class::Pin => "pins",
        Class::Label => "labels",
        Class::Port => "ports",
        Class::Junction => "junctions",
        _ => return None,
    })
}
fn origins(
    model: &DesignModel,
    authority: &NativeSelectionResolution,
    members: &BTreeSet<Id>,
) -> BTreeMap<Id, BTreeSet<Occurrence>> {
    let board = model
        .materialized_source_shard_value(SourceShardKind::BoardRoot)
        .ok()
        .and_then(|b| crate::placed_pad_projection::project(model, b).ok());
    let mut pads: BTreeMap<crate::SelectionProjectId, BTreeSet<Occurrence>> = BTreeMap::new();
    for p in board
        .as_ref()
        .and_then(|b| b.get("pads"))
        .and_then(Value::as_object)
        .into_iter()
        .flat_map(|m| m.values())
    {
        if let Some((parent, id)) = p
            .get("package")
            .and_then(Value::as_str)
            .and_then(|s| s.parse().ok())
            .zip(
                p.get("uuid")
                    .and_then(Value::as_str)
                    .and_then(|s| s.parse().ok()),
            )
        {
            pads.entry(parent).or_default().insert(Occurrence {
                class: "pads".into(),
                source_id: id,
                instance_path: vec![],
            });
        }
    }
    let symbols: BTreeMap<_, _> = authority
        .snapshot()
        .schematic_source()
        .into_iter()
        .flat_map(|s| s.sheets.values())
        .flat_map(|s| s.symbols.values())
        .map(|s| (s.uuid, &s.pins))
        .collect();
    members
        .iter()
        .map(|id| {
            let origins = if let Some(class) = class(id.class) {
                BTreeSet::from([Occurrence {
                    class: class.into(),
                    source_id: id.id,
                    instance_path: id.instance_path.clone(),
                }])
            } else if id.class == Class::Footprint {
                pads.get(&id.id).cloned().unwrap_or_default()
            } else if id.class == Class::Symbol {
                symbols
                    .get(&id.id)
                    .into_iter()
                    .flat_map(|pins| pins.iter())
                    .map(|p| Occurrence {
                        class: "pins".into(),
                        source_id: p.uuid,
                        instance_path: id.instance_path.clone(),
                    })
                    .collect()
            } else {
                BTreeSet::new()
            };
            (id.clone(), origins)
        })
        .collect()
}
pub(super) fn append(
    view: &mut SelectionView,
    model: &DesignModel,
    authority: &NativeSelectionResolution,
) {
    let origins = origins(model, authority, &view.members);
    let mut cache = BTreeMap::new();
    for origin in origins.values().flatten() {
        if cache.contains_key(origin) {
            continue;
        }
        match authority
            .snapshot()
            .net_for_origin(authority.revision(), origin)
        {
            Ok(membership) => {
                // Complete successful results amortize queries over their whole
                // domain. Names, graph component ordinals and scene IDs do not.
                for member in membership.owned {
                    cache.insert(member, Ok(membership.subject_id));
                }
                cache.insert(origin.clone(), Ok(membership.subject_id));
            }
            Err(error) => {
                cache.insert(origin.clone(), Err(format!("{error:?}")));
            }
        }
    }
    for scope in &mut view.inspection {
        let mut nets: BTreeMap<crate::SelectionProjectId, BTreeSet<Id>> = BTreeMap::new();
        let mut unavailable = Vec::new();
        let mut non_electrical = BTreeSet::new();
        for (id, origins) in &origins {
            if scope.scope.is_some_and(|class| class != id.class) {
                continue;
            }
            if origins.is_empty() {
                non_electrical.insert(id);
            }
            for origin in origins {
                match &cache[origin] {
                    Ok(net) => {
                        nets.entry(*net).or_default().insert(id.clone());
                    }
                    Err(reason) => unavailable
                        .push(serde_json::json!({"member":id,"origin":origin,"reason":reason})),
                }
            }
        }
        scope.fields.insert(
            "resolved_net_coverage".into(),
            InspectionValue::Common(serde_json::json!(nets)),
        );
        scope.fields.insert(
            "net_coverage".into(),
            if unavailable.is_empty() {
                InspectionValue::Common(serde_json::json!(nets))
            } else {
                InspectionValue::Unavailable(serde_json::to_string(&unavailable).unwrap())
            },
        );
        scope.fields.insert(
            "non_electrical_members".into(),
            InspectionValue::Common(serde_json::json!(non_electrical)),
        );
    }
}
