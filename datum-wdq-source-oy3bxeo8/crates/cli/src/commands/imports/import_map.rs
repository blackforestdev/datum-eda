use std::collections::BTreeMap;
use std::path::Path;

use anyhow::Result;
use eda_engine::substrate::{ImportMapEntry, ProjectResolver};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectImportMapView {
    pub(crate) contract: &'static str,
    pub(crate) project_id: String,
    pub(crate) model_revision: String,
    pub(crate) import_map_count: usize,
    pub(crate) entries: BTreeMap<String, ImportMapEntry>,
}

pub(crate) fn query_native_project_import_map(root: &Path) -> Result<NativeProjectImportMapView> {
    let model = ProjectResolver::new(root).resolve()?;
    Ok(NativeProjectImportMapView {
        contract: "import_map_query_v1",
        project_id: model.project.project_id.to_string(),
        model_revision: model.model_revision.0,
        import_map_count: model.import_map.len(),
        entries: model.import_map,
    })
}
