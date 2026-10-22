//! Canonical large Wire/Junction Run input; visibility remains an ordinary UI action.
use anyhow::{Context, Result, ensure};
use eda_engine::{
    api::native_write::{self, BatchComposer, WriteProvenance, electrical},
    ir::geometry::Point,
    schematic::{Junction, SchematicWire},
    substrate::{
        CommitSource, ElectricalIdentity, ElectricalIdentityRecord, ElectricalOccurrence,
        ElectricalSelectionSnapshot, NetAnchorReason, ObjectRevision, Operation, ProjectResolver,
    },
};
use std::{collections::BTreeSet, path::Path};
use uuid::Uuid;

const STRIDE_NM: i64 = 1_000_000;
const Y_NM: i64 = 10_000_000;
const ORIGIN_INDEX: usize = 127;
const MEMBER_INDEX: usize = 128;
fn id(n: usize) -> Uuid {
    Uuid::from_u128(0x7c000000000040008000000000000000 + n as u128)
}
fn provenance(wire_count: usize) -> WriteProvenance {
    WriteProvenance::new(
        "s5a-wire-run-hidden-fixture",
        CommitSource::Test,
        format!(
            "approved {wire_count}-Wire chain with one intermediate Junction; UI pre-hide pending"
        ),
    )
}
fn occurrence(class: &str, source_id: Uuid) -> ElectricalOccurrence {
    ElectricalOccurrence {
        class: class.into(),
        source_id,
        instance_path: vec![],
    }
}
fn gui_identity(class: &str, source_id: Uuid) -> serde_json::Value {
    serde_json::json!({"class":class,"id":source_id,"instance_path":[]})
}
pub(super) fn build(root: &Path, total_members: usize) -> Result<()> {
    ensure!(
        matches!(total_members, 256 | 257),
        "Wire/Junction total must be 256 or 257"
    );
    let wire_count = total_members - 1;
    ensure!(
        !root.exists() && !root.is_symlink(),
        "fresh output required; prior artifacts preserved"
    );
    // Declare the full topology, membership and semantic owner before any query.
    let wires: Vec<_> = (0..wire_count)
        .map(|i| SchematicWire {
            uuid: id(i + 1),
            from: Point::new(i as i64 * STRIDE_NM, Y_NM),
            to: Point::new((i as i64 + 1) * STRIDE_NM, Y_NM),
        })
        .collect();
    let junction = Junction {
        uuid: id(0x1000),
        position: Point::new(MEMBER_INDEX as i64 * STRIDE_NM, Y_NM),
    };
    let mut expected: BTreeSet<_> = wires.iter().map(|w| occurrence("wires", w.uuid)).collect();
    expected.insert(occurrence("junctions", junction.uuid));
    let mut expected_gui: Vec<_> = wires.iter().map(|w| gui_identity("wire", w.uuid)).collect();
    expected_gui.push(gui_identity("junction", junction.uuid));
    let origin = occurrence("wires", wires[ORIGIN_INDEX].uuid);
    let member = occurrence("wires", wires[MEMBER_INDEX].uuid);
    let net_id = id(0x2000);
    let record = ElectricalIdentityRecord {
        id: net_id,
        object_revision: ObjectRevision(0),
        identity: ElectricalIdentity::Net {
            anchor: origin.clone(),
            anchor_reason: NetAnchorReason::Authored,
            retired: false,
            predecessors: BTreeSet::new(),
        },
    };
    ensure!(
        expected.len() == total_members,
        "independent complete {total_members}-member oracle"
    );
    let group = if total_members == 256 {
        0x29000000
    } else {
        0x28000000
    };
    let base = super::recipe::build_with_project_group(root, 1, group)?;
    let sheet: Uuid = serde_json::from_value(base["sheet_id"].clone())?;
    let mut model = ProjectResolver::new(root).resolve()?;
    for source_id in expected.iter().map(|o| o.source_id).chain([net_id]) {
        ensure!(
            !model.objects.contains_key(&source_id)
                && !model.electrical_identities.contains_key(&source_id),
            "chosen fixture identity collides with existing base: {source_id}"
        );
    }
    let mut ops: Vec<_> = wires
        .iter()
        .map(|wire| {
            Ok(Operation::CreateSchematicWire {
                sheet_id: sheet,
                wire_id: wire.uuid,
                wire: serde_json::to_value(wire)?,
            })
        })
        .collect::<Result<_>>()?;
    ops.push(Operation::CreateSchematicJunction {
        sheet_id: sheet,
        junction_id: junction.uuid,
        junction: serde_json::to_value(&junction)?,
    });
    let write = BatchComposer::compose(&model, provenance(wire_count))
        .push_ops(ops)
        .finish()?;
    native_write::commit_prepared(&mut model, root, write)?;
    let write =
        electrical::build_create_electrical_identity(&model, provenance(wire_count), record)?;
    native_write::commit_prepared(&mut model, root, write)?;
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
        .schematic_run(&revision, &origin)
        .map_err(|e| anyhow::anyhow!("Wire Run: {e:?}"))?;
    ensure!(
        run.owned == expected,
        "complete independent Wire/Junction set"
    );
    ensure!(
        run.subject_id == net_id,
        "predeclared logical Net authority"
    );
    ensure!(
        model.model_revision == revision,
        "queries preserve revision"
    );
    let reopened = ProjectResolver::new(root).resolve()?;
    ensure!(
        reopened.model_revision == revision,
        "fresh resolver binds frozen revision"
    );
    let reopened_snapshot = ElectricalSelectionSnapshot::capture(&reopened)
        .map_err(|e| anyhow::anyhow!("reopened capture: {e:?}"))?;
    let reopened_run = reopened_snapshot
        .schematic_run(&revision, &origin)
        .map_err(|e| anyhow::anyhow!("reopened Wire Run: {e:?}"))?;
    ensure!(
        reopened_run.owned == expected && reopened_run.subject_id == net_id,
        "reopened Run matches independent complete membership and Net"
    );
    for (path, bytes) in before {
        ensure!(
            std::fs::read(path)? == bytes,
            "queries and reopen preserve source/journal"
        );
    }
    let manifest = serde_json::json!({
        "schema_version":1,"family":format!("wire-run-hidden-{total_members}"),
        "recipe":"existing canonical Wire/Junction topology; large pre-hidden input v1",
        "source_recipes":["crates/test-harness/src/bin/s5a_fixture/semantic.rs",
            "crates/engine/src/api/native_write/electrical/tests.rs::net"],
        "geometry":{"wire_count":wire_count,"junction_count":1,"stride_nm":STRIDE_NM,
            "y_nm":Y_NM,"junction_position_nm":{"x":128_000_000,"y":Y_NM}},
        "base_project":base["project_id"],"sheet_id":sheet,"model_revision":revision,
        "declared_net_id":net_id,"expected_members":expected_gui,
        "expected_selection":{"kind":"run","origin":gui_identity("wire",origin.source_id),
            "focus":null,"scope":null},
        "origin":{"index":ORIGIN_INDEX,"identity":gui_identity("wire",origin.source_id),
            "interior_point_nm":{"x":127_500_000,"y":Y_NM}},
        "member_noop":{"index":MEMBER_INDEX,"identity":gui_identity("wire",member.source_id),
            "interior_point_nm":{"x":128_500_000,"y":Y_NM}},
        "prehidden_member":{"identity":gui_identity("junction",junction.uuid),"layer":"L202",
            "setup":"ordinary UI hide before Run acquisition; builder does not hide geometry",
            "expected_hidden_count":1,"expected_unprojected_count":0},
        "engine_observation":{"complete_independent_set_equal":true,"member_count":run.owned.len(),
            "semantic_net_id":run.subject_id,"source_basis":snapshot.source_basis(),
            "reopened_membership_and_net_equal":true,"source_journal_queries_unchanged":true},
        "rebuild":format!("python3 scripts/run_cargo_guarded.py --workload proof -- cargo run --locked -p eda-test-harness --bin s5a_fixture -- wire-run-hidden-{total_members} <fresh-owned-root>"),
        "rebuild_limit":"Chosen IDs/geometry repeat; canonical transactions are newly authored. Freeze each actual source/history input before native execution.",
        "native_input":"pending actual source/binary/input/presented-event binding and independent review",
        "acceptance":"not established by setup; no pre-filtered/omitted-reason/fullC04/C05/HUMAN/successor claim"
    });
    std::fs::write(
        root.join("s5a-fixture.json"),
        eda_test_harness::canonical_json(&manifest)?,
    )
    .context("write fixture metadata, not authored design geometry")?;
    println!("{}", root.join("s5a-fixture.json").display());
    Ok(())
}
