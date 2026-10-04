//! Extra native classes use existing source owners and the actual bounded fill producer.
use anyhow::{Result, ensure};
use eda_engine::{
    api::native_write::{self, BatchComposer, WriteProvenance},
    board::{Net, NetClass, PadLayerConnection, PadShape, PlacedPad, Track, Via, Zone},
    ir::geometry::{Arc, Point, Polygon},
    schematic::SchematicPrimitive,
    substrate::{
        CommitSource, ElectricalSelectionSnapshot, Operation, ProjectResolver, ZoneFill,
        ZoneFillCopperContext, ZoneFillState, compute_bounded_zone_fill,
    },
};
use std::{collections::BTreeSet, path::Path};
use uuid::Uuid;
fn id(n: u128) -> Uuid {
    Uuid::from_u128(0x71000000000040008000000000000000 + n)
}
fn provenance() -> WriteProvenance {
    WriteProvenance::new(
        "s5a-class-fixture",
        CommitSource::Test,
        "existing native class source recipe",
    )
}
fn rectangle(x: i64, y: i64, width: i64, height: i64) -> Polygon {
    Polygon::new(vec![
        Point::new(x, y),
        Point::new(x + width, y),
        Point::new(x + width, y + height),
        Point::new(x, y + height),
    ])
}
pub(super) fn build(root: &Path) -> Result<serde_json::Value> {
    let path = root.join("variants/class-geometry");
    super::lifecycle::copy_project(root, &path)?;
    let mut model = ProjectResolver::new(&path).resolve()?;
    let snapshot = ElectricalSelectionSnapshot::capture(&model)
        .map_err(|e| anyhow::anyhow!("capture: {e:?}"))?;
    let sheet = snapshot
        .schematic_source()
        .unwrap()
        .sheets
        .values()
        .next()
        .unwrap()
        .uuid;
    let net = id(11);
    let origin_net = id(12);
    // Explicit authored fixture rules, never a manufacturing allowance or budget.
    let class = NetClass {
        uuid: id(10),
        name: "Class fixture".into(),
        clearance: 200000,
        track_width: 100000,
        via_drill: 250000,
        via_diameter: 1000000,
        diffpair_width: 100000,
        diffpair_gap: 200000,
    };
    let nets = [
        Net::new(net, "zone source", class.uuid),
        Net::new(origin_net, "barrier source", class.uuid),
    ];
    let pad = PlacedPad {
        uuid: id(1),
        package: Uuid::nil(),
        name: "P".into(),
        net: Some(net),
        position: Point::new(100_000_000, 0),
        layer: 1,
        copper_layers: vec![1],
        shape: PadShape::Rect,
        diameter: 0,
        width: 2_000_000,
        height: 2_000_000,
        drill: 0,
        rotation: 0,
        roundrect_rratio_ppm: 250000,
        mask_layers: vec![],
        paste_layers: vec![],
        solder_mask_margin_nm: 0,
        solder_paste_margin_nm: 0,
        solder_paste_margin_ratio_ppm: 0,
        layer_connection: PadLayerConnection::Separate,
    };
    let via = Via {
        uuid: id(2),
        net,
        position: Point::new(120_000_000, 0),
        drill: 250000,
        diameter: 1_000_000,
        from_layer: 1,
        to_layer: 1,
    };
    let zone = Zone {
        uuid: id(3),
        net,
        polygon: rectangle(140_000_000, 0, 4_000_000, 4_000_000),
        layer: 1,
        priority: 0,
        thermal_relief: false,
        thermal_gap: 0,
        thermal_spoke_width: 0,
    };
    let curve = Track {
        midpoint: Some(Point::new(85_000_000, 5_000_000)),
        ..Track::straight(
            id(4),
            net,
            Point::new(80_000_000, 0),
            Point::new(90_000_000, 0),
            100000,
            1,
        )
    };
    let barrier = Track::straight(
        id(5),
        origin_net,
        Point::new(142_000_000, -1_000_000),
        Point::new(142_000_000, 5_000_000),
        500000,
        1,
    );
    let drawings = [
        SchematicPrimitive::Line {
            uuid: id(6),
            from: Point::new(0, 10_000_000),
            to: Point::new(10_000_000, 10_000_000),
        },
        SchematicPrimitive::Rect {
            uuid: id(7),
            min: Point::new(20_000_000, 10_000_000),
            max: Point::new(25_000_000, 15_000_000),
        },
        SchematicPrimitive::Circle {
            uuid: id(8),
            center: Point::new(35_000_000, 12_500_000),
            radius: 2_500_000,
        },
        SchematicPrimitive::Arc {
            uuid: id(9),
            arc: Arc {
                center: Point::new(45_000_000, 12_500_000),
                radius: 2_500_000,
                start_angle: 0,
                end_angle: 1800,
            },
        },
    ];
    let mut operations = vec![
        Operation::CreateBoardNetClass {
            net_class_id: class.uuid,
            net_class: serde_json::to_value(&class)?,
        },
        Operation::CreateBoardNet {
            net_id: nets[0].uuid,
            net: serde_json::to_value(&nets[0])?,
        },
        Operation::CreateBoardNet {
            net_id: nets[1].uuid,
            net: serde_json::to_value(&nets[1])?,
        },
        Operation::CreateBoardPad {
            pad_id: pad.uuid,
            pad: serde_json::to_value(&pad)?,
        },
        Operation::CreateBoardVia {
            via_id: via.uuid,
            via: serde_json::to_value(&via)?,
        },
        Operation::CreateBoardZone {
            zone_id: zone.uuid,
            zone: serde_json::to_value(&zone)?,
        },
    ];
    for track in [&curve, &barrier] {
        operations.push(Operation::CreateBoardTrack {
            track_id: track.uuid,
            track: serde_json::to_value(track)?,
        });
    }
    for (ordinal, drawing) in drawings.iter().enumerate() {
        operations.push(Operation::CreateSchematicDrawing {
            sheet_id: sheet,
            drawing_id: id(6 + ordinal as u128),
            drawing: serde_json::to_value(drawing)?,
        });
    }
    let write = BatchComposer::compose(&model, provenance())
        .push_ops(operations)
        .finish()?;
    native_write::commit_prepared(&mut model, &path, write)?;
    let value =
        model.materialized_source_shard_value(eda_engine::substrate::SourceShardKind::BoardRoot)?;
    let net_classes: std::collections::BTreeMap<Uuid, NetClass> =
        serde_json::from_value(value["net_classes"].clone())?;
    let persisted_nets: std::collections::BTreeMap<Uuid, Net> =
        serde_json::from_value(value["nets"].clone())?;
    let clearances = persisted_nets
        .values()
        .filter_map(|net| {
            net_classes
                .get(&net.class)
                .map(|class| (net.uuid, class.clearance))
        })
        .collect();
    let context = ZoneFillCopperContext {
        tracks: serde_json::from_value::<std::collections::BTreeMap<Uuid, Track>>(
            value["tracks"].clone(),
        )?
        .into_values()
        .collect(),
        net_clearance_nm: clearances,
        pads: vec![pad.clone()],
        vias: vec![via],
        ..Default::default()
    };
    let (state, islands, reason) = compute_bounded_zone_fill(&zone, &context);
    ensure!(
        state == ZoneFillState::Filled && islands.len() == 2,
        "required real two-island fill: {reason}"
    );
    let fill = ZoneFill {
        schema_version: 2,
        zone_id: zone.uuid,
        state,
        source_zone_revision: model.objects[&zone.uuid].object_revision,
        model_revision: model.model_revision.clone(),
        islands: islands.clone(),
        provenance: Some(reason),
    };
    let write = native_write::board_routing::build_set_zone_fills(&model, provenance(), &[fill])?;
    native_write::commit_prepared(&mut model, &path, write)?;
    model = ProjectResolver::new(&path).resolve()?;
    ensure!(
        model.current_zone_fill(zone.uuid).is_some(),
        "current fill survives reopen"
    );
    let mut files: Vec<_> = model
        .source_shards
        .iter()
        .map(|s| Ok((s.path.clone(), std::fs::read(&s.path)?)))
        .collect::<Result<_>>()?;
    let journal = eda_engine::substrate::transaction_journal_path(&path);
    files.push((journal.clone(), std::fs::read(&journal)?));
    let current = ElectricalSelectionSnapshot::capture(&model)
        .map_err(|e| anyhow::anyhow!("class capture: {e:?}"))?;
    let sources = current
        .authored_sources()
        .map_err(|e| anyhow::anyhow!("class inventory: {e:?}"))?;
    let expected: BTreeSet<_> = (1..=9).map(id).collect();
    let actual: BTreeSet<_> = sources
        .iter()
        .map(|s| s.occurrence.source_id)
        .filter(|id| expected.contains(id))
        .collect();
    ensure!(
        actual == expected,
        "complete actual additional source identities"
    );
    for (file, before) in files {
        ensure!(
            before == std::fs::read(file)?,
            "class query mutated source/journal"
        );
    }
    let pad_anchors = vec![
        Point::new(99_000_000, -1_000_000),
        Point::new(100_000_000, -1_000_000),
        Point::new(101_000_000, -1_000_000),
        Point::new(101_000_000, 0),
        Point::new(101_000_000, 1_000_000),
        Point::new(100_000_000, 1_000_000),
        Point::new(99_000_000, 1_000_000),
        Point::new(99_000_000, 0),
    ];
    Ok(
        serde_json::json!({"relative_root":"variants/class-geometry","model_revision":model.model_revision,"source_basis":current.source_basis(),"expected_additional_ids":expected,"source_inventory":sources,"authored_net_class":class,"pad_id":pad.uuid,"pad_anchors":pad_anchors,"pad_majority_oracle":{"qualified_4":false,"qualified_5":true},"track_arc_id":curve.uuid,"track_arc_authored_anchors":[curve.from,curve.midpoint.unwrap(),curve.to],"track_arc_majority_oracle":{"qualified_1":false,"qualified_2":true},"zone_id":zone.uuid,"fill_islands":islands,"filled_area_oracle":"all islands require full enclosure; no authored-boundary or centroid substitute","native_acquisition":"pending; this verifies source availability only","remaining":"placed-parent/Pin, board text, graphics, hidden/filter and native gesture bindings remain explicit additional cases"}),
    )
}
