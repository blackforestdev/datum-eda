//! Disposable field rows keyed by their index in the captured Symbol revision.
use super::*;
use crate::library_browser::LibraryControl as C;

const PREFIX: &str = "symbol_prefix";
const ROW_FIELDS: &[(&str, &str, &str)] = &[
    ("symbol_field_key", "Field key", "/key"),
    ("symbol_field_value", "Field value", "/value"),
    ("symbol_field_x", "Field X (optional)", "/position/x"),
    ("symbol_field_y", "Field Y (optional)", "/position/y"),
    ("symbol_field_visible", "Field visibility", "/visible"),
];

#[derive(Debug, Clone, PartialEq, Default)]
pub struct SymbolFieldDraft {
    pub rows: BTreeMap<usize, Value>,
    pub selected: Option<usize>,
    next: usize,
    clear_prefix: bool,
}

impl SymbolFieldDraft {
    pub fn controls(&self) -> impl Iterator<Item = C> + '_ {
        self.rows
            .keys()
            .copied()
            .map(C::SymbolField)
            .chain([C::AddSymbolField])
            .chain(self.selected.map(|_| C::RemoveSymbolField))
            .chain([C::ClearSymbolPrefix])
    }
}

impl LibraryDraft {
    pub(super) fn hydrate_symbol_fields(&mut self, raw: &Value) -> Result<(), String> {
        let rows = match raw.get("fields") {
            None => BTreeMap::new(),
            Some(fields) => fields
                .as_array()
                .ok_or("Saved Symbol fields are unavailable")?
                .iter()
                .cloned()
                .enumerate()
                .collect(),
        };
        self.electrical.symbol_fields = SymbolFieldDraft {
            next: rows.len(),
            rows,
            ..Default::default()
        };
        self.field(
            PREFIX,
            "Default reference prefix",
            raw.get("default_refdes_prefix"),
            vec![],
        );
        Ok(())
    }

    pub fn symbol_prefix(&self) -> Option<&str> {
        if self.dirty.contains(PREFIX) {
            Some(self.value(PREFIX))
        } else if self.electrical.symbol_fields.clear_prefix {
            None
        } else {
            self.electrical
                .original
                .as_ref()?
                .get("default_refdes_prefix")?
                .as_str()
        }
    }

    pub fn switch_symbol_field_control(
        &mut self,
        control: &C,
    ) -> Option<Result<&'static str, String>> {
        let selected = match *control {
            C::SymbolField(index) => Some(index),
            C::AddSymbolField | C::RemoveSymbolField | C::ClearSymbolPrefix => None,
            _ => return None,
        };
        if self.create || self.kind != "symbols" || self.electrical.original.is_none() {
            return Some(Err(
                "Field and prefix editing requires a saved Symbol".into()
            ));
        }
        if *control == C::ClearSymbolPrefix {
            self.electrical.symbol_fields.clear_prefix = true;
            self.dirty.remove(PREFIX);
            if let Some(field) = self.fields.iter_mut().find(|field| field.key == PREFIX) {
                field.set_text(String::new());
            }
            self.preview = None;
            self.refusal = None;
            return Some(Ok(PREFIX));
        }
        Some(self.switch_symbol_field(selected, *control == C::RemoveSymbolField))
    }

    fn switch_symbol_field(
        &mut self,
        selected: Option<usize>,
        remove: bool,
    ) -> Result<&'static str, String> {
        let draft = &self.electrical.symbol_fields;
        if selected.is_some_and(|index| !draft.rows.contains_key(&index)) {
            return Err("Choose an existing Symbol field".into());
        }
        if remove {
            let index = draft.selected.ok_or("Select a Symbol field to remove")?;
            self.electrical.symbol_fields.rows.remove(&index);
        } else if let Some(index) = draft.selected {
            let row = self.pending_symbol_field(&draft.rows[&index])?;
            self.electrical.symbol_fields.rows.insert(index, row);
        }

        let draft = &mut self.electrical.symbol_fields;
        draft.selected = if selected.is_none() && !remove {
            let index = draft.next;
            draft.next += 1;
            draft.rows.insert(index, json!({"key":"","value":""}));
            Some(index)
        } else {
            selected
        };
        let row = draft.selected.map(|index| draft.rows[&index].clone());
        self.fields
            .retain(|field| !ROW_FIELDS.iter().any(|(key, _, _)| *key == field.key));
        for &(key, label, pointer) in ROW_FIELDS {
            self.dirty.remove(key);
            if let Some(row) = &row {
                let mut value = row.pointer(pointer);
                let mut choices = Vec::new();
                if key == "symbol_field_visible" {
                    value = value.or(Some(&Value::Bool(false)));
                    choices = vec![
                        ("false".into(), "Hidden".into()),
                        ("true".into(), "Visible".into()),
                    ];
                }
                self.field(key, label, value, choices);
                self.fields.last_mut().unwrap().selector = key == "symbol_field_visible";
            }
        }
        self.preview = None;
        self.refusal = None;
        Ok(if row.is_some() {
            "symbol_field_key"
        } else {
            PREFIX
        })
    }

    fn pending_symbol_field(&self, basis: &Value) -> Result<Value, String> {
        let mut row = basis.clone();
        for (key, leaf) in [("symbol_field_key", "key"), ("symbol_field_value", "value")] {
            if self.dirty.contains(key) {
                row[leaf] = json!(self.value(key));
            }
        }
        if self.dirty.contains("symbol_field_visible") {
            let visible = self
                .value("symbol_field_visible")
                .parse::<bool>()
                .map_err(|_| "Choose Visible or Hidden")?;
            if visible != row["visible"].as_bool().unwrap_or(false) {
                row["visible"] = json!(visible);
            }
        }
        let (x, y) = ("symbol_field_x", "symbol_field_y");
        if self.dirty.contains(x) || self.dirty.contains(y) {
            if self.value(x).is_empty() && self.value(y).is_empty() {
                row.as_object_mut().unwrap().remove("position");
            } else {
                let position = row["position"].as_object().cloned().unwrap_or_default();
                row["position"] = position.into();
                row["position"]["x"] = json!(self.length(x)?);
                row["position"]["y"] = json!(self.length(y)?);
            }
        }
        Ok(row)
    }

    pub(super) fn symbol_field_request(
        &self,
        original: &Value,
        fields: &mut Map<String, Value>,
    ) -> Result<(), String> {
        let draft = &self.electrical.symbol_fields;
        if self.dirty.contains(PREFIX) || draft.clear_prefix {
            let prefix = json!(self.symbol_prefix());
            if original.get("default_refdes_prefix").is_some() || !prefix.is_null() {
                fields.insert("default_refdes_prefix".into(), prefix);
            }
        }
        let mut rows = draft.rows.clone();
        if let Some(index) = draft.selected {
            rows.insert(index, self.pending_symbol_field(&draft.rows[&index])?);
        }
        let previous = original["fields"]
            .as_array()
            .map(Vec::as_slice)
            .unwrap_or_default();
        let mut edits = Vec::new();
        let mut removals = Vec::new();
        for (index, previous) in previous.iter().enumerate() {
            match rows.remove(&index) {
                Some(field) if &field != previous => edits.push(json!({
                    "index": index, "previous": previous, "field": field,
                })),
                None => removals.push(json!({"index": index, "previous": previous})),
                _ => {}
            }
        }
        if !edits.is_empty() {
            fields.insert("symbol_field_edits".into(), edits.into());
        }
        if !removals.is_empty() {
            fields.insert("symbol_field_removals".into(), removals.into());
        }
        if !rows.is_empty() {
            fields.insert(
                "symbol_field_additions".into(),
                Value::Array(rows.into_values().collect()),
            );
        }
        Ok(())
    }
}
