//! Current CI/Pad/Net closure through actual canonical source and history owners.
use super::*;
use crate::api::native_write::electrical::binding_fixture::{self, BindingFixture};
use crate::api::native_write::{
    self, BatchComposer, commit_prepared, electrical, library, schematic_connectivity,
};
use crate::board::{Net, Track};
use crate::ir::geometry::Point;
use crate::schematic::{LabelKind, NetLabel, SchematicWire};
use crate::substrate::{
    ElectricalIdentity, ElectricalOccurrence, ElectricalSelectionSnapshot, NetCorrespondenceStatus,
    ObjectRevision, ProjectResolver, component_pad_source::component_pad_projection,
    net_correspondence_status,
};
use serde_json::json;
mod tamper;

fn provenance() -> WriteProvenance {
    WriteProvenance::new(
        "component-binding-proof",
        crate::substrate::CommitSource::Test,
        "canonical current component/Pad/Net reference closure",
    )
}
fn authored_bytes(root: &std::path::Path, model: &DesignModel) -> BTreeMap<String, Vec<u8>> {
    model
        .source_shards
        .iter()
        .map(|s| {
            (
                s.relative_path.clone(),
                std::fs::read(root.join(&s.relative_path)).unwrap(),
            )
        })
        .collect()
}

fn occurrence(class: &str, source_id: Uuid) -> ElectricalOccurrence {
    ElectricalOccurrence {
        class: class.into(),
        source_id,
        instance_path: vec![],
    }
}
fn setup() -> (
    BindingFixture,
    ComponentPadMaterializationRequest,
    BTreeSet<ElectricalOccurrence>,
    BTreeSet<ElectricalOccurrence>,
) {
    let mut f = binding_fixture::fixture("current_component_pad_net");
    let (old_pad, library_pad, map, ci) =
        if let ElectricalIdentity::NetRelationship { evidence, .. } = &f.relationship.identity {
            (
                evidence[0].board_pad.object_id,
                evidence[0].library_pad.as_ref().unwrap().object_id,
                evidence[0].pin_pad_map.object_id,
                evidence[0].component_instance.object_id,
            )
        } else {
            unreachable!()
        };
    let board = f
        .model
        .materialized_source_shard_value(SourceShardKind::BoardRoot)
        .unwrap();
    let old: crate::board::PlacedPad =
        serde_json::from_value(board["pads"][old_pad.to_string()].clone()).unwrap();
    let placement: PlacedPackage =
        serde_json::from_value(board["packages"][old.package.to_string()].clone()).unwrap();
    let package = f
        .model
        .materialized_source_shard_value_by_relative_path(&format!(
            "pool/packages/{}.json",
            placement.package
        ))
        .unwrap();
    let stack: Uuid =
        serde_json::from_value(package["pads"][library_pad.to_string()]["padstack"].clone())
            .unwrap();
    let write=library::build_pool_library_write(&f.model,provenance(),None,vec![library::PoolLibraryOperationSpec::Create {
        target:library::PoolLibraryObjectTarget::new("pool","padstacks",stack),
        object:json!({"schema_version":1,"uuid":stack,"name":"exact","aperture":{"circle":{"diameter_nm":600000}},"drill_nm":null,"plated":false})
    }]).unwrap();
    commit_prepared(&mut f.model, &f.root, write).unwrap();
    let write = native_write::board_routing::build_delete_board_pad(
        &f.model,
        provenance(),
        old.uuid,
        serde_json::to_value(old).unwrap(),
    )
    .unwrap();
    commit_prepared(&mut f.model, &f.root, write).unwrap();
    let board_id = serde_json::from_value(board["uuid"].clone()).unwrap();
    let write = native_write::board_layout::source_profile::build_adopt_board_source_profile(
        &f.model,
        provenance(),
        board_id,
    )
    .unwrap();
    commit_prepared(&mut f.model, &f.root, write).unwrap();
    let mut request = ComponentPadMaterializationRequest {
        placement,
        land_pattern: PadLandPatternSource::LegacyPackage,
        component_instance: Some(ci),
        pin_pad_map: Some(map),
        correspondences: vec![],
        adopt_existing: false,
    };
    let write = build_materialize_component_pad_sources(&f.model, provenance(), &request).unwrap();
    commit_prepared(&mut f.model, &f.root, write).unwrap();
    let source = f
        .model
        .materialized_source_shard_value(SourceShardKind::BoardRoot)
        .unwrap();
    let pad: ComponentPadSource = serde_json::from_value(
        source["component_pads"][request.placement.uuid.to_string()][0].clone(),
    )
    .unwrap();
    request.correspondences.push(PlacedPadCorrespondence {
        placed_pad_id: pad.uuid,
        target_library_pad: pad.source_binding.as_ref().unwrap().library_pad.clone(),
    });
    let wire = SchematicWire {
        uuid: Uuid::new_v4(),
        from: Point::zero(),
        to: Point::new(10_000_000, 0),
    };
    let write =
        schematic_connectivity::build_create_schematic_wire(&f.model, provenance(), f.sheet, &wire)
            .unwrap();
    commit_prepared(&mut f.model, &f.root, write).unwrap();
    let label = NetLabel {
        uuid: Uuid::new_v4(),
        kind: LabelKind::Global,
        name: "complete".into(),
        position: wire.to,
    };
    let write = schematic_connectivity::build_create_schematic_label(
        &f.model,
        provenance(),
        f.sheet,
        &label,
    )
    .unwrap();
    commit_prepared(&mut f.model, &f.root, write).unwrap();
    let board_net = if let ElectricalIdentity::NetRelationship {
        board_net: Some(net),
        ..
    } = &f.relationship.identity
    {
        net.object_id
    } else {
        unreachable!()
    };
    let foreign = Net::new(Uuid::new_v4(), "unrelated net display", Uuid::nil());
    let connected = Track::straight(
        Uuid::new_v4(),
        board_net,
        Point::zero(),
        Point::new(1_000_000, 0),
        100_000,
        1,
    );
    let disconnected = Track::straight(
        Uuid::new_v4(),
        board_net,
        Point::new(4_000_000, 0),
        Point::new(5_000_000, 0),
        100_000,
        1,
    );
    let other = Track::straight(
        Uuid::new_v4(),
        foreign.uuid,
        Point::new(0, 2_000_000),
        Point::new(1_000_000, 2_000_000),
        100_000,
        1,
    );
    let write = BatchComposer::compose(&f.model, provenance())
        .push_ops([
            Operation::CreateBoardNet {
                net_id: foreign.uuid,
                net: serde_json::to_value(foreign).unwrap(),
            },
            Operation::CreateBoardTrack {
                track_id: connected.uuid,
                track: serde_json::to_value(&connected).unwrap(),
            },
            Operation::CreateBoardTrack {
                track_id: disconnected.uuid,
                track: serde_json::to_value(&disconnected).unwrap(),
            },
            Operation::CreateBoardTrack {
                track_id: other.uuid,
                track: serde_json::to_value(&other).unwrap(),
            },
        ])
        .finish()
        .unwrap();
    commit_prepared(&mut f.model, &f.root, write).unwrap();
    if let ElectricalIdentity::NetRelationship { evidence, .. } = &mut f.relationship.identity {
        evidence[0].board_pad.object_id = pad.uuid;
        evidence[0].board_pad.object_revision = f.model.objects[&pad.uuid].object_revision;
        evidence[0].component_instance.object_revision = f.model.objects[&ci].object_revision;
    }
    let write = electrical::build_adopt_pin_correspondence(
        &f.model,
        provenance(),
        f.relationship.clone(),
        &[electrical::PlacedPinAdoption {
            sheet_id: f.sheet,
            symbol_id: f.symbol.uuid,
            placed_pin_id: f.symbol.pins[0].uuid,
            library_pin_id: f.library_pin,
        }],
    )
    .unwrap();
    commit_prepared(&mut f.model, &f.root, write).unwrap();
    f.model = ProjectResolver::new(&f.root).resolve().unwrap();
    // The explicit target is a fresh producer pin, including its history anchor;
    // the persisted old Pad binding remains the continuity source.
    let owner = source::capture_root(&f.model, request.placement.package, T::PoolPackage).unwrap();
    request.correspondences[0].target_library_pad =
        source::capture_pad(&f.model, &owner, library_pad).unwrap();
    let expected = BTreeSet::from([
        occurrence("pins", f.symbol.pins[0].uuid),
        occurrence("pads", pad.uuid),
        occurrence("wires", wire.uuid),
        occurrence("labels", label.uuid),
        occurrence("tracks", connected.uuid),
        occurrence("tracks", disconnected.uuid),
    ]);
    (
        f,
        request,
        expected,
        BTreeSet::from([
            occurrence("pads", pad.uuid),
            occurrence("tracks", connected.uuid),
        ]),
    )
}

fn check(
    model: &DesignModel,
    request: &ComponentPadMaterializationRequest,
    logical: Uuid,
    expected: &BTreeSet<ElectricalOccurrence>,
    expected_run: &BTreeSet<ElectricalOccurrence>,
) {
    assert_eq!(
        net_correspondence_status(model, logical),
        NetCorrespondenceStatus::Complete
    );
    let snapshot = ElectricalSelectionSnapshot::capture(model).unwrap();
    assert_eq!(
        snapshot
            .global_net(snapshot.revision(), logical)
            .unwrap()
            .owned,
        *expected
    );
    for origin in expected {
        assert_eq!(
            snapshot
                .net_for_origin(snapshot.revision(), origin)
                .unwrap()
                .owned,
            *expected
        );
    }
    let pad = expected.iter().find(|o| o.class == "pads").unwrap().clone();
    let run = snapshot.board_run(snapshot.revision(), &pad, None).unwrap();
    assert_eq!(&run.members, expected_run);
    let ci = &model.component_instances[&request.component_instance.unwrap()];
    for (class, id, target) in [
        ("symbols", ci.placed_symbol_refs[0], request.placement.uuid),
        ("packages", request.placement.uuid, ci.placed_symbol_refs[0]),
    ] {
        let context = snapshot
            .component_selection_context(
                snapshot.revision(),
                &BTreeSet::from([occurrence(class, id)]),
            )
            .unwrap();
        assert!(context.unavailable.is_empty());
        assert_eq!(context.relations.len(), 1);
        assert_eq!(
            context.relations.first().unwrap().counterpart.source_id,
            target
        );
    }
    let board = model
        .materialized_source_shard_value(SourceShardKind::BoardRoot)
        .unwrap();
    assert!(
        component_pad_projection(model, &board)
            .unwrap()
            .unavailable
            .is_empty()
    );
}

#[test]
fn materialization_refresh_preserves_current_net_run_and_bidirectional_component_context() {
    let (mut f, request, expected, expected_run) = setup();
    check(&f.model, &request, f.logical, &expected, &expected_run);
    let write = build_materialize_component_pad_sources(&f.model, provenance(), &request).unwrap();
    commit_prepared(&mut f.model, &f.root, write).unwrap();
    f.model = ProjectResolver::new(&f.root).resolve().unwrap();
    check(&f.model, &request, f.logical, &expected, &expected_run);
    std::fs::remove_dir_all(f.root).unwrap();
}

#[test]
fn duplicate_or_forged_prospective_ci_refuses_without_authored_or_journal_changes() {
    let (mut f, request, _, _) = setup();
    let before = authored_bytes(&f.root, &f.model);
    let journal = std::fs::read(crate::substrate::transaction_journal_path(&f.root)).unwrap();
    let valid = build_materialize_component_pad_sources(&f.model, provenance(), &request).unwrap();
    let mut duplicate = valid.clone();
    let ci = valid
        .batch
        .operations
        .iter()
        .find(|op| matches!(op, Operation::SetComponentInstance { .. }))
        .unwrap()
        .clone();
    duplicate.batch.operations.push(ci);
    assert!(commit_prepared(&mut f.model, &f.root, duplicate).is_err());
    let mut missing = valid.clone();
    missing
        .batch
        .operations
        .retain(|op| !matches!(op, Operation::SetElectricalIdentity { .. }));
    assert!(commit_prepared(&mut f.model, &f.root, missing).is_err());
    let mut bad_evidence = valid.clone();
    for op in &mut bad_evidence.batch.operations {
        if let Operation::SetElectricalIdentity { record, .. } = op
            && let ElectricalIdentity::NetRelationship { evidence, .. } = &mut record.identity
        {
            evidence[0].board_pad.object_revision = ObjectRevision(999);
        }
    }
    assert!(commit_prepared(&mut f.model, &f.root, bad_evidence).is_err());
    let mut forged = valid;
    for op in &mut forged.batch.operations {
        if let Operation::SetComponentInstance {
            component_instance, ..
        } = op
        {
            component_instance["placed_package_refs"][0]["object_revision"] = json!(999);
        }
    }
    assert!(commit_prepared(&mut f.model, &f.root, forged).is_err());
    assert_eq!(authored_bytes(&f.root, &f.model), before);
    assert_eq!(
        std::fs::read(crate::substrate::transaction_journal_path(&f.root)).unwrap(),
        journal
    );
    std::fs::remove_dir_all(f.root).unwrap();
}

#[test]
fn materialization_reference_closure_survives_recorded_undo_redo_and_reopen() {
    let (mut f, request, expected, expected_run) = setup();
    let before = authored_bytes(&f.root, &f.model);
    let write = build_materialize_component_pad_sources(&f.model, provenance(), &request).unwrap();
    commit_prepared(&mut f.model, &f.root, write).unwrap();
    let after = authored_bytes(&f.root, &f.model);
    let prefix = std::fs::read(crate::substrate::transaction_journal_path(&f.root)).unwrap();
    let ci = request.component_instance.unwrap();
    let durable = ProjectResolver::new(&f.root).resolve().unwrap();
    let undo_revisions = next_electrical_revisions(&durable, ci);
    println!(
        "COMPENSATION_CANONICAL_PREBASIS {}",
        json!({"caller_next":next_electrical_revisions(&f.model,ci),"durable_next":undo_revisions})
    );
    let undo_epochs = next_bound_epochs(&f.model, &request, &expected);
    f.model
        .commit_journal_undo(&f.root, provenance().into())
        .unwrap();
    assert_bound_epochs(&f.model, &undo_epochs);
    assert_history_snapshot(
        &before,
        &authored_bytes(&f.root, &f.model),
        &f.model,
        ci,
        &undo_revisions,
    );
    check(&f.model, &request, f.logical, &expected, &expected_run);
    f.model = ProjectResolver::new(&f.root).resolve().unwrap();
    assert_bound_epochs(&f.model, &undo_epochs);
    check(&f.model, &request, f.logical, &expected, &expected_run);
    tamper::refuse(&f.root, &f.model);
    let redo_revisions = next_electrical_revisions(&f.model, ci);
    let redo_epochs = next_bound_epochs(&f.model, &request, &expected);
    f.model
        .commit_journal_redo(&f.root, provenance().into())
        .unwrap();
    assert_bound_epochs(&f.model, &redo_epochs);
    assert_history_snapshot(
        &after,
        &authored_bytes(&f.root, &f.model),
        &f.model,
        ci,
        &redo_revisions,
    );
    check(&f.model, &request, f.logical, &expected, &expected_run);
    f.model = ProjectResolver::new(&f.root).resolve().unwrap();
    assert_bound_epochs(&f.model, &redo_epochs);
    check(&f.model, &request, f.logical, &expected, &expected_run);
    tamper::refuse(&f.root, &f.model);
    assert!(
        std::fs::read(crate::substrate::transaction_journal_path(&f.root))
            .unwrap()
            .starts_with(&prefix)
    );
    std::fs::remove_dir_all(f.root).unwrap();
}

fn next_electrical_revisions(model: &DesignModel, ci: Uuid) -> BTreeMap<Uuid, ObjectRevision> {
    model
        .electrical_identities
        .values()
        .filter(|r|matches!(&r.identity,ElectricalIdentity::NetRelationship{evidence,..} if evidence.iter().any(|e|e.component_instance.object_id==ci)))
        .map(|r| (r.id, ObjectRevision(model.objects[&r.id].object_revision.0 + 1)))
        .collect()
}
fn next_bound_epochs(
    model: &DesignModel,
    request: &ComponentPadMaterializationRequest,
    members: &BTreeSet<ElectricalOccurrence>,
) -> BTreeMap<Uuid, ObjectRevision> {
    members
        .iter()
        .filter(|o| o.class == "pads")
        .map(|o| o.source_id)
        .chain([request.component_instance.unwrap(), request.placement.uuid])
        .map(|id| (id, ObjectRevision(model.objects[&id].object_revision.0 + 1)))
        .collect()
}
fn assert_bound_epochs(model: &DesignModel, expected: &BTreeMap<Uuid, ObjectRevision>) {
    for (id, revision) in expected {
        assert_eq!(
            model.objects[id].object_revision, *revision,
            "one monotonic advance for {id}"
        );
    }
}
fn assert_history_snapshot(
    expected: &BTreeMap<String, Vec<u8>>,
    actual: &BTreeMap<String, Vec<u8>>,
    model: &DesignModel,
    ci: Uuid,
    electrical: &BTreeMap<Uuid, ObjectRevision>,
) {
    assert_eq!(
        expected.keys().collect::<Vec<_>>(),
        actual.keys().collect::<Vec<_>>()
    );
    for (path, old) in expected {
        let new = &actual[path];
        if old == new {
            continue;
        }
        let mut left: serde_json::Value = serde_json::from_slice(old).unwrap();
        let mut right: serde_json::Value = serde_json::from_slice(new).unwrap();
        let mut allowed = false;
        if right["component_instance"]["uuid"] == json!(ci) {
            assert_eq!(
                right["component_instance"]["object_revision"],
                json!(model.objects[&ci].object_revision)
            );
            left["component_instance"]["object_revision"] = json!(null);
            right["component_instance"]["object_revision"] = json!(null);
            for (a, b) in left["component_instance"]["placed_package_refs"]
                .as_array_mut()
                .unwrap()
                .iter_mut()
                .zip(
                    right["component_instance"]["placed_package_refs"]
                        .as_array_mut()
                        .unwrap(),
                )
            {
                let id: Uuid = serde_json::from_value(b["object_id"].clone()).unwrap();
                assert_eq!(
                    b["object_revision"],
                    json!(model.objects[&id].object_revision)
                );
                a["object_revision"] = json!(null);
                b["object_revision"] = json!(null);
            }
            allowed = true;
        }
        if let Some(id) = right["record"]["id"]
            .as_str()
            .and_then(|s| s.parse::<Uuid>().ok())
        {
            if let Some(revision) = electrical.get(&id) {
                assert_eq!(right["record"]["object_revision"], json!(revision));
                left["record"]["object_revision"] = json!(null);
                right["record"]["object_revision"] = json!(null);
                for (a, b) in left["record"]["identity"]["evidence"]
                    .as_array_mut()
                    .unwrap()
                    .iter_mut()
                    .zip(
                        right["record"]["identity"]["evidence"]
                            .as_array_mut()
                            .unwrap(),
                    )
                {
                    if b["component_instance"]["object_id"] != json!(ci) {
                        continue;
                    }
                    for key in ["component_instance", "board_pad"] {
                        let id: Uuid = serde_json::from_value(b[key]["object_id"].clone()).unwrap();
                        assert_eq!(
                            b[key]["object_revision"],
                            json!(model.objects[&id].object_revision)
                        );
                        a[key]["object_revision"] = json!(null);
                        b[key]["object_revision"] = json!(null);
                    }
                }
                allowed = true;
            }
        }
        if let (Some(a), Some(b)) = (
            left["component_pads"].as_object_mut(),
            right["component_pads"].as_object_mut(),
        ) {
            for (key, pads) in a {
                for (a, b) in pads
                    .as_array_mut()
                    .unwrap()
                    .iter_mut()
                    .zip(b[key].as_array_mut().unwrap())
                {
                    if b["source_binding"]["component_instance"]["object_id"] != json!(ci) {
                        continue;
                    }
                    assert_eq!(
                        b["source_binding"]["component_instance"]["object_revision"],
                        json!(model.objects[&ci].object_revision)
                    );
                    a["source_binding"]["component_instance"]["object_revision"] = json!(null);
                    b["source_binding"]["component_instance"]["object_revision"] = json!(null);
                }
            }
            allowed = true;
        }
        assert!(allowed, "unrelated source changed: {path}");
        assert_eq!(
            left, right,
            "nonallowlisted semantic/metadata change: {path}"
        );
    }
}

#[test]
fn history_stale_missing_duplicate_and_foreign_controls_preserve_source_and_journal() {
    let (mut f, request, _, _) = setup();
    let write = build_materialize_component_pad_sources(&f.model, provenance(), &request).unwrap();
    commit_prepared(&mut f.model, &f.root, write).unwrap();
    let before = authored_bytes(&f.root, &f.model);
    let journal = std::fs::read(crate::substrate::transaction_journal_path(&f.root)).unwrap();
    for control in 0..7 {
        let mut model = f.model.clone();
        match control {
            0 => {
                model
                    .objects
                    .get_mut(&request.placement.uuid)
                    .unwrap()
                    .object_revision
                    .0 += 1
            }
            1 => model
                .journal
                .last_mut()
                .unwrap()
                .inverse_operations
                .retain(|op| !matches!(op, Operation::SetElectricalIdentity { .. })),
            2 => {
                let ops = &mut model.journal.last_mut().unwrap().inverse_operations;
                let ci = ops
                    .iter()
                    .find(|op| matches!(op, Operation::SetComponentInstance { .. }))
                    .unwrap()
                    .clone();
                ops.push(ci);
            }
            3 => {
                for op in &mut model.journal.last_mut().unwrap().inverse_operations {
                    if let Operation::SetComponentInstance {
                        component_instance, ..
                    } = op
                    {
                        component_instance["part_ref"]["object_id"] = json!(Uuid::new_v4());
                    }
                }
            }
            4 => model.model_revision.0.push('x'),
            5 => {
                let id = *next_electrical_revisions(&model, request.component_instance.unwrap())
                    .keys()
                    .next()
                    .unwrap();
                model.objects.get_mut(&id).unwrap().object_revision.0 += 7;
            }
            _ => model.journal[0].after_model_revision.0.push('x'),
        }
        assert!(
            model
                .commit_journal_undo(&f.root, provenance().into())
                .is_err(),
            "control {control}"
        );
        assert_eq!(authored_bytes(&f.root, &f.model), before);
        assert_eq!(
            std::fs::read(crate::substrate::transaction_journal_path(&f.root)).unwrap(),
            journal
        );
    }
    std::fs::remove_dir_all(f.root).unwrap();
}
