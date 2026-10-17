//! Disposable Package body and terminal drafts; no persistence or placed updates.
use super::*;

impl LibraryDraft {
    pub(super) fn package_fields(&mut self) {
        for (key, label) in [
            ("package_family", "Family (optional text)"),
            ("package_code", "Code (optional text)"),
            ("mounting_type", "Mounting (optional text)"),
            ("body_x", "Nominal body X (nm; blank = absent)"),
            ("body_y", "Nominal body Y (nm; blank = absent)"),
            ("body_z", "Nominal body Z (nm; blank = absent)"),
            ("body_height_nm", "Body height (nm; blank = unspecified)"),
            (
                "body_height_mounted_nm",
                "Mounted height (nm; blank = unspecified)",
            ),
            ("terminal_name", "Terminal name (blank = no new terminal)"),
            ("terminal_role", "Terminal role (optional text)"),
        ] {
            self.field(key, label, None, vec![]);
        }
    }

    pub(super) fn package_request(&self, fields: &mut Map<String, Value>) -> Result<(), String> {
        for key in ["package_family", "package_code", "mounting_type"] {
            if !self.value(key).is_empty() {
                fields.insert(key.into(), json!(self.value(key)));
            }
        }
        let mut dimensions = Map::new();
        for (key, target) in [("body_x", "x_nm"), ("body_y", "y_nm"), ("body_z", "z_nm")] {
            if !self.value(key).is_empty() {
                dimensions.insert(target.into(), json!(self.length(key)?));
            }
        }
        if !dimensions.is_empty() {
            fields.insert("body_dimensions".into(), dimensions.into());
        }
        for key in ["body_height_nm", "body_height_mounted_nm"] {
            if !self.value(key).is_empty() {
                fields.insert(key.into(), json!(self.length(key)?));
            }
        }
        let terminals = self.package_terminals()?;
        if !terminals.is_empty() {
            fields.insert("terminals".into(), terminals.into());
        }
        Ok(())
    }

    fn package_terminals(&self) -> Result<Map<String, Value>, String> {
        let mut terminals = self.terminals.clone();
        let name = self.value("terminal_name");
        if !name.trim().is_empty() {
            let id = self
                .terminal_id
                .ok_or("Start a named terminal before preview")?;
            let mut terminal = json!({"uuid":id,"name":name});
            if !self.value("terminal_role").is_empty() {
                terminal["role"] = json!(self.value("terminal_role"));
            }
            terminals.insert(id.to_string(), terminal);
        } else if !self.value("terminal_role").is_empty()
            || self
                .terminal_id
                .is_some_and(|id| terminals.contains_key(&id.to_string()))
        {
            return Err("Enter a terminal name, or remove the draft terminal".into());
        }
        Ok(terminals)
    }

    pub fn switch_terminal(&mut self, index: Option<usize>, remove: bool) -> Result<(), String> {
        let selected_id = index.and_then(|i| self.terminals.keys().nth(i)).cloned();
        if remove {
            if let Some(id) = self.terminal_id {
                self.terminals.remove(&id.to_string());
            }
        } else {
            self.terminals = self.package_terminals()?;
        }
        let selected =
            selected_id.and_then(|id| self.terminals.get(&id).cloned().map(|raw| (id, raw)));
        self.terminal_id = selected
            .as_ref()
            .and_then(|(id, _)| id.parse().ok())
            .or_else(|| (!remove).then(Uuid::new_v4));
        for (key, target) in [("terminal_name", "name"), ("terminal_role", "role")] {
            let value = selected
                .as_ref()
                .and_then(|(_, raw)| raw.get(target))
                .and_then(Value::as_str)
                .unwrap_or("");
            let field = self.fields.iter_mut().find(|f| f.key == key).unwrap();
            field.text = value.into();
            self.dirty.remove(key);
        }
        self.preview = None;
        self.refusal = None;
        Ok(())
    }
}

#[cfg(test)]
mod tests;
