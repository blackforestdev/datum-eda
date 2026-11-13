//! Source-qualified manual board Outline admission over the canonical builder.
//! Historical low-level operations/replay may restore an empty outline; newly
//! authored intent must instead describe one closed, simple, nonzero polygon.
use super::build_set_board_outline;
use crate::api::native_write::{
    PreparedWrite, WriteProvenance,
    board_routing::zone::{BoardBasis, capture_board_basis},
    registry::{NativeWriteContext, NativeWriteVerb, parse_verb_params},
};
use crate::{
    board::occupied_copper, error::EngineError, ir::geometry::Polygon, substrate::DesignModel,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OutlineRequest {
    pub basis: BoardBasis,
    pub polygon: Polygon,
}

pub fn prepare_outline(
    model: &DesignModel,
    provenance: WriteProvenance,
    request: &OutlineRequest,
) -> Result<PreparedWrite, EngineError> {
    if capture_board_basis(model)? != request.basis {
        return Err(EngineError::Validation(
            "Board Outline source changed; reacquire the current board".into(),
        ));
    }
    occupied_copper::validate_polygon(&request.polygon).map_err(|error| {
        EngineError::Validation(format!("Board Outline validation failed: {error:?}"))
    })?;
    build_set_board_outline(model, provenance, request.basis.board, &request.polygon)
}

pub(crate) const VERBS: &[NativeWriteVerb] = &[NativeWriteVerb {
    id: "datum.pcb.set_outline",
    build,
}];
fn build(
    context: &NativeWriteContext<'_>,
    provenance: WriteProvenance,
    params: serde_json::Value,
) -> Result<PreparedWrite, EngineError> {
    let request = parse_verb_params("datum.pcb.set_outline", params)?;
    prepare_outline(context.model, provenance, &request)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::native_write::{
        commit_prepared,
        genesis::{GenesisSpec, bootstrap_native_project},
        registry::find_native_write_verb,
        test_support::temp_project_root,
    };
    use crate::{
        ir::geometry::Point,
        substrate::{CommitSource, ProjectResolver, SourceShardKind},
    };
    use std::{collections::BTreeMap, path::PathBuf};
    use uuid::Uuid;
    struct Fixture {
        root: PathBuf,
        model: DesignModel,
    }
    impl Fixture {
        fn new() -> Self {
            let root = temp_project_root("outline");
            bootstrap_native_project(
                &root,
                GenesisSpec {
                    project_name: "Manual Outline fixture".into(),
                    existing_ids: None,
                },
            )
            .unwrap();
            let model = ProjectResolver::new(&root).resolve().unwrap();
            Self { root, model }
        }
        fn request(&self, polygon: Polygon) -> OutlineRequest {
            OutlineRequest {
                basis: capture_board_basis(&self.model).unwrap(),
                polygon,
            }
        }
        fn outline(&self) -> serde_json::Value {
            self.model
                .materialized_source_shard_value(SourceShardKind::BoardRoot)
                .unwrap()["outline"]
                .clone()
        }
        fn bytes(&self) -> BTreeMap<PathBuf, Vec<u8>> {
            self.model
                .source_shards
                .iter()
                .map(|s| s.path.clone())
                .chain([
                    crate::substrate::transaction_journal_path(&self.root),
                    self.root.join(".datum/journal/cursor.json"),
                ])
                .filter(|p| p.exists())
                .map(|p| {
                    let bytes = std::fs::read(&p).unwrap();
                    (p, bytes)
                })
                .collect()
        }
        fn cold(&mut self) {
            self.model = ProjectResolver::new(&self.root).resolve().unwrap();
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.root);
        }
    }
    fn provenance() -> WriteProvenance {
        WriteProvenance::new("outline-test", CommitSource::Test, "Manual Outline")
    }
    fn rectangle() -> Polygon {
        Polygon::new(vec![
            Point::new(0, 0),
            Point::new(10000000, 0),
            Point::new(10000000, 6000000),
            Point::new(0, 6000000),
        ])
    }
    #[test]
    fn registered_outline_prepare_cancel_first_empty_history_cold_and_replacement() {
        let mut f = Fixture::new();
        let initial = f.outline();
        let before = f.bytes();
        let count = f.model.journal.len();
        let request = f.request(rectangle());
        let prepared = (find_native_write_verb("datum.pcb.set_outline")
            .unwrap()
            .build)(
            &NativeWriteContext {
                model: &f.model,
                project_root: &f.root,
            },
            provenance(),
            serde_json::to_value(&request).unwrap(),
        )
        .unwrap();
        assert_eq!(prepared.primary_object_id, Some(request.basis.board));
        assert_eq!(
            before,
            f.bytes(),
            "preparation/discard is no-write cancellation"
        );
        commit_prepared(&mut f.model, &f.root, prepared).unwrap();
        assert_eq!(f.model.journal.len(), count + 1);
        f.cold();
        let first = f.outline();
        assert_eq!(first, serde_json::to_value(&request.polygon).unwrap());
        f.model
            .commit_journal_undo(&f.root, provenance().into())
            .unwrap();
        f.cold();
        assert_eq!(
            f.outline(),
            initial,
            "first-outline Undo restores initial empty value exactly"
        );
        f.model
            .commit_journal_redo(&f.root, provenance().into())
            .unwrap();
        f.cold();
        assert_eq!(f.outline(), first);
        let replacement = f.request(Polygon::new(vec![
            Point::new(0, 0),
            Point::new(12000000, 0),
            Point::new(0, 8000000),
        ]));
        let prepared = prepare_outline(&f.model, provenance(), &replacement).unwrap();
        commit_prepared(&mut f.model, &f.root, prepared).unwrap();
        f.cold();
        assert_eq!(
            f.outline(),
            serde_json::to_value(&replacement.polygon).unwrap()
        );
        f.model
            .commit_journal_undo(&f.root, provenance().into())
            .unwrap();
        f.cold();
        assert_eq!(
            f.outline(),
            first,
            "replacement Undo restores exact old vertices"
        );
    }
    #[test]
    fn invalid_geometry_and_forged_source_basis_are_no_write_refusals() {
        let f = Fixture::new();
        let before = f.bytes();
        let mut invalid = vec![
            Polygon::new(vec![]),
            Polygon::new(vec![Point::new(0, 0), Point::new(1, 0)]),
            Polygon::new(vec![Point::new(0, 0), Point::new(1, 0), Point::new(2, 0)]),
            Polygon::new(vec![
                Point::new(0, 0),
                Point::new(2, 2),
                Point::new(0, 2),
                Point::new(2, 0),
            ]),
        ];
        let mut open = rectangle();
        open.closed = false;
        invalid.push(open);
        for polygon in invalid {
            assert!(prepare_outline(&f.model, provenance(), &f.request(polygon)).is_err());
        }
        let extreme = f.request(Polygon::new(vec![
            Point::new(i64::MIN, i64::MIN),
            Point::new(i64::MAX, i64::MIN),
            Point::new(i64::MAX, i64::MAX),
        ]));
        assert!(
            prepare_outline(&f.model, provenance(), &extreme)
                .unwrap_err()
                .to_string()
                .contains("ArithmeticRange")
        );
        for variant in 0..4 {
            let mut request = f.request(rectangle());
            match variant {
                0 => request.basis.project = Uuid::new_v4(),
                1 => request.basis.board = Uuid::new_v4(),
                2 => request.basis.revision.0.push_str("stale"),
                3 => request.basis.tip = Some(Uuid::new_v4()),
                _ => unreachable!(),
            };
            assert!(
                prepare_outline(&f.model, provenance(), &request)
                    .unwrap_err()
                    .to_string()
                    .contains("source changed")
            );
        }
        let mut params = serde_json::to_value(f.request(rectangle())).unwrap();
        params["extra"] = true.into();
        assert!(
            (find_native_write_verb("datum.pcb.set_outline")
                .unwrap()
                .build)(
                &NativeWriteContext {
                    model: &f.model,
                    project_root: &f.root
                },
                provenance(),
                params
            )
            .is_err()
        );
        assert_eq!(before, f.bytes());
    }
    #[test]
    fn intervening_canonical_change_invalidates_request_and_prepared_write() {
        let mut f = Fixture::new();
        let request = f.request(rectangle());
        let prepared = prepare_outline(&f.model, provenance(), &request).unwrap();
        let rename = super::super::build_set_board_name(
            &f.model,
            provenance(),
            request.basis.board,
            "Changed board".into(),
        )
        .unwrap();
        commit_prepared(&mut f.model, &f.root, rename).unwrap();
        f.cold();
        let before = f.bytes();
        assert!(prepare_outline(&f.model, provenance(), &request).is_err());
        assert!(commit_prepared(&mut f.model, &f.root, prepared).is_err());
        assert_eq!(before, f.bytes());
    }
}
