//! Qualified CLI adapter for the shared native body alignment builder.
use super::*;
impl ProjectAlignSymbolsArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        use eda_engine::{
            api::native_write::{
                WriteProvenance, commit_prepared,
                registry::{NativeWriteContext, find_native_write_verb},
            },
            substrate::ProjectResolver,
        };
        anyhow::ensure!(
            !self.expected_model_revision.trim().is_empty(),
            "expected-model-revision must be nonempty"
        );
        let expected_tip = match self.expected_tip_transaction.as_str() {
            "none" => None,
            value => Some(
                Uuid::parse_str(value)
                    .context("expected-tip-transaction must be a UUID or none")?,
            ),
        };
        let mut model = ProjectResolver::new(&self.path).resolve()?;
        anyhow::ensure!(
            model.model_revision.0 == self.expected_model_revision,
            "model revision mismatch: stale Symbol alignment"
        );
        anyhow::ensure!(
            eda_engine::revision::transaction_tip(&model.journal) == expected_tip,
            "transaction tip mismatch: stale Symbol alignment"
        );
        let builder = find_native_write_verb("datum.schematic.align_symbols")
            .context("Native alignment unavailable")?;
        let prepared = (builder.build)(
            &NativeWriteContext {
                model: &model,
                project_root: &self.path,
            },
            WriteProvenance::new("datum-eda-cli", cli_commit_source()?, "align Symbol bodies"),
            serde_json::json!({"symbols":self.symbols,"instance_path":self.instance_path,
                "op":self.op,"axis":self.axis,"position":self.position,"reference":self.reference}),
        )?;
        let report = commit_prepared(&mut model, &self.path, prepared)?;
        let output = render_report(format, &report, |report| {
            format!(
                "Aligned Symbol bodies\nRevision: {}\nTransaction: {}",
                report.transaction.after_model_revision.0, report.transaction.transaction_id
            )
        });
        Ok((output, 0))
    }
}
