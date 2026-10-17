//! Creation-only physical Package intent over the existing canonical pool writer.
use super::*;

pub(super) fn build(
    context: &NativeWriteContext<'_>,
    provenance: WriteProvenance,
    request: LibraryDefinitionRequest,
    create: bool,
) -> Result<PreparedWrite, EngineError> {
    if !create {
        return Err(refusal(
            "Saved Package editing is unavailable in this increment",
        ));
    }
    const FIELDS: &[&str] = &[
        "name",
        "package_family",
        "package_code",
        "mounting_type",
        "body_dimensions",
        "body_height_nm",
        "body_height_mounted_nm",
        "terminals",
    ];
    if request
        .fields
        .keys()
        .any(|key| !FIELDS.contains(&key.as_str()))
    {
        return Err(refusal("Package creation contains an unsupported field"));
    }
    let name = request
        .fields
        .get("name")
        .and_then(Value::as_str)
        .filter(|name| !name.trim().is_empty())
        .ok_or_else(|| refusal("Library name must be non-empty"))?;
    let mut object = library::pool_package_payload(request.object_id, name, json!({}));
    object.as_object_mut().unwrap().extend(request.fields);
    if let Some(dimensions) = object.get("body_dimensions").filter(|v| !v.is_null()) {
        let dimensions = dimensions
            .as_object()
            .ok_or_else(|| refusal("Package body dimensions must be an object"))?;
        if dimensions
            .keys()
            .any(|k| !["x_nm", "y_nm", "z_nm"].contains(&k.as_str()))
        {
            return Err(refusal(
                "Package body supports nominal X/Y/Z dimensions only",
            ));
        }
    }
    // Validate raw rows before typed HashMap admission can collapse UUID aliases.
    if let Some(terminals) = object.get("terminals").and_then(Value::as_object) {
        let mut identities = std::collections::HashSet::new();
        for (key, terminal) in terminals {
            let id = Uuid::parse_str(key)
                .map_err(|_| refusal("Package terminal identity is invalid"))?;
            if !identities.insert(id) {
                return Err(refusal("Duplicate Package terminal identity"));
            }
            if terminal.as_object().is_none_or(|fields| {
                fields
                    .keys()
                    .any(|k| !["uuid", "name", "role"].contains(&k.as_str()))
            }) {
                return Err(refusal(
                    "Package terminals support identity, name and role only",
                ));
            }
            let typed: crate::pool::PackageTerminal = serde_json::from_value(terminal.clone())?;
            if typed.uuid != id {
                return Err(refusal("Package terminal UUID must match its map identity"));
            }
        }
    }
    // Keep the existing signed-nanometre schema; no new numeric or mounting policy.
    let _: crate::pool::Package = serde_json::from_value(object.clone())?;
    library::build_pool_library_write(
        context.model,
        provenance,
        Some("pool"),
        vec![PoolLibraryOperationSpec::Create {
            target: PoolLibraryObjectTarget::new("pool", "packages", request.object_id),
            object,
        }],
    )
}
