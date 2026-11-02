//! Independent text configures the shared literal/choice form and source guards.
use super::*;
use eda_engine::schematic::SchematicText;

#[derive(Debug, Clone, PartialEq)]
pub struct TextEditDraft {
    pub target: AuthoredSelectionIdentity,
    pub basis: AuthoringViewBasis,
    pub tip: Option<Uuid>,
    pub original: SchematicText,
}
impl TextEditDraft {
    pub fn replacement(&self, value: &str, rotation: i32) -> Result<SchematicText, String> {
        validate_text(value)?;
        if rotation != self.original.rotation && rotation.rem_euclid(90) != 0 {
            return Err("Choose a cardinal text rotation".into());
        }
        Ok(SchematicText {
            text: value.into(),
            rotation,
            ..self.original.clone()
        })
    }
    pub fn request(
        &self,
        value: &str,
        rotation: i32,
    ) -> Result<Option<SchematicEditRequest>, String> {
        let next = self.replacement(value, rotation)?;
        if next == self.original {
            return Ok(None);
        }
        Ok(Some(SchematicEditRequest::Text {
            sheet: self.basis.source,
            text: Some(self.target.id),
            instance_path: self.target.instance_path.clone(),
            value: (next.text != self.original.text).then_some(next.text),
            x_nm: None,
            y_nm: None,
            rotation_deg: (rotation != self.original.rotation).then_some(rotation),
        }))
    }
}
pub fn validate_text(value: &str) -> Result<(), String> {
    if value.trim().is_empty() || value.chars().any(char::is_control) {
        return Err("Enter nonempty single-line text".into());
    }
    Ok(())
}
pub fn text_admission(state: &ReviewWorkspaceState) -> Result<&'static str, String> {
    let (_, target) = target(state)?;
    if target.class != Class::SchematicText {
        return Err("Select one current Text annotation".into());
    }
    if matches!(&state.selection,SelectionTarget::Native(view) if view.locked.contains(&target)) {
        return Err("Text is locked".into());
    }
    let verb = "datum.schematic.place_text";
    crate::gui_menu_model::native_edit_unavailable_reason(state, verb)
        .map_or(Ok(verb), |r| Err(r.into()))
}
