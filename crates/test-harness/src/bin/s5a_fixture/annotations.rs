//! Real native no-connect and board-text producers extend the placed-parent
//! fixture. Child/source identities come from the existing CLI and model.
use anyhow::{Context, Result, ensure};
use eda_engine::substrate::{
    ElectricalSelectionSnapshot, ProjectResolver, transaction_journal_path,
};
use serde_json::{Value, json};
use std::{path::Path, process::Command};
fn invoke(cli: &Path, root: &Path, command: &str, args: &[String]) -> Result<Value> {
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
    Ok(serde_json::from_slice(&output.stdout)?)
}
pub(super) fn build(root: &Path, cli: &Path) -> Result<()> {
    super::parents::build(root, cli)?;
    let model = ProjectResolver::new(root).resolve()?;
    let authority = ElectricalSelectionSnapshot::capture(&model)
        .map_err(|e| anyhow::anyhow!("sources: {e:?}"))?;
    let schematic = authority.schematic_source().context("actual schematic")?;
    let sheet = schematic
        .sheets
        .values()
        .find(|s| !s.symbols.is_empty())
        .context("actual placed-symbol Sheet")?;
    let symbol = sheet
        .symbols
        .values()
        .min_by_key(|s| s.reference.clone())
        .context("actual Symbol")?;
    let pin = symbol
        .pins
        .iter()
        .min_by_key(|p| p.uuid)
        .context("actual instance Pin")?;
    let marker = invoke(
        cli,
        root,
        "place-noconnect",
        &[
            "--sheet".into(),
            sheet.uuid.to_string(),
            "--symbol".into(),
            symbol.uuid.to_string(),
            "--pin".into(),
            pin.uuid.to_string(),
            format!("--x-nm={}", pin.position.x),
            format!("--y-nm={}", pin.position.y),
        ],
    )?;
    let text = invoke(
        cli,
        root,
        "place-board-text",
        &[
            "--text".into(),
            "Native board note".into(),
            "--x-nm".into(),
            "30000000".into(),
            "--y-nm".into(),
            "10000000".into(),
            "--rotation-deg".into(),
            "30".into(),
            "--layer".into(),
            "3".into(),
        ],
    )?;
    let reopened = ProjectResolver::new(root).resolve()?;
    let sources: Vec<_> = reopened
        .source_shards
        .iter()
        .map(|s| Ok((s.path.clone(), std::fs::read(&s.path)?)))
        .collect::<Result<_>>()?;
    let journal = transaction_journal_path(root);
    let journal_bytes = std::fs::read(&journal)?;
    let snapshot = ElectricalSelectionSnapshot::capture(&reopened)
        .map_err(|e| anyhow::anyhow!("capture: {e:?}"))?;
    let schematic = snapshot.schematic_source().context("actual schematic")?;
    let markers: Vec<_> = schematic
        .sheets
        .values()
        .flat_map(|s| s.noconnects.values())
        .collect();
    ensure!(markers.len() == 1, "one actual no-connect marker");
    let actual = markers[0];
    ensure!(
        actual.symbol == symbol.uuid && actual.pin == pin.uuid && actual.position == pin.position,
        "actual owning instance Pin, not library or scene identity"
    );
    let authored = snapshot
        .authored_sources()
        .map_err(|e| anyhow::anyhow!("authored: {e:?}"))?;
    let markers: Vec<_> = authored
        .iter()
        .filter(|s| s.occurrence.class == "noconnects")
        .collect();
    let texts: Vec<_> = authored
        .iter()
        .filter(|s| {
            s.occurrence.class == "texts"
                && s.domain == eda_engine::substrate::SelectionSourceDomain::Board
        })
        .collect();
    ensure!(
        markers.len() == 1 && texts.len() == 1,
        "complete marker/text inventory"
    );
    for (path, before) in sources {
        ensure!(
            std::fs::read(path)? == before,
            "source unchanged around inspection"
        );
    }
    ensure!(
        std::fs::read(&journal)? == journal_bytes,
        "journal unchanged around inspection"
    );
    let mut report: Value = serde_json::from_slice(&std::fs::read(root.join("s5a-fixture.json"))?)?;
    report["annotations"] = json!({"project_id":reopened.project.project_id,"model_revision":reopened.model_revision.0,
        "marker":markers[0],"board_text":texts[0],"actual_marker":actual,"cli_receipts":[marker,text],
        "nonmutation":"complete source shards and full journal compared around snapshot/inspection/reopen",
        "native_input":"pending marker object-only acquisition, strict text majority and shared compound inspection replay",
        "acceptance":"partial source readiness; component-Pad/graphics positives and C04/C05/C06 remain pending"});
    std::fs::write(
        root.join("s5a-fixture.json"),
        eda_test_harness::canonical_json(&report)?,
    )?;
    println!("{}", root.join("s5a-fixture.json").display());
    Ok(())
}
