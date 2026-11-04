//! Qualified CLI field edits use the same native builder as the designer UI.
use super::*;
use eda_engine::{
    api::native_write::{
        WriteProvenance, commit_prepared,
        registry::{NativeWriteContext, find_native_write_verb},
    },
    substrate::ProjectResolver,
};

impl ProjectSetSymbolFieldArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        anyhow::ensure!(
            !self.expected_model_revision.trim().is_empty(),
            "expected-model-revision must be nonempty"
        );
        let expected_tip = match self.expected_tip_transaction.as_str() {
            "none" => None,
            text => Some(
                Uuid::parse_str(text).context("expected-tip-transaction must be a UUID or none")?,
            ),
        };
        let mut model = ProjectResolver::new(&self.path).resolve()?;
        anyhow::ensure!(
            model.model_revision.0 == self.expected_model_revision,
            "model revision mismatch: stale symbol field intent"
        );
        anyhow::ensure!(
            eda_engine::revision::transaction_tip(&model.journal) == expected_tip,
            "transaction tip mismatch: stale symbol field intent"
        );
        let builder = find_native_write_verb("datum.schematic.set_symbol_field")
            .context("Native symbol field editor unavailable")?;
        let prepared = (builder.build)(
            &NativeWriteContext {
                model: &model,
                project_root: &self.path,
            },
            WriteProvenance::new(
                "datum-eda-cli",
                cli_commit_source()?,
                "edit schematic symbol field",
            ),
            serde_json::json!({"symbol":self.symbol,"instance_path":self.instance_path,
                "field":self.field,"value":self.value,"clear":self.clear}),
        )?;
        // The canonical commit checks this exact snapshot against fresh persisted
        // revision, Project identity and journal tip while holding the writer lease.
        let report = commit_prepared(&mut model, &self.path, prepared)?;
        let output = render_report(format, &report, |report| {
            format!(
                "Updated Symbol {}\nRevision: {}\nTransaction: {}",
                self.symbol,
                report.transaction.after_model_revision.0,
                report.transaction.transaction_id
            )
        });
        Ok((output, 0))
    }
}
