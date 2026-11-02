//! Shared two-text-field edit controller, configured for Symbol, Bus and custom metadata.
use super::*;
use crate::{AuthoredSelectionIdentity, AuthoringViewBasis};
/// The existing two-text-field controller serves Symbol fields and Bus members.
#[derive(Debug, Clone, PartialEq)]
pub enum FieldsEditDraft {
    Symbol(selected::SymbolEditDraft),
    Bus(BusEditDraft),
    Custom(CustomFieldDraft),
}
impl FieldsEditDraft {
    pub fn text_editable(&self) -> bool {
        !matches!(self, Self::Custom(d) if d.remove)
    }
    pub fn basis(&self) -> &AuthoringViewBasis {
        match self {
            Self::Symbol(d) => &d.basis,
            Self::Bus(d) => &d.basis,
            Self::Custom(d) => &d.symbol.basis,
        }
    }
    pub fn target(&self) -> &AuthoredSelectionIdentity {
        match self {
            Self::Symbol(d) => &d.target,
            Self::Bus(d) => &d.target,
            Self::Custom(d) => &d.symbol.target,
        }
    }
    pub fn tip(&self) -> Option<Uuid> {
        match self {
            Self::Symbol(d) => d.tip,
            Self::Bus(d) => d.tip,
            Self::Custom(d) => d.symbol.tip,
        }
    }
    pub fn fields(&self) -> (String, String) {
        match self {
            Self::Symbol(d) => (d.original_reference.clone(), d.original_value.clone()),
            Self::Bus(d) => (d.original.name.clone(), d.original.members.join(", ")),
            Self::Custom(d) => d.fields(),
        }
    }
    pub fn request(
        &self,
        state: &SchematicAuthoring,
    ) -> Result<Option<SchematicEditRequest>, String> {
        match self {
            Self::Symbol(d) => {
                if state.source.as_ref().is_some_and(|source| {
                    source.sheets.values().any(|sheet| {
                        sheet
                            .symbols
                            .values()
                            .any(|s| s.uuid != d.target.id && s.reference == state.reference)
                    })
                }) {
                    return Err("That reference is already in use".into());
                }
                d.request(&state.reference, &state.value)
            }
            Self::Custom(d) => {
                d.check_source(state)?;
                d.request(&state.reference, &state.value)
            }
            Self::Bus(d) => {
                let current = state
                    .source
                    .as_ref()
                    .and_then(|s| s.sheets.get(&d.basis.source))
                    .and_then(|s| s.buses.get(&d.target.id));
                if current != Some(&d.original) {
                    return Err("Bus source changed; cancel and reopen the edit".into());
                }
                d.request(&state.reference, &state.value)
            }
        }
    }
}
