//! Width checks and native source qualification.
use super::*;
use crate::error::EngineError;
use crate::rules::native_policy::{PolicyRules, track_width_from_rules, width_parameters};
use crate::substrate::{DesignModel, SourceShardKind};

pub(in crate::drc) fn run_track_width_checks(board: &Board) -> Vec<DrcViolation> {
    let rules = match PolicyRules::from_typed(&board.rules) {
        Ok(rules) => rules,
        Err(error) => {
            return vec![finding(
                board.uuid,
                None,
                "track_width_policy_unresolved",
                error.to_string(),
            )];
        }
    };
    check_tracks(board, |track| {
        if !board.rules.is_empty() {
            let net = board.nets.get(&track.net).ok_or_else(|| {
                EngineError::Validation("Track Net has no current board source".into())
            })?;
            if net.uuid != track.net {
                return Err(EngineError::Validation(
                    "Track Net identity differs from its source".into(),
                ));
            }
            if let Some(rule) = rules.select("track_width", net, track.layer)? {
                let policy = width_parameters(rule)?;
                let minimum = policy.minimum.ok_or_else(|| {
                    EngineError::Validation("Selected TrackWidth rule has no minimum".into())
                })?;
                return Ok((minimum, policy.maximum));
            }
        }
        Ok((required_track_width_nm(board, track.net), None))
    })
}

/// The native model's raw RulesRoot is authoritative, not the legacy RuleSet.
/// Refuse a board projection from a different source rather than certifying it.
pub(in crate::drc) fn run_native_track_width_checks(
    board: &Board,
    model: &DesignModel,
) -> Vec<DrcViolation> {
    let source =
        model
            .source_shards
            .iter()
            .find(|s| s.kind == SourceShardKind::BoardRoot)
            .ok_or_else(|| EngineError::Validation("Model has no Board source".into()))
            .and_then(|shard| model.capture_source_shard_value(shard.shard_id))
            .and_then(|source| {
                let same = serde_json::from_value::<Uuid>(source["uuid"].clone())? == board.uuid
                    && serde_json::from_value::<std::collections::HashMap<Uuid, Track>>(
                        source["tracks"].clone(),
                    )? == board.tracks
                    && serde_json::from_value::<std::collections::HashMap<Uuid, crate::board::Net>>(
                        source["nets"].clone(),
                    )? == board.nets
                    && serde_json::from_value::<
                        std::collections::HashMap<Uuid, crate::board::NetClass>,
                    >(source["net_classes"].clone())?
                        == board.net_classes;
                if !same {
                    return Err(EngineError::Validation(
                        "Unverified native Board/width source basis".into(),
                    ));
                }
                Ok(source)
            });
    let source = match source {
        Ok(source) => source,
        Err(error) => {
            return vec![finding(
                board.uuid,
                None,
                "track_width_policy_unresolved",
                error.to_string(),
            )];
        }
    };
    let rules = match PolicyRules::from_model(model) {
        Ok(rules) => rules,
        Err(error) => {
            return vec![finding(
                board.uuid,
                None,
                "track_width_policy_unresolved",
                error.to_string(),
            )];
        }
    };
    check_tracks(board, |track| {
        let net = board.nets.get(&track.net).ok_or_else(|| {
            EngineError::Validation("Track Net has no current board source".into())
        })?;
        if net.uuid != track.net {
            return Err(EngineError::Validation(
                "Track Net identity differs from its source".into(),
            ));
        }
        let policy = track_width_from_rules(&rules, &source, net, track.layer)?;
        Ok((policy.minimum.unwrap_or(100_000), policy.maximum))
    })
}

fn check_tracks(
    board: &Board,
    mut policy: impl FnMut(&Track) -> Result<(i64, Option<i64>), EngineError>,
) -> Vec<DrcViolation> {
    let mut violations = Vec::new();
    let mut tracks: Vec<_> = board.tracks.values().collect();
    tracks.sort_by_key(|track| track.uuid);
    for track in tracks {
        let point = midpoint(track.from, track.to);
        let location = Some(DrcLocation {
            x_nm: point.x,
            y_nm: point.y,
            layer: Some(track.layer),
        });
        let (minimum, maximum) = match policy(track) {
            Ok(range) => range,
            Err(error) => {
                violations.push(finding(
                    track.uuid,
                    location,
                    "track_width_policy_unresolved",
                    error.to_string(),
                ));
                continue;
            }
        };
        let bound = if track.width < minimum {
            Some((
                "track_width_below_min",
                format!("below required {minimum}nm"),
            ))
        } else if maximum.is_some_and(|max| track.width > max) {
            Some((
                "track_width_above_max",
                format!("above required {}nm", maximum.unwrap()),
            ))
        } else {
            None
        };
        if let Some((code, requirement)) = bound {
            violations.push(finding(
                track.uuid,
                location,
                code,
                format!(
                    "track width {}nm is {} on layer {}",
                    track.width, requirement, track.layer
                ),
            ));
        }
    }
    violations
}

fn finding(
    object: Uuid,
    location: Option<DrcLocation>,
    code: &str,
    message: String,
) -> DrcViolation {
    DrcViolation {
        id: stable_violation_id(code, RuleType::TrackWidth, location.as_ref(), &[object]),
        code: code.into(),
        rule_type: RuleType::TrackWidth,
        severity: DrcSeverity::Error,
        message,
        location,
        objects: vec![object],
        fingerprint: None,
        standards_basis: Some("datum.process_aperture_and_geometry.current".into()),
        rule_revision: Some("v1".into()),
        import_key: None,
        waived: false,
    }
}

fn required_track_width_nm(board: &Board, net_uuid: Uuid) -> i64 {
    let from_class = || -> Option<i64> {
        let net = board.nets.get(&net_uuid)?;
        let class = board.net_classes.get(&net.class)?;
        Some(class.track_width)
    };
    from_class().filter(|value| *value > 0).unwrap_or(100_000)
}
