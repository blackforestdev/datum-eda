use eda_engine::import::ImportReport;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub(crate) struct ImportReportView {
    pub(crate) kind: &'static str,
    pub(crate) source: String,
    pub(crate) counts: ImportCountsView,
    pub(crate) warnings: Vec<String>,
    pub(crate) metadata: std::collections::BTreeMap<String, String>,
}

#[derive(Debug, Serialize)]
pub(crate) struct ImportCountsView {
    pub(crate) units: usize,
    pub(crate) symbols: usize,
    pub(crate) entities: usize,
    pub(crate) padstacks: usize,
    pub(crate) packages: usize,
    pub(crate) parts: usize,
}

impl From<ImportReport> for ImportReportView {
    fn from(report: ImportReport) -> Self {
        Self {
            kind: report.kind.as_str(),
            source: report.source.display().to_string(),
            counts: ImportCountsView {
                units: report.counts.units,
                symbols: report.counts.symbols,
                entities: report.counts.entities,
                padstacks: report.counts.padstacks,
                packages: report.counts.packages,
                parts: report.counts.parts,
            },
            warnings: report.warnings,
            metadata: report.metadata,
        }
    }
}
