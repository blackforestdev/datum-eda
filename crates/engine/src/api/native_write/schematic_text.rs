//! Standalone literal text through qualified canonical schematic operations.
use super::{
    context::{BatchComposer, PreparedWrite, WriteProvenance},
    registry::{NativeWriteContext, NativeWriteVerb, parse_verb_params},
    schematic_edit::prepare_native_birth,
    schematic_selected_source::{SelectedSchematicSource, selected_source},
    schematic_sheets::build_create_schematic_text,
    schematic_symbol_payload::{member_payload, set_point, source_sheet_payload},
};
use crate::{
    error::EngineError,
    ir::geometry::Point,
    schematic::SchematicText,
    substrate::{ElectricalSelectionSnapshot, Operation},
};
use serde::Deserialize;
use serde_json::{Value, json};
use uuid::Uuid;

pub(crate) const VERBS: &[NativeWriteVerb] = &[
    NativeWriteVerb {
        id: "datum.schematic.place_text",
        build: place_text,
    },
    NativeWriteVerb {
        id: "datum.schematic.delete_text",
        build: delete_text,
    },
];

/// A note has an absolute anchor and no electrical attachments or owned tether.
#[derive(Debug, Clone, PartialEq)]
pub struct TextMoveBasis {
    pub sheet_id: Uuid,
    pub text: SchematicText,
    raw: Value,
}
impl TextMoveBasis {
    pub fn capture(
        snapshot: &ElectricalSelectionSnapshot,
        text: Uuid,
        instance_path: &[Uuid],
    ) -> Result<Self, EngineError> {
        let source = SelectedSchematicSource::from_snapshot(snapshot)?;
        let sheet_id = source.sheet("texts", text, instance_path)?;
        let original = &source.source.sheets[&sheet_id].texts[&text];
        let mut raw = None;
        for shard in snapshot.source_basis() {
            let payload = snapshot
                .source_shard_value(snapshot.revision(), shard.shard_id)
                .map_err(|e| invalid(format!("Text source unavailable: {e:?}")))?;
            if payload.get("uuid") == Some(&json!(sheet_id)) {
                if raw
                    .replace(member_payload(&payload, "texts", text, original)?)
                    .is_some()
                {
                    return Err(invalid("Text source sheet is ambiguous"));
                }
            }
        }
        let raw = raw.ok_or_else(|| invalid("Text source payload is missing"))?;
        admit_unlocked(&raw)?;
        Ok(Self {
            sheet_id,
            text: original.clone(),
            raw,
        })
    }
    pub fn destination(
        &self,
        position: Point,
        rotation_deg: i32,
    ) -> Result<SchematicText, EngineError> {
        let mut text = self.text.clone();
        text.position = position;
        text.rotation = if rotation_deg == self.text.rotation {
            rotation_deg
        } else {
            cardinal(rotation_deg)?
        };
        Ok(text)
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PlaceParams {
    sheet: Uuid,
    text: Option<Uuid>,
    instance_path: Option<Vec<Uuid>>,
    value: Option<String>,
    x_nm: Option<i64>,
    y_nm: Option<i64>,
    rotation_deg: Option<i32>,
}
fn place_text(
    context: &NativeWriteContext<'_>,
    provenance: WriteProvenance,
    params: Value,
) -> Result<PreparedWrite, EngineError> {
    let params: PlaceParams = parse_verb_params("datum.schematic.place_text", params)?;
    if params
        .value
        .as_ref()
        .is_some_and(|v| v.trim().is_empty() || v.chars().any(char::is_control))
    {
        return Err(invalid("Text must be nonempty and single-line"));
    }
    let position = match (params.x_nm, params.y_nm) {
        (Some(x), Some(y)) => Some(Point::new(x, y)),
        (None, None) => None,
        _ => return Err(invalid("Text position requires both x_nm and y_nm")),
    };
    let rotation = params.rotation_deg;
    let snapshot = ElectricalSelectionSnapshot::capture(context.model)
        .map_err(|e| invalid(format!("Text source unavailable: {e:?}")))?;
    if let Some(id) = params.text {
        let path = params
            .instance_path
            .as_deref()
            .ok_or_else(|| invalid("Text edit requires its occurrence path"))?;
        let basis = TextMoveBasis::capture(&snapshot, id, path)?;
        if basis.sheet_id != params.sheet {
            return Err(invalid("Text belongs to a different source sheet"));
        }
        let mut raw = basis.raw.clone();
        if let Some(value) = params.value {
            raw["text"] = json!(value);
        }
        if let Some(position) = position {
            set_point(&mut raw["position"], position);
        }
        if let Some(rotation) = rotation {
            raw["rotation"] = json!(if rotation == basis.text.rotation {
                rotation
            } else {
                cardinal(rotation)?
            });
        }
        if raw == basis.raw {
            return Err(invalid("Text is unchanged"));
        }
        return BatchComposer::compose(context.model, provenance)
            .push_op(Operation::SetSchematicText {
                sheet_id: params.sheet,
                text_id: id,
                text: raw,
            })
            .primary_object(id)
            .finish();
    }
    if params.instance_path.as_ref().is_some_and(|p| !p.is_empty()) {
        return Err(invalid(
            "Text creation requires the supported source-sheet view",
        ));
    }
    let source = snapshot
        .schematic_source()
        .ok_or_else(|| invalid("Native schematic source unavailable"))?;
    if !source
        .sheets
        .get(&params.sheet)
        .is_some_and(|s| s.uuid == params.sheet)
    {
        return Err(invalid("Native sheet not found"));
    }
    if source.sheet_instances.values().any(|instance| {
        source
            .sheet_definitions
            .get(&instance.definition)
            .is_some_and(|d| d.root_sheet == params.sheet)
    }) {
        return Err(invalid(
            "Text creation requires an uninstantiated source-sheet view",
        ));
    }
    let mut id = Uuid::new_v4();
    while context.model.objects.contains_key(&id) {
        id = Uuid::new_v4();
    }
    let text = SchematicText {
        uuid: id,
        text: params
            .value
            .ok_or_else(|| invalid("New text requires a literal value"))?,
        position: position.ok_or_else(|| invalid("New text requires an explicit position"))?,
        rotation: rotation.map(cardinal).transpose()?.unwrap_or(0),
    };
    let write = build_create_schematic_text(context.model, provenance, params.sheet, &text)?;
    prepare_native_birth(context.model, write)
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DeleteParams {
    text: Uuid,
    instance_path: Vec<Uuid>,
}
fn delete_text(
    context: &NativeWriteContext<'_>,
    provenance: WriteProvenance,
    params: Value,
) -> Result<PreparedWrite, EngineError> {
    let params: DeleteParams = parse_verb_params("datum.schematic.delete_text", params)?;
    let (source, sheet) = selected_source(context, "texts", params.text, &params.instance_path)?;
    delete_in_sheet(
        context,
        provenance,
        sheet,
        &source.sheets[&sheet].texts[&params.text],
    )
}
/// Shared with complete-set Proposal compilation, after its occurrence admission.
pub(super) fn delete_in_sheet(
    context: &NativeWriteContext<'_>,
    provenance: WriteProvenance,
    sheet: Uuid,
    text: &SchematicText,
) -> Result<PreparedWrite, EngineError> {
    let raw_sheet = source_sheet_payload(context.model, sheet, text.uuid)?;
    let raw = member_payload(&raw_sheet, "texts", text.uuid, text)?;
    admit_unlocked(&raw)?;
    BatchComposer::compose(context.model, provenance)
        .push_op(Operation::DeleteSchematicText {
            sheet_id: sheet,
            text_id: text.uuid,
            text: raw,
        })
        .primary_object(text.uuid)
        .finish()
}
fn admit_unlocked(raw: &Value) -> Result<(), EngineError> {
    if raw.get("locked").and_then(Value::as_bool) == Some(true) {
        return Err(invalid("Selected Text is locked"));
    }
    Ok(())
}
fn cardinal(degrees: i32) -> Result<i32, EngineError> {
    if degrees % 90 != 0 {
        return Err(invalid("Text rotation must be cardinal"));
    }
    Ok(degrees.rem_euclid(360))
}
fn invalid(reason: impl Into<String>) -> EngineError {
    EngineError::Validation(reason.into())
}

#[cfg(test)]
#[path = "schematic_text_tests.rs"]
mod tests;
