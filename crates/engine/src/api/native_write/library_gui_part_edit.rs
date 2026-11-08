//! Saved Part metadata never rewrites relationships or placed components.
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
        "parts",
        &[
            "manufacturer",
            "mpn",
            "value",
            "description",
            "datasheet",
            "tags",
        ],
        &["manufacturer", "mpn"],
    )
}
