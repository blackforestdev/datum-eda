//! Exact old map correspondence across source revision and immutable query frames.
use super::*;
use crate::api::native_write::{electrical, schematic_connectivity};
use crate::ir::geometry::Point;
use crate::substrate::{
    ElectricalIdentityRecord, NetCorrespondence, NetCorrespondenceStatus, NetRelationshipIntent,
    ObjectRevision, RevisionedRef, SourceShardKind, component_pad_source as pins,
    net_correspondence_status,
};

fn reference(model: &DesignModel, id: Uuid) -> RevisionedRef {
    RevisionedRef {
        object_id: id,
        object_revision: model.objects[&id].object_revision,
    }
}
pub(super) fn connected(
    model: &mut DesignModel,
    root: &Path,
    symbol: Uuid,
    package: Uuid,
    ci: Uuid,
    map: Uuid,
) -> (Uuid, Uuid, BTreeSet<ElectricalOccurrence>) {
    let snapshot = ElectricalSelectionSnapshot::capture(model).unwrap();
    let (sheet, placed) = snapshot
        .schematic_source()
        .unwrap()
        .sheets
        .values()
        .find_map(|sheet| sheet.symbols.get(&symbol).map(|s| (sheet.uuid, s.clone())))
        .unwrap();
    let mapping = raw(model, map);
    let (pin, library_pad) = placed
        .pins
        .iter()
        .find_map(|pin| {
            let logical = pin.library_pin?;
            mapping["mappings"]
                .as_object()?
                .iter()
                .find_map(|(pad, cell)| {
                    (cell["gate"] == json!(placed.gate?) && cell["pin"] == json!(logical))
                        .then(|| (pin.clone(), pad.parse::<Uuid>().unwrap()))
                })
        })
        .expect("native placed gate has an explicit mapped library pin");
    let wire = crate::schematic::SchematicWire {
        uuid: Uuid::new_v4(),
        from: pin.position,
        to: Point::new(pin.position.x - 5_000_000, pin.position.y),
    };
    let p = schematic_connectivity::build_create_schematic_wire(model, provenance(), sheet, &wire)
        .unwrap();
    commit_prepared(model, root, p).unwrap();
    let label = crate::schematic::NetLabel {
        uuid: Uuid::new_v4(),
        kind: crate::schematic::LabelKind::Global,
        name: format!("PINNED_MAP_NET_{ci}"),
        position: wire.to,
    };
    let p =
        schematic_connectivity::build_create_schematic_label(model, provenance(), sheet, &label)
            .unwrap();
    commit_prepared(model, root, p).unwrap();
    let source = model
        .materialized_source_shard_value(SourceShardKind::BoardRoot)
        .unwrap();
    let pad: pins::ComponentPadSource = source["component_pads"][package.to_string()]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| serde_json::from_value::<pins::ComponentPadSource>(p.clone()).unwrap())
        .find(|p| p.source_binding.as_ref().unwrap().library_pad.object_id == library_pad)
        .unwrap();
    let projection = pins::component_pad_projection(model, &source).unwrap();
    let terminal = projection.query_pads[&pad.uuid].terminal();
    let net = crate::board::Net::new(Uuid::new_v4(), &format!("PINNED_MAP_NET_{ci}"), Uuid::nil());
    let track = crate::board::Track::straight(
        Uuid::new_v4(),
        net.uuid,
        terminal.position,
        Point::new(terminal.position.x - 1_000_000, terminal.position.y),
        100_000,
        terminal.layer,
    );
    let p = BatchComposer::compose(model, provenance())
        .push_ops([
            Operation::CreateBoardNet {
                net_id: net.uuid,
                net: serde_json::to_value(&net).unwrap(),
            },
            Operation::CreateBoardTrack {
                track_id: track.uuid,
                track: serde_json::to_value(&track).unwrap(),
            },
        ])
        .finish()
        .unwrap();
    commit_prepared(model, root, p).unwrap();
    let groups = crate::substrate::electrical_test_partitions(model, &[]).unwrap();
    let group = groups
        .iter()
        .find(|g| g.contains(&origin("pins", pin.uuid)))
        .unwrap();
    let logical = model
        .electrical_identities
        .values()
        .find_map(|record| match &record.identity {
            ElectricalIdentity::Net {
                anchor,
                retired: false,
                ..
            } if group.contains(anchor) => Some(record.id),
            _ => None,
        })
        .expect("use the actual automatic identity of the connected source partition");
    let record = ElectricalIdentityRecord {
        id: Uuid::new_v4(),
        object_revision: ObjectRevision(0),
        identity: ElectricalIdentity::NetRelationship {
            logical_net: logical,
            board_net: Some(reference(model, net.uuid)),
            intent: NetRelationshipIntent::Implemented,
            evidence: vec![NetCorrespondence {
                component_instance: reference(model, ci),
                pin_pad_map: reference(model, map),
                schematic_terminal: origin("pins", pin.uuid),
                board_pad: reference(model, pad.uuid),
                library_pad: Some(reference(model, library_pad)),
                library_pin: Some(reference(model, pin.library_pin.unwrap())),
            }],
        },
    };
    let p = electrical::build_adopt_pin_correspondence(
        model,
        provenance(),
        record,
        &[electrical::PlacedPinAdoption {
            sheet_id: sheet,
            symbol_id: symbol,
            placed_pin_id: pin.uuid,
            library_pin_id: pin.library_pin.unwrap(),
        }],
    )
    .unwrap();
    commit_prepared(model, root, p).unwrap();
    assert_eq!(
        net_correspondence_status(model, logical),
        NetCorrespondenceStatus::Complete
    );
    (
        logical,
        library_pad,
        BTreeSet::from([
            origin("pins", pin.uuid),
            origin("wires", wire.uuid),
            origin("labels", label.uuid),
            origin("pads", pad.uuid),
            origin("tracks", track.uuid),
        ]),
    )
}

#[test]
fn saved_map_revision_keeps_connected_eight_pads_exact_and_held_snapshot_immutable() {
    let (root, mut model, symbol, ci, _part, package, map, _request) =
        physical_fixture::eight_pad_fixture("saved-map-connected-eight");
    let (logical, selected, witnesses) = connected(&mut model, &root, symbol, package, ci, map);
    model = ProjectResolver::new(&root).resolve().unwrap();
    let held = ElectricalSelectionSnapshot::capture(&model).unwrap();
    let membership = held.global_net(held.revision(), logical).unwrap();
    assert!(witnesses.is_subset(&membership.owned));
    let original = raw(&model, map);
    let board = model
        .materialized_source_shard_value(SourceShardKind::BoardRoot)
        .unwrap();
    let ci_raw = raw(&model, ci);
    let declared = model.component_instances[&ci].clone();
    let identities = model.electrical_identities.clone();
    let projection = pins::component_pad_projection(&model, &board).unwrap();
    assert_eq!(projection.query_pads.len(), 8);
    assert!(
        projection
            .statuses
            .values()
            .all(|s| *s == pins::ComponentPadMaterializationStatus::Current)
    );
    let geometry = serde_json::to_value(&projection.pads).unwrap();
    let old_binding = board["component_pads"][package.to_string()].clone();
    let mut mappings = original["mappings"].as_object().unwrap().clone();
    let other = mappings
        .keys()
        .find(|k| k.parse::<Uuid>().unwrap() != selected)
        .unwrap()
        .clone();
    let first = mappings[&selected.to_string()].clone();
    let second = mappings[&other].clone();
    assert_ne!(first, second);
    mappings.insert(selected.to_string(), second);
    mappings.insert(other, first);
    let p=(find_native_write_verb("datum.library.edit").unwrap().build)(
        &NativeWriteContext { model:&model,project_root:&root },provenance(),
        json!({"kind":"pin_pad_maps","object_id":map,"pool_path":"pool","fields":{"mappings":mappings}}),
    ).unwrap();
    use crate::substrate::component_instance::pool_history::PoolHistoryBasis;
    let next = ObjectRevision(model.objects[&map].object_revision.0 + 1);
    let pending = PoolHistoryBasis {
        model: &model,
        operations: &p.batch.operations,
        applied_operations: &[],
    };
    assert!(!pending.definition_ref_matches(&model.objects, map, next, "pin_pad_maps"));
    let mut staged = model.objects.clone();
    staged.get_mut(&map).unwrap().object_revision = next;
    let applied = PoolHistoryBasis {
        model: &model,
        operations: &p.batch.operations,
        applied_operations: &p.batch.operations,
    };
    assert!(applied.definition_ref_matches(&staged, map, ObjectRevision(0), "pin_pad_maps"));
    assert!(applied.definition_ref_matches(&staged, map, next, "pin_pad_maps"));
    assert!(!applied.definition_ref_matches(
        &staged,
        map,
        ObjectRevision(next.0 + 1),
        "pin_pad_maps"
    ));
    let journal = std::fs::read(crate::substrate::transaction_journal_path(&root)).unwrap();
    let stale = p.clone();
    assert_eq!(
        std::fs::read(crate::substrate::transaction_journal_path(&root)).unwrap(),
        journal
    );
    commit_prepared(&mut model, &root, p).unwrap();
    assert!(commit_prepared(&mut model, &root, stale).is_err());
    let mut revised = original.clone();
    revised["mappings"] = Value::Object(mappings);
    assert_eq!(raw(&model, map), revised);
    let verify = |model: &DesignModel| {
        assert_eq!(model.component_instances[&ci], declared);
        assert_eq!(raw(model, ci), ci_raw);
        assert_eq!(model.electrical_identities, identities);
        assert_eq!(
            model
                .materialized_source_shard_value(SourceShardKind::BoardRoot)
                .unwrap(),
            board
        );
        assert_eq!(
            model
                .materialized_pool_object_revision(map, "pin_pad_maps", ObjectRevision(0))
                .unwrap(),
            original
        );
        assert_eq!(
            net_correspondence_status(model, logical),
            NetCorrespondenceStatus::Complete
        );
        // Both a held immutable query and a fresh query retain the declared old
        // map, while the editable current source above has different cells.
        let fresh = ElectricalSelectionSnapshot::capture(model).unwrap();
        assert_eq!(
            held.global_net(held.revision(), logical).unwrap(),
            membership
        );
        assert_eq!(
            fresh.global_net(fresh.revision(), logical).unwrap().owned,
            membership.owned
        );
        for witness in &witnesses {
            assert_eq!(
                fresh
                    .net_for_origin(fresh.revision(), witness)
                    .unwrap()
                    .owned,
                membership.owned
            );
        }
        for (class, id) in [("symbols", symbol), ("packages", package)] {
            let at = origin(class, id);
            assert_eq!(
                held.component_selection_context(held.revision(), &BTreeSet::from([at.clone()]))
                    .unwrap()
                    .bindings[&at]
                    .counterpart_status,
                ComponentCounterpartStatus::Resolved
            );
            assert_eq!(
                context(model, at.clone()).bindings[&at].counterpart_status,
                ComponentCounterpartStatus::Resolved
            );
        }
        let p = pins::component_pad_projection(model, &board).unwrap();
        assert_eq!(serde_json::to_value(&p.pads).unwrap(), geometry);
        assert_eq!(p.query_pads.len(), 8);
        assert!(p.unavailable.is_empty());
        assert!(
            p.statuses
                .values()
                .all(|s| *s == pins::ComponentPadMaterializationStatus::Current)
        );
        assert_eq!(board["component_pads"][package.to_string()], old_binding);
    };
    let recorded: pins::ComponentPadSource =
        serde_json::from_value(old_binding[0].clone()).unwrap();
    let pin = recorded
        .source_binding
        .as_ref()
        .unwrap()
        .pin_pad_map
        .as_ref()
        .unwrap();
    assert_eq!(pins::source_value(&model, pin).unwrap(), original);
    for tamper in 0..5 {
        let mut bad = pin.clone();
        match tamper {
            0 => bad.owner_value_hash = "altered".into(),
            1 => bad.record_hash = "altered".into(),
            2 => bad.incarnation = Some(Uuid::new_v4()),
            3 => bad.source_shard_id = Uuid::new_v4(),
            _ => bad.object_revision = ObjectRevision(next.0 + 1),
        }
        assert!(pins::check_pin(&model, &bad).is_err());
        assert!(pins::source_value(&model, &bad).is_err());
    }
    let mut wrong = model.clone();
    wrong.objects.get_mut(&map).unwrap().kind = "parts".into();
    assert!(pins::check_pin(&wrong, pin).is_err());
    // Counterfactual negative evidence only: a deletion/reuse epoch cannot
    // authenticate the recorded map pin. No fake state supplies a positive case.
    let mut reused = model.clone();
    reused
        .journal
        .iter_mut()
        .find(|t| t.transaction_kind == crate::substrate::TransactionKind::Normal)
        .unwrap()
        .operations
        .push(Operation::DeletePoolLibraryObject {
            object_id: map,
            relative_path: pin.relative_path.clone(),
            object_kind: "pin_pad_maps".into(),
            object: revised.clone(),
        });
    assert!(pins::check_pin(&reused, pin).is_err());
    verify(&model);
    model
        .commit_journal_undo(&root, provenance().into())
        .unwrap();
    verify(&model);
    model
        .commit_journal_redo(&root, provenance().into())
        .unwrap();
    verify(&model);
    model = ProjectResolver::new(&root).resolve().unwrap();
    verify(&model);
    assert_eq!(raw(&model, map), revised);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn new_explicit_current_mapping_materialization_keeps_old_bound_net_unchanged() {
    use crate::api::native_write::board_components::pad_materialization::{
        PadLandPatternSource, build_materialize_component_pad_sources,
    };
    let physical_fixture::CurrentMaterializationFixture {
        root,
        mut model,
        ci,
        old_ci,
        old_net,
        old_members,
        package,
        old_pads,
        map,
        revised,
        new_symbol,
        new_ci,
        new_package,
        selected,
        mut request,
        ..
    } = physical_fixture::current_materialization_fixture("saved-map-new-current-materialization");
    let p = build_materialize_component_pad_sources(&model, provenance(), &request).unwrap();
    commit_prepared(&mut model, &root, p).unwrap();
    // Initial absent/empty restoration has its own focused history cases.
    // This case exercises explicit refresh compensation over populated pads,
    // with producer-qualified target captures and exact retained identities.
    let source = model
        .materialized_source_shard_value(SourceShardKind::BoardRoot)
        .unwrap();
    let footprint = match &request.land_pattern {
        PadLandPatternSource::Footprint(id) => *id,
        _ => unreachable!(),
    };
    let owner = pins::capture_root(
        &model,
        footprint,
        crate::substrate::SourceShardTaxon::PoolFootprint,
    )
    .unwrap();
    request.correspondences=source["component_pads"][new_package.to_string()].as_array().unwrap().iter().map(|raw| {
        let pad:pins::ComponentPadSource=serde_json::from_value(raw.clone()).unwrap();
        crate::api::native_write::board_components::pad_materialization::PlacedPadCorrespondence {
            placed_pad_id:pad.uuid,
            target_library_pad:pins::capture_pad(&model,&owner,pad.source_binding.as_ref().unwrap().library_pad.object_id).unwrap(),
        }
    }).collect();
    let p = build_materialize_component_pad_sources(&model, provenance(), &request).unwrap();
    commit_prepared(&mut model, &root, p).unwrap();
    model
        .commit_journal_undo(&root, provenance().into())
        .unwrap();
    assert_eq!(
        net_correspondence_status(&model, old_net),
        NetCorrespondenceStatus::Complete
    );
    model
        .commit_journal_redo(&root, provenance().into())
        .unwrap();
    let (new_net, new_library_pad, _) =
        connected(&mut model, &root, new_symbol, new_package, new_ci, map);
    assert_ne!(
        new_library_pad, selected,
        "revised correspondence uses the new cell assignment"
    );
    model = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(model.component_instances[&ci], old_ci);
    let snapshot = ElectricalSelectionSnapshot::capture(&model).unwrap();
    assert_eq!(
        snapshot
            .global_net(snapshot.revision(), old_net)
            .unwrap()
            .owned,
        old_members
    );
    assert_eq!(
        net_correspondence_status(&model, new_net),
        NetCorrespondenceStatus::Complete
    );
    let board = model
        .materialized_source_shard_value(SourceShardKind::BoardRoot)
        .unwrap();
    assert_eq!(board["component_pads"][package.to_string()], old_pads);
    for pad in board["component_pads"][new_package.to_string()]
        .as_array()
        .unwrap()
    {
        let pad: pins::ComponentPadSource = serde_json::from_value(pad.clone()).unwrap();
        let pin = pad
            .source_binding
            .as_ref()
            .unwrap()
            .pin_pad_map
            .as_ref()
            .unwrap();
        assert_eq!(pin.object_revision, ObjectRevision(1));
        assert_eq!(pins::source_value(&model, pin).unwrap(), revised);
    }
    assert_eq!(
        board["component_pads"][new_package.to_string()]
            .as_array()
            .unwrap()
            .len(),
        8
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn ordinary_pending_map_values_never_supply_replacement_cells_as_an_old_revision() {
    let (root, mut model, _, _, _, _, map, _) =
        physical_fixture::eight_pad_fixture("map-pending-reader-boundary");
    let original = raw(&model, map);
    let mut rows = original["mappings"].as_object().unwrap().clone();
    let first = rows.keys().next().unwrap().clone();
    let second = rows.keys().nth(1).unwrap().clone();
    let a = rows[&first].clone();
    let b = rows[&second].clone();
    assert_ne!(a, b);
    rows.insert(first, b);
    rows.insert(second, a);
    let p=(find_native_write_verb("datum.library.edit").unwrap().build)(
        &NativeWriteContext { model:&model,project_root:&root },provenance(),
        json!({"kind":"pin_pad_maps","object_id":map,"pool_path":"pool","fields":{"mappings":rows}}),
    ).unwrap();
    let basis =
        crate::substrate::electrical_basis::ElectricalBasis::new(&model, &p.batch.operations)
            .unwrap();
    let old = reference(&model, map);
    let mut replacement = original.clone();
    replacement["mappings"] = Value::Object(rows);
    // Reproduce the former unqualified composition on the very same frame:
    // the old counter plus the projected callback returns future cell values.
    let unqualified = model
        .pool_object_revision_with_reader(map, "pin_pad_maps", old.object_revision, &|_| {
            Ok(basis.object(&model, map, "pin_pad_maps").unwrap().clone())
        })
        .unwrap();
    assert_eq!(unqualified, replacement);
    assert_ne!(unqualified, original);
    assert!(basis.pinned_map(&model, &old).is_err());
    assert!(
        basis
            .pinned_map(
                &model,
                &RevisionedRef {
                    object_id: map,
                    object_revision: ObjectRevision(old.object_revision.0 + 1)
                }
            )
            .is_err()
    );
    eprintln!(
        "pending Map regression: unqualified before-counter callback returns replacement cells; declared old/future revision reads refuse this pending frame"
    );
    // Accepted post-commit frames continue to resolve both declared payloads.
    drop(basis);
    commit_prepared(&mut model, &root, p).unwrap();
    let accepted = crate::substrate::electrical_basis::ElectricalBasis::new(&model, &[]).unwrap();
    assert_eq!(accepted.pinned_map(&model, &old).unwrap(), original);
    assert_eq!(
        accepted
            .pinned_map(&model, &reference(&model, map))
            .unwrap(),
        replacement
    );
    std::fs::remove_dir_all(root).unwrap();
}
