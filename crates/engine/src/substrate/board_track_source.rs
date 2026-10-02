//! Canonical Track source, feature schema and history shared by all mutation surfaces.
use super::{EngineError, Operation};

pub(in crate::substrate) fn validate(operation: &Operation) -> Result<(), EngineError> {
    match operation {
        Operation::CreateBoardTrack { track_id, track }
        | Operation::SetBoardTrack { track_id, track } => {
            validate_track_source(track)?;
            if track.get("midpoint").is_some_and(|point| !point.is_null())
                && track
                    .get("uuid")
                    .and_then(serde_json::Value::as_str)
                    .and_then(|id| uuid::Uuid::parse_str(id).ok())
                    != Some(*track_id)
            {
                return Err(EngineError::Validation(format!(
                    "arc Track {track_id} source UUID differs from operation identity"
                )));
            }
            Ok(())
        }
        _ => Ok(()),
    }
}

fn validate_track_source(value: &serde_json::Value) -> Result<(), EngineError> {
    // Preserve the existing straight-operation contract, including sparse
    // substrate history fixtures; only new arc source requires full Track data.
    if value.get("midpoint").is_none_or(serde_json::Value::is_null) {
        return Ok(());
    }
    // All write surfaces reach this canonical validation, not only the facade.
    let track: crate::board::Track = serde_json::from_value(value.clone())?;
    if let Some(midpoint) = track.midpoint {
        if track.width <= 0 {
            return Err(EngineError::Validation(format!(
                "arc Track {} width must be positive",
                track.uuid
            )));
        }
        crate::board::nominal_geometry::CertifiedArc::new(track.from, midpoint, track.to).map_err(
            |reason| {
                EngineError::Validation(format!(
                    "invalid/unsupported arc Track {}: {reason:?}",
                    track.uuid
                ))
            },
        )?;
    }
    Ok(())
}

use super::super::board_json_maps::{
    board_map_value, insert_board_map_value, remove_board_map_value, replace_board_map_value,
};

pub(in crate::substrate) const BOARD_ARC_SCHEMA_VERSION: u64 = 2;
fn stamp_schema(value: &mut serde_json::Value) {
    let has_arcs = value
        .get("tracks")
        .and_then(serde_json::Value::as_object)
        .is_some_and(|tracks| {
            tracks
                .values()
                .any(|track| track.get("midpoint").is_some_and(|point| !point.is_null()))
        });
    if has_arcs {
        value["schema_version"] = serde_json::json!(
            value
                .get("schema_version")
                .and_then(serde_json::Value::as_u64)
                .unwrap_or(1)
                .max(BOARD_ARC_SCHEMA_VERSION)
        );
    }
    // Format adoption is retained on undo; authored Track anchors still invert.
}
pub(in crate::substrate) fn validate_native(value: &serde_json::Value) -> Result<(), EngineError> {
    if let Some(tracks) = value.get("tracks").and_then(serde_json::Value::as_object) {
        for track in tracks.values() {
            if track.get("midpoint").is_some_and(|point| !point.is_null()) {
                if value
                    .get("schema_version")
                    .and_then(serde_json::Value::as_u64)
                    .unwrap_or(1)
                    < BOARD_ARC_SCHEMA_VERSION
                {
                    return Err(EngineError::Validation(format!(
                        "unsupported BoardRoot schema_version declaration for arc Track {}; schema_version 2 required, legacy declaration cannot describe a curve",
                        track.get("uuid").unwrap_or(&serde_json::Value::Null)
                    )));
                }
                validate_track_source(track)?;
            }
        }
    }
    Ok(())
}
pub(in crate::substrate) fn apply(
    value: &mut serde_json::Value,
    operation: &Operation,
) -> Result<Option<bool>, EngineError> {
    let board_value = value;
    match operation {
        Operation::CreateBoardTrack { track_id, track } => {
            insert_board_map_value(board_value, "tracks", *track_id, track.clone())?;
        }
        Operation::SetBoardTrack { track_id, track } => {
            replace_board_map_value(board_value, "tracks", *track_id, track.clone())?;
        }
        Operation::DeleteBoardTrack { track_id, .. } => {
            remove_board_map_value(board_value, "tracks", *track_id)?;
        }
        _ => return Ok(None),
    };
    stamp_schema(board_value);
    Ok(Some(true))
}
pub(in crate::substrate) fn inverse(
    board_value: &mut serde_json::Value,
    operation: &Operation,
    inverse_operations: &mut Vec<Operation>,
) -> Result<bool, EngineError> {
    match operation {
        Operation::CreateBoardTrack { track_id, track } => {
            inverse_operations.push(Operation::DeleteBoardTrack {
                track_id: *track_id,
                track: track.clone(),
            });
            insert_board_map_value(board_value, "tracks", *track_id, track.clone())?;
        }
        Operation::SetBoardTrack { track_id, track } => {
            let previous = board_map_value(board_value, "tracks", *track_id)?.clone();
            inverse_operations.push(Operation::SetBoardTrack {
                track_id: *track_id,
                track: previous,
            });
            replace_board_map_value(board_value, "tracks", *track_id, track.clone())?;
        }
        Operation::DeleteBoardTrack { track_id, .. } => {
            let previous = board_map_value(board_value, "tracks", *track_id)?.clone();
            inverse_operations.push(Operation::CreateBoardTrack {
                track_id: *track_id,
                track: previous,
            });
            remove_board_map_value(board_value, "tracks", *track_id)?;
        }
        _ => return Ok(false),
    }
    stamp_schema(board_value);
    Ok(true)
}
