//! Persisted lifecycle variants use the same canonical write/undo path as editors.
use anyhow::{Result, ensure};
use eda_engine::{
    api::native_write::{self, WriteProvenance},
    substrate::{CommitSource, ElectricalSelectionSnapshot, ProjectResolver},
};
use std::path::{Path, PathBuf};
fn provenance() -> WriteProvenance {
    WriteProvenance::new(
        "s5a-lifetime-fixture",
        CommitSource::Test,
        "canonical revision variant",
    )
}
fn snapshot(model: &eda_engine::substrate::DesignModel) -> Result<ElectricalSelectionSnapshot> {
    ElectricalSelectionSnapshot::capture(model)
        .map_err(|e| anyhow::anyhow!("lifetime capture: {e:?}"))
}
pub(super) fn copy_project(source: &Path, target: &Path) -> Result<()> {
    ensure!(!target.exists(), "fresh variant required");
    let model = ProjectResolver::new(source).resolve()?;
    let files = model
        .source_shards
        .iter()
        .map(|s| s.path.clone())
        .chain([eda_engine::substrate::transaction_journal_path(source)]);
    for from in files {
        let to = target.join(from.strip_prefix(source)?);
        std::fs::create_dir_all(to.parent().unwrap())?;
        std::fs::copy(from, to)?;
    }
    Ok(())
}
pub(super) fn build(root: &Path) -> Result<Vec<serde_json::Value>> {
    let initial = ProjectResolver::new(root).resolve()?;
    let basis = snapshot(&initial)?;
    let sheet = basis
        .schematic_source()
        .unwrap()
        .sheets
        .values()
        .next()
        .unwrap();
    ensure!(sheet.texts.len() == 1, "one explicit lifetime origin");
    let original = sheet.texts.values().next().unwrap().clone();
    let mut reports = Vec::new();
    for name in ["same-identity", "deleted", "undo", "replacement"] {
        let path = root.join("variants").join(name);
        copy_project(root, &path)?;
        let mut model = ProjectResolver::new(&path).resolve()?;
        let expected = match name {
            "same-identity" => {
                let mut text = original.clone();
                text.rotation = 30;
                let write = native_write::schematic_sheets::build_set_schematic_text(
                    &model,
                    provenance(),
                    sheet.uuid,
                    &text,
                )?;
                native_write::commit_prepared(&mut model, &path, write)?;
                Some(original.uuid)
            }
            _ => {
                let write = native_write::schematic_sheets::build_delete_schematic_text(
                    &model,
                    provenance(),
                    sheet.uuid,
                    &original,
                )?;
                native_write::commit_prepared(&mut model, &path, write)?;
                match name {
                    "undo" => {
                        model.commit_journal_undo(&path, provenance().into())?;
                        Some(original.uuid)
                    }
                    "replacement" => {
                        let mut text = original.clone();
                        text.uuid = uuid::Uuid::from_u128(0x70000000000040008000000000000001);
                        let write = native_write::schematic_sheets::build_create_schematic_text(
                            &model,
                            provenance(),
                            sheet.uuid,
                            &text,
                        )?;
                        native_write::commit_prepared(&mut model, &path, write)?;
                        Some(text.uuid)
                    }
                    _ => None,
                }
            }
        };
        model = ProjectResolver::new(&path).resolve()?;
        let mut files: Vec<(PathBuf, Vec<u8>)> = model
            .source_shards
            .iter()
            .map(|s| Ok((s.path.clone(), std::fs::read(&s.path)?)))
            .collect::<Result<_>>()?;
        let journal = eda_engine::substrate::transaction_journal_path(&path);
        files.push((journal.clone(), std::fs::read(journal)?));
        let current = snapshot(&model)?;
        let text_ids: std::collections::BTreeSet<_> = current.schematic_source().unwrap().sheets
            [&sheet.uuid]
            .texts
            .keys()
            .copied()
            .collect();
        ensure!(
            text_ids == expected.into_iter().collect(),
            "exact lifecycle identity oracle"
        );
        if name == "undo" {
            ensure!(
                model.model_revision == initial.model_revision,
                "undo restores content-derived revision"
            );
            ensure!(
                model.journal.len() == initial.journal.len() + 2,
                "undo remains explicit journal history"
            );
        } else {
            ensure!(
                model.model_revision != initial.model_revision,
                "changed content changes revision"
            );
        }
        for (file, before) in files {
            ensure!(
                before == std::fs::read(file)?,
                "lifetime query changed authored bytes"
            );
        }
        reports.push(serde_json::json!({"variant": name, "relative_root": path.strip_prefix(root)?.to_string_lossy(), "previous_id": original.uuid, "expected_id": expected, "model_revision": model.model_revision, "source_basis": current.source_basis(), "native_selection_lifecycle": "pending actual invocation; engine identity/reopen oracle only"}));
    }
    Ok(reports)
}
