//! Qualified named-point edits share one controller and immutable-source preview.
use super::*;
use crate::schematic_authoring::LabelDraft;
use eda_engine::schematic::{HierarchicalPort, PortDirection, Sheet};

#[derive(Debug, Clone, PartialEq)]
pub struct PortEditDraft {
    pub target: AuthoredSelectionIdentity,
    pub basis: AuthoringViewBasis,
    pub tip: Option<Uuid>,
    pub original: HierarchicalPort,
}
impl PortEditDraft {
    pub fn replacement(
        &self,
        name: &str,
        direction: PortDirection,
    ) -> Result<HierarchicalPort, String> {
        if name.trim().is_empty() || name.chars().any(char::is_control) {
            return Err("Enter a nonempty single-line port name".into());
        }
        let mut port = self.original.clone();
        port.name = name.trim().into();
        port.direction = direction;
        Ok(port)
    }
    pub fn request(
        &self,
        name: &str,
        direction: PortDirection,
    ) -> Result<Option<SchematicEditRequest>, String> {
        let port = self.replacement(name, direction)?;
        if port == self.original {
            return Ok(None);
        }
        Ok(Some(SchematicEditRequest::PlacePort {
            sheet: self.basis.source,
            port: Some(self.target.id),
            instance_path: self.target.instance_path.clone(),
            name: port.name,
            direction: port.direction,
            x_nm: port.position.x,
            y_nm: port.position.y,
        }))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum NamedPointEditDraft {
    Label(LabelEditDraft),
    Port(PortEditDraft),
    Text(TextEditDraft),
}
impl From<LabelEditDraft> for NamedPointEditDraft {
    fn from(draft: LabelEditDraft) -> Self {
        Self::Label(draft)
    }
}
impl NamedPointEditDraft {
    pub fn basis(&self) -> &AuthoringViewBasis {
        match self {
            Self::Label(d) => &d.basis,
            Self::Port(d) => &d.basis,
            Self::Text(d) => &d.basis,
        }
    }
    pub fn target(&self) -> &AuthoredSelectionIdentity {
        match self {
            Self::Label(d) => &d.target,
            Self::Port(d) => &d.target,
            Self::Text(d) => &d.target,
        }
    }
    pub fn tip(&self) -> Option<Uuid> {
        match self {
            Self::Label(d) => d.tip,
            Self::Port(d) => d.tip,
            Self::Text(d) => d.tip,
        }
    }
    pub fn form(&self) -> LabelDraft {
        let (mut form, name) = match self {
            Self::Text(d) => (LabelDraft::text(d.original.rotation), &d.original.text),
            Self::Label(d) => (LabelDraft::new(d.original.kind.clone()), &d.original.name),
            Self::Port(d) => (
                LabelDraft::port(d.original.direction.clone()),
                &d.original.name,
            ),
        };
        form.name = name.clone();
        form.editor = crate::single_line_editor::SingleLineEditor::at_end(name);
        form
    }
    pub fn position(&self) -> crate::PointNm {
        let at = match self {
            Self::Label(d) => d.original.position,
            Self::Port(d) => d.original.position,
            Self::Text(d) => d.original.position,
        };
        crate::PointNm { x: at.x, y: at.y }
    }
    pub fn source_matches(&self, sheet: &Sheet) -> bool {
        match self {
            Self::Label(d) => sheet.labels.get(&d.target.id) == Some(&d.original),
            Self::Port(d) => sheet.ports.get(&d.target.id) == Some(&d.original),
            Self::Text(d) => sheet.texts.get(&d.target.id) == Some(&d.original),
        }
    }
    pub fn request(&self, form: &LabelDraft) -> Result<Option<SchematicEditRequest>, String> {
        if let Self::Text(d) = self {
            return d.request(
                &form.name,
                form.text_rotation
                    .ok_or("Text form changed; reopen the edit")?,
            );
        }
        if form.text_rotation.is_some() {
            return Err("Object type changed; reopen the edit".into());
        }
        match (self, &form.port_direction) {
            (Self::Label(d), None) => d.request(&form.name, form.kind.clone()),
            (Self::Port(d), Some(direction)) => d.request(&form.name, direction.clone()),
            _ => Err("Object type changed; cancel and reopen the edit".into()),
        }
    }
    pub fn preview(&self, form: &LabelDraft, sheet: &mut Sheet) {
        if !self.source_matches(sheet) {
            return;
        }
        if let Self::Text(d) = self {
            if let Some(rotation) = form.text_rotation
                && let Ok(text) = d.replacement(&form.name, rotation)
            {
                sheet.texts.insert(text.uuid, text);
            }
            return;
        }
        // Invalid intermediate text keeps source ink and cannot form a request.
        match (self, &form.port_direction) {
            (Self::Label(d), None) => {
                if let Ok(label) = d.replacement(&form.name, form.kind.clone()) {
                    sheet.labels.insert(label.uuid, label);
                }
            }
            (Self::Port(d), Some(direction)) => {
                if let Ok(port) = d.replacement(&form.name, direction.clone()) {
                    sheet.ports.insert(port.uuid, port);
                }
            }
            _ => {}
        }
    }
}

pub fn port_admission(state: &ReviewWorkspaceState) -> Result<&'static str, String> {
    let (_, target) = target(state)?;
    if target.class != Class::Port {
        return Err("Select one current Port to edit its name or direction".into());
    }
    let verb = "datum.schematic.place_port";
    if let Some(reason) = crate::gui_menu_model::native_edit_unavailable_reason(state, verb) {
        return Err(reason.into());
    }
    Ok(verb)
}
