//! Disposable named gate intent; definitions are qualified again by the engine.
use super::*;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct EntityDraft {
    pub gates: Map<String, Value>,
    pub saved_gates: Map<String, Value>,
    pub gate_id: Option<Uuid>,
    symbols: Vec<(Uuid, Uuid, String)>,
}
fn gate_key(gates: &Map<String, Value>, id: Uuid) -> Result<String, String> {
    let keys: Vec<_> = gates
        .keys()
        .filter(|k| k.parse::<Uuid>().ok() == Some(id))
        .collect();
    if keys.len() != 1 {
        return Err("Saved gate identity is missing or ambiguous".into());
    }
    Ok(keys[0].clone())
}
impl EntityDraft {
    pub fn selected_gate(&self) -> Option<&Value> {
        self.gates.get(&gate_key(&self.gates, self.gate_id?).ok()?)
    }
}
impl LibraryDraft {
    /// Configure the retained target's exact identities, not a name-based guess.
    pub fn retain_entity_symbol(&mut self, symbol: Uuid, unit: Uuid) -> Result<(), String> {
        if !self.create
            || self.kind != "entities"
            || !self
                .entity
                .symbols
                .iter()
                .any(|(s, u, _)| *s == symbol && *u == unit)
        {
            return Err("Retained Symbol and Unit are unavailable for this Entity".into());
        }
        self.fields
            .iter_mut()
            .find(|f| f.key == "gate_unit")
            .ok_or("Entity Unit control unavailable")?
            .set_text(unit.to_string());
        self.refresh_entity_symbols();
        self.fields
            .iter_mut()
            .find(|f| f.key == "gate_symbol")
            .ok_or("Entity Symbol control unavailable")?
            .set_text(symbol.to_string());
        Ok(())
    }
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
        if !self.create {
            self.entity.gates = raw
                .get("gates")
                .and_then(Value::as_object)
                .ok_or("Saved Entity gate data is unavailable")?
                .clone();
            self.entity.saved_gates = self.entity.gates.clone();
            self.field("gate_name", "Gate name", None, vec![]);
            self.field("gate_unit", "Named Unit", None, units);
            self.field("gate_symbol", "Compatible named Symbol", None, vec![]);
            return Ok(());
        }
        self.entity.gate_id = Some(Uuid::new_v4());
        self.field("gate_name", "Gate name", None, vec![]);
        self.field("gate_unit", "Named Unit", None, units);
        self.field("gate_symbol", "Compatible named Symbol", None, vec![]);
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
            if self.create && !field.choices.iter().any(|(id, _)| id == &field.text) {
                field.text.clear();
            }
        }
    }
    pub fn entity_gates(&self) -> Result<Map<String, Value>, String> {
        let mut gates = self.entity.gates.clone();
        let existing = self.entity.gate_id.is_none_or(|id| {
            self.entity
                .saved_gates
                .keys()
                .any(|key| key.parse::<Uuid>().ok() == Some(id))
        });
        if !self.create && existing {
            if self.dirty.contains("gate_name") {
                let gate = self.entity.gate_id.ok_or("Choose an existing gate")?;
                let key = gate_key(&gates, gate)?;
                let name = self.value("gate_name");
                if name.trim().is_empty() {
                    return Err("Enter a gate name".into());
                }
                gates
                    .get_mut(&key)
                    .and_then(Value::as_object_mut)
                    .ok_or("Saved gate is malformed")?
                    .insert("name".into(), json!(name));
            }
            if self.dirty.contains("gate_unit") || self.dirty.contains("gate_symbol") {
                let gate = self.entity.gate_id.ok_or("Choose an existing gate")?;
                let key = gate_key(&gates, gate)?;
                let raw = gates.get_mut(&key).ok_or("Saved gate is unavailable")?;
                let unit: Uuid = if self.dirty.contains("gate_unit") {
                    let unit = self.uuid("gate_unit")?;
                    if !self
                        .fields
                        .iter()
                        .find(|f| f.key == "gate_unit")
                        .unwrap()
                        .choices
                        .iter()
                        .any(|(id, _)| id == &unit.to_string())
                    {
                        return Err("Choose an available named Unit".into());
                    }
                    unit
                } else {
                    serde_json::from_value(raw["unit"].clone())
                        .map_err(|_| "Saved Unit is malformed")?
                };
                let symbol: Uuid = if self.dirty.contains("gate_symbol") {
                    self.uuid("gate_symbol")?
                } else {
                    serde_json::from_value(raw["symbol"].clone())
                        .map_err(|_| "Saved Symbol is malformed")?
                };
                if !self
                    .entity
                    .symbols
                    .iter()
                    .any(|(s, u, _)| *s == symbol && *u == unit)
                {
                    return Err("Choose an available Symbol for the selected gate Unit".into());
                }
                let row = raw.as_object_mut().ok_or("Saved gate is malformed")?;
                if self.dirty.contains("gate_unit") {
                    row.insert("unit".into(), json!(unit));
                }
                if self.dirty.contains("gate_symbol") {
                    row.insert("symbol".into(), json!(symbol));
                }
            }
            return Ok(gates);
        }
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
            let gates = self.entity_gates()?;
            let removals: Vec<_> = self
                .entity
                .saved_gates
                .keys()
                .filter(|key| !gates.contains_key(*key))
                .map(|key| {
                    key.parse::<Uuid>()
                        .map_err(|_| "Saved gate identity is malformed")
                })
                .collect::<Result<_, _>>()?;
            if !removals.is_empty() {
                if gates.is_empty() {
                    return Err("Add one or more named gates".into());
                }
                fields.insert("gate_removals".into(), json!(removals));
            }
            for (property, field) in [
                ("name", "gate_names"),
                ("unit", "gate_units"),
                ("symbol", "gate_symbols"),
            ] {
                let changes: Map<_, _> = gates
                    .iter()
                    .filter(|(id, g)| {
                        self.entity
                            .saved_gates
                            .get(*id)
                            .is_some_and(|old| old[property] != g[property])
                    })
                    .map(|(id, g)| (id.clone(), g[property].clone()))
                    .collect();
                if !changes.is_empty() {
                    fields.insert(field.into(), changes.into());
                }
            }
            let additions: Map<_, _> = gates
                .into_iter()
                .filter(|(id, _)| !self.entity.saved_gates.contains_key(id))
                .collect();
            if !additions.is_empty() {
                fields.insert("gate_additions".into(), additions.into());
            }
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
        if self.kind != "entities" {
            return Err("Gate authoring requires an Entity draft".into());
        }
        if !self.create && remove && self.entity.gate_id.is_none() {
            return Err("Choose an existing gate to remove".into());
        }
        if let Some(index) = index {
            let key = self
                .entity
                .gates
                .keys()
                .nth(index)
                .ok_or("Choose an existing gate")?;
            let id = key
                .parse()
                .map_err(|_| "Saved gate identity is malformed")?;
            gate_key(&self.entity.gates, id)?;
        }
        let selected = index.and_then(|i| self.entity.gates.keys().nth(i)).cloned();
        if remove {
            if let Some(id) = self.entity.gate_id {
                // Resolve the original raw key; saved UUID spellings are retained.
                let key = gate_key(&self.entity.gates, id).or_else(|error| {
                    if !self
                        .entity
                        .gates
                        .keys()
                        .any(|key| key.parse::<Uuid>().ok() == Some(id))
                        && !self
                            .entity
                            .saved_gates
                            .keys()
                            .any(|key| key.parse::<Uuid>().ok() == Some(id))
                    {
                        Ok(id.to_string()) // Discard an unbuffered new row.
                    } else {
                        Err(error)
                    }
                })?;
                self.entity.gates.remove(&key);
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
        if !self.create {
            for key in ["gate_name", "gate_unit", "gate_symbol"] {
                self.dirty.remove(key);
            }
        }
        self.preview = None;
        self.refusal = None;
        Ok(())
    }
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod edit_tests;

#[cfg(test)]
mod unit_edit_tests;

#[cfg(test)]
mod add_gate_tests;

#[cfg(test)]
mod remove_gate_tests;
