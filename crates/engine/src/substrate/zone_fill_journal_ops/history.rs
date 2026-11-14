//! Historical native fill evidence is restored, never recertified as current.
use crate::substrate::{DesignModel, EngineError, Operation, OperationBatch, TransactionKind};
use uuid::Uuid;

/// Only this module constructs the proof, from the fresh leased journal.
pub(in crate::substrate) struct HistoricalFills<'a> {
    operations: &'a [Operation],
}

impl HistoricalFills<'_> {
    pub(in crate::substrate) fn restores(&self, operation: &Operation) -> bool {
        native_fill(operation) && self.operations.contains(operation)
    }
}

fn native_fill(operation: &Operation) -> bool {
    matches!(operation, Operation::SetZoneFill { zone_fill, .. }
        if !zone_fill["provenance"].as_str()
            .is_some_and(|p| p.starts_with("datum:kicad-saved-fill:v1;")))
}

pub(in crate::substrate) fn authenticate<'a>(
    model: &'a DesignModel,
    batch: &OperationBatch,
    kind: TransactionKind,
    undo_of: Option<Uuid>,
    redo_of: Option<Uuid>,
    inverse: &[Operation],
) -> Result<Option<HistoricalFills<'a>>, EngineError> {
    if kind == TransactionKind::Normal || !batch.operations.iter().any(native_fill) {
        return Ok(None);
    }
    let invalid = || EngineError::Validation("Unauthenticated historical zone fill restore".into());
    let history = model.journal_history_targets()?;
    let target = match kind {
        TransactionKind::Undo => history.undo,
        TransactionKind::Redo => history.redo,
        TransactionKind::Normal => None,
    }
    .ok_or_else(invalid)?;
    let (link, other) = match kind {
        TransactionKind::Undo => (undo_of, redo_of),
        TransactionKind::Redo => (redo_of, undo_of),
        TransactionKind::Normal => unreachable!(),
    };
    let mut expected = batch.clone();
    expected.operations = target.inverse_operations.clone();
    // Reuse the existing imported-fill policy on the actual current basis.
    // No native payload or imported policy is changed by this authentication.
    super::super::board_batch_basis::bind_imported_fill_history(model, &mut expected)?;
    if batch.expected_model_revision.as_ref() != Some(&model.model_revision)
        || link != Some(target.transaction_id)
        || other.is_some()
        || batch.operations != expected.operations
        || inverse != target.operations
    {
        return Err(invalid());
    }
    // Actual resulting transaction links are still validated by the commit;
    // no fabricated prospective revision or diff authorizes source staging.
    Ok(Some(HistoricalFills {
        operations: &target.inverse_operations,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::native_write::{BatchComposer, WriteProvenance, commit_prepared};
    use crate::{
        api::native_write::genesis::{GenesisSpec, bootstrap_native_project},
        substrate::{
            CommitSource, ObjectRevision, ProjectResolver, ZONE_FILL_SCHEMA_VERSION, ZoneFill,
            ZoneFillState, commit::CommitPolicyContext,
        },
    };

    #[test]
    fn forged_fill_compensation_cannot_borrow_a_real_history_target() {
        let root = std::env::temp_dir().join(format!("datum-fill-history-{}", Uuid::new_v4()));
        bootstrap_native_project(
            &root,
            GenesisSpec {
                project_name: "Fill history refusal".into(),
                existing_ids: None,
            },
        )
        .unwrap();
        let mut model = ProjectResolver::new(&root).resolve().unwrap();
        let zone_id = Uuid::new_v4();
        let provenance =
            || WriteProvenance::new("history-test", CommitSource::Manual, "Test fill history");
        let mut previous = None;
        for n in 0..3 {
            let value = serde_json::to_value(ZoneFill {
                schema_version: ZONE_FILL_SCHEMA_VERSION,
                zone_id,
                state: ZoneFillState::Stale,
                source_zone_revision: ObjectRevision(0),
                model_revision: model.model_revision.clone(),
                islands: vec![],
                provenance: Some(format!("Historical evidence {n}")),
            })
            .unwrap();
            let write = BatchComposer::compose(&model, provenance())
                .push_op(Operation::SetZoneFill {
                    zone_id,
                    previous_zone_fill: previous,
                    zone_fill: value.clone(),
                })
                .finish()
                .unwrap();
            commit_prepared(&mut model, &root, write).unwrap();
            previous = Some(value);
        }
        let target = model.journal.last().unwrap().clone();
        let source = root.join(format!(".datum/zone_fills/{zone_id}.json"));
        let original = std::fs::read(&source).unwrap();
        let journal = model.journal.clone();
        for variant in 0..5 {
            let mut batch = BatchComposer::compose(&model, provenance())
                .push_ops(target.inverse_operations.clone())
                .finish()
                .unwrap()
                .batch;
            let mut undo_of = Some(target.transaction_id);
            let mut kind = TransactionKind::Undo;
            let mut inverse = target.operations.clone();
            match variant {
                0 => undo_of = Some(Uuid::new_v4()),
                1 => kind = TransactionKind::Redo,
                2 => {
                    let Operation::SetZoneFill { zone_fill, .. } = &mut batch.operations[0] else {
                        panic!()
                    };
                    zone_fill["provenance"] = serde_json::json!("forged geometry authority");
                }
                3 => {
                    let old = &model.journal[model.journal.len() - 2];
                    undo_of = Some(old.transaction_id);
                    batch.operations = old.inverse_operations.clone();
                    inverse = old.operations.clone();
                }
                _ => inverse.clear(),
            }
            assert!(
                model
                    .commit_journaled_with_links_and_inverse(
                        &root,
                        batch,
                        kind,
                        undo_of,
                        None,
                        Some(inverse),
                        CommitPolicyContext::Direct,
                        None,
                        None,
                        Default::default(),
                    )
                    .is_err(),
                "forged history variant {variant}"
            );
            assert_eq!(std::fs::read(&source).unwrap(), original);
            let cold = ProjectResolver::new(&root).resolve().unwrap();
            assert_eq!(cold.journal, journal);
            assert_eq!(model.journal, journal);
        }
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn mixed_native_and_imported_history_keeps_native_source_and_existing_import_rebinding() {
        use serde_json::json;
        let root =
            std::env::temp_dir().join(format!("datum-mixed-fill-history-{}", Uuid::new_v4()));
        bootstrap_native_project(
            &root,
            GenesisSpec {
                project_name: "Mixed historical evidence".into(),
                existing_ids: None,
            },
        )
        .unwrap();
        let mut model = ProjectResolver::new(&root).resolve().unwrap();
        let zones = [Uuid::new_v4(), Uuid::new_v4()];
        let provenance = || {
            WriteProvenance::new(
                "history-test",
                CommitSource::Manual,
                "Mixed history control",
            )
        };
        let net = Uuid::new_v4();
        let shapes = zones.map(|id| json!({"uuid":id,"net":net,
            "polygon":{"vertices":[{"x":0,"y":0},{"x":10000,"y":0},{"x":10000,"y":10000}],"closed":true},
            "layer":1,"priority":0,"thermal_relief":false,"thermal_gap":0,"thermal_spoke_width":0}));
        let write = BatchComposer::compose(&model, provenance())
            .push_ops(
                zones
                    .iter()
                    .zip(&shapes)
                    .map(|(id, zone)| Operation::CreateBoardZone {
                        zone_id: *id,
                        zone: zone.clone(),
                    })
                    .collect::<Vec<_>>(),
            )
            .finish()
            .unwrap();
        commit_prepared(&mut model, &root, write).unwrap();
        let initial: Vec<_> = zones
            .iter()
            .enumerate()
            .map(|(i, id)| {
                serde_json::to_value(ZoneFill {
                    schema_version: ZONE_FILL_SCHEMA_VERSION,
                    zone_id: *id,
                    state: ZoneFillState::Stale,
                    source_zone_revision: model.objects[id].object_revision,
                    model_revision: model.model_revision.clone(),
                    islands: vec![],
                    provenance: Some(if i == 0 {
                        "native historical fill".into()
                    } else {
                        "datum:kicad-saved-fill:v1;test".into()
                    }),
                })
                .unwrap()
            })
            .collect();
        let write = BatchComposer::compose(&model, provenance())
            .push_ops(
                zones
                    .iter()
                    .zip(&initial)
                    .map(|(id, fill)| Operation::SetZoneFill {
                        zone_id: *id,
                        previous_zone_fill: None,
                        zone_fill: fill.clone(),
                    })
                    .collect::<Vec<_>>(),
            )
            .finish()
            .unwrap();
        commit_prepared(&mut model, &root, write).unwrap();
        // A real source edit makes historical epochs differ from compensation.
        let mut operations = vec![];
        for (i, id) in zones.iter().enumerate() {
            let mut zone = shapes[i].clone();
            zone["priority"] = json!(1);
            operations.push(Operation::SetBoardZone { zone_id: *id, zone });
            let mut fill = initial[i].clone();
            fill["islands"] = json!([]);
            fill["provenance"] = json!(if i == 0 {
                "native replacement"
            } else {
                "datum:kicad-saved-fill:v1;replacement"
            });
            operations.push(Operation::SetZoneFill {
                zone_id: *id,
                previous_zone_fill: Some(initial[i].clone()),
                zone_fill: fill,
            });
        }
        let write = BatchComposer::compose(&model, provenance())
            .push_ops(operations)
            .finish()
            .unwrap();
        commit_prepared(&mut model, &root, write).unwrap();
        model
            .commit_journal_undo(&root, provenance().into())
            .unwrap();
        let raw = |model: &DesignModel, id: Uuid| {
            let shard = model
                .source_shards
                .iter()
                .find(|s| {
                    s.kind == crate::substrate::SourceShardKind::ZoneFill
                        && s.relative_path == format!(".datum/zone_fills/{id}.json")
                })
                .unwrap();
            model.capture_source_shard_value(shard.shard_id).unwrap()
        };
        assert_eq!(raw(&model, zones[0]), initial[0]);
        let restored = raw(&model, zones[1]);
        let mut expected = initial[1].clone();
        expected["model_revision"] = json!(model.model_revision);
        expected["source_zone_revision"] = json!(model.objects[&zones[1]].object_revision);
        assert_eq!(
            restored, expected,
            "only the existing imported basis fields may rebind"
        );
        model
            .commit_journal_redo(&root, provenance().into())
            .unwrap();
        let cold = ProjectResolver::new(&root).resolve().unwrap();
        assert_eq!(cold.journal, model.journal);
        for id in zones {
            assert_eq!(raw(&cold, id), raw(&model, id));
        }
        std::fs::remove_dir_all(root).unwrap();
    }
}
