//! Disposable named gate intent; definitions are qualified again by the engine.
use super::*;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct EntityDraft {
    pub gates: Map<String, Value>,
    pub gate_id: Option<Uuid>,
    symbols: Vec<(Uuid, Uuid, String)>,
}
impl LibraryDraft {
    pub(super) fn entity_fields(
        &mut self,
        browser: &LibraryBrowser,
        raw: &Value,
    ) -> Result<(), String> {
        self.field("prefix", "Reference prefix", raw.get("prefix"), vec![]);
        self.field(
            "manufacturer",
            "Manufacturer",
            raw.get("manufacturer"),
            vec![],
        );
        let tags = match raw.get("tags") {
            None => String::new(),
            Some(value) => serde_json::from_value::<Vec<String>>(value.clone())
                .map_err(|_| "Saved Entity tags are unavailable")?
                .join(", "),
        };
        self.field("tags", "Tags (comma separated)", Some(&json!(tags)), vec![]);
        if !self.create {
            self.entity.gates = raw
                .get("gates")
                .and_then(Value::as_object)
                .ok_or("Saved Entity gate data is unavailable")?
                .clone();
            return Ok(());
        }
        self.entity.gate_id = Some(Uuid::new_v4());
        self.field("gate_name", "Gate name", None, vec![]);
        let units = named(browser, "units")
            .into_iter()
            .filter(|(id, _)| {
                browser
                    .entries
                    .iter()
                    .filter(|e| e.kind == "units" && e.id.to_string() == *id)
                    .count()
                    == 1
            })
            .collect();
        self.field("gate_unit", "Named Unit", None, units);
        self.field("gate_symbol", "Compatible named Symbol", None, vec![]);
        self.entity.symbols = browser
            .entries
            .iter()
            .filter(|e| {
                e.kind == "symbols"
                    && e.warning.is_none()
                    && browser
                        .entries
                        .iter()
                        .filter(|other| other.kind == "symbols" && other.id == e.id)
                        .count()
                        == 1
            })
            .filter_map(|e| {
                Some((
                    e.id,
                    e.raw.get("unit")?.as_str()?.parse().ok()?,
                    format!("{} · {}", e.name, e.source_path),
                ))
            })
            .collect();
        Ok(())
    }
    pub(super) fn refresh_entity_symbols(&mut self) {
        let unit = self.value("gate_unit").parse::<Uuid>().ok();
        if let Some(field) = self.fields.iter_mut().find(|f| f.key == "gate_symbol") {
            field.choices = self
                .entity
                .symbols
                .iter()
                .filter(|(_, u, _)| Some(*u) == unit)
                .map(|(id, _, label)| (id.to_string(), label.clone()))
                .collect();
            if !field.choices.iter().any(|(id, _)| id == &field.text) {
                field.text.clear();
            }
        }
    }
    pub fn entity_gates(&self) -> Result<Map<String, Value>, String> {
        let mut gates = self.entity.gates.clone();
        let name = self.value("gate_name");
        if !name.trim().is_empty() {
            let id = self.entity.gate_id.ok_or("Start a named gate")?;
            let unit = self.uuid("gate_unit")?;
            let symbol = self.uuid("gate_symbol")?;
            if !self
                .fields
                .iter()
                .find(|f| f.key == "gate_unit")
                .unwrap()
                .choices
                .iter()
                .any(|(id, _)| id == &unit.to_string())
                || !self
                    .entity
                    .symbols
                    .iter()
                    .any(|(s, u, _)| *s == symbol && *u == unit)
            {
                return Err("Choose an available Unit and a compatible Symbol".into());
            }
            gates.insert(
                id.to_string(),
                json!({"uuid":id,"name":name,"unit":unit,"symbol":symbol}),
            );
        } else if !self.value("gate_unit").is_empty()
            || !self.value("gate_symbol").is_empty()
            || self
                .entity
                .gate_id
                .is_some_and(|id| gates.contains_key(&id.to_string()))
        {
            return Err("Enter a gate name, or remove the draft gate".into());
        }
        Ok(gates)
    }
    pub(super) fn entity_request(&self, fields: &mut Map<String, Value>) -> Result<(), String> {
        for key in ["prefix", "manufacturer"] {
            if self.create || self.dirty.contains(key) {
                fields.insert(key.into(), json!(self.value(key)));
            }
        }
        let tags: BTreeSet<_> = self
            .value("tags")
            .split(',')
            .map(str::trim)
            .filter(|v| !v.is_empty())
            .collect();
        if self.create || self.dirty.contains("tags") {
            fields.insert("tags".into(), json!(tags));
        }
        if !self.create {
            return Ok(());
        }
        let gates = self.entity_gates()?;
        if gates.is_empty() {
            return Err("Add one or more named gates".into());
        }
        fields.insert("gates".into(), gates.into());
        Ok(())
    }
    pub fn switch_gate(&mut self, index: Option<usize>, remove: bool) -> Result<(), String> {
        if !self.create || self.kind != "entities" {
            return Err("Gate authoring requires a new Entity draft".into());
        }
        let selected = index.and_then(|i| self.entity.gates.keys().nth(i)).cloned();
        if remove {
            if let Some(id) = self.entity.gate_id {
                self.entity.gates.remove(&id.to_string());
            }
        } else {
            self.entity.gates = self.entity_gates()?;
        }
        let selected =
            selected.and_then(|id| self.entity.gates.get(&id).cloned().map(|raw| (id, raw)));
        self.entity.gate_id = selected
            .as_ref()
            .and_then(|(id, _)| id.parse().ok())
            .or_else(|| (!remove).then(Uuid::new_v4));
        let raw = selected.map(|(_, raw)| raw).unwrap_or(Value::Null);
        for (key, property) in [("gate_name", "name"), ("gate_unit", "unit")] {
            self.fields
                .iter_mut()
                .find(|f| f.key == key)
                .unwrap()
                .set_text(
                    raw.get(property)
                        .and_then(Value::as_str)
                        .unwrap_or("")
                        .into(),
                );
        }
        self.refresh_entity_symbols();
        self.fields
            .iter_mut()
            .find(|f| f.key == "gate_symbol")
            .unwrap()
            .set_text(
                raw.get("symbol")
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .into(),
            );
        self.preview = None;
        self.refusal = None;
        Ok(())
    }
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod edit_tests;
