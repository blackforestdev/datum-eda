//! Revision-qualified single-object editing; displayed Inspector values are not authority.
use super::SchematicEditRequest;
#[path = "schematic_label_edit.rs"]
mod label_edit;
#[path = "schematic_port_edit.rs"]
mod port_edit;
use crate::{
    AuthoredSelectionClass as Class, AuthoredSelectionIdentity, AuthoringViewBasis, PaneContent,
    ReviewWorkspaceState, SelectionSubject, SelectionTarget,
};
pub use label_edit::{LabelEditDraft, label_admission};
pub use port_edit::{NamedPointEditDraft, PortEditDraft, port_admission};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum DeleteKind {
    Symbol,
    Wire,
    Label,
    Junction,
    NoConnect,
    Port,
}
impl DeleteKind {
    pub fn from_class(class: Class) -> Result<Self, String> {
        match class {
            Class::Symbol => Ok(Self::Symbol),
            Class::Wire => Ok(Self::Wire),
            Class::Label => Ok(Self::Label),
            Class::Junction => Ok(Self::Junction),
            Class::NoConnect => Ok(Self::NoConnect),
            Class::Port => Ok(Self::Port),
            _ => {
                Err("Select one Symbol, Wire, Label, Port, Junction or No-Connect to delete".into())
            }
        }
    }
    pub fn verb_and_key(self) -> (&'static str, &'static str) {
        match self {
            Self::Symbol => ("datum.schematic.delete_symbol", "symbol"),
            Self::Wire => ("datum.schematic.delete_wire", "wire"),
            Self::Label => ("datum.schematic.delete_label", "label"),
            Self::Junction => ("datum.schematic.delete_junction", "junction"),
            Self::NoConnect => ("datum.schematic.delete_noconnect", "noconnect"),
            Self::Port => ("datum.schematic.delete_port", "port"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SymbolEditDraft {
    pub target: AuthoredSelectionIdentity,
    pub basis: AuthoringViewBasis,
    pub tip: Option<Uuid>,
    pub original_reference: String,
    pub original_value: String,
}
impl SymbolEditDraft {
    pub fn request(
        &self,
        reference: &str,
        value: &str,
    ) -> Result<Option<SchematicEditRequest>, String> {
        if reference.trim().is_empty()
            || reference.chars().any(char::is_control)
            || value.chars().any(char::is_control)
        {
            return Err("Enter a nonempty reference and single-line value".into());
        }
        if reference == self.original_reference && value == self.original_value {
            return Ok(None);
        }
        Ok(Some(SchematicEditRequest::EditSymbol {
            symbol: self.target.id,
            instance_path: self.target.instance_path.clone(),
            reference: reference.into(),
            value: value.into(),
        }))
    }
}

pub fn target(
    state: &ReviewWorkspaceState,
) -> Result<(AuthoringViewBasis, AuthoredSelectionIdentity), String> {
    let basis = AuthoringViewBasis::capture(state, state.ui.layout.focused)?;
    if basis.content != PaneContent::Schematic {
        return Err("Focus the selected object's schematic pane".into());
    }
    let SelectionTarget::Native(view) = &state.selection else {
        return Err("Select one schematic object first".into());
    };
    if view.stale || view.project != basis.project || view.revision.0 != basis.revision {
        return Err("Selection changed; select the current object again".into());
    }
    let SelectionSubject::Object(id) = &view.subject else {
        return Err("This edit requires one complete selected object".into());
    };
    if basis
        .document
        .as_ref()
        .is_none_or(|d| d.instance_path != id.instance_path)
    {
        return Err("Selection belongs to a different schematic occurrence".into());
    }
    let sheet = state
        .schematic_authoring
        .source
        .as_ref()
        .and_then(|s| s.sheets.get(&basis.source))
        .ok_or("Selected schematic source is unavailable")?;
    let present = match id.class {
        Class::Symbol => sheet.symbols.contains_key(&id.id),
        Class::Wire => sheet.wires.contains_key(&id.id),
        Class::Label => sheet.labels.contains_key(&id.id),
        Class::Junction => sheet.junctions.contains_key(&id.id),
        Class::NoConnect => sheet.noconnects.contains_key(&id.id),
        Class::Port => sheet.ports.contains_key(&id.id),
        _ => false,
    };
    if !present {
        return Err(
            "Select a Symbol, Wire, Label, Port, Junction or No-Connect in this source sheet"
                .into(),
        );
    }
    Ok((basis, id.clone()))
}

pub fn admission(state: &ReviewWorkspaceState, edit_fields: bool) -> Result<&'static str, String> {
    let (_, id) = target(state)?;
    let verb = if edit_fields {
        if id.class != Class::Symbol {
            return Err("Select one Symbol to edit its fields".into());
        }
        "datum.schematic.set_symbol_reference"
    } else {
        DeleteKind::from_class(id.class)?.verb_and_key().0
    };
    if let Some(reason) = crate::gui_menu_model::native_edit_unavailable_reason(state, verb) {
        return Err(reason.into());
    }
    Ok(verb)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn marker_delete_kinds_keep_class_identity_and_reject_other_classes() {
        assert_eq!(
            DeleteKind::from_class(Class::Junction).unwrap(),
            DeleteKind::Junction
        );
        assert_eq!(
            DeleteKind::from_class(Class::NoConnect).unwrap(),
            DeleteKind::NoConnect
        );
        assert!(DeleteKind::from_class(Class::Track).is_err());
        assert_eq!(
            DeleteKind::from_class(Class::Port).unwrap(),
            DeleteKind::Port
        );
    }
    #[test]
    fn field_apply_is_atomic_and_unchanged_or_invalid_input_has_no_request() {
        let target = AuthoredSelectionIdentity {
            class: Class::Symbol,
            id: Uuid::new_v4(),
            instance_path: vec![],
        };
        let draft = SymbolEditDraft {
            target: target.clone(),
            basis: AuthoringViewBasis {
                project: Uuid::new_v4(),
                revision: "revision".into(),
                pane: crate::PaneId(1),
                content: PaneContent::Schematic,
                source: Uuid::new_v4(),
                document: None,
            },
            tip: None,
            original_reference: "R1".into(),
            original_value: "10k".into(),
        };
        assert!(draft.request("R1", "10k").unwrap().is_none());
        assert!(draft.request(" ", "22k").is_err());
        assert!(draft.request("R2", "two\nlines").is_err());
        let (verb, params) = draft.request("R2", "22k").unwrap().unwrap().verb_params();
        assert_eq!(verb, "datum.schematic.set_symbol_reference");
        assert_eq!(
            params,
            serde_json::json!({"symbol":target.id,"instance_path":[],"reference":"R2","value":"22k"})
        );
        for kind in [
            DeleteKind::Symbol,
            DeleteKind::Wire,
            DeleteKind::Label,
            DeleteKind::Junction,
            DeleteKind::NoConnect,
            DeleteKind::Port,
        ] {
            let request = SchematicEditRequest::DeleteObject {
                object: target.id,
                instance_path: vec![Uuid::nil()],
                kind,
            };
            let (verb, params) = request.verb_params();
            assert_eq!(verb, kind.verb_and_key().0);
            assert_eq!(params[kind.verb_and_key().1], target.id.to_string());
            assert_eq!(params["instance_path"], serde_json::json!([Uuid::nil()]));
        }
    }
}
