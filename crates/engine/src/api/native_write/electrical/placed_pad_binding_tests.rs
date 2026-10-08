//! Reuse the complete PM053 fixture to prove PM057 positive electrical delivery.
use super::super::board_components::pad_materialization::*;
use super::super::{
    commit_prepared,
    library::{
        PoolLibraryObjectTarget as Target, PoolLibraryOperationSpec as Spec,
        build_pool_library_write,
    },
};
use super::tests::provenance;
use super::*;
use crate::substrate::{
    ElectricalIdentity, NetCorrespondenceStatus, ProjectResolver, SourceShardKind,
    component_pad_source::component_pad_projection, net_correspondence_status,
};
use serde_json::json;
#[test]
fn journaled_placed_pad_registration_uses_complete_existing_pm053_correspondence() {
    prove(false);
}
#[test]
fn exact30_placed_pad_preserves_complete_electrical_run_and_net() {
    prove(true);
}
fn prove(rotated: bool) {
    let mut f = super::binding_fixture::fixture("placed_pad_pm053_delivery");
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
    let package_id: uuid::Uuid =
        serde_json::from_value(board["pads"][old_pad.to_string()]["package"].clone()).unwrap();
    let placement: crate::board::PlacedPackage =
        serde_json::from_value(board["packages"][package_id.to_string()].clone()).unwrap();
    let package = f
        .model
        .materialized_source_shard_value_by_relative_path(&format!(
            "pool/packages/{}.json",
            placement.package
        ))
        .unwrap();
    let stack: uuid::Uuid =
        serde_json::from_value(package["pads"][library_pad.to_string()]["padstack"].clone())
            .unwrap();
    let write=build_pool_library_write(&f.model,provenance(),None,vec![Spec::Create{target:Target::new("pool","padstacks",stack),object:json!({"schema_version":1,"uuid":stack,"name":"exact pad","aperture":{"circle":{"diameter_nm":600000}},"drill_nm":null,"plated":false})}]).unwrap();
    commit_prepared(&mut f.model, &f.root, write).unwrap();
    let old: crate::board::PlacedPad =
        serde_json::from_value(board["pads"][old_pad.to_string()].clone()).unwrap();
    let write = super::super::board_routing::build_delete_board_pad(
        &f.model,
        provenance(),
        old.uuid,
        serde_json::to_value(&old).unwrap(),
    )
    .unwrap();
    commit_prepared(&mut f.model, &f.root, write).unwrap();
    let id = serde_json::from_value(board["uuid"].clone()).unwrap();
    let write = super::super::board_layout::source_profile::build_adopt_board_source_profile(
        &f.model,
        provenance(),
        id,
    )
    .unwrap();
    commit_prepared(&mut f.model, &f.root, write).unwrap();
    let request = ComponentPadMaterializationRequest {
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
    let pad: crate::substrate::component_pad_source::ComponentPadSource =
        serde_json::from_value(source["component_pads"][package_id.to_string()][0].clone())
            .unwrap();
    assert_eq!(
        component_pad_projection(&f.model, &source).unwrap().pads[&pad.uuid].net,
        None
    );
    if let ElectricalIdentity::NetRelationship { evidence, .. } = &mut f.relationship.identity {
        evidence[0].board_pad.object_id = pad.uuid;
        evidence[0].board_pad.object_revision = f.model.objects[&pad.uuid].object_revision;
    }
    let write = build_adopt_pin_correspondence(
        &f.model,
        provenance(),
        f.relationship.clone(),
        &[PlacedPinAdoption {
            sheet_id: f.sheet,
            symbol_id: f.symbol.uuid,
            placed_pin_id: f.symbol.pins[0].uuid,
            library_pin_id: f.library_pin,
        }],
    )
    .unwrap();
    commit_prepared(&mut f.model, &f.root, write).unwrap();
    assert_eq!(
        net_correspondence_status(&f.model, f.logical),
        NetCorrespondenceStatus::Complete
    );
    if rotated {
        let write = super::super::board_components::build_edit_board_package(
            &f.model,
            provenance(),
            package_id,
            super::super::board_components::BoardPackageEdit::Rotation { rotation: 30 },
        )
        .unwrap();
        commit_prepared(&mut f.model, &f.root, write).unwrap();
    }
    let source = f
        .model
        .materialized_source_shard_value(SourceShardKind::BoardRoot)
        .unwrap();
    let projection = component_pad_projection(&f.model, &source).unwrap();
    let net = projection.query_pads[&pad.uuid].terminal().net.unwrap();
    let mut expected = std::collections::BTreeSet::from([crate::substrate::ElectricalOccurrence {
        class: "pads".into(),
        source_id: pad.uuid,
        instance_path: vec![],
    }]);
    if rotated {
        let track = crate::board::Track::straight(
            uuid::Uuid::new_v4(),
            net,
            crate::ir::geometry::Point::new(-1_000_000, 0),
            crate::ir::geometry::Point::new(1_000_000, 0),
            100_000,
            1,
        );
        let write =
            super::super::board_routing::build_place_board_track(&f.model, provenance(), &track)
                .unwrap();
        commit_prepared(&mut f.model, &f.root, write).unwrap();
        let origin = crate::substrate::ElectricalOccurrence {
            class: "tracks".into(),
            source_id: track.uuid,
            instance_path: vec![],
        };
        expected.insert(origin.clone());
        let before = std::fs::read(f.root.join("board/board.json")).unwrap();
        let snapshot = crate::substrate::ElectricalSelectionSnapshot::capture(&f.model).unwrap();
        let captured = snapshot.board_source().unwrap();
        let typed = captured
            .query_pads
            .get(&pad.uuid)
            .expect("captured exact Pad");
        assert_eq!(
            typed.terminal().net,
            Some(net),
            "source qualification lost after Track write"
        );
        assert!(
            typed.contact_track(&track).unwrap(),
            "exact occupied contact lost"
        );
        let run = snapshot
            .board_run(
                &f.model.model_revision,
                &origin,
                Some(crate::ir::geometry::Point::zero()),
            )
            .unwrap();
        assert_eq!(run.members, expected);
        let members = snapshot
            .global_net(&f.model.model_revision, f.logical)
            .unwrap();
        assert_eq!(
            members
                .owned
                .iter()
                .filter(|o| matches!(o.class.as_str(), "pads" | "tracks"))
                .cloned()
                .collect::<std::collections::BTreeSet<_>>(),
            expected
        );
        assert_eq!(
            std::fs::read(f.root.join("board/board.json")).unwrap(),
            before
        );
    } else {
        let snapshot = crate::substrate::ElectricalSelectionSnapshot::capture(&f.model).unwrap();
        let members = snapshot
            .global_net(&f.model.model_revision, f.logical)
            .unwrap();
        assert!(members.owned.iter().any(|m| m.source_id == pad.uuid));
    }
    let reopened = ProjectResolver::new(&f.root).resolve().unwrap();
    assert_eq!(
        net_correspondence_status(&reopened, f.logical),
        NetCorrespondenceStatus::Complete
    );
    for id in [pad.uuid, library_pad, package_id, stack, ci, map] {
        assert_eq!(reopened.objects[&id], f.model.objects[&id]);
    }
    assert_eq!(reopened.model_revision, f.model.model_revision);
    let set_part = super::super::board_components::build_edit_board_package(
        &f.model,
        provenance(),
        package_id,
        super::super::board_components::BoardPackageEdit::Part {
            part_id: uuid::Uuid::new_v4(),
        },
    )
    .unwrap();
    commit_prepared(&mut f.model, &f.root, set_part).unwrap();
    assert_ne!(
        net_correspondence_status(&f.model, f.logical),
        NetCorrespondenceStatus::Complete
    );
    let source = f
        .model
        .materialized_source_shard_value(SourceShardKind::BoardRoot)
        .unwrap();
    assert_eq!(
        component_pad_projection(&f.model, &source)
            .unwrap()
            .query_pads[&pad.uuid]
            .terminal()
            .net,
        None
    );
    std::fs::remove_dir_all(f.root).unwrap();
}
