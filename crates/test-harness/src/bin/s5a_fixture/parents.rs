//! Actual CLI materializers supply placed identities; the recipe never mints children.
use anyhow::{Context, Result, ensure};
use eda_engine::substrate::{
    ElectricalOccurrence, ElectricalSelectionSnapshot, ProjectResolver, SourceShardKind,
};
use serde_json::{Value, json};
use std::{collections::BTreeSet, path::Path, process::Command};
use uuid::Uuid;
fn occurrence(class: &str, id: Uuid) -> ElectricalOccurrence {
    ElectricalOccurrence {
        class: class.into(),
        source_id: id,
        instance_path: vec![],
    }
}
fn execute(
    cli: &Path,
    root: &Path,
    command: &str,
    args: &[&str],
    receipts: &mut Vec<Value>,
) -> Result<Value> {
    let output = Command::new(cli)
        .args(["--format", "json", "project", command])
        .arg(root)
        .args(args)
        .output()?;
    ensure!(
        output.status.success(),
        "{command}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let result: Value = serde_json::from_slice(&output.stdout)?;
    receipts.push(json!({"command":command,"arguments":args,"result":result}));
    Ok(result)
}
fn field(value: &Value, key: &str) -> Result<Uuid> {
    Ok(Uuid::parse_str(
        value[key]
            .as_str()
            .with_context(|| format!("actual CLI {key}"))?,
    )?)
}
pub(super) fn build(root: &Path, cli: &Path) -> Result<()> {
    ensure!(
        cli.is_file(),
        "pin an existing CLI binary; this recipe never builds it"
    );
    super::library::prepare(root)?;
    let mut receipts = vec![];
    let mut placements = vec![];
    for (reference, x, rotation, locked) in
        [("U1", "0", "0", true), ("U2", "20000000", "30", false)]
    {
        let symbol = execute(
            cli,
            root,
            "place-symbol",
            &[
                "--sheet",
                "10000000-0000-4000-8000-000000000004",
                "--reference",
                reference,
                "--value",
                "native fixture",
                "--lib-id",
                "30000000-0000-4000-8000-000000000001",
                "--x-nm",
                x,
                "--y-nm",
                "0",
                "--rotation-deg",
                rotation,
            ],
            &mut receipts,
        )?;
        let package = execute(
            cli,
            root,
            "place-board-component",
            &[
                "--part",
                "90000000-0000-4000-8000-000000000001",
                "--package",
                "50000000-0000-4000-8000-000000000001",
                "--reference",
                reference,
                "--value",
                "native fixture",
                "--x-nm",
                x,
                "--y-nm",
                "0",
                "--layer",
                "1",
            ],
            &mut receipts,
        )?;
        let symbol_id = field(&symbol, "symbol_uuid")?;
        let package_id = field(&package, "component_uuid")?;
        let instance_id = field(&symbol, "component_instance_uuid")?;
        if rotation != "0" {
            execute(
                cli,
                root,
                "rotate-board-component",
                &[
                    "--component",
                    &package_id.to_string(),
                    "--rotation-deg",
                    rotation,
                ],
                &mut receipts,
            )?;
        }
        if locked {
            execute(
                cli,
                root,
                "set-board-component-locked",
                &["--component", &package_id.to_string()],
                &mut receipts,
            )?;
        }
        execute(
            cli,
            root,
            "set-component-instance",
            &[
                "--component-instance",
                &instance_id.to_string(),
                "--symbol",
                &symbol_id.to_string(),
                "--package",
                &package_id.to_string(),
                "--part",
                "90000000-0000-4000-8000-000000000001",
            ],
            &mut receipts,
        )?;
        placements.push((symbol_id, package_id, instance_id, locked));
    }
    let model = ProjectResolver::new(root).resolve()?;
    let mut before = model
        .source_shards
        .iter()
        .map(|s| Ok((s.path.clone(), std::fs::read(&s.path)?)))
        .collect::<Result<Vec<_>>>()?;
    let journal = eda_engine::substrate::transaction_journal_path(root);
    before.push((journal.clone(), std::fs::read(&journal)?));
    let snapshot = ElectricalSelectionSnapshot::capture(&model)
        .map_err(|e| anyhow::anyhow!("capture: {e:?}"))?;
    let inventory = snapshot
        .authored_sources()
        .map_err(|e| anyhow::anyhow!("sources: {e:?}"))?;
    let actual: BTreeSet<_> = inventory.into_iter().map(|s| s.occurrence).collect();
    let schematic = snapshot.schematic_source().context("actual schematic")?;
    let board = model.materialized_source_shard_value(SourceShardKind::BoardRoot)?;
    let pads = board["component_pads"]
        .as_object()
        .context("actual component materialization")?;
    let expected_library_pins: BTreeSet<Uuid> = [0x101, 0x102, 0x103, 0x104, 0x106]
        .into_iter()
        .map(|n| Uuid::from_u128(0x20000000000040008000000000000000 + n))
        .collect();
    let mut oracles = vec![];
    let mut expected = BTreeSet::new();
    for (symbol_id, package_id, instance_id, locked) in placements {
        let symbol = schematic
            .sheets
            .values()
            .find_map(|s| s.symbols.get(&symbol_id))
            .context("actual placed Symbol")?;
        let pin_ids: BTreeSet<_> = symbol
            .pins
            .iter()
            .map(|p| occurrence("pins", p.uuid))
            .collect();
        ensure!(
            symbol.pins.len() == 5
                && symbol
                    .pins
                    .iter()
                    .filter_map(|p| p.library_pin)
                    .collect::<BTreeSet<_>>()
                    == expected_library_pins,
            "complete five source-library Pins materialized"
        );
        let package = &board["packages"][package_id.to_string()];
        ensure!(package["locked"] == locked, "actual Package lock state");
        let selected_pads: Vec<_> = pads[&package_id.to_string()]
            .as_array()
            .context("actual package Pad payload")?
            .iter()
            .collect();
        ensure!(
            selected_pads.len() == 8,
            "all eight footprint Pads materialized"
        );
        ensure!(
            selected_pads
                .iter()
                .map(|p| p["name"].as_str().unwrap())
                .collect::<BTreeSet<_>>()
                == BTreeSet::from(["1", "2", "3", "4", "5", "6", "7", "8"]),
            "complete independent pad names"
        );
        let library_pad_ids = selected_pads
            .iter()
            .map(|p| field(p, "uuid"))
            .collect::<Result<BTreeSet<_>>>()?;
        ensure!(
            library_pad_ids
                == (1..=8)
                    .map(|n| Uuid::from_u128(0x80000000000040008000000000000000 + n))
                    .collect(),
            "persisted component payload still uses library Pad UUIDs"
        );
        ensure!(
            library_pad_ids
                .iter()
                .all(|id| !actual.contains(&occurrence("pads", *id))),
            "missing component Pad identity must remain explicit"
        );
        let parent = occurrence("symbols", symbol_id);
        let related = snapshot
            .related_parents(&model.model_revision, &pin_ids)
            .map_err(|e| anyhow::anyhow!("parent query: {e:?}"))?;
        ensure!(
            related == BTreeSet::from([parent.clone()]),
            "exact Pin source parent only; no sibling expansion"
        );
        expected.insert(parent);
        expected.insert(occurrence("packages", package_id));
        expected.extend(pin_ids.iter().cloned());
        let instance = model
            .component_instances
            .get(&instance_id)
            .context("actual ComponentInstance")?;
        ensure!(
            instance.placed_symbol_refs == vec![symbol_id]
                && instance.placed_package_refs == vec![package_id],
            "actual cross-editor parent correspondence"
        );
        oracles.push(json!({"symbol":symbol_id,"package":package_id,"component_instance":instance_id,
            "locked":locked,"rotation_deg":symbol.rotation,"pins":symbol.pins,"pads":selected_pads,
            "expected_pin_members":pin_ids,"unqualified_component_library_pad_ids":library_pad_ids,
            "parent_queries":"exact single source parent; children retained separately; no sibling membership"}));
    }
    ensure!(
        expected.is_subset(&actual) && expected.len() == 14,
        "all four parents and ten authored instance Pins present"
    );
    for (path, bytes) in &before {
        ensure!(
            std::fs::read(path)? == *bytes,
            "read-only source/journal changed: {}",
            path.display()
        );
    }
    let reopened = ProjectResolver::new(root).resolve()?;
    ensure!(
        reopened.model_revision == model.model_revision,
        "reopen source basis"
    );
    let report = json!({"schema_version":1,"recipe":"s5a-library-parents-v1","project_id":model.project.project_id,
        "model_revision":model.model_revision,"source_basis":snapshot.source_basis(),"expected_authored_members":expected,
        "placements":oracles,"cli_receipts":receipts,"nonmutation":"all source shards and full journal byte-equal around read-only queries",
        "native_input":"pending: parent acquisition, projections, locked disclosure and inspection",
        "component_pad_authority":"unavailable: library UUIDs repeat across Packages in component_pads; absent from canonical pads class. Positive component-Pad membership/corner proofs remain pending; no IDs fabricated",
        "acceptance":"partial C03 fixture preparation; no C04/C05/C06 or graphics/manufacturing acceptance"});
    std::fs::write(
        root.join("s5a-fixture.json"),
        eda_test_harness::canonical_json(&report)?,
    )?;
    println!("{}", root.join("s5a-fixture.json").display());
    Ok(())
}
