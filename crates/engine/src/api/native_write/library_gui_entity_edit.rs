//! Saved Entity metadata shares canonical qualification and raw patching.
use super::*;
pub(super) fn build(
    context: &NativeWriteContext<'_>,
    provenance: WriteProvenance,
    request: LibraryDefinitionRequest,
) -> Result<PreparedWrite, EngineError> {
    super::super::metadata_edit::build(
        context,
        provenance,
        request,
        "entities",
        &["name", "prefix", "manufacturer", "tags"],
        &["name"],
    )
}
