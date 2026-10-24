//! Canonical logical Net coverage, distinct from board-local assignments.
use super::*;
use eda_engine::substrate::ElectricalOccurrence as Occurrence;

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
    authority: &NativeSelectionResolution,
    members: &BTreeSet<Id>,
    board: Option<&Value>,
) -> BTreeMap<Id, BTreeSet<Occurrence>> {
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
    authority: &NativeSelectionResolution,
    board: Option<&Value>,
    model: &eda_engine::substrate::DesignModel,
) {
    let origins = origins(authority, &view.members, board);
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
        if let SelectionSubject::GlobalNet(net) = view.subject {
            let buses: BTreeSet<_> = model
                .electrical_identities
                .values()
                .filter_map(|record| match &record.identity {
                    eda_engine::substrate::ElectricalIdentity::Bus {
                        scalar_nets,
                        retired: false,
                        ..
                    } if scalar_nets.contains(&net) => Some(record.id),
                    _ => None,
                })
                .collect();
            scope.insert_field(
                "declared_bus_references",
                InspectionValue::Common(serde_json::json!(buses)),
                InspectionDetailKind::Members,
            );
        }
        let label_names: BTreeSet<_> = view
            .members
            .iter()
            .filter(|id| scope.scope.is_none_or(|class| class == id.class))
            .filter_map(|id| view.values.get(id)?.get("net_label_name")?.as_str())
            .collect();
        scope.insert_field(
            "member_label_names",
            InspectionValue::Common(serde_json::json!(label_names)),
            InspectionDetailKind::Members,
        );
        // Follow explicit member -> BoardNet -> class UUID references. Repeated
        // names never identify a Net or supply a default class assignment.
        let assignments: BTreeSet<_> = view
            .members
            .iter()
            .filter(|id| scope.scope.is_none_or(|class| class == id.class))
            .filter_map(|id| view.values.get(id)?.get("board_net_id")?.as_str())
            .collect();
        let rows: Vec<_> = assignments
            .into_iter()
            .map(|id| board_net(board, model, id))
            .collect();
        scope.insert_field(
            "board_net_assignments",
            InspectionValue::Common(serde_json::json!(rows)),
            InspectionDetailKind::Members,
        );
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
        let net_detail = if nets.is_empty() {
            InspectionDetailKind::Ordinary
        } else {
            InspectionDetailKind::Members
        };
        let unavailable_detail = if unavailable.is_empty() {
            net_detail
        } else {
            InspectionDetailKind::Members
        };
        let non_electrical_detail = if non_electrical.is_empty() {
            InspectionDetailKind::Ordinary
        } else {
            InspectionDetailKind::Members
        };
        scope.insert_field(
            "resolved_net_coverage",
            InspectionValue::Common(serde_json::json!(nets)),
            net_detail,
        );
        scope.insert_field(
            "net_coverage",
            if unavailable.is_empty() {
                InspectionValue::Common(serde_json::json!(nets))
            } else {
                InspectionValue::Unavailable(serde_json::to_string(&unavailable).unwrap())
            },
            unavailable_detail,
        );
        scope.insert_field(
            "non_electrical_members",
            InspectionValue::Common(serde_json::json!(non_electrical)),
            non_electrical_detail,
        );
    }
}

fn board_net(board: Option<&Value>, model: &eda_engine::substrate::DesignModel, id: &str) -> Value {
    let net = board.and_then(|board| board.get("nets")?.get(id));
    let Some(net) = net else {
        return serde_json::json!({"board_net_id":id,"state":"Unavailable — explicit BoardNet reference is absent"});
    };
    let class_id = net.get("class").and_then(Value::as_str);
    let class = class_id.and_then(|id| board?.get("net_classes")?.get(id));
    let units = eda_engine::api::native_write::project::project_display_units(model)
        .ok()
        .and_then(|profile| profile.resolve().ok());
    let mut constraints = serde_json::Map::new();
    if let Some(class) = class {
        for name in [
            "clearance",
            "track_width",
            "via_drill",
            "via_diameter",
            "diffpair_width",
            "diffpair_gap",
        ] {
            let Some(nm) = class.get(name).and_then(Value::as_i64) else {
                continue;
            };
            let rendered = units.and_then(|units| {
                let display = if name == "via_drill" {
                    units.drill
                } else {
                    units.board
                };
                eda_engine::ir::units::format_length(eda_engine::ir::units::FormatLengthRequest {
                    value: eda_engine::ir::units::Nanometers::new(nm),
                    quantity: eda_engine::ir::units::QuantityKind::Length,
                    display_unit: display.unit,
                    measurement_system: units.system,
                    precision: display.precision,
                })
                .ok()
            });
            constraints.insert(name.into(), serde_json::json!({"canonical_nm":nm,
                "display":rendered.as_ref().map(|value| value.rendered.as_str()), "rounded":rendered.as_ref().map(|value| value.rounded)}));
        }
    }
    let status = if class_id
        .and_then(|id| id.parse::<uuid::Uuid>().ok())
        .is_some_and(|id| id.is_nil())
    {
        "Unassigned — explicit nil class reference"
    } else if class.is_some() {
        "Resolved explicit class reference"
    } else {
        "Unavailable — referenced class is absent"
    };
    serde_json::json!({"board_net_id":id,"name":net.get("name"),"class_id":class_id,"class_state":status,
        "class_name":class.and_then(|class| class.get("name")),"class_constraints":constraints})
}
