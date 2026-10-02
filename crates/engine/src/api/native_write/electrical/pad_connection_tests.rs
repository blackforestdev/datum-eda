//! Actual canonical pad process/span facts, exact layer components and history.
use super::*;
use crate::board::{PadLayerConnection, PlacedPad};
fn pad(net: Uuid, connection: PadLayerConnection) -> PlacedPad {
    serde_json::from_value(serde_json::json!({
        "uuid":Uuid::new_v4(),"package":Uuid::nil(),"name":"1","net":net,
        "position":{"x":0,"y":0},"layer":8,"copper_layers":[8,9],
        "diameter":10,"drill":2,"layer_connection":connection
    }))
    .unwrap()
}
fn write_pad(
    model: &mut DesignModel,
    root: &Path,
    pad: &PlacedPad,
    create: bool,
) -> Result<(), crate::error::EngineError> {
    let value = serde_json::to_value(pad).unwrap();
    let op = if create {
        Operation::CreateBoardPad {
            pad_id: pad.uuid,
            pad: value,
        }
    } else {
        Operation::SetBoardPad {
            pad_id: pad.uuid,
            pad: value,
        }
    };
    let write = BatchComposer::compose(model, provenance())
        .push_op(op)
        .finish()?;
    commit_prepared(model, root, write)?;
    Ok(())
}
fn schema(root: &Path) -> u64 {
    let value: serde_json::Value =
        serde_json::from_slice(&std::fs::read(root.join("board/board.json")).unwrap()).unwrap();
    value["schema_version"].as_u64().unwrap()
}
#[test]
fn explicit_plating_joins_actual_layers_separate_apertures_do_not_and_history_preserves_identity() {
    let (root, mut model, net) = fixture("pad_layer_connection_history");
    let mut p = pad(net, PadLayerConnection::Separate);
    write_pad(&mut model, &root, &p, true).unwrap();
    let front = line(net, (3, 0), (20, 0));
    let back = Track {
        layer: 9,
        ..line(net, (3, 0), (0, 20))
    };
    tracks(&mut model, &root, &[front.clone(), back.clone()]);
    let before = query(&model, &member("tracks", front.uuid), None).unwrap();
    assert_eq!(
        before.members,
        BTreeSet::from([member("tracks", front.uuid), member("pads", p.uuid)])
    );
    assert_eq!(
        before.copper_layers[&member("pads", p.uuid)],
        BTreeSet::from([8])
    );
    let second = query(&model, &member("tracks", back.uuid), None).unwrap();
    assert_eq!(
        second.copper_layers[&member("pads", p.uuid)],
        BTreeSet::from([9])
    );
    assert!(matches!(
        query(&model, &member("pads", p.uuid), None),
        Err(ElectricalQueryFailure::Physical(
            PhysicalQueryFailure::AmbiguousOrigin { .. }
        ))
    ));
    p.layer_connection = PadLayerConnection::PlatedThrough;
    write_pad(&mut model, &root, &p, false).unwrap();
    let joined = query(&model, &member("tracks", front.uuid), None).unwrap();
    assert_eq!(
        joined.members,
        BTreeSet::from([
            member("tracks", front.uuid),
            member("tracks", back.uuid),
            member("pads", p.uuid)
        ])
    );
    assert_eq!(
        joined.copper_layers[&member("pads", p.uuid)],
        BTreeSet::from([8, 9])
    );
    assert_eq!(joined.board_net, net);
    assert_eq!(schema(&root), 3);
    assert!(matches!(
        query(&model, &member("pads", p.uuid), Some(Point::zero())),
        Err(ElectricalQueryFailure::Physical(
            PhysicalQueryFailure::OriginOutsideCopper { .. }
        ))
    ));
    model
        .commit_journal_undo(&root, provenance().into())
        .unwrap();
    let undone = query(&model, &member("tracks", front.uuid), None).unwrap();
    assert_eq!(undone.members, before.members);
    assert_eq!(undone.copper_layers, before.copper_layers);
    assert_eq!(schema(&root), 3, "format adoption is forward only");
    model
        .commit_journal_redo(&root, provenance().into())
        .unwrap();
    let reopened = ProjectResolver::new(&root).resolve().unwrap();
    let replay = query(&reopened, &member("tracks", front.uuid), None).unwrap();
    assert_eq!(replay.members, joined.members);
    assert_eq!(replay.copper_layers, joined.copper_layers);
    assert_eq!(replay.board_net, net);
    // An arc write cannot downgrade a previously adopted pad feature version.
    let curve = Track {
        midpoint: Some(Point::new(30, 5)),
        ..line(net, (25, 0), (35, 0))
    };
    tracks(&mut model, &root, &[curve]);
    assert_eq!(schema(&root), 3);
    std::fs::remove_dir_all(root).unwrap();
}
#[test]
fn unknown_or_incomplete_barrel_basis_is_unavailable_and_malformed_source_refuses_atomically() {
    let (root, mut model, net) = fixture("pad_layer_connection_refusal");
    let mut p = pad(net, PadLayerConnection::Unknown);
    assert!(
        serde_json::to_value(&p)
            .unwrap()
            .get("layer_connection")
            .is_none()
    );
    write_pad(&mut model, &root, &p, true).unwrap();
    assert!(matches!(
        query(&model, &member("pads", p.uuid), None),
        Err(ElectricalQueryFailure::Physical(
            PhysicalQueryFailure::UnavailableGeometry { .. }
        ))
    ));
    let board = std::fs::read(root.join("board/board.json")).unwrap();
    let journal = std::fs::read(crate::substrate::transaction_journal_path(&root)).unwrap();
    let original = model.clone();
    for (connection, drill) in [
        (PadLayerConnection::PlatedThrough, 0),
        (
            PadLayerConnection::PlatedSpan {
                start_layer: 8,
                end_layer: 8,
            },
            2,
        ),
        (PadLayerConnection::Separate, -1),
    ] {
        p.layer_connection = connection;
        p.drill = drill;
        assert!(write_pad(&mut model, &root, &p, false).is_err());
        assert_eq!(model.objects, original.objects);
        assert_eq!(model.model_revision, original.model_revision);
        assert_eq!(std::fs::read(root.join("board/board.json")).unwrap(), board);
        assert_eq!(
            std::fs::read(crate::substrate::transaction_journal_path(&root)).unwrap(),
            journal
        );
    }
    p.drill = 2;
    p.layer_connection = PadLayerConnection::PlatedSpan {
        start_layer: 8,
        end_layer: 9,
    };
    p.copper_layers = vec![8];
    write_pad(&mut model, &root, &p, false).unwrap();
    assert!(matches!(
        query(&model, &member("pads", p.uuid), None),
        Err(ElectricalQueryFailure::Physical(
            PhysicalQueryFailure::UnavailableGeometry { .. }
        ))
    ));
    p.copper_layers = vec![8, 9];
    write_pad(&mut model, &root, &p, false).unwrap();
    assert!(query(&model, &member("pads", p.uuid), None).is_ok());
    let path = root.join("board/board.json");
    let mut value: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    for version in [1, 2, 4] {
        value["schema_version"] = serde_json::json!(version);
        std::fs::write(&path, serde_json::to_vec_pretty(&value).unwrap()).unwrap();
        assert!(ProjectResolver::new(&root).resolve().is_err());
    }
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn explicit_blind_span_uses_stackup_order_and_never_joins_out_of_span_copper() {
    let (root, mut model, net) = fixture("pad_blind_span");
    let layers = Stackup {
        layers: vec![
            StackupLayer::new(8, "front", StackupLayerType::Copper, 35_000),
            StackupLayer::new(7, "core", StackupLayerType::Dielectric, 1_000_000),
            StackupLayer::new(3, "inner", StackupLayerType::Copper, 35_000),
            StackupLayer::new(9, "back", StackupLayerType::Copper, 35_000),
        ],
    };
    let board = model
        .materialized_source_shard_value(crate::substrate::SourceShardKind::BoardRoot)
        .unwrap();
    let write = BatchComposer::compose(&model, provenance())
        .push_op(Operation::SetBoardStackup {
            board_id: serde_json::from_value(board["uuid"].clone()).unwrap(),
            stackup: serde_json::to_value(layers).unwrap(),
        })
        .finish()
        .unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    let mut p = pad(
        net,
        PadLayerConnection::PlatedSpan {
            start_layer: 8,
            end_layer: 3,
        },
    );
    p.copper_layers = vec![8, 3];
    write_pad(&mut model, &root, &p, true).unwrap();
    let front = line(net, (3, 0), (20, 0));
    let inner = Track {
        layer: 3,
        ..line(net, (3, 0), (0, 20))
    };
    let back = Track {
        layer: 9,
        ..line(net, (3, 0), (-20, 0))
    };
    tracks(
        &mut model,
        &root,
        &[front.clone(), inner.clone(), back.clone()],
    );
    let result = query(&model, &member("tracks", front.uuid), None).unwrap();
    assert_eq!(
        result.members,
        BTreeSet::from([
            member("tracks", front.uuid),
            member("tracks", inner.uuid),
            member("pads", p.uuid)
        ])
    );
    assert_eq!(
        result.copper_layers[&member("pads", p.uuid)],
        BTreeSet::from([3, 8])
    );
    assert_eq!(
        query(&model, &member("tracks", back.uuid), None)
            .unwrap()
            .members,
        BTreeSet::from([member("tracks", back.uuid)])
    );
    p.layer_connection = PadLayerConnection::PlatedThrough;
    write_pad(&mut model, &root, &p, false).unwrap();
    assert!(matches!(
        query(&model, &member("pads", p.uuid), None),
        Err(ElectricalQueryFailure::Physical(
            PhysicalQueryFailure::UnavailableGeometry { .. }
        ))
    ));
    std::fs::remove_dir_all(root).unwrap();
}
