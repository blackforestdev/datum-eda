//! One canonical filled BoardGraphic; setup archives are not native acceptance.
use super::source_history::preserve_stage;
use anyhow::{Result, ensure};
use eda_engine::{
    api::native_write::{
        self, WriteProvenance, board_annotations::graphics, board_layout, genesis,
    },
    board::{
        StackupLayer, StackupLayerType,
        graphics::{BoardGraphic, GraphicGeometry},
    },
    ir::geometry::Point,
    substrate::{CommitSource, ProjectResolver, SourceShardKind},
};
use std::{io::Write, path::Path};
use uuid::Uuid;

fn id(n: u128) -> Uuid {
    Uuid::from_u128(0x7d000000000040008000000000000000 + n)
}
fn provenance(reason: &str) -> WriteProvenance {
    WriteProvenance::new(
        "s5a-filled-board-graphic-fixture",
        CommitSource::Test,
        reason,
    )
}

pub(super) fn build(root: &Path) -> Result<()> {
    ensure!(
        !root.exists() && !root.is_symlink(),
        "fresh output required; prior artifacts preserved"
    );
    // The complete source identity and geometry oracle precede every query.
    let roots = genesis::GenesisRootIds {
        project: id(0x10),
        schematic: id(0x11),
        board: id(0x12),
        rules: Some(id(0x13)),
    };
    let graphic = BoardGraphic {
        uuid: id(1),
        layer: 37,
        width: 0,
        import_key: None,
        geometry: GraphicGeometry::Rectangle {
            from: Point::new(20_000_000, 20_000_000),
            to: Point::new(40_000_000, 40_000_000),
            filled: true,
        },
    };
    let project = root.join("project");
    genesis::bootstrap_native_project(
        &project,
        genesis::GenesisSpec {
            project_name: "S5A filled BoardGraphic".into(),
            existing_ids: Some(roots),
        },
    )?;
    let mut model = ProjectResolver::new(&project).resolve()?;
    let mut stages = vec![preserve_stage(root, &project, "00-genesis", &model)?];
    let write = board_layout::build_set_board_stackup(
        &model,
        provenance("documented nonconductive F.SilkS layer"),
        roots.board,
        &[StackupLayer::new(
            37,
            "F.SilkS",
            StackupLayerType::Silkscreen,
            0,
        )],
    )?;
    native_write::commit_prepared(&mut model, &project, write)?;
    stages.push(preserve_stage(
        root,
        &project,
        "01-pre-adoption-layer",
        &model,
    )?);
    let write = board_layout::source_profile::build_adopt_board_source_profile(
        &model,
        provenance("explicit forward-only BoardRoot profile-4 adoption"),
        roots.board,
    )?;
    native_write::commit_prepared(&mut model, &project, write)?;
    stages.push(preserve_stage(root, &project, "02-profile-4", &model)?);
    let write = graphics::build_create_board_graphic(
        &model,
        provenance("one standalone filled Rectangle; native proof pending"),
        roots.board,
        &graphic,
    )?;
    native_write::commit_prepared(&mut model, &project, write)?;
    stages.push(preserve_stage(
        root,
        &project,
        "03-filled-rectangle",
        &model,
    )?);
    let board = model.materialized_source_shard_value(SourceShardKind::BoardRoot)?;
    ensure!(
        board["schema_version"] == 4,
        "explicit profile adoption persisted"
    );
    let actual = board["graphics"]
        .as_object()
        .ok_or_else(|| anyhow::anyhow!("missing graphics map"))?;
    ensure!(
        actual.len() == 1
            && actual.get(&graphic.uuid.to_string()) == Some(&serde_json::to_value(&graphic)?),
        "complete standalone graphic catalogue matches predeclared singleton"
    );
    let manifest = serde_json::json!({
        "schema_version":1,"family":"filled-board-graphic","project_root":project,
        "project_id":roots.project,"board_id":roots.board,"schematic_id":roots.schematic,"rules_id":roots.rules,
        "model_revision":model.model_revision,"graphic":graphic,"stages":stages,
        "expected_members":[{"class":"board_graphic","id":graphic.uuid,"instance_path":[]}],
        "proposed_workflow_nm":{
            "partial_enclosure":{"from":[19_000_000,19_000_000],"to":[35_000_000,41_000_000],"expected_kind":"none"},
            "full_enclosure":{"from":[19_000_000,19_000_000],"to":[41_000_000,41_000_000],"expected_kind":"compound","focus":null},
            "plain_interior":{"point":[30_000_000,30_000_000],"expected_kind":"object","focus":graphic.uuid}
        },
        "acceptance":"not established; setup only, native pixels/camera/consumer proof and independent review pending"
    });
    std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(root.join("s5a-fixture.json"))?
        .write_all(eda_test_harness::canonical_json(&manifest)?.as_bytes())?;
    println!("{}", root.join("s5a-fixture.json").display());
    Ok(())
}
