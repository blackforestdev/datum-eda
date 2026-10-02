use super::super::commit_prepared;
use super::tests::{create, provenance};
use super::*;
use crate::ir::geometry::Point;
use crate::substrate::{
    NetCorrespondenceStatus, NetRelationshipIntent, ProjectResolver, RevisionedRef,
    net_correspondence_status,
};
use uuid::Uuid;
fn reference(id: Uuid) -> RevisionedRef {
    RevisionedRef {
        object_id: id,
        object_revision: ObjectRevision(0),
    }
}

#[test]
fn explicit_pinned_pin_pad_correspondence_reopen_and_refusal() {
    let super::binding_fixture::BindingFixture {
        root,
        mut model,
        sheet,
        symbol,
        logical,
        relationship,
        library_pin,
        ..
    } = super::binding_fixture::fixture("electrical_native_binding");
    create(&mut model, &root, relationship.clone());
    assert_eq!(
        net_correspondence_status(&model, logical),
        NetCorrespondenceStatus::Unverified
    );
    let before_pin = symbol.pins[0].uuid;
    let prepared = build_adopt_pin_correspondence(
        &model,
        provenance(),
        relationship.clone(),
        &[PlacedPinAdoption {
            sheet_id: sheet,
            symbol_id: symbol.uuid,
            placed_pin_id: before_pin,
            library_pin_id: library_pin,
        }],
    )
    .unwrap();
    commit_prepared(&mut model, &root, prepared).unwrap();
    let relationship = model.electrical_identities[&relationship.id].clone();
    let source = model
        .materialized_source_shard_value_by_relative_path("schematic/sheets/main.json")
        .unwrap();
    assert_eq!(
        source["symbols"][symbol.uuid.to_string()]["pins"][0]["uuid"],
        before_pin.to_string()
    );
    assert_eq!(
        net_correspondence_status(&model, logical),
        NetCorrespondenceStatus::Complete
    );
    assert_eq!(
        net_correspondence_status(&ProjectResolver::new(&root).resolve().unwrap(), logical),
        NetCorrespondenceStatus::Complete
    );
    model
        .commit_journal_undo(&root, provenance().into())
        .unwrap();
    let reopened = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(
        net_correspondence_status(&reopened, logical),
        NetCorrespondenceStatus::Unverified
    );
    let source = reopened
        .materialized_source_shard_value_by_relative_path("schematic/sheets/main.json")
        .unwrap();
    assert_eq!(
        source["symbols"][symbol.uuid.to_string()]["pins"][0]["uuid"],
        before_pin.to_string()
    );
    assert!(
        source["symbols"][symbol.uuid.to_string()]["pins"][0]
            .get("library_pin")
            .is_none()
    );
    model
        .commit_journal_redo(&root, provenance().into())
        .unwrap();
    assert_eq!(
        net_correspondence_status(&ProjectResolver::new(&root).resolve().unwrap(), logical),
        NetCorrespondenceStatus::Complete
    );
    let mut wrong = relationship.clone();
    if let ElectricalIdentity::NetRelationship { evidence, .. } = &mut wrong.identity {
        evidence[0].library_pad = Some(reference(library_pin));
    }
    let prepared = build_set_electrical_identity(&model, provenance(), wrong).unwrap();
    let before = model.clone();
    assert!(commit_prepared(&mut model, &root, prepared).is_err());
    assert_eq!(model, before);
    let mut unverified = relationship.clone();
    if let ElectricalIdentity::NetRelationship { evidence, .. } = &mut unverified.identity {
        evidence[0].library_pad = None;
    }
    let prepared = build_set_electrical_identity(&model, provenance(), unverified).unwrap();
    commit_prepared(&mut model, &root, prepared).unwrap();
    assert_eq!(
        net_correspondence_status(&model, logical),
        NetCorrespondenceStatus::Unverified
    );
    model
        .commit_journal_undo(&root, provenance().into())
        .unwrap();
    assert_eq!(
        net_correspondence_status(&model, logical),
        NetCorrespondenceStatus::Complete
    );
    // A real topology edit invalidates correspondence in the same transaction;
    // physical board source is retained exactly and undo restores certification.
    let board_before = model
        .materialized_source_shard_value_by_relative_path("board/board.json")
        .unwrap();
    let wire = crate::schematic::SchematicWire {
        uuid: Uuid::new_v4(),
        from: Point::new(0, 0),
        to: Point::new(10, 0),
    };
    let prepared = super::super::schematic_connectivity::build_create_schematic_wire(
        &model,
        provenance(),
        sheet,
        &wire,
    )
    .unwrap();
    let before = model.electrical_identities.clone();
    let report = commit_prepared(&mut model, &root, prepared).unwrap();
    assert!(matches!(
        model.electrical_identities[&relationship.id].identity,
        ElectricalIdentity::NetRelationship {
            intent: NetRelationshipIntent::Pending,
            ..
        }
    ));
    assert!(report.transaction.operations.iter().any(|op| matches!(op,Operation::SetElectricalIdentity {record,..} if record.id == relationship.id)));
    assert_eq!(
        model
            .materialized_source_shard_value_by_relative_path("board/board.json")
            .unwrap(),
        board_before
    );
    let after = model.electrical_identities.clone();
    assert_eq!(
        ProjectResolver::new(&root)
            .resolve()
            .unwrap()
            .electrical_identities,
        after
    );
    model
        .commit_journal_undo(&root, provenance().into())
        .unwrap();
    assert_eq!(model.electrical_identities, before);
    assert_eq!(
        net_correspondence_status(&model, logical),
        NetCorrespondenceStatus::Complete
    );
    model
        .commit_journal_redo(&root, provenance().into())
        .unwrap();
    assert_eq!(model.electrical_identities, after);
}
