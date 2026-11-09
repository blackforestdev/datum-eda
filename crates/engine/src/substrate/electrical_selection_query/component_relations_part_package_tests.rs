//! Incompatible Package choices cannot migrate an existing Complete certificate.
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
fn saved_part_package_default_and_nondefault_maps_refuse_preserving_complete_eight_pad_history() {
    let (root, mut model, symbol, ci, part, placed, map, _) =
        physical_fixture::eight_pad_fixture("part-package-complete-refusal");
    let (net, _, members) = map_tests::connected(&mut model, &root, symbol, placed, ci, map);
    let original = raw(&model, part);
    let package: Uuid = serde_json::from_value(original["package"].clone()).unwrap();
    let other = Uuid::new_v4();
    let mut descriptor = raw(&model, package);
    descriptor["uuid"] = json!(other);
    descriptor["name"] = json!("Distinct current Package");
    for field in ["terminals", "pads"] {
        if let Some(rows) = descriptor[field].as_object() {
            descriptor[field] = Value::Object(
                rows.values()
                    .map(|row| {
                        let id = Uuid::new_v4();
                        let mut r = row.clone();
                        r["uuid"] = json!(id);
                        (id.to_string(), r)
                    })
                    .collect(),
            );
        }
    }
    let p = library::build_pool_library_write(
        &model,
        provenance(),
        Some("pool"),
        vec![Spec::Create {
            target: Target::new("pool", "packages", other),
            object: descriptor,
        }],
    )
    .unwrap();
    commit_prepared(&mut model, &root, p).unwrap();
    for selected in [false, true] {
        if selected {
            let p = edit(&model, &root, part, json!({"default_pin_pad_map":map})).unwrap();
            commit_prepared(&mut model, &root, p).unwrap();
        }
        let before_part = raw(&model, part);
        let before_ci = raw(&model, ci);
        let before_symbol = raw(&model, symbol);
        let before_map = raw(&model, map);
        let before_board = model
            .materialized_source_shard_value(crate::substrate::SourceShardKind::BoardRoot)
            .unwrap();
        let ids = model.electrical_identities.clone();
        let revision = model.model_revision.clone();
        let journal = crate::substrate::transaction_journal_path(&root);
        let bytes = std::fs::read(&journal).unwrap();
        assert_eq!(
            net_correspondence_status(&model, net),
            NetCorrespondenceStatus::Complete
        );
        let held = ElectricalSelectionSnapshot::capture(&model).unwrap();
        for fields in [
            json!({"package":other}),
            json!({"package":other,"default_footprint":null,"default_pin_pad_map":null}),
        ] {
            assert!(edit(&model, &root, part, fields).is_err());
            assert_eq!(std::fs::read(&journal).unwrap(), bytes);
            assert_eq!(model.model_revision, revision);
            assert_eq!(raw(&model, part), before_part);
            assert_eq!(raw(&model, ci), before_ci);
            assert_eq!(raw(&model, symbol), before_symbol);
            assert_eq!(raw(&model, map), before_map);
            assert_eq!(model.electrical_identities, ids);
            assert_eq!(
                model
                    .materialized_source_shard_value(crate::substrate::SourceShardKind::BoardRoot)
                    .unwrap(),
                before_board
            );
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
        assert_eq!(raw(&model, ci), before_ci);
        assert_eq!(raw(&model, map), before_map);
        assert_eq!(
            model
                .materialized_source_shard_value(crate::substrate::SourceShardKind::BoardRoot)
                .unwrap(),
            before_board
        );
    }
    std::fs::remove_dir_all(root).unwrap();
}
