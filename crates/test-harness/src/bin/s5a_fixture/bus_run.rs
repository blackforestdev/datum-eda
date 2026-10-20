//! Visible-input instance of the existing uncapped Bus spine recipe, not replay evidence.
use anyhow::{Context, Result, ensure};
use eda_engine::{
    api::native_write::{self, BatchComposer, WriteProvenance, electrical},
    ir::geometry::Point,
    schematic::Bus,
    substrate::{
        CommitSource, ElectricalIdentity, ElectricalIdentityRecord, ElectricalOccurrence,
        ElectricalSelectionSnapshot, ObjectRevision, Operation, ProjectResolver,
    },
};
use std::{collections::BTreeSet, path::Path};
use uuid::Uuid;

const COUNT: usize = 257;
const STRIDE_NM: i64 = 1_000_000;
const Y_NM: i64 = 10_000_000;
const ORIGIN_INDEX: usize = 128;
const SUBTRACT_INDEX: usize = 129;
fn id(n: usize) -> Uuid {
    Uuid::from_u128(0x7b000000000040008000000000000000 + n as u128)
}
fn provenance() -> WriteProvenance {
    WriteProvenance::new(
        "s5a-bus-run-fixture",
        CommitSource::Test,
        "approved Bus257 topology with frozen native-visible geometry",
    )
}
fn occurrence(id: Uuid) -> ElectricalOccurrence {
    ElectricalOccurrence {
        class: "buses".into(),
        source_id: id,
        instance_path: vec![],
    }
}
fn gui_identity(id: Uuid) -> serde_json::Value {
    serde_json::json!({"class":"bus_section","id":id,"instance_path":[]})
}
pub(super) fn build(root: &Path) -> Result<()> {
    ensure!(
        !root.exists(),
        "fresh output required; prior artifacts preserved"
    );
    let base = super::recipe::build_with_project_group(root, 1, 0x27000000)?;
    let sheet: Uuid = serde_json::from_value(base["sheet_id"].clone())?;
    let mut model = ProjectResolver::new(root).resolve()?;
    // Choose every source identity and expected member before graph querying.
    let buses: Vec<_> = (0..COUNT)
        .map(|i| Bus {
            uuid: id(i + 1),
            name: "DATA".into(),
            members: vec!["DATA0".into()],
            segments: vec![
                Point::new(i as i64 * STRIDE_NM, Y_NM),
                Point::new((i as i64 + 1) * STRIDE_NM, Y_NM),
            ],
        })
        .collect();
    let expected: BTreeSet<_> = buses.iter().map(|b| occurrence(b.uuid)).collect();
    let expected_gui: Vec<_> = buses.iter().map(|b| gui_identity(b.uuid)).collect();
    let bus_id = id(0x2000);
    let origin = occurrence(buses[ORIGIN_INDEX].uuid);
    let subtract = occurrence(buses[SUBTRACT_INDEX].uuid);
    let write = BatchComposer::compose(&model, provenance())
        .push_ops(buses.iter().map(|b| Operation::CreateSchematicBus {
            sheet_id: sheet,
            bus_id: b.uuid,
            bus: serde_json::to_value(b).unwrap(),
        }))
        .finish()?;
    native_write::commit_prepared(&mut model, root, write)?;
    let record = ElectricalIdentityRecord {
        id: bus_id,
        object_revision: ObjectRevision(0),
        identity: ElectricalIdentity::Bus {
            name: "DATA".into(),
            representations: expected.clone(),
            scalar_nets: BTreeSet::new(),
            retired: false,
        },
    };
    let write = electrical::build_create_electrical_identity(&model, provenance(), record)?;
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
        .bus_run(snapshot.revision(), &origin)
        .map_err(|e| anyhow::anyhow!("Bus Run: {e:?}"))?;
    ensure!(
        run.membership.owned == expected,
        "complete independent Bus257 set"
    );
    ensure!(
        run.membership.subject_id == bus_id,
        "declared Bus authority"
    );
    ensure!(
        run.contacts.is_empty(),
        "chain has no foreign contact evidence"
    );
    ensure!(
        model.model_revision == revision,
        "queries preserve revision"
    );
    for (path, bytes) in before {
        ensure!(
            std::fs::read(path)? == bytes,
            "setup queries preserve source/journal"
        );
    }
    ensure!(
        ProjectResolver::new(root).resolve()?.model_revision == revision,
        "fresh resolver binds the same frozen source revision"
    );
    let manifest = serde_json::json!({
        "schema_version":1,
        "family":"bus-run-refusal",
        "recipe":"existing Bus257 topology; native-visible parameterization v1",
        "source_recipe":"crates/engine/src/api/native_write/electrical/bus_run_query_tests.rs::bus_run_complete_257_spines_and_full_integer_contacts",
        "source_recipe_sha256":"69814342e15beb6d14f0da98c64646ca0e911c3e2dcd03480dd1b4757fb94b94",
        "geometry":{"count":COUNT,"stride_nm":STRIDE_NM,"y_nm":Y_NM,"original_stride_nm":10,
            "note":"Explicit visible-input parameters; not byte-exact historical family4 reconstruction."},
        "base_project":base["project_id"],"sheet_id":sheet,"model_revision":revision,
        "declared_bus_id":bus_id,"expected_members":expected_gui,
        "expected_selection":{"kind":"run","origin":gui_identity(origin.source_id),"focus":null,"scope":null},
        "origin":{"index":ORIGIN_INDEX,"identity":gui_identity(origin.source_id),
            "interior_point_nm":{"x":(ORIGIN_INDEX as i64 * STRIDE_NM)+STRIDE_NM/2,"y":Y_NM}},
        "subtract":{"index":SUBTRACT_INDEX,"identity":gui_identity(subtract.source_id),
            "interior_point_nm":{"x":(SUBTRACT_INDEX as i64 * STRIDE_NM)+STRIDE_NM/2,"y":Y_NM}},
        "expected_refusal":"Cannot remove members from a run selection. Membership follows connectivity. Select an object without modifiers to start an explicit selection.",
        "engine_observation":{"complete_independent_set_equal":true,"member_count":run.membership.owned.len(),
            "semantic_bus_id":run.membership.subject_id,"source_basis":snapshot.source_basis()},
        "rebuild":"python3 scripts/run_cargo_guarded.py --workload proof -- cargo run -p eda-test-harness --bin s5a_fixture -- bus-run-refusal <fresh-owned-root>",
        "rebuild_limit":"Chosen semantic IDs and geometry are repeatable; canonical write transactions are newly authored. Freeze each actual input/history manifest before native execution, never claim historical byte identity.",
        "native_input":"pending actual source/binary/input/presented-event binding and independent review",
        "acceptance":"not established by setup; no basic cap replay/fullC04/HUMAN/successor claim"
    });
    std::fs::write(
        root.join("s5a-fixture.json"),
        eda_test_harness::canonical_json(&manifest)?,
    )
    .context("write fixture metadata, not authored design geometry")?;
    println!("{}", root.join("s5a-fixture.json").display());
    Ok(())
}
