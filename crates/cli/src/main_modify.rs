use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ModifyReportView {
    pub(crate) actions: Vec<String>,
    pub(crate) last_result: Option<eda_engine::api::OperationResult>,
    pub(crate) saved_path: Option<String>,
    pub(crate) applied_scoped_replacement_manifests: Vec<AppliedScopedReplacementManifestView>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct AppliedScopedReplacementManifestView {
    pub(crate) path: String,
    pub(crate) source_version: u32,
    pub(crate) version: u32,
    pub(crate) migration_applied: bool,
    pub(crate) replacements: usize,
}
