//! Initial mapped materialization restores absent and explicit-empty source states.
use super::physical_fixture::{CurrentMaterializationFixture, current_materialization_fixture};
use super::*;
use crate::api::native_write::board_components::pad_materialization::build_materialize_component_pad_sources;
use crate::substrate::{
    NetCorrespondenceStatus, ObjectRevision, SourceShardKind, component_pad_source as pins,
    net_correspondence_status,
};

fn board(model: &DesignModel) -> Value {
    model
        .materialized_source_shard_value(SourceShardKind::BoardRoot)
        .unwrap()
}
fn prior_collection(
    f: &CurrentMaterializationFixture,
    value: Value,
) -> Result<crate::api::native_write::PreparedWrite, crate::error::EngineError> {
    let previous = raw(&f.model, f.new_ci)["component_instance"].clone();
    let mut next = previous.clone();
    next["object_revision"] = json!(f.model.objects[&f.new_ci].object_revision.0 + 1);
    next["placed_package_refs"][0]["object_revision"] =
        json!(f.model.objects[&f.new_package].object_revision.0 + 1);
    BatchComposer::compose(&f.model, provenance())
        .push_ops([
            Operation::SetBoardPackagePackage {
                package_id: f.new_package,
                package_ref_id: f.request.placement.package,
                previous_materialized: json!({}),
                materialized: json!({"component_pads":value}),
            },
            Operation::SetComponentInstance {
                component_instance_id: f.new_ci,
                previous_component_instance: previous,
                component_instance: next,
            },
        ])
        .finish()
}
fn unaffected(f: &CurrentMaterializationFixture) -> BTreeMap<String, Vec<u8>> {
    let ci_shard = f.model.objects[&f.new_ci].source_shard_id;
    f.model
        .source_shards
        .iter()
        .filter(|s| s.kind != SourceShardKind::BoardRoot && s.shard_id != ci_shard)
        .map(|s| {
            (
                s.relative_path.clone(),
                std::fs::read(f.root.join(&s.relative_path)).unwrap(),
            )
        })
        .collect()
}
fn ci_semantics(mut raw: Value) -> Value {
    raw["component_instance"]["object_revision"] = Value::Null;
    for r in raw["component_instance"]["placed_package_refs"]
        .as_array_mut()
        .unwrap()
    {
        r["object_revision"] = Value::Null;
    }
    raw
}
fn old_component(f: &CurrentMaterializationFixture, bytes: &BTreeMap<String, Vec<u8>>) {
    assert_eq!(f.model.component_instances[&f.ci], f.old_ci);
    assert_eq!(
        board(&f.model)["component_pads"][f.package.to_string()],
        f.old_pads
    );
    assert_eq!(
        net_correspondence_status(&f.model, f.old_net),
        NetCorrespondenceStatus::Complete
    );
    let snapshot = ElectricalSelectionSnapshot::capture(&f.model).unwrap();
    assert_eq!(
        snapshot
            .global_net(snapshot.revision(), f.old_net)
            .unwrap()
            .owned,
        f.old_members
    );
    assert_eq!(
        unaffected(f),
        *bytes,
        "other source files, including authored unknown Symbol/pin metadata, stay byte exact"
    );
}

#[test]
fn initial_absent_and_explicit_empty_materialization_repeat_history_and_cold_without_phantoms() {
    for explicit_empty in [false, true] {
        let mut f = current_materialization_fixture(if explicit_empty {
            "initial-mapped-explicit-empty"
        } else {
            "initial-mapped-absent"
        });
        if explicit_empty {
            let p = prior_collection(&f, json!([])).unwrap();
            commit_prepared(&mut f.model, &f.root, p).unwrap();
        }
        let prior = board(&f.model);
        let key = f.new_package.to_string();
        let original_slot = prior["component_pads"].get(&key).cloned();
        assert_eq!(original_slot, explicit_empty.then(|| json!([])));
        let symbol_source = raw(&f.model, f.symbol);
        assert!(
            symbol_source["symbols"][f.symbol.to_string()]["extension"].is_object(),
            "fixture contains real authored unknown siblings"
        );
        let other_bytes = unaffected(&f);
        let original_ci = f.model.component_instances[&f.new_ci].clone();
        let original_raw_ci = ci_semantics(raw(&f.model, f.new_ci));
        let p =
            build_materialize_component_pad_sources(&f.model, provenance(), &f.request).unwrap();
        commit_prepared(&mut f.model, &f.root, p).unwrap();
        let recorded = board(&f.model)["component_pads"][&key].clone();
        assert_eq!(recorded.as_array().unwrap().len(), 8);
        let ids: BTreeSet<Uuid> = recorded
            .as_array()
            .unwrap()
            .iter()
            .map(|p| serde_json::from_value(p["uuid"].clone()).unwrap())
            .collect();
        assert_eq!(ids.len(), 8);
        let verify_ci = |f: &CurrentMaterializationFixture| {
            let mut expected = original_ci.clone();
            expected.object_revision = f.model.objects[&f.new_ci].object_revision;
            assert_eq!(f.model.component_instances[&f.new_ci], expected);
            let raw_ci = raw(&f.model, f.new_ci);
            assert_eq!(ci_semantics(raw_ci.clone()), original_raw_ci);
            assert_eq!(
                raw_ci["component_instance"]["placed_package_refs"][0]["object_revision"],
                json!(f.model.objects[&f.new_package].object_revision.0)
            );
        };
        old_component(&f, &other_bytes);
        for _ in 0..2 {
            let ci_before = f.model.objects[&f.new_ci].object_revision.0;
            let package_before = f.model.objects[&f.new_package].object_revision.0;
            f.model
                .commit_journal_undo(&f.root, provenance().into())
                .unwrap();
            assert_eq!(f.model.objects[&f.new_ci].object_revision.0, ci_before + 1);
            assert_eq!(
                f.model.objects[&f.new_package].object_revision.0,
                package_before + 1
            );
            for cold in [false, true] {
                if cold {
                    f.model = ProjectResolver::new(&f.root).resolve().unwrap();
                }
                assert_eq!(
                    board(&f.model)["component_pads"].get(&key).cloned(),
                    original_slot
                );
                assert!(
                    ids.iter().all(|id| !f.model.objects.contains_key(id)),
                    "Undo leaves no phantom Pad registrations"
                );
                let projection =
                    pins::component_pad_projection(&f.model, &board(&f.model)).unwrap();
                assert_eq!(
                    projection.query_pads.len(),
                    8,
                    "only the other component's eight pads remain"
                );
                verify_ci(&f);
                old_component(&f, &other_bytes);
            }
            f.model
                .commit_journal_redo(&f.root, provenance().into())
                .unwrap();
            assert_eq!(f.model.objects[&f.new_ci].object_revision.0, ci_before + 2);
            assert_eq!(
                f.model.objects[&f.new_package].object_revision.0,
                package_before + 2
            );
            for cold in [false, true] {
                if cold {
                    f.model = ProjectResolver::new(&f.root).resolve().unwrap();
                }
                let current = board(&f.model)["component_pads"][&key].clone();
                let mut expected = recorded.clone();
                for pad in expected.as_array_mut().unwrap() {
                    pad["source_binding"]["component_instance"]["object_revision"] =
                        json!(f.model.objects[&f.new_ci].object_revision.0);
                }
                assert_eq!(
                    current, expected,
                    "same requested Pad IDs, geometry and exact source choices; only the lawful CI pin advances"
                );
                assert!(
                    ids.iter()
                        .all(|id| f.model.objects[id].object_revision == ObjectRevision(0))
                );
                let projection =
                    pins::component_pad_projection(&f.model, &board(&f.model)).unwrap();
                assert_eq!(projection.query_pads.len(), 16);
                assert!(projection.unavailable.is_empty());
                assert!(
                    projection
                        .statuses
                        .values()
                        .all(|s| *s == pins::ComponentPadMaterializationStatus::Current)
                );
                verify_ci(&f);
                old_component(&f, &other_bytes);
            }
        }
        std::fs::remove_dir_all(f.root).unwrap();
    }
}

#[test]
fn malformed_initial_pad_collections_refuse_without_becoming_empty_or_writing() {
    let mut f = current_materialization_fixture("initial-mapped-malformed-refusal");
    let before_board = std::fs::read(
        f.root.join(
            f.model
                .source_shards
                .iter()
                .find(|s| s.kind == SourceShardKind::BoardRoot)
                .unwrap()
                .relative_path
                .clone(),
        ),
    )
    .unwrap();
    let journal = std::fs::read(crate::substrate::transaction_journal_path(&f.root)).unwrap();
    let original = f.model.model_revision.clone();
    for malformed in [Value::Null, json!({}), json!("not a pad sequence")] {
        let refused =
            prior_collection(&f, malformed).and_then(|p| commit_prepared(&mut f.model, &f.root, p));
        assert!(refused.is_err());
        assert_eq!(
            std::fs::read(crate::substrate::transaction_journal_path(&f.root)).unwrap(),
            journal
        );
        assert_eq!(
            std::fs::read(
                f.root.join(
                    f.model
                        .source_shards
                        .iter()
                        .find(|s| s.kind == SourceShardKind::BoardRoot)
                        .unwrap()
                        .relative_path
                        .clone()
                )
            )
            .unwrap(),
            before_board
        );
        assert_eq!(f.model.model_revision, original);
        assert!(
            board(&f.model)["component_pads"]
                .get(f.new_package.to_string())
                .is_none()
        );
    }
    std::fs::remove_dir_all(f.root).unwrap();
}
