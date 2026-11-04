//! Selected label text/kind edits retain source identity and canonical history.
use super::{
    context::{BatchComposer, PreparedWrite, WriteProvenance},
    registry::{NativeWriteContext, NativeWriteVerb, parse_verb_params},
    schematic_selection_edit::selected_source,
    schematic_symbol_payload::{member_payload, source_sheet_payload},
};
use crate::{error::EngineError, schematic::LabelKind, substrate::Operation};
use serde::Deserialize;
use uuid::Uuid;

pub(crate) const VERBS: &[NativeWriteVerb] = &[NativeWriteVerb {
    id: "datum.schematic.rename_label",
    build: rename_label,
}];

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
enum Kind {
    Local,
    Global,
    Hierarchical,
    Power,
}
impl From<Kind> for LabelKind {
    fn from(kind: Kind) -> Self {
        match kind {
            Kind::Local => Self::Local,
            Kind::Global => Self::Global,
            Kind::Hierarchical => Self::Hierarchical,
            Kind::Power => Self::Power,
        }
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Params {
    label: Uuid,
    instance_path: Vec<Uuid>,
    name: String,
    kind: Option<Kind>,
}

fn rename_label(
    context: &NativeWriteContext<'_>,
    provenance: WriteProvenance,
    params: serde_json::Value,
) -> Result<PreparedWrite, EngineError> {
    let params: Params = parse_verb_params("datum.schematic.rename_label", params)?;
    if params.name.trim().is_empty() || params.name.chars().any(char::is_control) {
        return Err(EngineError::Validation(
            "Label name must be nonempty and single-line".into(),
        ));
    }
    let (source, sheet) = selected_source(context, "labels", params.label, &params.instance_path)?;
    let original = &source.sheets[&sheet].labels[&params.label];
    let mut label = original.clone();
    label.name = params.name;
    let kind_supplied = params.kind.is_some();
    if let Some(kind) = params.kind {
        label.kind = kind.into();
    }
    if &label == original {
        return Err(EngineError::Validation("Label fields are unchanged".into()));
    }
    let raw = source_sheet_payload(context.model, sheet, original.uuid)?;
    let mut payload = member_payload(&raw, "labels", original.uuid, original)?;
    payload["name"] = serde_json::Value::String(label.name);
    if kind_supplied {
        payload["kind"] = serde_json::to_value(label.kind)?;
    }
    BatchComposer::compose(context.model, provenance)
        .push_op(Operation::SetSchematicLabel {
            sheet_id: sheet,
            label_id: original.uuid,
            label: payload,
        })
        .primary_object(original.uuid)
        .finish()
}

#[cfg(test)]
#[path = "schematic_label_edit_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "schematic_label_port_payload_tests.rs"]
mod payload_tests;
