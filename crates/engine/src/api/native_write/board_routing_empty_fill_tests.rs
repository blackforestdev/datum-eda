use super::tests::{resolved_model_with_net_and_zone, test_provenance, test_zone_fill};
use super::*;
use crate::api::native_write::commit_prepared;
use crate::substrate::{ObjectRevision, ProjectResolver, ZONE_FILL_SCHEMA_VERSION};
// PM054 successful-empty proof through canonical native producers.
#[test]
fn empty_fill_is_current_zero_copper_and_preserves_authored_bytes_on_replay() {
    use crate::board::Keepout;
    use crate::substrate::{
        ZoneFillCopperContext, compute_bounded_zone_fill, zone_fill_copper_projection_zones,
    };
    let (root, mut model, _, zone) = resolved_model_with_net_and_zone("pm054_empty_fill");
    let board_before = std::fs::read(root.join("board/board.json")).unwrap();
    let revision = model.model_revision.clone();
    for count in [1, 2] {
        let context = ZoneFillCopperContext {
            keepouts: (0..count)
                .map(|_| Keepout {
                    uuid: Uuid::new_v4(),
                    polygon: zone.polygon.clone(),
                    layers: vec![zone.layer],
                    kind: "copper".into(),
                })
                .collect(),
            ..Default::default()
        };
        let (state, islands, reason) = compute_bounded_zone_fill(&zone, &context);
        assert_eq!(state, ZoneFillState::Filled);
        assert!(islands.is_empty());
        let fill = ZoneFill {
            schema_version: ZONE_FILL_SCHEMA_VERSION,
            zone_id: zone.uuid,
            state,
            source_zone_revision: model.objects[&zone.uuid].object_revision,
            model_revision: revision.clone(),
            islands,
            provenance: Some(reason),
        };
        let write =
            build_set_zone_fills(&model, test_provenance(), std::slice::from_ref(&fill)).unwrap();
        commit_prepared(&mut model, &root, write).unwrap();
        assert_eq!(model.model_revision, revision);
        let (copper, unavailable) =
            zone_fill_copper_projection_zones(std::slice::from_ref(&zone), &model.zone_fills);
        assert!(copper.is_empty());
        assert!(
            unavailable.is_empty(),
            "successful empty is available zero copper"
        );
        assert_eq!(
            ProjectResolver::new(&root).resolve().unwrap().zone_fills[&zone.uuid],
            fill
        );
        model
            .commit_journal_undo(&root, test_provenance().into())
            .unwrap();
        if count == 1 {
            assert_eq!(
                ProjectResolver::new(&root).resolve().unwrap().zone_fills[&zone.uuid].state,
                ZoneFillState::Unfilled
            );
        }
        model
            .commit_journal_redo(&root, test_provenance().into())
            .unwrap();
        assert_eq!(
            ProjectResolver::new(&root).resolve().unwrap().zone_fills[&zone.uuid],
            fill
        );
        assert_eq!(
            std::fs::read(root.join("board/board.json")).unwrap(),
            board_before
        );
    }
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn empty_fill_refuses_legacy_missing_provenance_and_wrong_basis_atomically() {
    let (root, mut model, _, zone) = resolved_model_with_net_and_zone("pm054_empty_refusal");
    let before = model.clone();
    let journal_before = std::fs::read(crate::substrate::transaction_journal_path(&root)).ok();
    let path = root.join(format!(".datum/zone_fills/{}.json", zone.uuid));
    let mut fill = test_zone_fill(&model, zone.uuid, ZoneFillState::Filled);
    fill.source_zone_revision = model.objects[&zone.uuid].object_revision;
    for case in 0..4 {
        let mut invalid = fill.clone();
        match case {
            0 => invalid.schema_version = 1,
            1 => invalid.provenance = None,
            2 => invalid.source_zone_revision = ObjectRevision(fill.source_zone_revision.0 + 1),
            _ => invalid.model_revision.0.push_str("-wrong"),
        }
        let write = build_set_zone_fills(&model, test_provenance(), &[invalid]);
        let result = write.and_then(|w| commit_prepared(&mut model, &root, w));
        assert!(result.is_err(), "case {case} must refuse");
        assert_eq!(model.model_revision, before.model_revision);
        assert_eq!(model.zone_fills, before.zone_fills);
        assert_eq!(model.journal, before.journal);
        assert!(!path.exists());
        assert_eq!(
            std::fs::read(crate::substrate::transaction_journal_path(&root)).ok(),
            journal_before
        );
    }
    // Historical unsupported output retains its meaning after version upgrade.
    let mut unsupported = fill;
    unsupported.schema_version = 1;
    unsupported.state = ZoneFillState::Unsupported;
    let write = build_set_zone_fills(&model, test_provenance(), &[unsupported]).unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    assert_eq!(
        ProjectResolver::new(&root).resolve().unwrap().zone_fills[&zone.uuid].state,
        ZoneFillState::Unsupported
    );
    std::fs::remove_dir_all(root).unwrap();
}
