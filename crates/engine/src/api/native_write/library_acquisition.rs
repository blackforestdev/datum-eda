//! Local library acquisition through the same atomic native write as authoring.
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{
    PreparedWrite, WriteProvenance,
    context::BatchComposer,
    imports::{import_map_shard_operation, source_shard_id_for_relative_path},
    library::{PoolLibraryObjectTarget, PoolLibraryOperationSpec, build_pool_library_write},
    registry::{NativeWriteContext, NativeWriteVerb, parse_verb_params},
};
use crate::{
    error::EngineError,
    import::{kicad_library, library_bundle},
    substrate::ImportMapEntry,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LibraryAcquisitionRequest {
    pub source_format: String,
    pub source_path: PathBuf,
    pub symbol_name: String,
    pub expected_source_hash: String,
}

pub const VERBS: &[NativeWriteVerb] = &[NativeWriteVerb {
    id: "datum.library.acquire",
    build: acquire,
}];

fn acquire(
    context: &NativeWriteContext<'_>,
    provenance: WriteProvenance,
    params: Value,
) -> Result<PreparedWrite, EngineError> {
    build_acquisition(
        context,
        provenance,
        parse_verb_params("datum.library.acquire", params)?,
    )
}

/// Re-read and validate the source at commit preparation; preview data is not
/// trusted as an object payload. Existing definitions require a separate edit.
pub fn build_acquisition(
    context: &NativeWriteContext<'_>,
    provenance: WriteProvenance,
    request: LibraryAcquisitionRequest,
) -> Result<PreparedWrite, EngineError> {
    let converted = match request.source_format.as_str() {
        "kicad_symbol" => kicad_library::convert(&request.source_path, &request.symbol_name)?,
        "datum_bundle" => library_bundle::preview(&request.source_path, &request.symbol_name)?,
        _ => {
            return Err(EngineError::Validation(
                "Unsupported library source format".into(),
            ));
        }
    };
    if converted.source_hash != request.expected_source_hash {
        return Err(EngineError::Validation(
            "Library source changed since preview; load and preview it again".into(),
        ));
    }
    let source_path = converted.source_path.to_string_lossy().into_owned();
    let mut specs = Vec::new();
    let mut entries = Vec::new();
    for (kind, id, mut object) in [
        (
            "units",
            converted.unit.uuid,
            serde_json::to_value(&converted.unit)?,
        ),
        (
            "symbols",
            converted.symbol.uuid,
            serde_json::to_value(&converted.symbol)?,
        ),
    ] {
        if context.model.objects.contains_key(&id) {
            return Err(EngineError::Validation(format!(
                "{} is already in the native library; select the existing symbol to place it",
                request.symbol_name
            )));
        }
        object["schema_version"] = serde_json::json!(1);
        let target = PoolLibraryObjectTarget::new("pool", kind, id);
        entries.push(ImportMapEntry {
            import_key: format!(
                "kicad:symbol:{source_path}:{}:{kind}",
                converted.source_symbol
            ),
            object_id: id,
            source_shard_id: source_shard_id_for_relative_path(&target.relative_path),
            status: Default::default(),
            source_tool: request.source_format.clone(),
            source_path: source_path.clone(),
            source_object_ref: converted.source_symbol.clone(),
            source_hash: converted.source_hash.clone(),
        });
        specs.push(PoolLibraryOperationSpec::Create { target, object });
    }
    let prepared =
        build_pool_library_write(context.model, provenance.clone(), Some("pool"), specs)?;
    // One map per source symbol keeps independent acquisitions from replacing
    // another symbol's provenance. Stable source identities prevent silent rebinding.
    let map_path = format!(
        ".datum/import_map/kicad-symbol-{}.json",
        converted.symbol.uuid
    );
    BatchComposer::compose(context.model, provenance)
        .push_ops(prepared.batch.operations)
        .push_ops([import_map_shard_operation(map_path, entries)?])
        .finish()
}
