//! Explicit native Track deletion over the existing canonical builder.
//! Removing obsolete copper needs no current fill or connectivity conclusion.
use super::{PreparedWrite, WriteProvenance, build_delete_board_track};
use crate::{
    api::native_write::registry::{NativeWriteContext, NativeWriteVerb, parse_verb_params},
    board::Track,
    error::EngineError,
    substrate::{DesignModel, SourceShardKind},
};
use serde::Deserialize;
use uuid::Uuid;

pub const VERB: &str = "datum.pcb.delete_track";
pub(crate) const VERBS: &[NativeWriteVerb] = &[NativeWriteVerb { id: VERB, build }];

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Params {
    track: Uuid,
}

fn invalid(reason: &str) -> EngineError {
    EngineError::Validation(format!("Delete PCB Track: {reason}"))
}

/// Capture the exact current authored preimage; callers never supply geometry.
/// The common native.write envelope and canonical commit guard project/revision/tip.
pub fn prepare(
    model: &DesignModel,
    provenance: WriteProvenance,
    track: Uuid,
) -> Result<PreparedWrite, EngineError> {
    let mut boards = model
        .source_shards
        .iter()
        .filter(|s| s.kind == SourceShardKind::BoardRoot);
    let source = boards
        .next()
        .ok_or_else(|| invalid("Native Board source is unavailable"))?;
    if boards.next().is_some() {
        return Err(invalid("Board source is ambiguous"));
    }
    let object = model
        .objects
        .get(&track)
        .ok_or_else(|| invalid("Track is no longer present"))?;
    if object.domain != "board" || object.source_shard_id != source.shard_id {
        return Err(invalid("Track does not belong to this Board source"));
    }
    let board = model.capture_source_shard_value(source.shard_id)?;
    let raw = board
        .get("tracks")
        .and_then(|v| v.get(track.to_string()))
        .ok_or_else(|| invalid("Select one current authored Track"))?;
    if raw.get("locked").and_then(serde_json::Value::as_bool) == Some(true) {
        return Err(invalid("Unlock the Track before deleting it"));
    }
    let typed: Track = serde_json::from_value(raw.clone())?;
    if typed.uuid != track {
        return Err(invalid("Track identity differs from its source key"));
    }
    build_delete_board_track(model, provenance, track, raw.clone())
}

fn build(
    context: &NativeWriteContext<'_>,
    provenance: WriteProvenance,
    value: serde_json::Value,
) -> Result<PreparedWrite, EngineError> {
    let params: Params = parse_verb_params(VERB, value)?;
    prepare(context.model, provenance, params.track)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        api::native_write::board_routing::route::{
            prepare_route,
            tests::{fixture, provenance},
        },
        api::native_write::{commit_prepared, registry::find_native_write_verb},
        substrate::ProjectResolver,
    };
    use serde_json::json;

    #[test]
    fn native_delete_removes_only_the_track_and_restores_exact_source_through_cold_history() {
        let mut f = fixture();
        let write = prepare_route(&f.model, provenance(), &f.request).unwrap();
        commit_prepared(&mut f.model, &f.root, write).unwrap();
        let original = f.board();
        let track: Uuid = original["tracks"]
            .as_object()
            .unwrap()
            .keys()
            .next()
            .unwrap()
            .parse()
            .unwrap();
        let before = f.bytes();
        let write = (find_native_write_verb(VERB).unwrap().build)(
            &NativeWriteContext {
                model: &f.model,
                project_root: &f.root,
            },
            provenance(),
            json!({"track":track}),
        )
        .unwrap();
        assert_eq!(f.bytes(), before, "preparing deletion must not write");
        commit_prepared(&mut f.model, &f.root, write).unwrap();
        let mut expected = original.clone();
        expected["tracks"]
            .as_object_mut()
            .unwrap()
            .remove(&track.to_string());
        assert_eq!(
            f.board(),
            expected,
            "all other authored source must stay exact"
        );
        f.model
            .commit_journal_undo(&f.root, provenance().into())
            .unwrap();
        f.model = ProjectResolver::new(&f.root).resolve().unwrap();
        assert_eq!(f.board(), original);
        f.model
            .commit_journal_redo(&f.root, provenance().into())
            .unwrap();
        f.model = ProjectResolver::new(&f.root).resolve().unwrap();
        assert_eq!(f.board(), expected);
    }

    #[test]
    fn stale_delete_and_wrong_class_or_extra_geometry_refuse_without_writes() {
        let mut f = fixture();
        let write = prepare_route(&f.model, provenance(), &f.request).unwrap();
        commit_prepared(&mut f.model, &f.root, write).unwrap();
        f.recapture();
        let track: Uuid = f.board()["tracks"]
            .as_object()
            .unwrap()
            .keys()
            .next()
            .unwrap()
            .parse()
            .unwrap();
        let before = f.bytes();
        for id in [Uuid::new_v4(), f.request.basis.start_pad] {
            assert!(prepare(&f.model, provenance(), id).is_err());
            assert_eq!(f.bytes(), before);
        }
        assert!(
            (find_native_write_verb(VERB).unwrap().build)(
                &NativeWriteContext {
                    model: &f.model,
                    project_root: &f.root
                },
                provenance(),
                json!({"track":track,"geometry":{}}),
            )
            .is_err()
        );
        assert_eq!(f.bytes(), before);
        let stale = prepare(&f.model, provenance(), track).unwrap();
        f.request.edit_seed = Uuid::new_v4();
        let other = prepare_route(&f.model, provenance(), &f.request).unwrap();
        commit_prepared(&mut f.model, &f.root, other).unwrap();
        let changed = f.bytes();
        assert!(commit_prepared(&mut f.model, &f.root, stale).is_err());
        assert_eq!(f.bytes(), changed);
    }
}
