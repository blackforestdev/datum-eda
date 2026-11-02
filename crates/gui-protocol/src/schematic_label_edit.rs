//! Source-qualified name/kind replacement; moving a Label is a separate gesture.
use super::*;
use eda_engine::schematic::{LabelKind, NetLabel};

#[derive(Debug, Clone, PartialEq)]
pub struct LabelEditDraft {
    pub target: AuthoredSelectionIdentity,
    pub basis: AuthoringViewBasis,
    pub tip: Option<Uuid>,
    pub original: NetLabel,
}
impl LabelEditDraft {
    pub fn replacement(&self, name: &str, kind: LabelKind) -> Result<NetLabel, String> {
        if name.trim().is_empty() || name.chars().any(char::is_control) {
            return Err("Enter a nonempty single-line net name".into());
        }
        let mut label = self.original.clone();
        label.name = name.trim().into();
        label.kind = kind;
        Ok(label)
    }
    pub fn request(
        &self,
        name: &str,
        kind: LabelKind,
    ) -> Result<Option<SchematicEditRequest>, String> {
        let label = self.replacement(name, kind)?;
        if label == self.original {
            return Ok(None);
        }
        Ok(Some(SchematicEditRequest::EditLabel {
            label: self.target.id,
            instance_path: self.target.instance_path.clone(),
            name: label.name,
            kind: label.kind,
        }))
    }
}

pub fn label_admission(state: &ReviewWorkspaceState) -> Result<&'static str, String> {
    let (_, target) = target(state)?;
    if target.class != Class::Label {
        return Err("Select one current Label to edit its name or kind".into());
    }
    let verb = "datum.schematic.rename_label";
    if let Some(reason) = crate::gui_menu_model::native_edit_unavailable_reason(state, verb) {
        return Err(reason.into());
    }
    Ok(verb)
}
