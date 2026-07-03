//! Board net/track/via/zone/pad/net-class and zone-fill builders for the
//! native write facade.
//!
//! Family D of the native-write migration: all operation authoring for board
//! routing objects lives here. The CLI callers in
//! `crates/cli/src/command_project_board_routing_net.rs`,
//! `crates/cli/src/command_project_board_pad.rs`, and the net-class half of
//! `crates/cli/src/command_project_board_netclass_dimension.rs` are thin
//! argument-parsers over this module: they assemble the typed board structs
//! (`Net`/`Track`/`Via`/`Zone`/`PlacedPad`/`NetClass`), call a `build_*`
//! function, and commit the returned [`PreparedWrite`] via
//! [`super::commit_prepared`]. The route-apply substrate paths are the
//! declared follow-on migration targets onto these same builders.
//!
//! Builders are build-only; they never touch disk. Payload shape (serde of
//! the board types; delete operations thread the stored raw value), guard
//! insertion, and the zone-fill previous-payload rules are byte-for-byte the
//! CLI's historical behavior — zone-fill payloads feed the derived-geometry
//! resolver (`substrate/zone_fill_geometry.rs`) and must not drift.

use uuid::Uuid;

use crate::board::{Net, NetClass, PlacedPad, Track, Via, Zone};
use crate::error::EngineError;
use crate::substrate::{DesignModel, ObjectId, Operation, ZoneFill, ZoneFillState};

use super::context::{BatchComposer, PreparedWrite, WriteProvenance};

macro_rules! board_routing_builders {
    (
        $entity:ty,
        $id_field:ident,
        $payload_field:ident,
        $create_variant:ident,
        $set_variant:ident,
        $delete_variant:ident,
        $place_fn:ident,
        $set_fn:ident,
        $delete_fn:ident
    ) => {
        /// Build the unguarded creation batch for one new board object; the
        /// payload is the serde serialization of the typed struct.
        pub fn $place_fn(
            model: &DesignModel,
            provenance: WriteProvenance,
            object: &$entity,
        ) -> Result<PreparedWrite, EngineError> {
            BatchComposer::compose(model, provenance)
                .push_op(Operation::$create_variant {
                    $id_field: object.uuid,
                    $payload_field: serde_json::to_value(object)?,
                })
                .primary_object(object.uuid)
                .finish()
        }

        /// Build the revision-guarded rewrite batch for one existing board
        /// object; the payload is the serde serialization of the typed
        /// struct.
        pub fn $set_fn(
            model: &DesignModel,
            provenance: WriteProvenance,
            object: &$entity,
        ) -> Result<PreparedWrite, EngineError> {
            BatchComposer::compose(model, provenance)
                .push_op(Operation::$set_variant {
                    $id_field: object.uuid,
                    $payload_field: serde_json::to_value(object)?,
                })
                .primary_object(object.uuid)
                .finish()
        }

        /// Build the revision-guarded delete batch for one existing board
        /// object; `stored` is the raw persisted payload (threaded verbatim
        /// for byte-exact journal parity with the pre-migration CLI).
        pub fn $delete_fn(
            model: &DesignModel,
            provenance: WriteProvenance,
            object_id: ObjectId,
            stored: serde_json::Value,
        ) -> Result<PreparedWrite, EngineError> {
            BatchComposer::compose(model, provenance)
                .push_op(Operation::$delete_variant {
                    $id_field: object_id,
                    $payload_field: stored,
                })
                .primary_object(object_id)
                .finish()
        }
    };
}

board_routing_builders!(
    Net,
    net_id,
    net,
    CreateBoardNet,
    SetBoardNet,
    DeleteBoardNet,
    build_place_board_net,
    build_set_board_net,
    build_delete_board_net
);
board_routing_builders!(
    Track,
    track_id,
    track,
    CreateBoardTrack,
    SetBoardTrack,
    DeleteBoardTrack,
    build_place_board_track,
    build_set_board_track,
    build_delete_board_track
);
board_routing_builders!(
    Via,
    via_id,
    via,
    CreateBoardVia,
    SetBoardVia,
    DeleteBoardVia,
    build_place_board_via,
    build_set_board_via,
    build_delete_board_via
);
board_routing_builders!(
    Zone,
    zone_id,
    zone,
    CreateBoardZone,
    SetBoardZone,
    DeleteBoardZone,
    build_place_board_zone,
    build_set_board_zone,
    build_delete_board_zone
);
board_routing_builders!(
    PlacedPad,
    pad_id,
    pad,
    CreateBoardPad,
    SetBoardPad,
    DeleteBoardPad,
    build_place_board_pad,
    build_set_board_pad,
    build_delete_board_pad
);
board_routing_builders!(
    NetClass,
    net_class_id,
    net_class,
    CreateBoardNetClass,
    SetBoardNetClass,
    DeleteBoardNetClass,
    build_place_board_net_class,
    build_set_board_net_class,
    build_delete_board_net_class
);

/// Build one atomic batch of `SetZoneFill` operations for the computed
/// `zone_fills`. Each operation's `previous_zone_fill` is resolved via
/// [`previous_persisted_zone_fill_value`] against the same pre-commit model,
/// and its payload is the serde serialization of the [`ZoneFill`] — both
/// byte-for-byte the pre-migration CLI behavior.
pub fn build_set_zone_fills(
    model: &DesignModel,
    provenance: WriteProvenance,
    zone_fills: &[ZoneFill],
) -> Result<PreparedWrite, EngineError> {
    let mut composer = BatchComposer::compose(model, provenance);
    for fill in zone_fills {
        composer = composer.push_op(Operation::SetZoneFill {
            zone_id: fill.zone_id,
            previous_zone_fill: previous_persisted_zone_fill_value(model, fill.zone_id),
            zone_fill: serde_json::to_value(fill)?,
        });
    }
    composer.finish()
}

/// The previous persisted zone-fill payload for `zone_id`: the resolved fill
/// when it is anything but `Unfilled`, otherwise the most recent journaled
/// `SetZoneFill` payload (cleared by a later `DeleteZoneFill`).
///
/// Moved verbatim from `crates/cli/src/command_project_board_routing_net.rs`
/// (the CLI re-exports this function); the rules feed `SetZoneFill`
/// journal records and must not drift.
pub fn previous_persisted_zone_fill_value(
    model: &DesignModel,
    zone_id: Uuid,
) -> Option<serde_json::Value> {
    model
        .zone_fills
        .get(&zone_id)
        .filter(|fill| fill.state != ZoneFillState::Unfilled)
        .map(|fill| {
            serde_json::to_value(fill).expect("resolved zone fill serialization must succeed")
        })
        .or_else(|| previous_journaled_zone_fill_value(model, zone_id))
}

fn previous_journaled_zone_fill_value(
    model: &DesignModel,
    zone_id: Uuid,
) -> Option<serde_json::Value> {
    let mut previous = None;
    for transaction in &model.journal {
        for operation in &transaction.operations {
            match operation {
                Operation::SetZoneFill {
                    zone_id: operation_zone_id,
                    zone_fill,
                    ..
                } if *operation_zone_id == zone_id => {
                    previous = Some(zone_fill.clone());
                }
                Operation::DeleteZoneFill {
                    zone_id: operation_zone_id,
                    ..
                } if *operation_zone_id == zone_id => {
                    previous = None;
                }
                _ => {}
            }
        }
    }
    previous
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::super::context::commit_prepared;
    use super::super::test_support::resolved_model_with_board_package;
    use super::*;
    use crate::ir::geometry::{Point, Polygon};
    use crate::substrate::{
        CommitSource, ObjectRevision, ProjectResolver, ZONE_FILL_SCHEMA_VERSION,
    };

    fn test_provenance() -> WriteProvenance {
        WriteProvenance::new("unit-test", CommitSource::Test, "board routing facade test")
    }

    fn test_net(class: Uuid) -> Net {
        Net {
            uuid: Uuid::new_v4(),
            name: "GND".to_string(),
            class,
            controlled_impedance: None,
        }
    }

    fn test_zone(net: Uuid) -> Zone {
        Zone {
            uuid: Uuid::new_v4(),
            net,
            polygon: Polygon {
                vertices: vec![
                    Point { x: 0, y: 0 },
                    Point { x: 1_000_000, y: 0 },
                    Point { x: 1_000_000, y: 1_000_000 },
                    Point { x: 0, y: 1_000_000 },
                ],
                closed: true,
            },
            layer: 1,
            priority: 0,
            thermal_relief: false,
            thermal_gap: 0,
            thermal_spoke_width: 0,
        }
    }

    fn test_zone_fill(model: &DesignModel, zone_id: Uuid, state: ZoneFillState) -> ZoneFill {
        ZoneFill {
            schema_version: ZONE_FILL_SCHEMA_VERSION,
            zone_id,
            state,
            source_zone_revision: ObjectRevision(0),
            model_revision: model.model_revision.clone(),
            islands: Vec::new(),
            provenance: Some("test-fill".to_string()),
        }
    }

    /// Fixture with one committed net and one committed zone (created through
    /// the facade itself, then re-resolved from disk).
    fn resolved_model_with_net_and_zone(name: &str) -> (PathBuf, DesignModel, Net, Zone) {
        let (root, mut model, _board_id, _package_id) = resolved_model_with_board_package(name);
        let net = test_net(Uuid::new_v4());
        let prepared = build_place_board_net(&model, test_provenance(), &net)
            .expect("net placement should build");
        commit_prepared(&mut model, &root, prepared).expect("net placement should commit");
        let zone = test_zone(net.uuid);
        let prepared = build_place_board_zone(&model, test_provenance(), &zone)
            .expect("zone placement should build");
        commit_prepared(&mut model, &root, prepared).expect("zone placement should commit");
        let model = ProjectResolver::new(&root)
            .resolve()
            .expect("fixture project should re-resolve");
        (root, model, net, zone)
    }

    #[test]
    fn place_builds_single_unguarded_create_with_serde_payload() {
        let (_root, model, _board_id, _package_id) =
            resolved_model_with_board_package("board_routing_place_net");
        let net = test_net(Uuid::new_v4());

        let prepared = build_place_board_net(&model, test_provenance(), &net)
            .expect("place should build");

        assert_eq!(prepared.primary_object_id, Some(net.uuid));
        assert_eq!(
            prepared.batch.operations,
            vec![Operation::CreateBoardNet {
                net_id: net.uuid,
                net: serde_json::to_value(&net).unwrap(),
            }]
        );
    }

    #[test]
    fn set_guards_existing_object() {
        let (_root, model, net, _zone) = resolved_model_with_net_and_zone("board_routing_set_net");
        let mut renamed = net.clone();
        renamed.name = "GND2".to_string();

        let prepared = build_set_board_net(&model, test_provenance(), &renamed)
            .expect("set should build");

        assert_eq!(
            prepared.batch.operations,
            vec![
                Operation::GuardObjectRevision {
                    object_id: net.uuid,
                    expected_object_revision: ObjectRevision(0),
                },
                Operation::SetBoardNet {
                    net_id: net.uuid,
                    net: serde_json::to_value(&renamed).unwrap(),
                },
            ]
        );
    }

    #[test]
    fn delete_threads_stored_payload_raw_and_guards() {
        let (_root, model, net, _zone) =
            resolved_model_with_net_and_zone("board_routing_delete_net");
        let stored = serde_json::json!({ "uuid": net.uuid, "name": "GND", "class": net.class });

        let prepared =
            build_delete_board_net(&model, test_provenance(), net.uuid, stored.clone())
                .expect("delete should build");

        assert_eq!(
            prepared.batch.operations,
            vec![
                Operation::GuardObjectRevision {
                    object_id: net.uuid,
                    expected_object_revision: ObjectRevision(0),
                },
                Operation::DeleteBoardNet {
                    net_id: net.uuid,
                    net: stored,
                },
            ]
        );
    }

    #[test]
    fn set_rejects_unknown_object() {
        let (_root, model, _board_id, _package_id) =
            resolved_model_with_board_package("board_routing_set_unknown");
        let track = Track {
            uuid: Uuid::new_v4(),
            net: Uuid::new_v4(),
            from: Point { x: 0, y: 0 },
            to: Point { x: 1, y: 1 },
            width: 200_000,
            layer: 1,
        };
        let error = build_set_board_track(&model, test_provenance(), &track)
            .expect_err("unknown track should fail");
        assert!(matches!(
            error,
            EngineError::NotFound {
                object_type: "domain_object",
                uuid,
            } if uuid == track.uuid
        ));
    }

    #[test]
    fn zone_fill_payload_is_byte_identical_serde_with_no_previous_on_fresh_model() {
        let (_root, model, _net, zone) = resolved_model_with_net_and_zone("board_routing_fill");
        let fill = test_zone_fill(&model, zone.uuid, ZoneFillState::Stale);

        let prepared =
            build_set_zone_fills(&model, test_provenance(), std::slice::from_ref(&fill))
                .expect("fill batch should build");

        // Zone fills are evidence records, not guard targets: exactly the
        // pushed operation, payload byte-identical to the CLI's
        // serde_json::to_value(&fill).
        assert_eq!(
            prepared.batch.operations,
            vec![Operation::SetZoneFill {
                zone_id: zone.uuid,
                previous_zone_fill: None,
                zone_fill: serde_json::to_value(&fill).unwrap(),
            }]
        );
        assert_eq!(
            prepared.batch.operations[0],
            Operation::SetZoneFill {
                zone_id: zone.uuid,
                previous_zone_fill: None,
                zone_fill: serde_json::json!({
                    "schema_version": ZONE_FILL_SCHEMA_VERSION,
                    "zone_id": zone.uuid,
                    "state": "stale",
                    "source_zone_revision": 0,
                    "model_revision": model.model_revision.0,
                    "islands": [],
                    "provenance": "test-fill",
                }),
            }
        );
    }

    /// The exact previous-payload rules the CLI applied pre-migration,
    /// reproduced verbatim as the parity oracle.
    fn cli_previous_persisted_zone_fill_value(
        model: &DesignModel,
        zone_id: Uuid,
    ) -> Option<serde_json::Value> {
        model
            .zone_fills
            .get(&zone_id)
            .filter(|fill| fill.state != ZoneFillState::Unfilled)
            .map(|fill| {
                serde_json::to_value(fill).expect("resolved zone fill serialization must succeed")
            })
            .or_else(|| {
                let mut previous = None;
                for transaction in &model.journal {
                    for operation in &transaction.operations {
                        match operation {
                            Operation::SetZoneFill {
                                zone_id: operation_zone_id,
                                zone_fill,
                                ..
                            } if *operation_zone_id == zone_id => {
                                previous = Some(zone_fill.clone());
                            }
                            Operation::DeleteZoneFill {
                                zone_id: operation_zone_id,
                                ..
                            } if *operation_zone_id == zone_id => {
                                previous = None;
                            }
                            _ => {}
                        }
                    }
                }
                previous
            })
    }

    #[test]
    fn zone_fill_previous_payload_matches_cli_oracle_across_states() {
        let (root, mut model, _net, zone) =
            resolved_model_with_net_and_zone("board_routing_fill_previous");
        let first = test_zone_fill(&model, zone.uuid, ZoneFillState::Stale);
        let prepared =
            build_set_zone_fills(&model, test_provenance(), std::slice::from_ref(&first))
                .expect("first fill should build");
        commit_prepared(&mut model, &root, prepared).expect("first fill should commit");
        let mut model = ProjectResolver::new(&root)
            .resolve()
            .expect("filled project should re-resolve");

        // Resolved non-unfilled fill wins.
        let previous = previous_persisted_zone_fill_value(&model, zone.uuid);
        assert_eq!(
            previous,
            cli_previous_persisted_zone_fill_value(&model, zone.uuid)
        );
        assert_eq!(
            previous,
            Some(serde_json::to_value(&model.zone_fills[&zone.uuid]).unwrap())
        );

        // An unfilled resolved fill is skipped in favor of the journal.
        model
            .zone_fills
            .get_mut(&zone.uuid)
            .expect("fill should resolve")
            .state = ZoneFillState::Unfilled;
        let previous = previous_persisted_zone_fill_value(&model, zone.uuid);
        assert_eq!(
            previous,
            cli_previous_persisted_zone_fill_value(&model, zone.uuid)
        );
        assert_eq!(previous, Some(serde_json::to_value(&first).unwrap()));

        // Unknown zones have no previous payload.
        let stray = Uuid::new_v4();
        assert_eq!(previous_persisted_zone_fill_value(&model, stray), None);
        assert_eq!(cli_previous_persisted_zone_fill_value(&model, stray), None);
    }

    #[test]
    fn zone_fill_batch_threads_previous_payload_on_refill() {
        let (root, mut model, _net, zone) =
            resolved_model_with_net_and_zone("board_routing_refill");
        let first = test_zone_fill(&model, zone.uuid, ZoneFillState::Stale);
        let prepared =
            build_set_zone_fills(&model, test_provenance(), std::slice::from_ref(&first))
                .expect("first fill should build");
        commit_prepared(&mut model, &root, prepared).expect("first fill should commit");
        let model = ProjectResolver::new(&root)
            .resolve()
            .expect("filled project should re-resolve");

        let second = test_zone_fill(&model, zone.uuid, ZoneFillState::Stale);
        let prepared =
            build_set_zone_fills(&model, test_provenance(), std::slice::from_ref(&second))
                .expect("second fill should build");

        let Operation::SetZoneFill {
            previous_zone_fill: Some(previous),
            ..
        } = &prepared.batch.operations[0]
        else {
            panic!("expected SetZoneFill with previous payload");
        };
        assert_eq!(
            previous,
            &serde_json::to_value(&model.zone_fills[&zone.uuid]).unwrap()
        );
    }
}
