//! A mapped Part cannot silently migrate its old Gate-qualified certificates.
use super::*;
use crate::api::native_write::library::{
    self, PoolLibraryObjectTarget as Target, PoolLibraryOperationSpec as Spec,
};
use crate::substrate::{NetCorrespondenceStatus, net_correspondence_status};

fn edit(
    model: &DesignModel,
    root: &Path,
    part: Uuid,
    fields: Value,
) -> Result<crate::api::native_write::PreparedWrite, crate::error::EngineError> {
    (find_native_write_verb("datum.library.edit").unwrap().build)(
        &NativeWriteContext {
            model,
            project_root: root,
        },
        provenance(),
        json!({"kind":"parts","object_id":part,"pool_path":"pool","fields":fields}),
    )
}
#[test]
fn saved_part_entity_refuses_default_and_nondefault_maps_preserving_complete_eight_pad_certificate()
{
    let (root, mut model, symbol, ci, part, placed, map, _) =
        physical_fixture::eight_pad_fixture("part-entity-complete-refusal");
    let (net, _, members) = map_tests::connected(&mut model, &root, symbol, placed, ci, map);
    let before_part = raw(&model, part);
    let old_entity: Uuid = serde_json::from_value(before_part["entity"].clone()).unwrap();
    let other = Uuid::new_v4();
    let mut entity = raw(&model, old_entity);
    entity["uuid"] = json!(other);
    entity["name"] = json!("Legal distinct Entity");
    entity["gates"] = Value::Object(
        entity["gates"]
            .as_object()
            .unwrap()
            .values()
            .map(|g| {
                let id = Uuid::new_v4();
                let mut g = g.clone();
                g["uuid"] = json!(id);
                (id.to_string(), g)
            })
            .collect(),
    );
    let write = library::build_pool_library_write(
        &model,
        provenance(),
        Some("pool"),
        vec![Spec::Create {
            target: Target::new("pool", "entities", other),
            object: entity,
        }],
    )
    .unwrap();
    commit_prepared(&mut model, &root, write).unwrap();
    let map_bytes = raw(&model, map);
    for selected in [false, true] {
        if selected {
            let write = edit(&model, &root, part, json!({"default_pin_pad_map":map})).unwrap();
            commit_prepared(&mut model, &root, write).unwrap();
        }
        let before_part = raw(&model, part);
        let before_board = model
            .materialized_source_shard_value(crate::substrate::SourceShardKind::BoardRoot)
            .unwrap();
        let before_ci = raw(&model, ci);
        let before_symbol = raw(&model, symbol);
        let before_identities = model.electrical_identities.clone();
        let before_revision = model.model_revision.clone();
        let journal = crate::substrate::transaction_journal_path(&root);
        let before_journal = std::fs::read(&journal).unwrap();
        assert_eq!(
            net_correspondence_status(&model, net),
            NetCorrespondenceStatus::Complete
        );
        let held = ElectricalSelectionSnapshot::capture(&model).unwrap();
        for fields in [
            json!({"entity":other}),
            json!({"entity":other,"default_pin_pad_map":null}),
        ] {
            assert!(edit(&model, &root, part, fields).is_err());
            assert_eq!(std::fs::read(&journal).unwrap(), before_journal);
            assert_eq!(model.model_revision, before_revision);
            assert_eq!(raw(&model, part), before_part);
            assert_eq!(raw(&model, map), map_bytes);
            assert_eq!(raw(&model, ci), before_ci);
            assert_eq!(raw(&model, symbol), before_symbol);
            assert_eq!(
                model
                    .materialized_source_shard_value(crate::substrate::SourceShardKind::BoardRoot)
                    .unwrap(),
                before_board
            );
            assert_eq!(model.electrical_identities, before_identities);
            assert_eq!(
                net_correspondence_status(&model, net),
                NetCorrespondenceStatus::Complete
            );
        }
        assert_eq!(
            held.global_net(held.revision(), net).unwrap().owned,
            members
        );
        model = ProjectResolver::new(&root).resolve().unwrap();
        assert_eq!(
            net_correspondence_status(&model, net),
            NetCorrespondenceStatus::Complete
        );
        assert_eq!(raw(&model, part), before_part);
        assert_eq!(raw(&model, map), map_bytes);
        assert_eq!(raw(&model, ci), before_ci);
        assert_eq!(raw(&model, symbol), before_symbol);
        assert_eq!(
            model
                .materialized_source_shard_value(crate::substrate::SourceShardKind::BoardRoot)
                .unwrap(),
            before_board
        );
    }
    std::fs::remove_dir_all(root).unwrap();
}
