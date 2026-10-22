//! Canonical two-layer Board Run; UI visibility and filtering remain native proof.
use anyhow::{Context, Result, ensure};
use eda_engine::{
    api::native_write::{self, BatchComposer, WriteProvenance, board_layout},
    board::{StackupLayer, StackupLayerType, Track, Via},
    ir::geometry::Point,
    substrate::{
        CommitSource, ElectricalOccurrence, ElectricalSelectionSnapshot, Operation,
        ProjectResolver, SourceShardKind,
    },
};
use std::{collections::BTreeMap, collections::BTreeSet, io::Write, path::Path};
use uuid::Uuid;

fn id(group: u128, ordinal: u128) -> Uuid {
    Uuid::from_u128((group << 96) | 0x40008000000000000000 | ordinal)
}
fn provenance() -> WriteProvenance {
    WriteProvenance::new(
        "s5a-board-derived-hidden-fixture",
        CommitSource::Test,
        "two-layer Track/Via chain with disconnected and foreign-Net controls",
    )
}
fn occurrence(class: &str, source_id: Uuid) -> ElectricalOccurrence {
    ElectricalOccurrence {
        class: class.into(),
        source_id,
        instance_path: vec![],
    }
}
fn gui_identity(source: &ElectricalOccurrence) -> serde_json::Value {
    serde_json::json!({
        "class":if source.class == "tracks" {"track"} else {"via"},
        "id":source.source_id,"instance_path":[]
    })
}

pub(super) fn build(root: &Path) -> Result<()> {
    ensure!(
        !root.exists() && !root.is_symlink(),
        "fresh output required; prior artifacts preserved"
    );
    // All geometry and complete membership sets precede the engine queries.
    let board_net = id(0x30000000, 1);
    let tracks = [
        Track::straight(
            id(0x40000000, 1),
            board_net,
            Point::new(0, 0),
            Point::new(10_000_000, 0),
            500_000,
            1,
        ),
        Track::straight(
            id(0x40000000, 2),
            board_net,
            Point::new(10_000_000, 0),
            Point::new(20_000_000, 0),
            500_000,
            2,
        ),
        Track::straight(
            id(0x40000000, 5),
            board_net,
            Point::new(20_000_000, 0),
            Point::new(30_000_000, 0),
            500_000,
            1,
        ),
        Track::straight(
            id(0x40000000, 3),
            board_net,
            Point::new(40_000_000, 0),
            Point::new(50_000_000, 0),
            500_000,
            1,
        ),
    ];
    let vias = [10_000_000, 20_000_000].map(|x| Via {
        uuid: id(0x7f000000, if x == 10_000_000 { 1 } else { 2 }),
        net: board_net,
        position: Point::new(x, 0),
        drill: 400_000,
        diameter: 1_000_000,
        from_layer: 1,
        to_layer: 2,
    });
    let expected_run: BTreeSet<_> = tracks[..3]
        .iter()
        .map(|t| occurrence("tracks", t.uuid))
        .chain(vias.iter().map(|v| occurrence("vias", v.uuid)))
        .collect();
    let disconnected = occurrence("tracks", tracks[3].uuid);
    let foreign = occurrence("tracks", id(0x40000000, 4));
    let expected_global: BTreeSet<_> = expected_run
        .iter()
        .cloned()
        .chain([disconnected.clone()])
        .collect();
    let expected_layers: BTreeMap<_, BTreeSet<_>> = tracks[..3]
        .iter()
        .map(|t| (occurrence("tracks", t.uuid), [t.layer].into()))
        .chain(
            vias.iter()
                .map(|v| (occurrence("vias", v.uuid), [1, 2].into())),
        )
        .collect();
    ensure!(
        expected_run.len() == 5 && expected_global.len() == 6,
        "complete independent sets"
    );
    let origin = occurrence("tracks", tracks[0].uuid);
    let base = super::recipe::build_with_project_group(root, 1, 0x2a000000)?;
    let board: Uuid = serde_json::from_value(base["board_id"].clone())?;
    let logical: Uuid = serde_json::from_value(base["expected_global_net"]["subject_id"].clone())?;
    ensure!(
        logical != board_net,
        "logical and board-local Net identities remain distinct"
    );
    let mut model = ProjectResolver::new(root).resolve()?;
    let before_board = model.materialized_source_shard_value(SourceShardKind::BoardRoot)?;
    let mut expected_tracks: BTreeMap<Uuid, Track> =
        serde_json::from_value(before_board["tracks"].clone())?;
    ensure!(
        expected_tracks.len() == 4,
        "unchanged four-Track base recipe"
    );
    ensure!(
        before_board["vias"]
            .as_object()
            .is_some_and(|m| m.is_empty())
            && before_board["zones"]
                .as_object()
                .is_some_and(|m| m.is_empty())
            && before_board["pads"]
                .as_object()
                .is_some_and(|m| m.is_empty())
            && before_board["packages"]
                .as_object()
                .is_some_and(|m| m.is_empty()),
        "no additional copper or placed-parent inputs"
    );
    ensure!(
        base["observed_run"]["board_net"] == serde_json::to_value(board_net)?,
        "base board Net binding"
    );
    for source_id in [tracks[2].uuid, vias[0].uuid, vias[1].uuid] {
        ensure!(
            !model.objects.contains_key(&source_id)
                && !model.electrical_identities.contains_key(&source_id),
            "new identity collision: {source_id}"
        );
    }
    let layers = [
        StackupLayer::new(1, "F.Cu", StackupLayerType::Copper, 35_000),
        StackupLayer::new(2, "B.Cu", StackupLayerType::Copper, 35_000),
    ];
    let write = board_layout::build_set_board_stackup(&model, provenance(), board, &layers)?;
    native_write::commit_prepared(&mut model, root, write)?;
    let mut operations = Vec::new();
    for track in &tracks {
        let payload = serde_json::to_value(track)?;
        operations.push(if track.uuid == tracks[2].uuid {
            Operation::CreateBoardTrack {
                track_id: track.uuid,
                track: payload,
            }
        } else {
            ensure!(
                expected_tracks.contains_key(&track.uuid),
                "existing Track identity"
            );
            Operation::SetBoardTrack {
                track_id: track.uuid,
                track: payload,
            }
        });
        expected_tracks.insert(track.uuid, track.clone());
    }
    for via in &vias {
        operations.push(Operation::CreateBoardVia {
            via_id: via.uuid,
            via: serde_json::to_value(via)?,
        });
    }
    let write = BatchComposer::compose(&model, provenance())
        .push_ops(operations)
        .finish()?;
    native_write::commit_prepared(&mut model, root, write)?;
    let actual_board = model.materialized_source_shard_value(SourceShardKind::BoardRoot)?;
    ensure!(
        serde_json::from_value::<BTreeMap<Uuid, Track>>(actual_board["tracks"].clone())?
            == expected_tracks,
        "exact complete Track catalogue, including foreign control"
    );
    let expected_vias: BTreeMap<_, _> = vias.iter().map(|v| (v.uuid, v.clone())).collect();
    ensure!(
        serde_json::from_value::<BTreeMap<Uuid, Via>>(actual_board["vias"].clone())?
            == expected_vias,
        "exact two Via catalogue"
    );
    ensure!(
        actual_board["zones"]
            .as_object()
            .is_some_and(|m| m.is_empty())
            && model.zone_fills.is_empty(),
        "zero Zones and generated-fill inputs"
    );
    let revision = model.model_revision.clone();
    let mut before: Vec<_> = model
        .source_shards
        .iter()
        .map(|s| Ok((s.path.clone(), std::fs::read(&s.path)?)))
        .collect::<Result<_>>()?;
    let journal = eda_engine::substrate::transaction_journal_path(root);
    before.push((journal.clone(), std::fs::read(&journal)?));
    let snapshot = ElectricalSelectionSnapshot::capture(&model)
        .map_err(|e| anyhow::anyhow!("capture: {e:?}"))?;
    let run = snapshot
        .board_run(&revision, &origin, None)
        .map_err(|e| anyhow::anyhow!("Board Run: {e:?}"))?;
    ensure!(
        run.origin == origin && run.board_net == board_net && run.members == expected_run,
        "Run must include A/B/C and both Vias, excluding disconnected and foreign Tracks"
    );
    ensure!(
        run.copper_layers == expected_layers
            && run.zone_copper.is_empty()
            && run.zone_regions.is_empty(),
        "complete layer authority without Zone qualifiers"
    );
    ensure!(
        run.contact_evidence.is_ok(),
        "foreign contact evidence must remain available"
    );
    let global = snapshot
        .global_net(&revision, logical)
        .map_err(|e| anyhow::anyhow!("Global Net: {e:?}"))?;
    ensure!(
        global.subject_id == logical && global.owned == expected_global,
        "Global Net includes disconnected Track and excludes foreign Track"
    );
    ensure!(
        !run.members.contains(&disconnected) && !global.owned.contains(&foreign),
        "independent disconnected and foreign controls"
    );
    let reopened = ProjectResolver::new(root).resolve()?;
    ensure!(
        reopened.model_revision == revision && model.model_revision == revision,
        "query/reopen revision invariance"
    );
    let reopened_snapshot = ElectricalSelectionSnapshot::capture(&reopened)
        .map_err(|e| anyhow::anyhow!("reopened capture: {e:?}"))?;
    let reopened_run = reopened_snapshot
        .board_run(&revision, &origin, None)
        .map_err(|e| anyhow::anyhow!("reopened Board Run: {e:?}"))?;
    let reopened_global = reopened_snapshot
        .global_net(&revision, logical)
        .map_err(|e| anyhow::anyhow!("reopened Global Net: {e:?}"))?;
    ensure!(
        reopened_run == run && reopened_global == global,
        "fresh resolver preserves complete query results"
    );
    for (path, bytes) in before {
        ensure!(
            std::fs::read(&path)? == bytes,
            "queries/reopen changed {}",
            path.display()
        );
    }
    let manifest = serde_json::json!({
        "schema_version":1,"family":"board-derived-hidden",
        "recipe":"existing canonical core plus two-layer Track/Via chain",
        "project_id":model.project.project_id,"board_id":board,"sheet_id":base["sheet_id"],
        "model_revision":revision,"logical_net_id":logical,"board_net_id":board_net,
        "construction_baseline":{"recipe":"recipe::build_with_project_group","project_group":"0x2a000000","text_count":1,"model_revision":base["model_revision"],"limit":"base observations precede the canonical topology changes; final observations below bind the new revision"},
        "geometry":{"tracks":tracks,"vias":vias,"layers":layers,"foreign_track":expected_tracks[&foreign.source_id]},
        "expected_run_members":expected_run.iter().map(gui_identity).collect::<Vec<_>>(),
        "expected_global_net_members":expected_global.iter().map(gui_identity).collect::<Vec<_>>(),
        "expected_run":{"kind":"run","origin":gui_identity(&origin),"scope":null,"focus":null},
        "expected_global_net":{"kind":"global_net","subject_id":logical,"scope":null,"focus":null},
        "visible_origin":{"identity":gui_identity(&origin),"layer":"L1","interior_point_nm":[5_000_000,0]},
        "prehidden_member":{"identity":gui_identity(&occurrence("tracks",tracks[1].uuid)),"layer":"L2","interior_point_nm":[15_000_000,0],"setup":"ordinary UI hide L2 before acquisition; no builder visibility mutation"},
        "downstream_member":{"identity":gui_identity(&occurrence("tracks",tracks[2].uuid)),"layer":"L1","interior_point_nm":[25_000_000,0]},
        "prefiltered_members":{"class":"via","identities":vias.iter().map(|v| gui_identity(&occurrence("vias",v.uuid))).collect::<Vec<_>>(),"setup":"ordinary Exclude Via before acquisition; no builder eligibility mutation"},
        "controls":{"disconnected_same_net":gui_identity(&disconnected),"foreign_net":base["foreign_net"],"foreign_track":gui_identity(&foreign),"zones":0,"unprojected_members_expected":0},
        "engine_observation":{"origin":run.origin,"board_net":run.board_net,"members":run.members,"copper_layers":run.copper_layers.iter().map(|(identity,layers)| serde_json::json!({"identity":identity,"layers":layers})).collect::<Vec<_>>(),"contact_evidence":run.contact_evidence,"global_net":global,"source_basis":snapshot.source_basis(),"reopen_full_result_equal":true,"source_journal_queries_unchanged":true},
        "rebuild":"python3 scripts/run_cargo_guarded.py --workload proof -- cargo run --locked -p eda-test-harness --bin s5a_fixture -- board-derived-hidden <fresh-owned-root>",
        "rebuild_limit":"Chosen identities and geometry repeat; canonical transactions are newly authored. Freeze actual files/history before native use.",
        "native_input":"pending actual camera/geometry, ordinary pre-hide/pre-filter input, binary, presentation and consumer binding",
        "acceptance":"not established by input preparation; no native hidden/filter/D4, lock, full C04/C05/C06 or successor credit"
    });
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(root.join("s5a-fixture.json"))
        .context("fresh fixture metadata")?;
    file.write_all(eda_test_harness::canonical_json(&manifest)?.as_bytes())?;
    println!("{}", root.join("s5a-fixture.json").display());
    Ok(())
}
