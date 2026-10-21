//! One canonical filled BoardGraphic; setup archives are not native acceptance.
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
    substrate::{CommitSource, DesignModel, ProjectResolver, SourceShardKind},
};
use std::{
    collections::BTreeMap,
    io::Write,
    path::{Path, PathBuf},
};
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

// Match native_project_source_inventory.py before descending or reading bytes.
fn operational(relative: &Path) -> bool {
    let mut suffix = PathBuf::new();
    let mut in_datum = false;
    for part in relative.components() {
        if in_datum {
            suffix.push(part.as_os_str());
        } else if part.as_os_str() == ".datum" {
            in_datum = true;
        }
    }
    in_datum
        && (suffix == Path::new("revision/writer.lock")
            || suffix == Path::new("gui-terminal-context.json")
            || suffix.starts_with("tool-sessions")
            || suffix.starts_with("terminal-contexts"))
}

// Read the complete canonical Project, including journal/revision history.
fn source_bytes(root: &Path, directory: &Path) -> Result<BTreeMap<PathBuf, Vec<u8>>> {
    let mut files = BTreeMap::new();
    for entry in std::fs::read_dir(directory)? {
        let path = entry?.path();
        if operational(path.strip_prefix(root)?) {
            continue;
        }
        ensure!(!path.is_symlink(), "fixture source symlink refused");
        if path.is_dir() {
            files.extend(source_bytes(root, &path)?);
        } else {
            files.insert(
                path.strip_prefix(root)?.to_path_buf(),
                std::fs::read(&path)?,
            );
        }
    }
    Ok(files)
}

fn preserve_stage(
    root: &Path,
    project: &Path,
    name: &str,
    model: &DesignModel,
) -> Result<serde_json::Value> {
    let before = source_bytes(project, project)?;
    let reopened = ProjectResolver::new(project).resolve()?;
    ensure!(
        reopened.model_revision == model.model_revision,
        "stage reopen revision"
    );
    ensure!(
        source_bytes(project, project)? == before,
        "reopen preserves complete source/history"
    );
    let archive = root.join("stages").join(name);
    ensure!(!archive.exists(), "stage archive must be fresh");
    // Byte copies only to evidence archives, never to active canonical sources.
    for (relative, bytes) in &before {
        let destination = archive.join(relative);
        std::fs::create_dir_all(destination.parent().unwrap())?;
        std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(destination)?
            .write_all(bytes)?;
    }
    let board = reopened.materialized_source_shard_value(SourceShardKind::BoardRoot)?;
    Ok(serde_json::json!({
        "name":name,"archive":archive,"revision":reopened.model_revision,
        "board_schema_version":board["schema_version"],
        "journal_transactions":reopened.journal.len(),
        "latest_operations":reopened.journal.last().map(|t| &t.operations),
        "files":before.iter().map(|(path,bytes)| (path.to_string_lossy().into_owned(), bytes.len())).collect::<BTreeMap<_,_>>(),
        "reopen_preserved_source_and_history":true
    }))
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
