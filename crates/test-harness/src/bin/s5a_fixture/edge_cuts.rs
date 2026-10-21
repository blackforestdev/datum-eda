//! Isolated negative fixture: synthetic component Edge.Cuts, never library mapping.
use super::source_history::{preserve_stage, source_bytes};
use anyhow::{Result, ensure};
use eda_engine::{
    api::native_write::{
        self, WriteProvenance,
        board_components::{
            self, BoardPackagePlacement,
            pad_materialization::{
                ComponentPadMaterializationRequest, PadLandPatternSource,
                prepare_component_pad_materialization,
            },
        },
        board_layout,
    },
    board::{PlacedPackage, StackupLayer, StackupLayerType},
    ir::geometry::{Point, Polygon},
    substrate::{CommitSource, ProjectResolver, SourceShardKind, SourceShardTaxon},
};
use std::{io::Write, path::Path};
use uuid::Uuid;

const SEED_REVISION: &str = "3ac065a1457f9e2277be5c04a4234b0a4a36e9b8effcf57ee519c21d411f1a0b";
fn provenance(reason: &str) -> WriteProvenance {
    WriteProvenance::new(
        "s5a-synthetic-component-edge-cuts-fixture",
        CommitSource::Test,
        reason,
    )
}
fn id(text: &str) -> Result<Uuid> {
    Ok(Uuid::parse_str(text)?)
}

pub(super) fn build(root: &Path, seed: &Path) -> Result<()> {
    ensure!(
        !root.exists() && !root.is_symlink(),
        "fresh output required; no retry overwrite"
    );
    ensure!(
        seed.is_dir() && !seed.is_symlink(),
        "real frozen seed directory required"
    );
    let seed = seed.canonicalize()?;
    ensure!(
        root.is_absolute() && !root.starts_with(&seed),
        "separate absolute output required"
    );
    let before = source_bytes(&seed, &seed)?;
    let original = ProjectResolver::new(&seed).resolve()?;
    ensure!(
        original.model_revision.0.as_str() == SEED_REVISION
            && original.journal.len() == 6
            && before.len() == 51,
        "exact frozen seed identity/history required"
    );
    ensure!(
        source_bytes(&seed, &seed)? == before,
        "seed resolve must preserve bytes"
    );
    let project = root.join("project");
    // Copy the complete frozen input/history once; all design changes below are journaled.
    for (relative, bytes) in &before {
        let destination = project.join(relative);
        std::fs::create_dir_all(destination.parent().unwrap())?;
        std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(destination)?
            .write_all(bytes)?;
    }
    ensure!(
        source_bytes(&project, &project)? == before,
        "exact canonical seed copy"
    );
    let mut model = ProjectResolver::new(&project).resolve()?;
    let mut stages = vec![preserve_stage(root, &project, "00-frozen-seed", &model)?];
    let board_id = id("72000000-0000-4000-8000-000000000003")?;
    let parent = PlacedPackage {
        uuid: id("7e000000-0000-4000-8000-000000000001")?,
        part: id("90000000-0000-4000-8000-000000000001")?,
        package: id("50000000-0000-4000-8000-000000000001")?,
        reference: "UEDGE".into(),
        value: "Synthetic Edge.Cuts negative".into(),
        position: Point::new(0, 0),
        rotation: 0,
        layer: 1,
        locked: false,
    };
    let footprint_id = id("60000000-0000-4000-8000-000000000001")?;
    let board = model.materialized_source_shard_value(SourceShardKind::BoardRoot)?;
    ensure!(
        board["uuid"] == serde_json::json!(board_id)
            && board["schema_version"] == 1
            && board["packages"].as_object().is_some_and(|v| v.is_empty()),
        "empty seed Board required"
    );
    let mut layers: Vec<StackupLayer> = serde_json::from_value(board["stackup"]["layers"].clone())?;
    ensure!(
        !layers.iter().any(|v| v.id == 44),
        "seed has no Edge.Cuts layer"
    );
    layers.push(StackupLayer::new(
        44,
        "Edge.Cuts",
        StackupLayerType::Mechanical,
        0,
    ));
    let write = board_layout::build_set_board_stackup(
        &model,
        provenance("declare existing canonical L44 Edge.Cuts for synthetic negative fixture"),
        board_id,
        &layers,
    )?;
    native_write::commit_prepared(&mut model, &project, write)?;
    stages.push(preserve_stage(root, &project, "01-edge-layer", &model)?);
    let write = board_layout::source_profile::build_adopt_board_source_profile(
        &model,
        provenance("separate existing BoardRoot profile-4 adoption"),
        board_id,
    )?;
    native_write::commit_prepared(&mut model, &project, write)?;
    stages.push(preserve_stage(root, &project, "02-profile-4", &model)?);
    let outline = Polygon::new(vec![
        Point::new(0, 0),
        Point::new(20_000_000, 0),
        Point::new(20_000_000, 10_000_000),
        Point::new(0, 10_000_000),
    ]);
    let write = board_layout::build_set_board_outline(
        &model,
        provenance("real board-owned outline, separate from synthetic component edge"),
        board_id,
        &outline,
    )?;
    native_write::commit_prepared(&mut model, &project, write)?;
    stages.push(preserve_stage(root, &project, "03-board-outline", &model)?);
    // Only this polyline is synthetic. No Footprint primitive gains an Edge.Cuts role.
    let edge = serde_json::json!({"layer":44,"width_nm":100_000,"vertices":[
        {"x":5_000_000,"y":0},{"x":15_000_000,"y":0},
        {"x":15_000_000,"y":5_000_000},{"x":5_000_000,"y":5_000_000},
        {"x":5_000_000,"y":0}]});
    let footprint_source = model
        .source_shards
        .iter()
        .find(|s| {
            s.taxon == Some(SourceShardTaxon::PoolFootprint)
                && model
                    .objects
                    .get(&footprint_id)
                    .is_some_and(|o| o.source_shard_id == s.shard_id)
        })
        .ok_or_else(|| anyhow::anyhow!("exact frozen Footprint source required"))?;
    let footprint: eda_engine::pool::Footprint = serde_json::from_value(
        model.materialized_source_shard_value_by_relative_path(&footprint_source.relative_path)?,
    )?;
    ensure!(
        footprint.uuid == footprint_id
            && footprint.package == parent.package
            && footprint.pads.len() == 8
            && footprint.silkscreen.is_empty()
            && footprint.mechanical.is_empty()
            && footprint.fab.is_empty()
            && footprint.assembly.is_empty(),
        "expected unmodified library Footprint required"
    );
    let courtyard = serde_json::json!({"layer":41,"vertices":footprint.courtyard.vertices});
    let request = ComponentPadMaterializationRequest {
        placement: parent.clone(),
        land_pattern: PadLandPatternSource::Footprint(footprint_id),
        component_instance: None,
        pin_pad_map: None,
        correspondences: vec![],
        adopt_existing: false,
    };
    let materialized = prepare_component_pad_materialization(
        &model,
        &request,
        &serde_json::json!({
            "component_mechanical_polylines":[edge.clone()],
            "component_mechanical_polygons":[courtyard.clone()]
        }),
    )?;
    ensure!(
        materialized["component_pads"]
            .as_array()
            .is_some_and(|v| v.len() == 8),
        "eight real source-bound Pads required"
    );
    let expected_pads = materialized["component_pads"].clone();
    let write = board_components::build_place_board_package(
        &model,
        provenance(
            "synthetic component L44 negative geometry with real pinned eight-Pad source; not library Edge.Cuts authoring",
        ),
        &BoardPackagePlacement {
            package: parent.clone(),
            materialized,
        },
    )?;
    native_write::commit_prepared(&mut model, &project, write)?;
    stages.push(preserve_stage(
        root,
        &project,
        "04-synthetic-component-edge",
        &model,
    )?);
    let actual = model.materialized_source_shard_value(SourceShardKind::BoardRoot)?;
    let key = parent.uuid.to_string();
    ensure!(
        actual["schema_version"] == 4
            && actual["outline"] == serde_json::to_value(&outline)?
            && actual["packages"].as_object().is_some_and(|v| v.len() == 1)
            && actual["packages"][&key] == serde_json::to_value(&parent)?
            && actual["component_pads"][&key] == expected_pads
            && actual["component_mechanical_polylines"][&key] == serde_json::json!([edge.clone()])
            && actual["component_mechanical_polygons"][&key] == serde_json::json!([courtyard]),
        "persisted complete fixture identities and geometry must match construction"
    );
    ensure!(
        model.journal.len() == original.journal.len() + 4
            && model.journal[..original.journal.len()] == original.journal[..],
        "four appended transactions preserve seed history"
    );
    ensure!(
        source_bytes(&seed, &seed)? == before,
        "frozen source remains byte-identical"
    );
    let manifest = serde_json::json!({"schema_version":1,"family":"component-edge-cuts",
        "origin":"synthetic isolated negative fixture, CommitSource::Test; not library-derived Edge.Cuts",
        "seed":seed,"seed_revision":SEED_REVISION,"project_root":project,
        "project_id":model.project.project_id,"board_id":board_id,"parent":parent,
        "footprint_id":footprint_id,"placed_pads":expected_pads,"component_edge":edge,
        "board_outline":outline,"model_revision":model.model_revision,"stages":stages,
        "proposed_workflow_nm":{
            "owned_only":{"point":[10_000_000,5_000_000],"expected_kind":"none","expected_members":[],"focus":null},
            "overlap":{"point":[10_000_000,0],"expected_kind":"object","expected_members":[{"class":"board_outline","id":board_id,"instance_path":[]}],"focus":board_id}},
        "acceptance":"not established; actual pixel/camera/consumer proof and independent review pending; no library mapping, electrical linkage, C04 closure or successor credit"});
    std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(root.join("s5a-fixture.json"))?
        .write_all(eda_test_harness::canonical_json(&manifest)?.as_bytes())?;
    println!("{}", root.join("s5a-fixture.json").display());
    Ok(())
}
