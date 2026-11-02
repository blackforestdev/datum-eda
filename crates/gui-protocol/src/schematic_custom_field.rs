//! UUID-qualified custom metadata uses the shared two-field form, without presentation edits.
use super::*;
use eda_engine::schematic::{PlacedSymbol, SymbolField};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CustomFieldControl {
    Open,
    Previous,
    Next,
    New,
    Remove,
    Symbol,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CustomFieldDraft {
    pub symbol: selected::SymbolEditDraft,
    pub original: PlacedSymbol,
    pub field_id: Option<Uuid>,
    pub remove: bool,
}
impl CustomFieldDraft {
    pub fn selected(&self) -> Option<&SymbolField> {
        self.original
            .fields
            .iter()
            .find(|f| Some(f.uuid) == self.field_id)
    }
    pub fn fields(&self) -> (String, String) {
        self.selected().map_or_else(
            || (String::new(), String::new()),
            |f| (f.key.clone(), f.value.clone()),
        )
    }
    pub fn check_source(&self, state: &SchematicAuthoring) -> Result<(), String> {
        let current = state
            .source
            .as_ref()
            .and_then(|s| s.sheets.get(&self.symbol.basis.source))
            .and_then(|s| s.symbols.get(&self.symbol.target.id));
        if current != Some(&self.original) {
            return Err("Symbol source changed; cancel and reopen its fields".into());
        }
        if let Some(id) = self.field_id
            && self.original.fields.iter().filter(|f| f.uuid == id).count() != 1
        {
            return Err("Field identity is unavailable or ambiguous".into());
        }
        Ok(())
    }
    pub fn require_clean(&self, key: &str, value: &str) -> Result<(), String> {
        let original = self.fields();
        if self.remove || key != original.0 || value != original.1 {
            return Err("Apply or cancel the current field changes first".into());
        }
        Ok(())
    }
    /// Changing the selected UUID never writes or silently discards edited text.
    pub fn navigate(
        &mut self,
        control: CustomFieldControl,
        key: &str,
        value: &str,
    ) -> Result<(), String> {
        self.require_clean(key, value)?;
        match control {
            CustomFieldControl::New => self.field_id = None,
            CustomFieldControl::Remove => {
                if self.selected().is_none() {
                    return Err("Choose an existing field to remove".into());
                }
                self.remove = true;
            }
            CustomFieldControl::Previous | CustomFieldControl::Next => {
                let count = self.original.fields.len();
                if count == 0 {
                    return Err("No custom fields yet; enter a new metadata key and value".into());
                }
                let selected = self
                    .original
                    .fields
                    .iter()
                    .position(|f| Some(f.uuid) == self.field_id);
                let index = match (selected, control) {
                    (Some(index), CustomFieldControl::Previous) => (index + count - 1) % count,
                    (Some(index), _) => (index + 1) % count,
                    (None, _) => 0,
                };
                self.field_id = Some(self.original.fields[index].uuid);
            }
            CustomFieldControl::Open | CustomFieldControl::Symbol => {}
        }
        Ok(())
    }
    pub fn request(&self, key: &str, value: &str) -> Result<Option<SchematicEditRequest>, String> {
        let (old_key, old_value) = self.fields();
        if self.remove {
            if self.field_id.is_none() || key != old_key || value != old_value {
                return Err("Cancel the removal before editing field text".into());
            }
        } else {
            if self.field_id.is_some() && key == old_key && value == old_value {
                return Ok(None);
            }
            if key.trim().is_empty()
                || key.chars().any(char::is_control)
                || value.chars().any(char::is_control)
            {
                return Err("Enter a nonempty single-line key and a single-line value".into());
            }
        }
        Ok(Some(SchematicEditRequest::CustomField {
            symbol: self.symbol.target.id,
            instance_path: self.symbol.target.instance_path.clone(),
            field_id: self.field_id,
            key: (!self.remove).then(|| key.into()),
            value: (!self.remove).then(|| value.into()),
            clear: self.remove,
        }))
    }
}
