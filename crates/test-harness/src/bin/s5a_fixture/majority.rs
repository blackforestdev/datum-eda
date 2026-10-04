//! Exact fourteen-pin and pinless sources use existing library/placement owners.
use anyhow::{Result, ensure};
use eda_engine::{
    api::native_write::{
        self, WriteProvenance,
        library::{PoolLibraryObjectTarget, PoolLibraryOperationSpec, build_pool_library_write},
    },
    substrate::{CommitSource, ElectricalSelectionSnapshot, ProjectResolver},
};
use serde_json::{Value, json};
use std::{path::Path, process::Command};
use uuid::Uuid;
fn id(n: u128) -> Uuid {
    Uuid::from_u128(0x75000000000040008000000000000000 + n)
}
pub(super) fn build(root: &Path, cli: &Path) -> Result<()> {
    ensure!(cli.is_file(), "existing pinned CLI required");
    super::library::prepare(root)?;
    let mut model = ProjectResolver::new(root).resolve()?;
    let mut definitions = vec![];
    for (unit, symbol, count) in [(id(1), id(2), 14), (id(3), id(4), 0)] {
        let mut pins = serde_json::Map::new();
        let mut anchors = vec![];
        for index in 0..count {
            let pin = id(100 + index);
            pins.insert(pin.to_string(), json!({"uuid":pin,"name":format!("P{index}"),"direction":"Passive","swap_group":0,"alternates":[]}));
            anchors.push(json!({"pin":pin,"position":{"x":index as i64 * 1_000_000,"y":0},"style":{"orientation":"Down","length_nm":500000,"decoration":"none"}}));
        }
        for (kind, uuid, object) in [
            (
                "units",
                unit,
                json!({"schema_version":1,"uuid":unit,"name":format!("S5A {count} pin fixture"),"manufacturer":"Datum","pins":pins,"tags":["s5a-native-fixture"]}),
            ),
            (
                "symbols",
                symbol,
                json!({"schema_version":1,"uuid":symbol,"name":format!("S5A {count} pin symbol"),"unit":unit,"default_refdes_prefix":"U","drawings":[{"Line":{"from":{"x":-500000,"y":-500000},"to":{"x":500000,"y":500000},"width":100000}}],"pin_anchors":anchors,"tags":["s5a-native-fixture"]}),
            ),
        ] {
            definitions.push(PoolLibraryOperationSpec::Create {
                target: PoolLibraryObjectTarget::new("pool", kind, uuid),
                object,
            });
        }
    }
    let write = build_pool_library_write(
        &model,
        WriteProvenance::new(
            "s5a-fixture",
            CommitSource::Test,
            "finite fourteen-pin and pinless qualification sources",
        ),
        Some("pool"),
        definitions,
    )?;
    native_write::commit_prepared(&mut model, root, write)?;
    let mut receipts = vec![];
    for (symbol, reference, x) in [(id(2), "U14", "0"), (id(4), "U0", "30000000")] {
        let output = Command::new(cli)
            .args(["--format", "json", "project", "place-symbol"])
            .arg(root)
            .args([
                "--sheet",
                "10000000-0000-4000-8000-000000000004",
                "--reference",
                reference,
                "--value",
                "majority fixture",
                "--lib-id",
                &symbol.to_string(),
                "--x-nm",
                x,
                "--y-nm",
                "0",
            ])
            .output()?;
        ensure!(
            output.status.success(),
            "actual Symbol materializer: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        receipts.push(serde_json::from_slice::<Value>(&output.stdout)?);
    }
    let reopened = ProjectResolver::new(root).resolve()?;
    let snapshot = ElectricalSelectionSnapshot::capture(&reopened)
        .map_err(|e| anyhow::anyhow!("snapshot: {e:?}"))?;
    let schematic = snapshot
        .schematic_source()
        .ok_or_else(|| anyhow::anyhow!("native schematic"))?;
    let mut subjects = vec![];
    for (receipt, count) in receipts.iter().zip([14, 0]) {
        let uuid: Uuid = receipt["symbol_uuid"].as_str().unwrap().parse()?;
        let symbol = schematic
            .sheets
            .values()
            .find_map(|s| s.symbols.get(&uuid))
            .ok_or_else(|| anyhow::anyhow!("actual placed Symbol"))?;
        ensure!(
            symbol.pins.len() == count,
            "complete materialized pin count"
        );
        let mut anchors: Vec<_> = symbol
            .pins
            .iter()
            .map(|p| json!({"source_id":p.uuid,"library_pin":p.library_pin,"position":p.position}))
            .collect();
        anchors.sort_by_key(|p| p["position"]["x"].as_i64());
        for (index, pin) in anchors.iter().enumerate() {
            ensure!(
                pin["library_pin"] == json!(id(100 + index as u128))
                    && pin["position"] == json!({"x":index as i64*1_000_000,"y":0}),
                "independent complete library-anchor oracle"
            );
        }
        subjects.push(json!({"source_id":uuid,"pin_count":count,"position":symbol.position,"anchors":anchors}));
    }
    let report = json!({"family":"symbol-majority","project_id":reopened.project.project_id,"model_revision":reopened.model_revision.0,"subjects":subjects,"producer_receipts":receipts,"oracles":{"fourteen_pin":{"seven_max_x_nm":6500000,"eight_max_x_nm":7500000,"min_x_nm":-1000000,"min_y_nm":-1000000,"max_y_nm":1000000},"pinless_anchor":{"x":30000000,"y":0}},"unverified":["actual native region input and visual/accessibility","component-Pad/Footprint majority remains blocked separately","C04/C05/C06 acceptance"]});
    std::fs::write(
        root.join("s5a-fixture.json"),
        eda_test_harness::canonical_json(&report)?,
    )?;
    println!("{}", root.join("s5a-fixture.json").display());
    Ok(())
}
