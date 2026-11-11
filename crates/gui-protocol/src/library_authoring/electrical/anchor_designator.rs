//! Saved schematic pin designators stay separate from captured anchor poses.
use super::*;

pub const FIELD: &str = "anchor_designator";

impl LibraryDraft {
    fn designator_anchor(&self) -> Option<Uuid> {
        self.electrical
            .anchor_id
            .filter(|&id| !self.create && self.original_anchor(id))
    }

    pub fn has_anchor_designator_control(&self) -> bool {
        self.designator_anchor().is_some()
    }

    pub fn clear_anchor_designator(&mut self) -> Result<&'static str, String> {
        let id = self
            .designator_anchor()
            .ok_or("Select an existing saved Symbol anchor")?;
        self.electrical.anchor_designators.insert(id, Value::Null);
        self.dirty.remove(FIELD);
        if let Some(field) = self.fields.iter_mut().find(|field| field.key == FIELD) {
            field.set_text(String::new());
        }
        self.preview = None;
        self.refusal = None;
        Ok(FIELD)
    }

    pub(super) fn leave_anchor_designator(&mut self, remove: bool) {
        if let Some(id) = self.designator_anchor() {
            if remove {
                self.electrical.anchor_designators.remove(&id);
            } else if self.dirty.contains(FIELD) {
                let value = json!(self.value(FIELD));
                self.electrical.anchor_designators.insert(id, value);
            }
        }
    }

    pub(super) fn bind_anchor_designator(&mut self, raw: Option<&Value>) {
        self.fields.retain(|field| field.key != FIELD);
        self.dirty.remove(FIELD);
        if let Some(id) = self.designator_anchor() {
            let value = self
                .electrical
                .anchor_designators
                .get(&id)
                .or_else(|| raw.and_then(|row| row.get("number")))
                .cloned();
            self.field(FIELD, "Schematic pin designator", value.as_ref(), vec![]);
        }
    }

    pub fn anchor_designator(&self) -> Option<&str> {
        let id = self.designator_anchor()?;
        if self.dirty.contains(FIELD) {
            return Some(self.value(FIELD));
        }
        if let Some(value) = self.electrical.anchor_designators.get(&id) {
            return value.as_str();
        }
        self.electrical.original.as_ref()?["pin_anchors"]
            .as_array()?
            .iter()
            .find(|row| row["pin"].as_str().and_then(|pin| pin.parse().ok()) == Some(id))
            .and_then(|row| row["number"].as_str())
    }

    pub(super) fn anchor_designator_request(
        &self,
        original: &Value,
        fields: &mut Map<String, Value>,
    ) {
        let mut values = self.electrical.anchor_designators.clone();
        if let Some(id) = self
            .designator_anchor()
            .filter(|_| self.dirty.contains(FIELD))
        {
            values.insert(id, json!(self.value(FIELD)));
        }
        let edits: Vec<_> = original["pin_anchors"]
            .as_array()
            .into_iter()
            .flatten()
            .enumerate()
            .filter_map(|(index, row)| {
                let id: Uuid = row["pin"].as_str()?.parse().ok()?;
                let number = values.get(&id)?;
                let changed = if number.is_null() {
                    row.get("number").is_some()
                } else {
                    row.get("number") != Some(number)
                };
                changed.then(|| json!({"index":index,"previous":row,"number":number}))
            })
            .collect();
        if !edits.is_empty() {
            fields.insert("anchor_designators".into(), edits.into());
        }
    }
}
