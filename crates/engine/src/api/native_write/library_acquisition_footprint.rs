//! Atomic acquisition of one supported source footprint and its full closure.
use super::{LibraryAcquisitionRequest, PreparedWrite, WriteProvenance};
use crate::api::native_write::{
    context::BatchComposer,
    imports::{import_map_shard_operation, source_shard_id_for_relative_path},
    library::{PoolLibraryObjectTarget, PoolLibraryOperationSpec, build_pool_library_write},
    registry::NativeWriteContext,
};
use crate::{error::EngineError, import::kicad_footprint_library};
use serde_json::json;

pub(super) fn build(
    context: &NativeWriteContext<'_>,
    provenance: WriteProvenance,
    request: LibraryAcquisitionRequest,
) -> Result<PreparedWrite, EngineError> {
    let converted = kicad_footprint_library::convert(&request.source_path)?;
    if converted.source_hash != request.expected_source_hash
        || converted.footprint.name != request.symbol_name
    {
        return Err(EngineError::Validation(
            "Footprint source changed since preview; load and preview it again".into(),
        ));
    }
    let mut objects = vec![
        (
            "packages",
            converted.package.uuid,
            serde_json::to_value(&converted.package)?,
        ),
        (
            "footprints",
            converted.footprint.uuid,
            serde_json::to_value(&converted.footprint)?,
        ),
    ];
    objects.extend(
        converted
            .padstacks
            .iter()
            .map(|p| Ok(("padstacks", p.uuid, serde_json::to_value(p)?)))
            .collect::<Result<Vec<_>, EngineError>>()?,
    );
    let mut specs = vec![];
    let mut entries = vec![];
    for (kind, id, mut object) in objects {
        if context.model.objects.contains_key(&id) {
            return Err(EngineError::Validation(
                "This footprint is already in the Project library; use the saved definition".into(),
            ));
        }
        let target = PoolLibraryObjectTarget::new("pool", kind, id);
        object["schema_version"] = json!(1);
        object["source_provenance"] = json!({
            "source_tool":"kicad_footprint","source_path":converted.source_path,
            "source_hash":converted.source_hash,"source_name":converted.footprint.name,
            "copyright":null,"license":null,
            "source_rights":"Source rights are not inferred from file format or directory name",
            "conversion":"Datum native source-library conversion; unsupported required features refuse",
            "warnings":converted.warnings,
        });
        if kind == "footprints" {
            object["original_kicad_source"] = json!(converted.source_document);
        }
        entries.push(crate::substrate::ImportMapEntry {
            import_key: format!("kicad:footprint-library:{id}"),
            object_id: id,
            source_shard_id: source_shard_id_for_relative_path(&target.relative_path),
            status: Default::default(),
            source_tool: "kicad_footprint".into(),
            source_path: converted.source_path.to_string_lossy().into_owned(),
            source_object_ref: converted.footprint.name.clone(),
            source_hash: converted.source_hash.clone(),
        });
        specs.push(PoolLibraryOperationSpec::Create { target, object });
    }
    let prepared =
        build_pool_library_write(context.model, provenance.clone(), Some("pool"), specs)?;
    BatchComposer::compose(context.model, provenance)
        .push_ops(prepared.batch.operations)
        .push_ops([import_map_shard_operation(
            format!(
                ".datum/import_map/kicad-footprint-library-{}.json",
                converted.footprint.uuid
            ),
            entries,
        )?])
        .finish()
}
