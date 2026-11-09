//! Shared canonical-source qualification and raw metadata patching for saved forms.
use super::*;

pub(super) fn build(
    context: &NativeWriteContext<'_>,
    provenance: WriteProvenance,
    request: LibraryDefinitionRequest,
    kind: &str,
    allowed: &[&str],
    required_text: &[&str],
) -> Result<PreparedWrite, EngineError> {
    if request.kind != kind || !matches!(kind, "entities" | "parts" | "packages") {
        return Err(refusal(
            "Expected a saved Entity, Part or Package metadata request",
        ));
    }
    let (target, mut object) = saved_source::load(context.model, request.object_id, kind)?;
    for (key, value) in request.fields {
        if !allowed.contains(&key.as_str()) {
            return Err(refusal(
                "Saved metadata editing refuses structural, identity and policy fields",
            ));
        }
        if key == "tags" {
            let _: Vec<String> = serde_json::from_value(value.clone())?;
        } else {
            let text = value
                .as_str()
                .ok_or_else(|| refusal(format!("{key} must be text")))?;
            if required_text.contains(&key.as_str()) && text.trim().is_empty() {
                return Err(refusal(format!("Enter {key}")));
            }
        }
        object
            .as_object_mut()
            .ok_or_else(|| refusal("Definition payload is not an object"))?
            .insert(key, value);
    }
    match kind {
        "entities" => {
            let _: crate::pool::Entity = serde_json::from_value(object.clone())?;
        }
        "packages" => {
            let _: crate::pool::Package = serde_json::from_value(object.clone())?;
            saved_source::preserve_pinned_revision(
                context.model,
                request.object_id,
                kind,
                &object,
            )?;
        }
        "parts" => {
            let _: crate::pool::Part = serde_json::from_value(object.clone())?;
            saved_source::preserve_pinned_revision(
                context.model,
                request.object_id,
                kind,
                &object,
            )?;
        }
        _ => unreachable!(),
    }
    library::build_pool_library_write(
        context.model,
        provenance,
        Some("pool"),
        vec![PoolLibraryOperationSpec::Set { target, object }],
    )
}
