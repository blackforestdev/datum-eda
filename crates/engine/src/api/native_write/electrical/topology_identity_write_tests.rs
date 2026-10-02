//! E1-IR-01: explicit writes cannot bypass immutable pre/final nomination.
use super::*;

fn refused(root: &std::path::Path, model: &mut DesignModel, write: PreparedWrite) {
    let before = model.clone();
    let bytes = super::super::binding_fixture::authored_bytes(root, model);
    let journal = std::fs::read(crate::substrate::transaction_journal_path(root)).unwrap();
    let source = model
        .materialized_source_shard_value(crate::substrate::SourceShardKind::SchematicSheet)
        .unwrap();
    assert!(commit_prepared(model, root, write).is_err());
    assert_eq!(*model, before);
    assert_eq!(
        super::super::binding_fixture::authored_bytes(root, model),
        bytes
    );
    assert_eq!(
        std::fs::read(crate::substrate::transaction_journal_path(root)).unwrap(),
        journal
    );
    assert_eq!(
        model
            .materialized_source_shard_value(crate::substrate::SourceShardKind::SchematicSheet)
            .unwrap(),
        source
    );
    persisted(root, model);
}

#[test]
fn unchanged_partition_refuses_identity_swap_with_or_without_display_edit() {
    let (root, mut model, _, sheet) = fixture("e1_ir01_anchor_swap");
    let a = wire(&mut model, &root, sheet, 0, 10);
    let b = wire(&mut model, &root, sheet, 100, 110);
    let label = NetLabel {
        uuid: Uuid::new_v4(),
        name: "A".into(),
        kind: LabelKind::Local,
        position: Point::new(0, 0),
    };
    let write = build_create_schematic_label(&model, provenance(), sheet, &label).unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    adopt(&mut model, &root);
    let a_id = id_at(&model, &member("wires", a.uuid, &[]));
    let b_id = id_at(&model, &member("wires", b.uuid, &[]));
    let mut new_a = model.electrical_identities[&a_id].clone();
    let mut new_b = model.electrical_identities[&b_id].clone();
    if let (ElectricalIdentity::Net { anchor: x, .. }, ElectricalIdentity::Net { anchor: y, .. }) =
        (&mut new_a.identity, &mut new_b.identity)
    {
        std::mem::swap(x, y);
    }
    for display_edit in [false, true] {
        let mut operations = vec![];
        if display_edit {
            let changed = NetLabel {
                name: "RENAMED".into(),
                ..label.clone()
            };
            operations.extend(
                build_set_schematic_label(&model, provenance(), sheet, &changed)
                    .unwrap()
                    .batch
                    .operations,
            );
        }
        operations.extend(
            build_set_electrical_identity(&model, provenance(), new_a.clone())
                .unwrap()
                .batch
                .operations,
        );
        operations.extend(
            build_set_electrical_identity(&model, provenance(), new_b.clone())
                .unwrap()
                .batch
                .operations,
        );
        let write = BatchComposer::compose(&model, provenance())
            .push_ops(operations)
            .finish()
            .unwrap();
        refused(&root, &mut model, write);
        assert_eq!(id_at(&model, &member("wires", a.uuid, &[])), a_id);
        assert_eq!(id_at(&model, &member("wires", b.uuid, &[])), b_id);
    }
    // Revision-only records and a lawful display edit retain the complete subject.
    let before = model.electrical_identities.clone();
    let changed = NetLabel {
        name: "RENAMED".into(),
        ..label
    };
    let mut operations = build_set_schematic_label(&model, provenance(), sheet, &changed)
        .unwrap()
        .batch
        .operations;
    operations.extend(
        build_set_electrical_identity(
            &model,
            provenance(),
            model.electrical_identities[&a_id].clone(),
        )
        .unwrap()
        .batch
        .operations,
    );
    let write = BatchComposer::compose(&model, provenance())
        .push_ops(operations)
        .finish()
        .unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    assert_eq!(
        model.electrical_identities[&a_id].identity,
        before[&a_id].identity
    );
    assert_eq!(model.electrical_identities[&b_id], before[&b_id]);
    let after = model.electrical_identities.clone();
    persisted(&root, &model);
    model
        .commit_journal_undo(&root, provenance().into())
        .unwrap();
    assert_eq!(model.electrical_identities, before);
    model
        .commit_journal_redo(&root, provenance().into())
        .unwrap();
    assert_eq!(model.electrical_identities, after);
    persisted(&root, &model);
}

#[test]
fn unchanged_connectivity_refuses_reanchor_retirement_provenance_and_type_conversion() {
    let (root, mut model, _, sheet) = fixture("e1_ir01_identity_only");
    let a = wire(&mut model, &root, sheet, 0, 10);
    let b = wire(&mut model, &root, sheet, 10, 20);
    adopt(&mut model, &root);
    let id = id_at(&model, &member("wires", a.uuid, &[]));
    let old = model.electrical_identities[&id].clone();
    for mode in 0..4 {
        let mut changed = old.clone();
        match (&mut changed.identity, mode) {
            (ElectricalIdentity::Net { anchor, .. }, 0) => {
                *anchor = if anchor.source_id == a.uuid {
                    member("wires", b.uuid, &[])
                } else {
                    member("wires", a.uuid, &[])
                }
            }
            (ElectricalIdentity::Net { retired, .. }, 1) => *retired = true,
            (ElectricalIdentity::Net { predecessors, .. }, 2) => {
                predecessors.insert(Uuid::new_v4());
            }
            (_, 3) => {
                changed.identity = ElectricalIdentity::Bus {
                    name: "converted".into(),
                    scalar_nets: BTreeSet::new(),
                    representations: BTreeSet::new(),
                    retired: false,
                }
            }
            _ => unreachable!(),
        }
        let write = build_set_electrical_identity(&model, provenance(), changed).unwrap();
        refused(&root, &mut model, write);
    }
}
