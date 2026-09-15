//! Route-strategy regression fixture board composition over the native-write
//! facade.
//!
//! Family F sub-step 3 of the native-write migration: the CLI's curated
//! route-strategy fixture seeds no longer raw-write `board/board.json`.
//! Instead a fixture is genesis-bootstrapped (`project new` path) and its
//! board content is committed as ONE composed batch whose operations are all
//! authored by the family-D `native_write` builders (stackup, outline, net
//! classes, nets, pads, tracks, vias). Per-builder guards are stripped and
//! the combined batch is re-guarded as one unit — the family-E composition
//! pattern — so the board root is guarded exactly once.
//!
//! The no-proposal variant needs a net whose persisted `class` is JSON
//! `null` (the historical fixture hand-wrote that into `board.json`; the
//! route pipeline's typed `Net` parse then fails, which is what makes every
//! candidate unavailable). A `null` class is unrepresentable in the typed
//! [`Net`], so [`build_route_strategy_fixture_net_class_clear`] authors the
//! one `SetBoardNet` operation with the null-class payload directly and runs
//! it through the facade's [`build_batch`] guard/stamping path. This is
//! fixture-generation-only authoring, never a product write path.

use crate::api::native_write::board_layout::{build_set_board_outline, build_set_board_stackup};
use crate::api::native_write::board_routing::{
    build_place_board_net, build_place_board_net_class, build_place_board_pad,
    build_place_board_track, build_place_board_via,
};
use crate::api::native_write::context::{PreparedWrite, WriteProvenance, build_batch};
use crate::board::{Net, NetClass, PlacedPad, StackupLayer, Track, Via};
use crate::error::EngineError;
use crate::ir::geometry::Polygon;
use crate::substrate::{DesignModel, ObjectId, Operation};

/// Everything one curated route-strategy fixture board authors on top of the
/// genesis scaffold.
pub struct RouteStrategyFixtureBoardSpec {
    pub stackup_layers: Vec<StackupLayer>,
    pub outline: Polygon,
    pub net_classes: Vec<NetClass>,
    pub nets: Vec<Net>,
    pub pads: Vec<PlacedPad>,
    pub tracks: Vec<Track>,
    pub vias: Vec<Via>,
}

/// Compose the single batch that authors a curated fixture board: replace
/// the genesis stackup and outline, then create the fixture's net classes,
/// nets, pads, tracks, and vias. Every operation comes from a family-D
/// facade builder; the whole batch is guarded as one unit.
pub fn build_route_strategy_fixture_board_write(
    model: &DesignModel,
    provenance: WriteProvenance,
    board_id: ObjectId,
    spec: &RouteStrategyFixtureBoardSpec,
) -> Result<PreparedWrite, EngineError> {
    let mut operations = Vec::new();
    let mut push = |prepared: PreparedWrite| {
        operations.extend(unguarded_operations(prepared));
    };
    push(build_set_board_stackup(
        model,
        provenance.clone(),
        board_id,
        &spec.stackup_layers,
    )?);
    push(build_set_board_outline(
        model,
        provenance.clone(),
        board_id,
        &spec.outline,
    )?);
    for net_class in &spec.net_classes {
        push(build_place_board_net_class(
            model,
            provenance.clone(),
            net_class,
        )?);
    }
    for net in &spec.nets {
        push(build_place_board_net(model, provenance.clone(), net)?);
    }
    for pad in &spec.pads {
        push(build_place_board_pad(model, provenance.clone(), pad)?);
    }
    for track in &spec.tracks {
        push(build_place_board_track(model, provenance.clone(), track)?);
    }
    for via in &spec.vias {
        push(build_place_board_via(model, provenance.clone(), via)?);
    }
    let batch = build_batch(model, provenance, operations)?;
    Ok(PreparedWrite {
        batch,
        primary_object_id: Some(board_id),
    })
}

/// Author the guarded `SetBoardNet` that clears one fixture net's class to
/// JSON `null` (the no-proposal fixture's defining state; see module docs
/// for why this cannot go through the typed `build_set_board_net`).
pub fn build_route_strategy_fixture_net_class_clear(
    model: &DesignModel,
    provenance: WriteProvenance,
    net: &Net,
) -> Result<PreparedWrite, EngineError> {
    let mut payload = serde_json::to_value(net)?;
    payload["class"] = serde_json::Value::Null;
    let batch = build_batch(
        model,
        provenance,
        vec![Operation::SetBoardNet {
            net_id: net.uuid,
            net: payload,
        }],
    )?;
    Ok(PreparedWrite {
        batch,
        primary_object_id: Some(net.uuid),
    })
}

/// Strip the object-revision guards a single-edit family-D builder inserted,
/// leaving only the authored mutation operations (the combined fixture batch
/// is re-guarded as a whole; duplicate guards on the board root would trip
/// the second guard after the first mutation bumps the revision).
fn unguarded_operations(prepared: PreparedWrite) -> impl Iterator<Item = Operation> {
    prepared
        .batch
        .operations
        .into_iter()
        .filter(|operation| !matches!(operation, Operation::GuardObjectRevision { .. }))
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use uuid::Uuid;

    use crate::api::native_write::context::commit_prepared;
    use crate::api::native_write::genesis::{GenesisSpec, bootstrap_native_project};
    use crate::board::{PadShape, StackupLayerType};
    use crate::ir::geometry::Point;
    use crate::substrate::{CommitSource, ProjectResolver, SourceShardKind};

    use super::*;

    fn test_provenance() -> WriteProvenance {
        WriteProvenance::new(
            "datum-eda-cli",
            CommitSource::Cli,
            "seed route strategy curated fixture board",
        )
    }

    fn fixture_spec(class_uuid: Uuid, net_uuid: Uuid) -> RouteStrategyFixtureBoardSpec {
        RouteStrategyFixtureBoardSpec {
            stackup_layers: vec![
                StackupLayer::new(1, "Top Copper", StackupLayerType::Copper, 35_000),
                StackupLayer::new(2, "Core", StackupLayerType::Dielectric, 1_600_000),
                StackupLayer::new(3, "Bottom Copper", StackupLayerType::Copper, 35_000),
            ],
            outline: Polygon {
                vertices: vec![
                    Point { x: 0, y: 0 },
                    Point { x: 5_000_000, y: 0 },
                    Point {
                        x: 5_000_000,
                        y: 3_000_000,
                    },
                    Point { x: 0, y: 3_000_000 },
                ],
                closed: true,
            },
            net_classes: vec![NetClass {
                uuid: class_uuid,
                name: "Default".to_string(),
                clearance: 150_000,
                track_width: 200_000,
                via_drill: 300_000,
                via_diameter: 600_000,
                diffpair_width: 0,
                diffpair_gap: 0,
            }],
            nets: vec![Net {
                uuid: net_uuid,
                name: "SIG".to_string(),
                class: class_uuid,
                controlled_impedance: None,
            }],
            pads: vec![PlacedPad {
                uuid: Uuid::from_u128(0xf1),
                package: Uuid::from_u128(0xf2),
                name: "1".to_string(),
                net: Some(net_uuid),
                position: Point {
                    x: 500_000,
                    y: 600_000,
                },
                layer: 1,
                copper_layers: Vec::new(),
                shape: PadShape::Circle,
                diameter: 450_000,
                width: 0,
                height: 0,
                drill: 0,
                rotation: 0,
                roundrect_rratio_ppm: 250_000,
                mask_layers: Vec::new(),
                paste_layers: Vec::new(),
                solder_mask_margin_nm: 0,
                solder_paste_margin_nm: 0,
                solder_paste_margin_ratio_ppm: 0,
            }],
            tracks: Vec::new(),
            vias: Vec::new(),
        }
    }

    fn bootstrap(label: &str) -> (PathBuf, DesignModel, ObjectId) {
        let root = std::env::temp_dir().join(format!(
            "datum-route-strategy-fixture-{label}-{}",
            Uuid::new_v4()
        ));
        std::fs::create_dir_all(&root).expect("temp root should create");
        let report = bootstrap_native_project(
            &root,
            GenesisSpec {
                project_name: "Fixture Compose Test".to_string(),
                existing_ids: None,
            },
        )
        .expect("genesis should succeed");
        let model = ProjectResolver::new(&root)
            .resolve()
            .expect("fresh project should resolve");
        (root, model, report.board_uuid)
    }

    #[test]
    fn fixture_board_batch_guards_the_board_root_exactly_once_and_commits() {
        let (root, mut model, board_id) = bootstrap("compose");
        let class_uuid = Uuid::from_u128(0xc202);
        let net_uuid = Uuid::from_u128(0xc200);
        let spec = fixture_spec(class_uuid, net_uuid);

        let prepared =
            build_route_strategy_fixture_board_write(&model, test_provenance(), board_id, &spec)
                .expect("fixture batch should build");

        let guard_count = prepared
            .batch
            .operations
            .iter()
            .filter(|operation| matches!(operation, Operation::GuardObjectRevision { .. }))
            .count();
        assert_eq!(guard_count, 1, "board root must be guarded exactly once");
        assert!(matches!(
            prepared.batch.operations[0],
            Operation::GuardObjectRevision { object_id, .. } if object_id == board_id
        ));
        // guard, stackup, outline, net class, net, pad
        assert_eq!(prepared.batch.operations.len(), 6);

        commit_prepared(&mut model, &root, prepared).expect("fixture batch should commit");
        let resolved = ProjectResolver::new(&root)
            .resolve()
            .expect("seeded fixture should resolve");
        let board = resolved
            .materialized_source_shard_value(SourceShardKind::BoardRoot)
            .expect("board shard should materialize");
        assert_eq!(board["stackup"]["layers"].as_array().unwrap().len(), 3);
        assert_eq!(board["outline"]["vertices"].as_array().unwrap().len(), 4);
        assert_eq!(
            board["nets"][net_uuid.to_string()]["class"],
            serde_json::json!(class_uuid)
        );
        assert!(board["pads"][Uuid::from_u128(0xf1).to_string()].is_object());
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn net_class_clear_persists_a_null_class_like_the_historical_fixture() {
        let (root, mut model, board_id) = bootstrap("null_class");
        let class_uuid = Uuid::from_u128(0xc202);
        let net_uuid = Uuid::from_u128(0xc200);
        let spec = fixture_spec(class_uuid, net_uuid);
        let prepared =
            build_route_strategy_fixture_board_write(&model, test_provenance(), board_id, &spec)
                .expect("fixture batch should build");
        commit_prepared(&mut model, &root, prepared).expect("fixture batch should commit");
        let model = ProjectResolver::new(&root)
            .resolve()
            .expect("seeded fixture should resolve");

        let prepared =
            build_route_strategy_fixture_net_class_clear(&model, test_provenance(), &spec.nets[0])
                .expect("net class clear should build");
        assert!(matches!(
            prepared.batch.operations[0],
            Operation::GuardObjectRevision { object_id, .. } if object_id == net_uuid
        ));
        let mut model = model;
        commit_prepared(&mut model, &root, prepared).expect("net class clear should commit");

        let resolved = ProjectResolver::new(&root)
            .resolve()
            .expect("cleared fixture should resolve");
        let board = resolved
            .materialized_source_shard_value(SourceShardKind::BoardRoot)
            .expect("board shard should materialize");
        assert!(
            board["nets"][net_uuid.to_string()]["class"].is_null(),
            "net class must persist as JSON null"
        );
        assert_eq!(board["nets"][net_uuid.to_string()]["name"], "SIG");
        let _ = std::fs::remove_dir_all(&root);
    }
}
