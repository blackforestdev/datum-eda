//! Authored identities chosen before canonical creation; query results are checked
//! against independent complete sets, never fed back as their own expected oracle.
use anyhow::{Context, Result, ensure};
use eda_engine::{
    api::native_write::{self, BatchComposer, WriteProvenance, genesis},
    board::{Net, StackupLayer, StackupLayerType, Track},
    ir::geometry::Point,
    schematic::{SchematicText, Sheet},
    substrate::{
        CommitSource, ElectricalOccurrence, ElectricalSelectionSnapshot, Operation, ProjectResolver,
    },
};
use std::{
    collections::{BTreeSet, HashMap},
    path::Path,
};
use uuid::Uuid;
#[path = "recipe/identities.rs"]
mod identities;
fn id(group: u128, ordinal: u128) -> Uuid {
    Uuid::from_u128((group << 96) | 0x40008000000000000000 | ordinal)
}
fn provenance() -> WriteProvenance {
    WriteProvenance::new(
        "s5a-fixture",
        CommitSource::Test,
        "finite native S5A fixture recipe",
    )
}
fn occurrence(class: &str, source_id: Uuid) -> ElectricalOccurrence {
    ElectricalOccurrence {
        class: class.into(),
        source_id,
        instance_path: vec![],
    }
}
pub(super) fn build_and_check(root: &Path, count: usize) -> Result<serde_json::Value> {
    build_with_project_group(root, count, 0x20000000)
}
pub(super) fn build_with_project_group(
    root: &Path,
    count: usize,
    group: u128,
) -> Result<serde_json::Value> {
    let roots = genesis::GenesisRootIds {
        project: id(group, 1),
        schematic: id(group, 2),
        board: id(group, 3),
        rules: Some(id(group, 4)),
    };
    genesis::bootstrap_native_project(
        root,
        genesis::GenesisSpec {
            project_name: "S5A native core".into(),
            existing_ids: Some(roots),
        },
    )?;
    let mut model = ProjectResolver::new(root).resolve()?;
    let layer = StackupLayer::new(1, "F.Cu", StackupLayerType::Copper, 35_000);
    let write = native_write::board_layout::build_set_board_stackup(
        &model,
        provenance(),
        roots.board,
        &[layer],
    )?;
    native_write::commit_prepared(&mut model, root, write)?;
    let sheet = Sheet {
        uuid: id(group, 5),
        name: "Selection".into(),
        frame: None,
        symbols: HashMap::new(),
        wires: HashMap::new(),
        junctions: HashMap::new(),
        labels: HashMap::new(),
        buses: HashMap::new(),
        bus_entries: HashMap::new(),
        ports: HashMap::new(),
        noconnects: HashMap::new(),
        texts: HashMap::new(),
        drawings: HashMap::new(),
    };
    // Persist the existing native shard header used by CLI create-sheet and
    // native_write schematic_sheets tests; Sheet is only the geometry payload.
    let mut payload = serde_json::to_value(&sheet)?;
    payload["schema_version"] = serde_json::json!(1);
    let write = native_write::schematic_sheets::build_create_schematic_sheet(
        &model,
        provenance(),
        roots.schematic,
        sheet.uuid,
        "selection.json",
        payload,
    )?;
    native_write::commit_prepared(&mut model, root, write)?;
    let nets = [
        Net::new(id(0x30000000, 1), "same label", Uuid::nil()),
        Net::new(id(0x30000000, 2), "same label", Uuid::nil()),
    ];
    let write = BatchComposer::compose(&model, provenance())
        .push_ops(nets.iter().map(|net| Operation::CreateBoardNet {
            net_id: net.uuid,
            net: serde_json::to_value(net).unwrap(),
        }))
        .finish()?;
    native_write::commit_prepared(&mut model, root, write)?;
    let tracks = [
        Track::straight(
            id(0x40000000, 1),
            nets[0].uuid,
            Point::new(0, 0),
            Point::new(10_000_000, 0),
            5_080_000,
            1,
        ),
        Track::straight(
            id(0x40000000, 2),
            nets[0].uuid,
            Point::new(10_000_000, 0),
            Point::new(20_000_000, 0),
            5_080_001,
            1,
        ),
        Track::straight(
            id(0x40000000, 3),
            nets[0].uuid,
            Point::new(40_000_000, 0),
            Point::new(50_000_000, 0),
            5_080_000,
            1,
        ),
        Track::straight(
            id(0x40000000, 4),
            nets[1].uuid,
            Point::new(10_000_000, -10_000_000),
            Point::new(10_000_000, 10_000_000),
            2,
            1,
        ),
    ];
    let mut ops: Vec<_> = tracks
        .iter()
        .map(|track| {
            Ok(Operation::CreateBoardTrack {
                track_id: track.uuid,
                track: serde_json::to_value(track)?,
            })
        })
        .collect::<Result<_>>()?;
    let mut expected_text = BTreeSet::new();
    for ordinal in 0..count {
        let text = SchematicText {
            uuid: id(0x50000000, 0x1000 + ordinal as u128),
            text: "I".into(),
            position: Point::new(
                (ordinal % 400) as i64 * 2_000_000,
                (ordinal / 400) as i64 * 2_000_000,
            ),
            rotation: 0,
        };
        ops.push(Operation::CreateSchematicText {
            sheet_id: sheet.uuid,
            text_id: text.uuid,
            text: serde_json::to_value(&text)?,
        });
        expected_text.insert(occurrence("texts", text.uuid));
    }
    let write = BatchComposer::compose(&model, provenance())
        .push_ops(ops)
        .finish()?;
    native_write::commit_prepared(&mut model, root, write)?;
    let logical = identities::bind_board_net(
        &mut model,
        root,
        nets[0].uuid,
        occurrence("tracks", tracks[0].uuid),
    )?;
    model = ProjectResolver::new(root).resolve()?;
    let mut files: Vec<_> = model
        .source_shards
        .iter()
        .map(|s| Ok((s.path.clone(), std::fs::read(&s.path)?)))
        .collect::<Result<_>>()?;
    let journal = eda_engine::substrate::transaction_journal_path(root);
    files.push((journal.clone(), std::fs::read(&journal)?));
    let snapshot = ElectricalSelectionSnapshot::capture(&model)
        .map_err(|e| anyhow::anyhow!("capture: {e:?}"))?;
    let expected_run: BTreeSet<_> = tracks[..2]
        .iter()
        .map(|t| occurrence("tracks", t.uuid))
        .collect();
    let result = snapshot
        .board_run(
            snapshot.revision(),
            &occurrence("tracks", tracks[0].uuid),
            None,
        )
        .map_err(|e| anyhow::anyhow!("board Run: {e:?}"))?;
    ensure!(
        result.members == expected_run,
        "complete Run must exclude disconnected and foreign-Net tracks"
    );
    let contact = result
        .contact_evidence
        .as_ref()
        .map_err(|e| anyhow::anyhow!("contact evidence: {e:?}"))?;
    let pairs: BTreeSet<_> = contact
        .iter()
        .map(|c| {
            ensure!(c.layers == BTreeSet::from([1]), "contact layer authority");
            ensure!(
                BTreeSet::from([c.left_net, c.right_net])
                    == BTreeSet::from([nets[0].uuid, nets[1].uuid]),
                "contact assignments"
            );
            Ok(BTreeSet::from([c.left.source_id, c.right.source_id]))
        })
        .collect::<Result<_>>()?;
    let expected_pairs = BTreeSet::from([
        BTreeSet::from([tracks[0].uuid, tracks[3].uuid]),
        BTreeSet::from([tracks[1].uuid, tracks[3].uuid]),
    ]);
    ensure!(
        pairs == expected_pairs,
        "complete foreign contact identities"
    );
    let expected_net: BTreeSet<_> = tracks[..3]
        .iter()
        .map(|t| occurrence("tracks", t.uuid))
        .collect();
    let global = snapshot
        .global_net(snapshot.revision(), logical)
        .map_err(|e| anyhow::anyhow!("Global Net: {e:?}"))?;
    ensure!(
        global.owned == expected_net,
        "complete Global Net includes disconnected Run and excludes foreign Net"
    );
    let authored = snapshot
        .authored_sources()
        .map_err(|e| anyhow::anyhow!("authored sources: {e:?}"))?;
    let actual_text: BTreeSet<_> = authored
        .iter()
        .filter(|s| s.occurrence.class == "texts")
        .map(|s| s.occurrence.clone())
        .collect();
    ensure!(
        actual_text == expected_text,
        "complete persisted text identity set must survive reopen"
    );
    ensure!(
        tracks[0].width != tracks[1].width,
        "canonical quantity oracle is distinct"
    );
    for (path, before) in &files {
        ensure!(
            *before == std::fs::read(path).with_context(|| path.display().to_string())?,
            "read-only query changed {}",
            path.display()
        );
    }
    Ok(serde_json::json!({
        "project_id": roots.project, "board_id": roots.board, "sheet_id": sheet.uuid,
        "model_revision": model.model_revision, "source_basis": snapshot.source_basis(),
        "expected_run_members": expected_run,
        "observed_run": { "origin": result.origin, "board_net": result.board_net, "members": result.members, "contact_evidence": contact },
        "expected_global_net": { "subject_id": logical, "members": expected_net },
        "observed_global_net": global,
        "disconnected_same_net": occurrence("tracks", tracks[2].uuid),
        "foreign_net": nets[1].uuid, "foreign_track": occurrence("tracks", tracks[3].uuid),
        "canonical_widths_nm": [tracks[0].width, tracks[1].width],
        "text_population": { "class": "texts", "instance_path": [], "first_uuid": id(0x50000000, 0x1000), "last_uuid": id(0x50000000, 0x1000 + count as u128 - 1), "count": count, "construction": "consecutive authored UUIDs, canonical CreateSchematicText batch", "grid_columns": 400, "spacing_nm": 2000000 },
        "transport_controls": { "first_256": if count >= 256 {Some(256)} else {None}, "full_257": if count == 257 {Some(257)} else {None} },
        "nonmutation": "all source shards and complete journal compared byte-for-byte around snapshot/queries",
        "journal_path": journal.strip_prefix(root)?.to_string_lossy(), "journal_bytes": files.last().unwrap().1.len()
    }))
}
