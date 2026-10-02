//! Z02-Z04 successor oracles over canonical native source/history and fill.
use super::*;
use crate::connectivity::{ZoneClearReason, ZoneRegionQualifier, ZoneRegionSuccessor};

fn selected(
    model: &DesignModel,
    zone: &Zone,
) -> (ElectricalSelectionSnapshot, ZoneRegionQualifier) {
    let snapshot = ElectricalSelectionSnapshot::capture(model).unwrap();
    let result = snapshot
        .board_run(
            snapshot.revision(),
            &member("zones", zone.uuid),
            Some(Point::new(5, 5)),
        )
        .unwrap();
    assert_eq!(result.zone_regions.len(), 1);
    (snapshot, result.zone_regions[0].clone())
}
fn evolve(
    model: &DesignModel,
    previous: &ElectricalSelectionSnapshot,
    token: &ZoneRegionQualifier,
) -> ZoneRegionSuccessor {
    let files: Vec<_> = model
        .source_shards
        .iter()
        .map(|s| (s.path.clone(), std::fs::read(&s.path).unwrap()))
        .collect();
    let snapshot = ElectricalSelectionSnapshot::capture(model).unwrap();
    let result = snapshot
        .zone_region_successor(snapshot.revision(), previous, token)
        .unwrap();
    for (path, bytes) in files {
        assert_eq!(std::fs::read(path).unwrap(), bytes);
    }
    result
}

#[test]
fn equivalent_refill_rebinds_region_without_array_winding_or_decomposition_identity() {
    let (root, mut model, net) = fixture("zone_region_equivalent");
    let zone = zone(&mut model, &root, net);
    fill(
        &mut model,
        &root,
        &zone,
        ZoneFillState::Filled,
        vec![rectangle(0, 10), rectangle(20, 10)],
    );
    let (old, token) = selected(&model, &zone);
    let mut reverse = rectangle(0, 5);
    reverse.vertices.reverse();
    fill(
        &mut model,
        &root,
        &zone,
        ZoneFillState::Filled,
        vec![rectangle(20, 10), rectangle(5, 5), reverse],
    );
    let ZoneRegionSuccessor::Unique { qualifier, run } = evolve(&model, &old, &token) else {
        panic!("same occupied region must preserve")
    };
    assert_eq!(qualifier.source(), token.source());
    assert_eq!(run.members, BTreeSet::from([member("zones", zone.uuid)]));
    assert_eq!(run.zone_copper[0].polygons.len(), 2);
    assert!(
        run.zone_copper[0]
            .polygons
            .iter()
            .all(|p| p.vertices.iter().all(|v| v.x <= 10))
    );
    assert_ne!(
        qualifier, token,
        "new fill basis requires a rebound engine token"
    );
    let reopened = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(
        evolve(&reopened, &old, &token),
        ZoneRegionSuccessor::Unique { qualifier, run }
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn certified_external_bridge_merge_preserves_and_its_removal_splits_physical_successors() {
    let (root, mut model, net) = fixture("zone_region_external_partition");
    let zone = zone(&mut model, &root, net);
    let copper = vec![rectangle(0, 10), rectangle(20, 10)];
    fill(
        &mut model,
        &root,
        &zone,
        ZoneFillState::Filled,
        copper.clone(),
    );
    let (old, token) = selected(&model, &zone);
    let bridge = line(net, (10, 5), (20, 5));
    tracks(&mut model, &root, std::slice::from_ref(&bridge));
    assert!(matches!(
        evolve(&model, &old, &token),
        ZoneRegionSuccessor::Suspended { .. }
    ));
    fill(
        &mut model,
        &root,
        &zone,
        ZoneFillState::Filled,
        copper.clone(),
    );
    let ZoneRegionSuccessor::Unique {
        qualifier: merged,
        run,
    } = evolve(&model, &old, &token)
    else {
        panic!("unique same-copper merge")
    };
    assert_eq!(
        run.members,
        BTreeSet::from([member("zones", zone.uuid), member("tracks", bridge.uuid)])
    );
    assert_eq!(run.zone_copper[0].polygons.len(), 2);
    let before_split = ElectricalSelectionSnapshot::capture(&model).unwrap();
    let write = build_delete_board_track(
        &model,
        provenance(),
        bridge.uuid,
        serde_json::to_value(&bridge).unwrap(),
    )
    .unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    fill(&mut model, &root, &zone, ZoneFillState::Filled, copper);
    let ZoneRegionSuccessor::Split { successors } = evolve(&model, &before_split, &merged) else {
        panic!("one-to-two physical split must clear downstream")
    };
    assert_eq!(successors.len(), 2);
    assert!(successors.iter().all(|q| q.source() == token.source()));
    let reopened = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(
        evolve(&reopened, &before_split, &merged),
        ZoneRegionSuccessor::Split { successors }
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn current_changed_copper_is_unknown_stale_suspends_and_only_certified_empty_or_deletion_clears() {
    let (root, mut model, net) = fixture("zone_region_unknown_empty");
    let zone = zone(&mut model, &root, net);
    fill(
        &mut model,
        &root,
        &zone,
        ZoneFillState::Filled,
        vec![rectangle(0, 10)],
    );
    let (old, token) = selected(&model, &zone);
    // Old hit and positive-area overlap survive, but neither supplies lineage.
    fill(
        &mut model,
        &root,
        &zone,
        ZoneFillState::Filled,
        vec![rectangle(1, 10)],
    );
    assert_eq!(evolve(&model, &old, &token), ZoneRegionSuccessor::Unknown);
    for state in [
        ZoneFillState::Stale,
        ZoneFillState::Unsupported,
        ZoneFillState::Unfilled,
    ] {
        fill(&mut model, &root, &zone, state, vec![]);
        assert!(matches!(
            evolve(&model, &old, &token),
            ZoneRegionSuccessor::Suspended {
                failure: PhysicalQueryFailure::UnavailableFill { .. }
            }
        ));
    }
    fill(&mut model, &root, &zone, ZoneFillState::Filled, vec![]);
    assert_eq!(
        evolve(&model, &old, &token),
        ZoneRegionSuccessor::Cleared {
            reason: ZoneClearReason::CurrentEmpty
        }
    );
    let write = build_delete_board_zone(
        &model,
        provenance(),
        zone.uuid,
        serde_json::to_value(&zone).unwrap(),
    )
    .unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    assert_eq!(
        evolve(&model, &old, &token),
        ZoneRegionSuccessor::Cleared {
            reason: ZoneClearReason::Deleted
        }
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn exact_canonical_translation_with_matching_fill_certifies_unique_successor() {
    let (root, mut model, net) = fixture("zone_region_transform");
    let zone = zone(&mut model, &root, net);
    fill(
        &mut model,
        &root,
        &zone,
        ZoneFillState::Filled,
        vec![rectangle(0, 10), rectangle(20, 10)],
    );
    let (old, token) = selected(&model, &zone);
    let mut moved = zone.clone();
    for v in &mut moved.polygon.vertices {
        v.x += 100;
        v.y += 50;
    }
    let write = build_set_board_zone(&model, provenance(), &moved).unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    assert!(matches!(
        evolve(&model, &old, &token),
        ZoneRegionSuccessor::Suspended { .. }
    ));
    let mut copper = vec![rectangle(100, 10), rectangle(120, 10)];
    for p in &mut copper {
        for v in &mut p.vertices {
            v.y += 50;
        }
    }
    fill(&mut model, &root, &moved, ZoneFillState::Filled, copper);
    let ZoneRegionSuccessor::Unique { qualifier, run } = evolve(&model, &old, &token) else {
        panic!("operation- and fill-bound translation")
    };
    assert_eq!(qualifier.source(), token.source());
    assert_eq!(
        run.zone_copper[0].polygons[0].vertices[0],
        Point::new(100, 50)
    );
    assert!(matches!(
        query(&model, &member("zones", zone.uuid), Some(Point::new(5, 5))),
        Err(ElectricalQueryFailure::Physical(
            PhysicalQueryFailure::OriginOutsideCopper { .. }
        ))
    ));
    // A canonical transform does not authorize an inconsistent producer result.
    fill(
        &mut model,
        &root,
        &moved,
        ZoneFillState::Filled,
        vec![rectangle(1, 10)],
    );
    assert_eq!(evolve(&model, &old, &token), ZoneRegionSuccessor::Unknown);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn region_qualifiers_refuse_wrong_component_revision_project_and_source_basis() {
    let (root, mut model, net) = fixture("zone_region_token");
    let zone = zone(&mut model, &root, net);
    fill(
        &mut model,
        &root,
        &zone,
        ZoneFillState::Filled,
        vec![rectangle(0, 10)],
    );
    let (old, token) = selected(&model, &zone);
    let snapshot = ElectricalSelectionSnapshot::capture(&model).unwrap();
    for (field, value) in [
        ("component", serde_json::json!(9999)),
        ("revision", serde_json::json!("wrong")),
        ("project_id", serde_json::json!(Uuid::new_v4())),
        ("source_basis", serde_json::json!([])),
    ] {
        let mut value_token = serde_json::to_value(&token).unwrap();
        value_token[field] = value;
        let forged = serde_json::from_value(value_token).unwrap();
        assert_eq!(
            snapshot.zone_region_successor(snapshot.revision(), &old, &forged),
            Err(ElectricalQueryFailure::Physical(
                PhysicalQueryFailure::InvalidQualifier
            ))
        );
    }
    assert_eq!(
        snapshot.zone_region_successor(&ModelRevision("wrong".into()), &old, &token),
        Err(ElectricalQueryFailure::StaleRevision)
    );
    let (other_root, other, _) = fixture("zone_region_other_project");
    let other = ElectricalSelectionSnapshot::capture(&other).unwrap();
    assert_eq!(
        other.zone_region_successor(other.revision(), &old, &token),
        Err(ElectricalQueryFailure::Physical(
            PhysicalQueryFailure::InvalidQualifier
        ))
    );
    std::fs::remove_dir_all(other_root).unwrap();
    std::fs::remove_dir_all(root).unwrap();
}
