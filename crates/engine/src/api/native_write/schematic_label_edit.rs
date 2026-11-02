//! Selected label text/kind edits retain source identity and canonical history.
use super::{
    context::{PreparedWrite, WriteProvenance},
    registry::{NativeWriteContext, NativeWriteVerb, parse_verb_params},
    schematic_connectivity::build_set_schematic_label,
    schematic_selection_edit::selected_source,
};
use crate::{error::EngineError, schematic::LabelKind};
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
    if let Some(kind) = params.kind {
        label.kind = kind.into();
    }
    if &label == original {
        return Err(EngineError::Validation("Label fields are unchanged".into()));
    }
    build_set_schematic_label(context.model, provenance, sheet, &label)
}

#[cfg(test)]
#[path = "schematic_label_edit_tests.rs"]
mod tests;
