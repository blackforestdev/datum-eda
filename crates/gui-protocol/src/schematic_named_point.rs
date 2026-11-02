//! Shared name/choice form for label and interface-port point authoring.
use super::*;

#[derive(Debug, Clone, PartialEq)]
pub struct LabelDraft {
    /// The same name/choice form configures a label or interface port.
    pub text_rotation: Option<i32>,
    pub port_direction: Option<PortDirection>,
    pub kind: LabelKind,
    pub name: String,
    pub editor: crate::single_line_editor::SingleLineEditor,
    /// Name, kind, Cancel, Preview; shared field/controller focus order.
    pub focus: usize,
}
impl LabelDraft {
    pub fn new(kind: LabelKind) -> Self {
        let name = if kind == LabelKind::Power { "GND" } else { "" }.to_owned();
        Self {
            port_direction: None,
            text_rotation: None,
            kind,
            editor: crate::single_line_editor::SingleLineEditor::at_end(&name),
            name,
            focus: 0,
        }
    }
    pub fn text(rotation: i32) -> Self {
        Self {
            text_rotation: Some(rotation),
            ..Self::new(LabelKind::Local)
        }
    }
    pub fn port(direction: PortDirection) -> Self {
        Self {
            port_direction: Some(direction),
            ..Self::new(LabelKind::Hierarchical)
        }
    }
    pub fn placement_request(
        &self,
        sheet: Uuid,
        at: PointNm,
    ) -> Result<SchematicEditRequest, String> {
        if let Some(rotation) = self.text_rotation {
            selected::validate_text(&self.name)?;
            return Ok(SchematicEditRequest::Text {
                sheet,
                text: None,
                instance_path: vec![],
                value: Some(self.name.clone()),
                x_nm: Some(at.x),
                y_nm: Some(at.y),
                rotation_deg: Some(rotation),
            });
        }
        if self.name.trim().is_empty() || self.name.chars().any(char::is_control) {
            return Err("Enter a nonempty single-line name".into());
        }
        let name = self.name.trim().to_owned();
        Ok(if let Some(direction) = &self.port_direction {
            SchematicEditRequest::PlacePort {
                sheet,
                port: None,
                instance_path: vec![],
                name,
                direction: direction.clone(),
                x_nm: at.x,
                y_nm: at.y,
            }
        } else {
            SchematicEditRequest::PlaceLabel {
                sheet,
                kind: self.kind.clone(),
                name,
                x_nm: at.x,
                y_nm: at.y,
            }
        })
    }
    pub fn kind_name(&self) -> &'static str {
        if let Some(direction) = &self.port_direction {
            return match direction {
                PortDirection::Input => "Input",
                PortDirection::Output => "Output",
                PortDirection::Bidirectional => "Bidirectional",
                PortDirection::Passive => "Passive",
            };
        }
        match self.kind {
            LabelKind::Local => "Local",
            LabelKind::Global => "Global",
            LabelKind::Power => "Power",
            LabelKind::Hierarchical => "Hierarchical",
        }
    }
    pub fn cycle_kind(&mut self) {
        if let Some(rotation) = &mut self.text_rotation {
            *rotation = ((rotation.rem_euclid(360) / 90 + 1) * 90).rem_euclid(360);
            return;
        }
        if let Some(direction) = &mut self.port_direction {
            *direction = match direction {
                PortDirection::Input => PortDirection::Output,
                PortDirection::Output => PortDirection::Bidirectional,
                PortDirection::Bidirectional => PortDirection::Passive,
                PortDirection::Passive => PortDirection::Input,
            };
            return;
        }
        self.kind = match self.kind {
            LabelKind::Local => LabelKind::Global,
            LabelKind::Global => LabelKind::Power,
            _ => LabelKind::Local,
        };
    }
}
